"""Run real rq code in threads, one Redis command or transaction at a time.

Every Redis call an actor thread makes waits at a gate until the scheduler
grants it. A granted call runs, then the actor continues until its next Redis
call or the end of its program. So one scheduler step is one atomic Redis
step, which is the grain at which Redis itself serialises work.
"""

from __future__ import annotations

import datetime
import hashlib
import json
import re
import sys
import threading
import zlib
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

import fakeredis
import redis
import redis.client

FROZEN_NOW = datetime.datetime(2026, 1, 1, tzinfo=datetime.timezone.utc)
GATE_TIMEOUT_S = 20.0

_tls = threading.local()


class GateTimeout(RuntimeError):
    pass


class Gate:
    """Hands control to one actor thread at a time."""

    def __init__(self) -> None:
        self.cv = threading.Condition()
        self.waiting: dict[str, str] = {}
        self.done: set[str] = set()
        self.granted: str | None = None
        self.running: str | None = None
        self.events: list[tuple[str, str, str, Any]] = []
        self.results: dict[str, list[tuple[str, Any]]] = {}

    def arrive(self, actor: str, label: str) -> None:
        with self.cv:
            self.waiting[actor] = label
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
        self._wait_settled(actor)

    def grant(self, actor: str) -> None:
        with self.cv:
            assert actor in self.waiting, f'{actor} is not at a gate'
            self.granted = actor
            self.running = actor
            self.cv.notify_all()
        self._wait_settled(actor)

    def _wait_settled(self, actor: str) -> None:
        with self.cv:
            if not self.cv.wait_for(lambda: self.running is None, GATE_TIMEOUT_S):
                raise GateTimeout(f'actor {actor} did not reach a gate or finish')

    def record(self, actor: str, kind: str, op: str, value: Any = None) -> None:
        with self.cv:
            self.events.append((actor, kind, op, value))


def _current_gate() -> tuple[Gate, str] | None:
    gate = getattr(_tls, 'gate', None)
    if gate is None:
        return None
    return gate, _tls.actor


ID_PATTERN = re.compile(r'\b(j\d+|w\d+|x\d+w\d+|default)\b')


def key_pattern(key: str) -> str:
    return ID_PATTERN.sub('*', key)


def _text(value: Any) -> str:
    return value.decode() if isinstance(value, bytes) else str(value)


def command_label(args: tuple[Any, ...]) -> str:
    name = _text(args[0]).upper()
    if name in ('EVALSHA', 'EVAL') and len(args) > 3:
        return f'{name} {key_pattern(_text(args[3]))}'
    if name in FIELD_READS and len(args) > 2:
        return f'{name} {key_pattern(_text(args[1]))} ' + ','.join(_text(a) for a in args[2:])
    if len(args) > 1:
        return f'{name} {key_pattern(_text(args[1]))}'
    return name


FIELD_READS = {'HGET', 'HMGET', 'HEXISTS'}


def _stack_label(stack: list[Any]) -> str:
    parts = []
    for entry in stack:
        args = entry[0] if isinstance(entry, tuple) and entry and isinstance(entry[0], tuple) else entry.args
        parts.append(command_label(args))
    return 'EXEC[' + '; '.join(parts) + ']'


def _after(gate_actor: tuple[Gate, str] | None, label: str, result: Any) -> None:
    if gate_actor is None:
        return
    gate, actor = gate_actor
    with gate.cv:
        gate.results.setdefault(actor, []).append((label, digest(normalize_value(result))))


_orig_execute_command = redis.client.Redis.execute_command
_orig_immediate = redis.client.Pipeline.immediate_execute_command
_orig_pipeline_execute = redis.client.Pipeline.execute
_orig_info = redis.client.Redis.info


UNGATED_COMMANDS = {'CLIENT'}


