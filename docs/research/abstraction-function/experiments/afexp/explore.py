"""Enumerate schedules of a scenario and merge every run into one state graph.

Schedules are enumerated by re-execution, as stateless model checkers do: run
once, then for every choice point branch to each other enabled actor, as long
as the number of preemptions (switching away from an actor in the middle of an
operation) stays within the bound. Switching at operation boundaries is free,
so bound 0 gives exactly the runs in which every operation is atomic.
"""

from __future__ import annotations

import time
from typing import Any

from afexp import harness

MAX_RUNS = 40000


def explore(name: str, scenario: Any, bound: int) -> dict[str, Any]:
    import afjobs

    setup, actors = scenario
    raw_kept: set[str] = set()

    def keep_raw(snap_id: str) -> bool:
        if snap_id in raw_kept:
            return False
        raw_kept.add(snap_id)
        return True

    graph: dict[str, Any] = {
        'scenario': name,
        'bound': bound,
        'actors': {a: [op.name for op in program] for a, program in actors.items()},
        'nodes': {},
        'snaps': {},
        'raw': {},
        'edges': {},
        'runs': [],
    }
    stack: list[list[str]] = [[]]
    started = time.time()
    truncated = False
    while stack:
        if len(graph['runs']) >= MAX_RUNS:
            truncated = True
            break
        prefix = stack.pop()
        record = harness.execute(
            setup, actors, prefix, lambda: dict(sorted(afjobs.EFFECTS.items())), afjobs.EFFECTS.clear, keep_raw
        )
        preemptions = 0
        for index, (enabled, chosen, preemptive) in enumerate(record.choices):
            if index >= len(prefix):
                last = record.schedule[index - 1] if index > 0 else None
                last_mid = last is not None and last in record.states[index]['mid']
                for alternative in enabled:
                    if alternative == chosen:
                        continue
                    extra = 1 if (last is not None and last in enabled and alternative != last and last_mid) else 0
                    if preemptions + extra <= bound:
                        stack.append(record.schedule[:index] + [alternative])
            preemptions += 1 if preemptive else 0
        _merge(graph, record, preemptions)
    graph['meta'] = {
        'runs': len(graph['runs']),
        'nodes': len(graph['nodes']),
        'snaps': len(graph['snaps']),
        'edges': len(graph['edges']),
        'seconds': round(time.time() - started, 1),
        'truncated': truncated,
    }
    graph['edges'] = list(graph['edges'].values())
    return graph


def _merge(graph: dict[str, Any], record: harness.RunRecord, preemptions: int) -> None:
    path = []
    for state in record.states:
        node_id = state['id']
        path.append(node_id)
        if node_id not in graph['nodes']:
            graph['nodes'][node_id] = {k: state[k] for k in ('snap_id', 'effects', 'mid', 'done', 'local')}
        graph['snaps'].setdefault(state['snap_id'], state['snap'])
        if 'raw' in state:
            graph['raw'].setdefault(state['snap_id'], state['raw'])
    current_op: dict[str, str] = {actor: op_id for actor, kind, op_id, _ in record.initial_events if kind == 'begin'}
    for index, step in enumerate(record.steps):
        op = current_op.get(step.actor)
        ended = []
        for actor, kind, op_id, value in step.events:
            if kind == 'begin':
                current_op[actor] = op_id
            else:
                ended.append((op_id, value))
        src, dst = path[index], path[index + 1]
        key = (src, dst, step.actor, step.label)
        if key not in graph['edges']:
            graph['edges'][key] = {
                'src': src,
                'dst': dst,
                'actor': step.actor,
                'label': step.label,
                'op': op,
                'ended': ended,
                'detail': step.detail,
            }
    graph['runs'].append(
        {
            'path': path,
            'schedule': record.schedule,
            'preemptions': preemptions,
            'outputs': record.outputs,
            'initial_events': record.initial_events,
            'events': [step.events for step in record.steps],
            'error': record.error,
        }
    )
