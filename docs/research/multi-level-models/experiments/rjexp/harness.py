"""Run report-jobs' real code in threads, one step at a time, and enumerate schedules.

One scheduler step is one of: a SELECT, a session commit (its flush and COMMIT together),
an S3 upload, a Celery message sent, a message received by a worker. The app writes only
inside `Session.commit()` and reads committed data outside it, so with these steps SQLite
behaves as Postgres under READ COMMITTED does for this code: a read sees the last commit, a
write is applied at once, and nothing is left uncommitted between steps.

Outside services are fakes: S3 (an upload can be made to fail, as an environment choice),
the Celery broker (a list of messages), the clock (`requeue_stuck` always sees the rows as
stuck: "15 minutes passed" is an environment choice, as in the specification, F-S5).

Schedules are enumerated by re-execution with a preemption bound, as in the rq harness:
switching away from an actor in the middle of an operation costs one preemption, switching
at operation boundaries is free, so bound 0 gives the runs where each operation is atomic.

Each run is written as plain data: one effect per step, in the shape that mlexp/levels.py
cuts into levels, plus outputs and the observable state at the end. A run is anomalous when its
settled state (after the messages still queued are handled) and the answers its API and CLI calls
gave differ from those of every run with no preemption.

usage: python -I harness.py <report-jobs src dir> <out dir> [bound]
"""

from __future__ import annotations

import collections
import datetime
import os
import pathlib
import pickle
import re
import shutil
import sys
import tempfile
import threading
import time
from typing import Any

GATE_TIMEOUT_S = 20.0
MAX_RUNS = 20000
_tls = threading.local()


class GateTimeout(RuntimeError):
    pass


class Gate:
    """Hands control to one actor thread at a time (the same protocol as the rq harness)."""

    def __init__(self) -> None:
        self.cv = threading.Condition()
        self.waiting: dict[str, str] = {}
        self.detail: dict[str, dict[str, Any]] = {}
        self.done: set[str] = set()
        self.granted: str | None = None
        self.running: str | None = None
        self.events: list[tuple[str, str, str, Any]] = []

    def arrive(self, actor: str, label: str, detail: dict[str, Any]) -> None:
        with self.cv:
            self.waiting[actor] = label
            self.detail[actor] = detail
            if self.running == actor:
                self.running = None
            self.cv.notify_all()
            if not self.cv.wait_for(lambda: self.granted == actor, GATE_TIMEOUT_S * 5):
                raise GateTimeout(f'actor {actor} was never granted {label}')
            self.granted = None
            del self.waiting[actor]

    def finish(self, actor: str) -> None:
        with self.cv:
            self.done.add(actor)
            if self.running == actor:
                self.running = None
            self.cv.notify_all()

    def start(self, actor: str, thread: threading.Thread) -> None:
        with self.cv:
            self.running = actor
        thread.start()
        self._settle(actor)

    def grant(self, actor: str) -> None:
        with self.cv:
            self.granted = actor
            self.running = actor
            self.cv.notify_all()
        self._settle(actor)

    def _settle(self, actor: str) -> None:
        with self.cv:
            if not self.cv.wait_for(lambda: self.running is None, GATE_TIMEOUT_S):
                raise GateTimeout(f'actor {actor} did not reach a gate or finish')

    def record(self, actor: str, kind: str, op: str, value: Any = None) -> None:
        with self.cv:
            self.events.append((actor, kind, op, value))


def gate_step(label: str, commands: list[tuple[str, list[str]]], **flags: Any) -> dict[str, Any]:
    """Wait for the scheduler, in an actor thread; a no-op elsewhere (setup, snapshots)."""
    detail: dict[str, Any] = {'label': label, 'commands': commands, 'rejected': False, **flags}
    gate = getattr(_tls, 'gate', None)
    if gate is not None:
        detail.update(app_stack() if 'stack' not in flags else {})
        gate.arrive(_tls.actor, label, detail)
    return detail


SRC: pathlib.Path


def app_stack() -> dict[str, Any]:
    """The app functions on the calling thread's stack, outermost first, and the innermost call site."""
    names, anchor = [], None
    frame = sys._getframe(2)
    while frame is not None:
        if frame.f_code.co_filename.startswith(str(SRC / 'app')):
            names.append(frame.f_code.co_qualname)
            if anchor is None:
                anchor = f'{frame.f_code.co_qualname}:{frame.f_lineno}'
        frame = frame.f_back
    return {'stack': names[::-1], 'anchor': anchor}