def _gated_execute_command(self: Any, *args: Any, **options: Any) -> Any:
    label = command_label(args)
    # CLIENT touches no key, so it commutes with every other step.
    ga = None if label.split(' ')[0] in UNGATED_COMMANDS else _current_gate()
    if ga is not None:
        ga[0].arrive(ga[1], label)
    result = _orig_execute_command(self, *args, **options)
    _after(ga, label, result)
    return result


def _gated_immediate(self: Any, *args: Any, **options: Any) -> Any:
    ga = _current_gate()
    label = command_label(args)
    if ga is not None:
        ga[0].arrive(ga[1], label)
    result = _orig_immediate(self, *args, **options)
    _after(ga, label, result)
    return result


def _gated_pipeline_execute(self: Any, raise_on_error: bool = True) -> Any:
    ga = _current_gate()
    stack = list(self.command_stack)
    if not stack:
        return _orig_pipeline_execute(self, raise_on_error)
    label = _stack_label(stack)
    if ga is not None:
        ga[0].arrive(ga[1], label)
    result = _orig_pipeline_execute(self, raise_on_error)
    _after(ga, label, result)
    return result


def _fake_info(self: Any, section: Any = None, *args: Any, **kwargs: Any) -> dict[str, Any]:
    # fakeredis has no INFO; rq only asks it for the server version.
    return {'redis_version': '7.2.0'}


def install_patches() -> None:
    redis.client.Redis.execute_command = _gated_execute_command  # type: ignore[method-assign]
    redis.client.Pipeline.immediate_execute_command = _gated_immediate  # type: ignore[method-assign]
    redis.client.Pipeline.execute = _gated_pipeline_execute  # type: ignore[method-assign]
    redis.client.Redis.info = _fake_info  # type: ignore[method-assign]


def freeze_rq_clock() -> None:
    """Make every rq timestamp the same instant, so snapshots are deterministic."""
    import rq.utils as utils

    replacements: dict[str, Callable[..., Any]] = {}
    if hasattr(utils, 'now'):
        replacements['now'] = lambda: FROZEN_NOW
    if hasattr(utils, 'utcnow'):
        replacements['utcnow'] = lambda: FROZEN_NOW.replace(tzinfo=None)
    if hasattr(utils, 'current_timestamp'):
        replacements['current_timestamp'] = lambda: int(FROZEN_NOW.timestamp())
    originals = {name: getattr(utils, name) for name in replacements}
    for module_name, module in list(sys.modules.items()):
        if module is None or not (module_name == 'rq' or module_name.startswith('rq.')):
            continue
        for name, original in originals.items():
            if getattr(module, name, None) is original:
                setattr(module, name, replacements[name])


def rq_now() -> datetime.datetime:
    """The frozen instant in the form this rq version uses (aware in 2.x, naive in 1.x)."""
    import rq.utils as utils

    return utils.utcnow() if hasattr(utils, 'utcnow') else utils.now()


def deterministic_ids() -> None:
    """Execution ids become x<n><actor>, numbered per actor thread."""
    try:
        import rq.executions as executions
    except ModuleNotFoundError:
        return  # rq 1.x records no executions

    class _Id:
        def __init__(self, text: str) -> None:
            self.hex = text

    def uuid4() -> _Id:
        actor = getattr(_tls, 'actor', 'main')
        n = getattr(_tls, 'exec_n', 0) + 1
        _tls.exec_n = n
        return _Id(f'x{n}{actor}')

    executions.uuid4 = uuid4  # type: ignore[assignment]


# ---------------------------------------------------------------- snapshots

TIME_FIELD = re.compile(r'(_at|heartbeat|birth|_time|time_|timestamp)$|^(birth|death)')
DATETIME_TEXT = re.compile(r'^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}')
EPOCH_TEXT = re.compile(r'^1\d{9}(\.\d+)?$')
STREAM_ID_TEXT = re.compile(r'^\d{13}-\d+$')
BASE64_BLOB_TEXT = re.compile(r'^[A-Za-z0-9+/]{40,}={0,2}$')


