"""Observer abstraction: the library's own query methods, found automatically.

For each entity class (Job, Queue, every registry) take every public property
and every public method callable without arguments, call it on objects bound to
a restored snapshot, and keep only the observers that never issue a write and
never change an outside effect. Their abstracted return values are the
abstract state. This follows ADABU (Dallmeier et al.), which abstracts object
state through inspector methods.
"""

from __future__ import annotations

import datetime
import enum
import inspect
import json
import pathlib
import pickle
import threading
from typing import Any

import fakeredis
import redis.client

from afexp import afs, harness

MUTATING_VERBS = (
    'add', 'cancel', 'cleanup', 'delete', 'dequeue', 'empty', 'enqueue', 'execute', 'expire', 'heartbeat',
    'kill', 'move', 'perform', 'persist', 'pop', 'refresh', 'register', 'remove', 'requeue', 'restore',
    'retry', 'run', 'save', 'schedule', 'send', 'set', 'start', 'stop', 'unregister', 'compact', 'prepare',
    'handle', 'create', 'clean', 'lpop', 'lmove', 'blpop',
)
READS = afs.READ_COMMANDS - {'LMOVE', 'LPOP', 'BLMOVE', 'EVALSHA'} | {'UNWATCH', 'MULTI', 'PTTL', 'TTL', 'SCAN'}

_capture = threading.local()


def _install_capture() -> None:
    inner_execute = redis.client.Redis.execute_command
    inner_pipeline = redis.client.Pipeline.execute
    inner_immediate = redis.client.Pipeline.immediate_execute_command

    def execute_command(self: Any, *args: Any, **options: Any) -> Any:
        log = getattr(_capture, 'log', None)
        if log is not None:
            log.append(harness._text(args[0]).upper())
        return inner_execute(self, *args, **options)

    def immediate(self: Any, *args: Any, **options: Any) -> Any:
        log = getattr(_capture, 'log', None)
        if log is not None:
            log.append(harness._text(args[0]).upper())
        return inner_immediate(self, *args, **options)

    def pipeline_execute(self: Any, raise_on_error: bool = True) -> Any:
        log = getattr(_capture, 'log', None)
        if log is not None:
            for entry in self.command_stack:
                log.append(harness._text(entry[0][0]).upper())
        return inner_pipeline(self, raise_on_error)

    redis.client.Redis.execute_command = execute_command  # type: ignore[method-assign]
    redis.client.Pipeline.immediate_execute_command = immediate  # type: ignore[method-assign]
    redis.client.Pipeline.execute = pipeline_execute  # type: ignore[method-assign]


def abstract(value: Any, depth: int = 0) -> Any:
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, enum.Enum):
        return abstract(value.value, depth)
    if isinstance(value, int):
        return min(value, 3)
    if isinstance(value, float):
        return '<num>'
    if isinstance(value, (datetime.datetime, datetime.timedelta, datetime.date)):
        return '<T>'
    if isinstance(value, bytes):
        return harness.normalize_value(value)
    if isinstance(value, str):
        text = harness.normalize_value(value)
        return text if len(text) <= 80 else '<long>'
    if depth > 2:
        return type(value).__name__
    if isinstance(value, (list, tuple, set, frozenset)):
        items = [abstract(v, depth + 1) for v in value]
        if len(items) > 8:
            return ['<len>', min(len(items), 3)]
        return sorted(items, key=lambda x: json.dumps(x, sort_keys=True, default=str))
    if isinstance(value, dict):
        return {str(k): abstract(v, depth + 1) for k, v in sorted(value.items(), key=lambda kv: str(kv[0]))}
    entity_id = getattr(value, 'id', None)
    if isinstance(entity_id, str):
        return [type(value).__name__, harness.normalize_value(entity_id)]
    return type(value).__name__


def candidate_observers(cls: type) -> list[tuple[str, str]]:
    out = []
    for name, attr in inspect.getmembers(cls):
        if name.startswith('_'):
            continue
        if isinstance(attr, property):
            out.append((name, 'prop'))
            continue
        if not callable(attr) or inspect.isclass(attr):
            continue
        if any(name == verb or name.startswith(verb + '_') for verb in MUTATING_VERBS):
            continue
        if isinstance(inspect.getattr_static(cls, name), (staticmethod, classmethod)):
            continue
        try:
            signature = inspect.signature(attr)
        except (TypeError, ValueError):
            continue
        params = [p for p in list(signature.parameters.values())[1:] if p.kind not in (p.VAR_POSITIONAL, p.VAR_KEYWORD)]
        if all(p.default is not p.empty for p in params):
            out.append((name, 'method'))
    return out