# ---------------------------------------------------------------------------
# The world outside the app: S3, the broker, the clock, ids.


class World:
    def __init__(self) -> None:
        self.broker: list[dict[str, Any]] = []
        self.uploads: collections.Counter[str] = collections.Counter()
        self.s3_fail = 0
        self.next_id = 0
        self.next_msg = 0
        self.inflight: collections.Counter[str] = collections.Counter()


WORLD = World()


def install(src: pathlib.Path) -> Any:
    """Import the app with fakes in place; return the modules."""
    global SRC
    SRC = src
    os.environ.update({'DATABASE_URL': 'sqlite://', 'CELERY_BROKER_URL': 'memory://', 'REPORTS_BUCKET': 'acme-reports',
                       'AWS_DEFAULT_REGION': 'us-east-1', 'AWS_ACCESS_KEY_ID': 'x', 'AWS_SECRET_ACCESS_KEY': 'x'})
    os.environ.pop('WEBHOOK_URL', None)
    sys.path.insert(0, str(src))
    import sqlalchemy
    from botocore.exceptions import ClientError
    from celery.exceptions import Retry
    from sqlalchemy import event
    from sqlalchemy.engine import Engine
    from sqlalchemy.orm import Session

    from app import api, cli, db, models, tasks

    commit = Session.commit

    def gated_commit(self: Session) -> None:
        detail = gate_step('COMMIT', [], tx_end=True)
        _tls.collect = detail['commands']
        try:
            commit(self)
        except sqlalchemy.exc.IntegrityError:
            detail['rejected'] = True
            raise
        finally:
            _tls.collect = None
            detail['label'] = 'COMMIT[' + '; '.join(f'{n} {" ".join(pattern(k) for k in ks)}' for n, ks in detail['commands']) + ']'

    Session.commit = gated_commit  # type: ignore[method-assign]

    def on_execute(conn: Any, cursor: Any, statement: str, params: Any, context: Any, executemany: bool) -> None:
        verb = statement.split(None, 1)[0].upper()
        keys = sql_keys(verb, statement, params)
        collect = getattr(_tls, 'collect', None)
        if collect is not None:
            collect.append((verb, keys))
        elif verb == 'SELECT':
            gate_step(f'SELECT {" ".join(pattern(k) for k in keys)}', [('SELECT', keys)], tx_open=True)

    event.listen(Engine, 'before_cursor_execute', on_execute)

    class FakeS3:
        def put_object(self, Bucket: str, Key: str, Body: str) -> None:  # noqa: N803
            detail = gate_step(f'S3PUT {pattern("s3:" + Key)}', [('S3PUT', ['s3:' + Key])])
            if WORLD.s3_fail > 0:
                WORLD.s3_fail -= 1
                detail['rejected'] = True
                raise ClientError({'Error': {'Code': '503', 'Message': 'slow down'}}, 'PutObject')
            WORLD.uploads[Key] += 1
            detail['outside'] = True

    tasks.s3 = FakeS3()
    task = tasks.generate_report

    def delay(report_id: str) -> None:
        gate_step('SEND generate_report', [('SEND', ['broker:q'])])
        WORLD.broker.append({'report': report_id, 'retries': 0})

    def retry(exc: Any = None, **kwargs: Any) -> Retry:
        gate_step('SEND generate_report (retry)', [('SEND', ['broker:q'])])
        WORLD.broker.append({'report': task.request.args_report, 'retries': task.request.retries + 1})
        return Retry()

    task.delay = delay
    task.retry = retry

    class Clock(datetime.datetime):
        @classmethod
        def utcnow(cls) -> datetime.datetime:  # type: ignore[override]
            return datetime.datetime(2100, 1, 1)

    tasks.datetime = Clock

    class Ids:
        @staticmethod
        def uuid4() -> str:
            WORLD.next_id += 1
            return f'r{WORLD.next_id}'

    api.uuid = Ids
    return {'api': api, 'cli': cli, 'db': db, 'models': models, 'tasks': tasks, 'Retry': Retry, 'sqlalchemy': sqlalchemy}