def normalize_value(value: Any, field_name: str | None = None) -> Any:
    if isinstance(value, bytes):
        try:
            value = value.decode()
        except UnicodeDecodeError:
            try:
                zlib.decompress(value)
                return '<blob>'
            except zlib.error:
                return '<bin>'
    if isinstance(value, str):
        if field_name is not None and TIME_FIELD.search(field_name):
            return '<T>' if value else ''
        if DATETIME_TEXT.match(value) or EPOCH_TEXT.match(value) or STREAM_ID_TEXT.match(value):
            return '<T>'
        if BASE64_BLOB_TEXT.match(value):
            return '<blob>'
        if 'Traceback' in value:
            return '<exc:' + value.strip().splitlines()[-1].split(':')[0] + '>'
        return value
    if isinstance(value, float):
        return '<T>' if value > 1e9 else value
    if isinstance(value, (list, tuple)):
        return [normalize_value(v) for v in value]
    if isinstance(value, dict):
        return {_text(k): normalize_value(v, _text(k)) for k, v in value.items()}
    return value


def snapshot(conn: redis.Redis) -> dict[str, Any]:
    """The whole keyspace as plain data, with clock values and blobs normalised."""
    out: dict[str, Any] = {}
    for raw in sorted(conn.keys('*')):
        key = _text(raw)
        kind = _text(conn.type(raw))
        value: Any
        if kind == 'string':
            value = normalize_value(conn.get(raw))
        elif kind == 'hash':
            value = {_text(f): normalize_value(v, _text(f)) for f, v in sorted(conn.hgetall(raw).items())}
        elif kind == 'list':
            value = [normalize_value(v) for v in conn.lrange(raw, 0, -1)]
        elif kind == 'set':
            value = sorted(normalize_value(v) for v in conn.smembers(raw))
        elif kind == 'zset':
            value = sorted(
                [normalize_value(m), 'inf' if s == float('inf') else 'T'] for m, s in conn.zrange(raw, 0, -1, withscores=True)
            )
        elif kind == 'stream':
            value = [normalize_value(fields) for _id, fields in conn.xrange(raw)]
        else:
            value = f'<{kind}>'
        out[key] = {'t': kind, 'v': value, 'ttl': conn.ttl(raw) >= 0}
    return out


def raw_snapshot(conn: redis.Redis) -> dict[str, tuple[bytes, int]]:
    """Every key as DUMP bytes plus its remaining TTL in ms (0 means none)."""
    out = {}
    for raw in conn.keys('*'):
        pttl = conn.pttl(raw)
        out[_text(raw)] = (conn.dump(raw), max(pttl, 0))
    return out


def restore(server: fakeredis.FakeServer, raw: dict[str, tuple[bytes, int]]) -> redis.Redis:
    conn = fakeredis.FakeRedis(server=server)
    for key, (dumped, pttl) in raw.items():
        conn.restore(key, pttl, dumped, replace=True)
    return conn


def digest(value: Any) -> str:
    return hashlib.sha1(json.dumps(value, sort_keys=True, default=str).encode()).hexdigest()[:16]


# ---------------------------------------------------------------- one run


@dataclass
class Op:
    name: str
    fn: Callable[[redis.Redis], Any]


@dataclass
class Step:
    actor: str
    label: str
    events: list[tuple[str, str, str, Any]]


@dataclass
class RunRecord:
    schedule: list[str] = field(default_factory=list)
    choices: list[tuple[list[str], str, bool]] = field(default_factory=list)
    states: list[dict[str, Any]] = field(default_factory=list)
    steps: list[Step] = field(default_factory=list)
    outputs: dict[str, list[tuple[str, Any]]] = field(default_factory=dict)
    initial_events: list[tuple[str, str, str, Any]] = field(default_factory=list)
    error: str | None = None