def entity_classes() -> dict[str, Any]:
    import rq.registry as registry
    from rq import Queue
    from rq.job import Job

    classes: dict[str, Any] = {'Job': Job, 'Queue': Queue}
    for name, cls in inspect.getmembers(registry, inspect.isclass):
        if issubclass(cls, registry.BaseRegistry) and cls is not registry.BaseRegistry and cls.__module__ == registry.__name__:
            classes[name] = cls
    return classes


class Observer:
    def __init__(self, purity: str) -> None:
        _install_capture()
        self.purity = purity
        self.housekeeping: set[str] = set()
        self.classes = entity_classes()
        self.candidates = {name: candidate_observers(cls) for name, cls in self.classes.items()}
        self.impure: set[str] = set()
        self.calls = 0

    def _bind(self, cls_name: str, entity: str, conn: Any) -> Any:
        cls = self.classes[cls_name]
        if cls_name == 'Job':
            return cls.fetch(entity, connection=conn)
        return cls('default', connection=conn)

    def observe(self, raw: dict[str, Any], snap: dict[str, Any]) -> dict[str, Any]:
        import afjobs

        values: dict[str, Any] = {}
        server = fakeredis.FakeServer()
        conn = harness.restore(server, raw)
        base = harness.digest(harness.snapshot(conn))
        targets = [('Job', j) for j in afs.mentioned_jobs(snap)] + [(c, 'default') for c in self.classes if c != 'Job']
        for cls_name, entity in targets:
            prefix = f'{cls_name}[{entity}]' if cls_name == 'Job' else cls_name
            try:
                obj = self._bind(cls_name, entity, conn)
            except Exception as exc:
                values[f'{prefix}'] = f'exc:{type(exc).__name__}'
                continue
            for name, kind in self.candidates[cls_name]:
                var = f'{cls_name}.{name}'
                if var in self.impure:
                    continue
                before = dict(afjobs.EFFECTS)
                _capture.log = []
                try:
                    value = getattr(obj, name) if kind == 'prop' else getattr(obj, name)()
                    result = abstract(value)
                except Exception as exc:
                    result = f'exc:{type(exc).__name__}'
                finally:
                    log, _capture.log = _capture.log, None
                self.calls += 1
                wrote = any(cmd not in READS for cmd in log)
                if wrote and self.purity == 'semantic' and dict(afjobs.EFFECTS) == before:
                    if harness.digest(harness.snapshot(conn)) == base:
                        self.housekeeping.add(var)
                        wrote = False
                if wrote or dict(afjobs.EFFECTS) != before:
                    self.impure.add(var)
                    afjobs.EFFECTS.clear()
                    afjobs.EFFECTS.update(before)
                    server = fakeredis.FakeServer()
                    conn = harness.restore(server, raw)
                    obj = self._bind(cls_name, entity, conn)
                    continue
                values[f'{prefix}.{name}'] = result
        return values


def observe_dir(graphs: pathlib.Path, purity: str) -> None:
    import rq

    version = str(getattr(rq, '__version__', '?'))
    observer = Observer(purity)
    per_graph: dict[str, dict[str, Any]] = {}
    files = sorted(graphs.glob('*.pkl'))
    for path in files:
        with open(path, 'rb') as fh:
            graph = pickle.load(fh)
        done = per_graph.setdefault(graph['scenario'], {})
        for snap_id, raw in graph['raw'].items():
            if snap_id not in done:
                done[snap_id] = observer.observe(raw, graph['snaps'][snap_id])
    kept: set[str] = set()
    for values in per_graph.values():
        for snap_id, vals in values.items():
            for var in [v for v in vals if v.split('.')[0].split('[')[0] + '.' + v.split('.')[-1] in observer.impure]:
                del vals[var]
            kept.update(vals)
    out = {
        'observer_rq': version,
        'candidates': {c: [n for n, _ in obs] for c, obs in observer.candidates.items()},
        'impure': sorted(observer.impure),
        'housekeeping': sorted(observer.housekeeping - observer.impure),
        'purity': purity,
        'variables': sorted(kept),
        'calls': observer.calls,
        'values': per_graph,
    }
    target = graphs / f'observer-{purity}@{version}.json'
    target.write_text(json.dumps(out, sort_keys=True, default=str))
    print(f'observer-{purity}@{version}: {len(kept)} variables, {len(observer.impure)} impure, {observer.calls} calls -> {target}')