ID = re.compile(r'\b(r\d+|k\d+)\b')


def pattern(key: str) -> str:
    return ID.sub('*', key)


def sql_keys(verb: str, statement: str, params: Any) -> list[str]:
    """The rows a statement touches, as keys: report:<id>, reports:key:<k>, reports:status, orders."""
    s = ' '.join(statement.split()).lower()
    p = list(params) if isinstance(params, (list, tuple)) else []
    if 'orders' in s.split(' where ')[0]:
        return ['orders']
    if verb == 'SELECT':
        if 'idempotency_key' in s.split(' where ')[-1]:
            return [f'reports:key:{p[0]}']
        if 'where reports.id' in s:
            return [f'report:{p[0]}']
        if 'reports.status' in s.split(' where ')[-1]:
            return ['reports:status']
        return ['reports']
    if verb == 'INSERT':
        return [f'report:{p[0]}', f'reports:key:{p[1]}', 'reports:status']
    if verb == 'UPDATE':
        keys = [f'report:{p[-1]}']
        return keys + (['reports:status'] if 'status=' in s.replace(' ', '') else [])
    return ['reports']


# ---------------------------------------------------------------------------
# Operations and scenarios


def operations(m: dict[str, Any]) -> dict[str, Any]:
    api, cli, tasks, retry_exc = m['api'], m['cli'], m['tasks'], m['Retry']
    from fastapi import HTTPException
    import click

    def create(key: str) -> str:
        req = api.ReportRequest(start=datetime.date(2026, 1, 1), end=datetime.date(2026, 1, 31))
        try:
            return f'202 {api.create_report(req, idempotency_key=key)["status"]}'
        except HTTPException as exc:
            return str(exc.status_code)
        except Exception as exc:  # an unhandled error is a 500 from FastAPI
            return f'500 {type(exc).__name__}'

    def work() -> str:
        gate_step('RECV', [('RECV', ['broker:q'])], stack=['celery worker'], anchor='celery worker:receive')
        if not WORLD.broker:
            return 'empty'
        msg = WORLD.broker.pop(0)
        WORLD.inflight[msg['report']] += 1
        WORLD.next_msg += 1
        task = tasks.generate_report
        task.push_request(id=f'm{WORLD.next_msg}', retries=msg['retries'], args_report=msg['report'])
        try:
            task.run(msg['report'])
            return 'ok'
        except retry_exc:
            return 'retry'
        except Exception as exc:
            return f'error {type(exc).__name__}'
        finally:
            task.pop_request()
            WORLD.inflight[msg['report']] -= 1

    def requeue() -> str:
        tasks.requeue_stuck.run()
        return 'ok'

    def retry(report_id: str) -> str:
        try:
            cli.retry.callback(report_id)
            return 'requeued'
        except click.ClickException as exc:
            return f'refused: {exc.message}'

    return {'create': create, 'work': work, 'requeue': requeue, 'retry': retry}


# name: (reports at the start as (id, status), messages for report ids, S3 failures, actors and their operations)
SCENARIOS: dict[str, dict[str, Any]] = {
    'create-and-work': {'reports': [], 'messages': [], 's3_fail': 0, 'actors': {'c': ['create:k1'], 'w1': ['work']}},
    'two-workers': {'reports': [('r1', 'PENDING')], 'messages': ['r1', 'r1'], 's3_fail': 0, 'actors': {'w1': ['work'], 'w2': ['work']}},
    'requeue-vs-work': {'reports': [('r1', 'PENDING')], 'messages': ['r1'], 's3_fail': 0, 'actors': {'w1': ['work'], 'beat': ['requeue']}},
    'duplicate-create': {'reports': [], 'messages': [], 's3_fail': 0, 'actors': {'c1': ['create:k1'], 'c2': ['create:k1']}},
    'fail-and-retry': {'reports': [('r1', 'PENDING')], 'messages': ['r1'], 's3_fail': 1, 'actors': {'w1': ['work'], 'w2': ['work']}},
    'operator-retry-vs-work': {'reports': [('r1', 'FAILED')], 'messages': ['r1'], 's3_fail': 0, 'actors': {'op': ['retry:r1'], 'w1': ['work']}},
}