def job_started() -> None:
    """Split the running operation in two at the start of a job body: take, then complete."""
    ga = _current_gate()
    if ga is None:
        return
    gate, actor = ga
    gate.record(actor, 'end', _tls.op_id, ('ok', 'taken'))
    _tls.op_id = _tls.op_id.split(':')[0] + 'b:complete'
    gate.record(actor, 'begin', _tls.op_id)


def _actor_main(gate: Gate, actor: str, program: list[Op], conn: redis.Redis) -> None:
    _tls.gate = gate
    _tls.actor = actor
    _tls.exec_n = 0
    try:
        for index, op in enumerate(program):
            _tls.op_id = f'{actor}.{index}:{op.name}'
            gate.record(actor, 'begin', _tls.op_id)
            try:
                value = op.fn(conn)
                result: Any = ('ok', normalize_value(value))
            except GateTimeout:
                raise
            except Exception as exc:  # the op's outcome, kept as data
                result = ('exc', type(exc).__name__)
            gate.record(actor, 'end', _tls.op_id, result)
    finally:
        gate.finish(actor)


def execute(
    setup: Callable[[redis.Redis], None],
    actors: dict[str, list[Op]],
    prefix: list[str],
    effects: Callable[[], dict[str, int]],
    reset_effects: Callable[[], None],
    keep_raw: Callable[[str], bool],
) -> RunRecord:
    """Run once: follow prefix, then keep the current actor going while it can."""
    reset_effects()
    server = fakeredis.FakeServer()
    observer = fakeredis.FakeRedis(server=server)
    setup(fakeredis.FakeRedis(server=server))
    gate = Gate()
    record = RunRecord()
    threads: dict[str, threading.Thread] = {}
    for actor, program in actors.items():
        conn = fakeredis.FakeRedis(server=server)
        thread = threading.Thread(target=_actor_main, args=(gate, actor, program, conn), daemon=True, name=actor)
        threads[actor] = thread
        gate.start(actor, thread)

    def state(mid: dict[str, bool]) -> dict[str, Any]:
        snap = snapshot(observer)
        snap_id = digest(snap)
        local = {a: digest(gate.results.get(a, [])) for a in actors}
        node = {
            'snap_id': snap_id,
            'snap': snap,
            'effects': effects(),
            'mid': sorted(a for a, m in mid.items() if m),
            'done': sorted(gate.done),
            'local': local,
        }
        node['id'] = digest([snap_id, node['effects'], node['mid'], node['done'], local])
        if keep_raw(snap_id):
            node['raw'] = raw_snapshot(observer)
        return node

    mid = {a: False for a in actors}

    def apply_events(events: list[tuple[str, str, str, Any]]) -> None:
        for actor, kind, op_id, value in events:
            if kind == 'end':
                record.outputs.setdefault(actor, []).append((op_id, value))

    record.initial_events = list(gate.events)
    apply_events(gate.events)
    gate.events = []
    record.states.append(state(mid))
    last: str | None = None
    try:
        while True:
            enabled = sorted(gate.waiting)
            if not enabled:
                break
            position = len(record.schedule)
            if position < len(prefix):
                chosen = prefix[position]
            elif last in enabled:
                chosen = last
            else:
                chosen = enabled[0]
            preemptive = last is not None and last in enabled and chosen != last and mid[last]
            record.choices.append((enabled, chosen, preemptive))
            label = gate.waiting[chosen]
            gate.grant(chosen)
            events, gate.events = gate.events, []
            apply_events(events)
            # An actor is mid-operation once it has taken a step that did not end its operation.
            mid[chosen] = not any(a == chosen and kind == 'end' for a, kind, _, _ in events)
            record.schedule.append(chosen)
            record.steps.append(Step(chosen, label, events))
            record.states.append(state(mid))
            last = chosen
    except GateTimeout as exc:
        record.error = str(exc)
    for thread in threads.values():
        thread.join(timeout=1)
    return record