def observe(engine: Any) -> dict[str, Any]:
    """What a user or operator can observe per report: status, whether it has a URL, how many
    times its file was uploaded, and whether it is stranded (PENDING, with no message queued or
    being handled for it, so nothing will ever pick it up)."""
    with engine.connect() as conn:
        rows = conn.exec_driver_sql('SELECT id, status, url FROM reports ORDER BY id').fetchall()
    queued = collections.Counter(m['report'] for m in WORLD.broker)
    return {rid: (status, url is not None, WORLD.uploads[f'reports/{rid}.csv'], status == 'PENDING' and queued[rid] + WORLD.inflight[rid] == 0)
            for rid, status, url in rows}


def execute(m: dict[str, Any], ops: dict[str, Any], template: pathlib.Path, scn: dict[str, Any], prefix: list[str]) -> dict[str, Any]:
    import sqlalchemy
    from sqlalchemy.pool import NullPool

    work = pathlib.Path(tempfile.mkdtemp(dir=template.parent))
    shutil.copy(template, work / 'db.sqlite')
    engine = sqlalchemy.create_engine(f'sqlite:///{work / "db.sqlite"}', poolclass=NullPool)
    m['db'].Session.configure(bind=engine)
    WORLD.__init__()
    WORLD.broker = [{'report': r, 'retries': 0} for r in scn['messages']]
    WORLD.s3_fail = scn['s3_fail']
    WORLD.next_id = len(scn['reports'])
    gate = Gate()
    outputs: dict[str, list[Any]] = collections.defaultdict(list)

    def main(actor: str, program: list[str]) -> None:
        _tls.gate, _tls.actor = gate, actor
        try:
            for n, op in enumerate(program):
                name, _, arg = op.partition(':')
                gate.record(actor, 'begin', f'{actor}#{n}:{name}')
                value = ops[name](arg) if arg else ops[name]()
                outputs[actor].append(value)
                gate.record(actor, 'end', f'{actor}#{n}:{name}', value)
        finally:
            gate.finish(actor)

    state = observe(engine)  # connects once in this thread, so the dialect's first-connect queries are not gated
    current: dict[str, str] = {}
    for actor, program in scn['actors'].items():
        n_events = len(gate.events)
        gate.start(actor, threading.Thread(target=main, args=(actor, program), daemon=True))
        for a, kind, op, _ in gate.events[n_events:]:
            if kind == 'begin':
                current[a] = op
    effects, choices, schedule = [], [], []
    mid = collections.Counter()
    last = None
    while gate.waiting:
        enabled = sorted(gate.waiting)
        i = len(schedule)
        chosen = prefix[i] if i < len(prefix) else (last if last in enabled else enabled[0])
        preemptive = last is not None and last in enabled and chosen != last and mid[last] > 0
        detail = gate.detail[chosen]
        op = current[chosen]
        n_events = len(gate.events)
        gate.grant(chosen)
        events = gate.events[n_events:]
        mid[chosen] += 1
        output = None
        for a, kind, op_id, value in events:
            if kind == 'begin':
                current[a] = op_id
                mid[a] = 0
            elif a == chosen and op_id == op:
                output = value
        after = observe(engine)
        changes = [[rid, state.get(rid), after.get(rid)] for rid in sorted(set(state) | set(after)) if state.get(rid) != after.get(rid)]
        reads = sorted({k for n, ks in detail['commands'] if n in ('SELECT', 'RECV') for k in ks})
        writes = [] if detail['rejected'] else sorted({k for n, ks in detail['commands'] if n != 'SELECT' for k in ks})
        effects.append({
            'i': i, 'actor': chosen, 'call': op.split(':')[0], 'api': op.split(':')[1], 'label': detail['label'],
            'commands': detail['commands'], 'stack': detail.get('stack', []), 'anchor': detail.get('anchor'),
            'rejected': detail['rejected'], 'changed': [c[0] for c in changes], 'writes': writes, 'reads': reads,
            'ref_change': changes, 'body_ran': bool(detail.get('outside')), 'output': output,
            'tx_open': bool(detail.get('tx_open')), 'tx_end': bool(detail.get('tx_end')),
        })
        choices.append((enabled, chosen, preemptive))
        schedule.append(chosen)
        state = after
        last = chosen
    # Settle: messages still queued are handled one by one, outside the explored schedule, as a worker
    # would later. The oracle compares settled states, so a retry still in the queue is not an anomaly.
    for _ in range(10):
        if not WORLD.broker:
            break
        ops['work']()
    settled = observe(engine)
    engine.dispose()
    shutil.rmtree(work, ignore_errors=True)
    return {'effects': effects, 'choices': choices, 'schedule': schedule, 'outputs': dict(outputs), 'end': state, 'final': settled}


def make_template(m: dict[str, Any], scn: dict[str, Any], path: pathlib.Path) -> None:
    import sqlalchemy

    engine = sqlalchemy.create_engine(f'sqlite:///{path}')
    m['models'].Base.metadata.create_all(engine)
    with engine.begin() as conn:
        for i in range(3):
            conn.exec_driver_sql('INSERT INTO orders (id, created, total_cents) VALUES (?, ?, ?)', (i + 1, f'2026-01-{i + 10}', 1000 * (i + 1)))
        for n, (rid, status) in enumerate(scn['reports']):
            conn.exec_driver_sql("INSERT INTO reports (id, idempotency_key, start, \"end\", status) VALUES (?, ?, '2026-01-01', '2026-01-31', ?)", (rid, f'k{n + 1}', status))
    engine.dispose()


def explore(m: dict[str, Any], ops: dict[str, Any], name: str, scn: dict[str, Any], bound: int, scratch: pathlib.Path) -> dict[str, Any]:
    template = scratch / f'{name}.sqlite'
    make_template(m, scn, template)
    runs = []
    stack: list[list[str]] = [[]]
    started = time.time()
    while stack and len(runs) < MAX_RUNS:
        prefix = stack.pop()
        rec = execute(m, ops, template, scn, prefix)
        preemptions = 0
        for index, (enabled, chosen, preemptive) in enumerate(rec['choices']):
            if index >= len(prefix):
                last = rec['schedule'][index - 1] if index > 0 else None
                for alt in enabled:
                    if alt == chosen:
                        continue
                    # Switching away from an actor in the middle of an operation is a preemption.
                    extra = 1 if (last is not None and last in enabled and alt != last and _mid(rec, index, last)) else 0
                    if preemptions + extra <= bound:
                        stack.append(rec['schedule'][:index] + [alt])
            preemptions += 1 if preemptive else 0
        rec['preemptions'] = preemptions
        del rec['choices']
        runs.append(rec)
    # What users and operators see: the settled state, and the answers of the API and the CLI.
    # A worker's return value ("ok", "empty", "retry") is internal and depends only on timing.
    public = {a for a, program in scn['actors'].items() if any(op.split(':')[0] in ('create', 'retry') for op in program)}
    key = lambda r: (sorted(r['final'].items()), sorted((a, tuple(v)) for a, v in r['outputs'].items() if a in public))  # noqa: E731
    sequential = {repr(key(r)) for r in runs if r['preemptions'] == 0}
    labels = [repr(key(r)) not in sequential for r in runs]
    return {'scenario': name, 'bound': bound, 'runs': runs, 'labels': labels,
            'meta': {'runs': len(runs), 'anomalous': sum(labels), 'seconds': round(time.time() - started, 1), 'truncated': bool(stack)}}


def _mid(rec: dict[str, Any], index: int, actor: str) -> bool:
    """Whether `actor` has taken a step of its current operation before step `index`."""
    effects = rec['effects'][:index]
    mine = [e for e in effects if e['actor'] == actor]
    if not mine:
        return False
    last_op = mine[-1]['call']
    finished = any(e['output'] is not None for e in mine if e['call'] == last_op)
    return not finished


def main() -> None:
    src, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2])
    bound = int(sys.argv[3]) if len(sys.argv) > 3 else 2
    out.mkdir(parents=True, exist_ok=True)
    m = install(src)
    ops = operations(m)
    scratch = pathlib.Path(tempfile.mkdtemp())
    result = {}
    for name, scn in SCENARIOS.items():
        result[name] = explore(m, ops, name, scn, bound, scratch)
        print(name, result[name]['meta'], flush=True)
    (out / 'rj-runs.pkl').write_bytes(pickle.dumps(result))
    shutil.rmtree(scratch, ignore_errors=True)


if __name__ == '__main__':
    main()
