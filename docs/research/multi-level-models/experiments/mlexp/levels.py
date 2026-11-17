"""Cut each explored run into levels by closing rules, with no author and no LLM.

Design B (the report's proposal), from the bottom:
- effect: one scheduler step, that is one Redis command, transaction or script.
- move: the on-chain effects of one activation, closed after each effect that
  writes to the store or is a rejected transaction; reads join the move they feed.
- activation: one actor's part of an API call, closed where the call returns or
  where it calls code outside the library (the job body).
- goal: one API call of one actor.
- outcome: per job, every goal that changes what the documentation lets a user
  observe about that job.
Candidate kinds that are tested and may not be admitted: exchange (a request
with its answer; in a one-store library this is the API call again) and stage
(a change of a job's status).

Design A (the specification's ladder) is read onto the same runs: business is a
status change, system a call across the library's boundary, services a round
trip to Redis, components the same plus calls between rq classes, statements
the single commands.

An effect is on the causal chain when it changes an observable, or writes a key
that a later on-chain effect reads, or is a read (or a rejected transaction) of
an actor that later, in the same API call, performs an on-chain effect. Every
other effect is a support step.
"""

from __future__ import annotations

import pathlib
import sys
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2] / 'abstraction-function' / 'experiments'))

from afexp import afs  # noqa: E402
from afexp.harness import digest  # noqa: E402
from afexp.harness import key_pattern as rq_key_pattern  # noqa: E402

import re  # noqa: E402

RJ_IDS = re.compile(r'\b(r\d+|k\d+)\b')


def key_pattern(key: str) -> str:
    """A key with its ids replaced by `*`: rq's jobs and workers, report-jobs' reports and keys."""
    return RJ_IDS.sub('*', rq_key_pattern(key))

READS = afs.READ_COMMANDS - {'EVALSHA'}
PURE_READS = READS - {'LMOVE', 'LPOP', 'BLMOVE'}


def effects_of(graph: dict[str, Any], run: dict[str, Any]) -> list[dict[str, Any]]:
    """One record per scheduler step of a run."""
    edges = graph.setdefault('_edge_index', {})
    if not edges:
        for e in graph['edges']:
            edges[(e['src'], e['dst'], e['actor'])] = e
    path = run['path']
    current = {a: op for a, kind, op, _ in run['initial_events'] if kind == 'begin'}
    out = []
    for i, actor in enumerate(run['schedule']):
        edge = edges[(path[i], path[i + 1], actor)]
        before, after = graph['nodes'][path[i]], graph['nodes'][path[i + 1]]
        snap0, snap1 = graph['snaps'][before['snap_id']], graph['snaps'][after['snap_id']]
        changed = sorted(k for k in set(snap0) | set(snap1) if snap0.get(k) != snap1.get(k))
        detail = edge['detail']
        reads = sorted({k for name, keys in detail['commands'] if name in READS for k in keys})
        # Writes come from the commands, not the snapshot diff: under the frozen clock a heartbeat rewrites the same value.
        writes = [] if detail['rejected'] else sorted({k for name, keys in detail['commands'] if name not in PURE_READS for k in keys})
        ref0, ref1 = afs.reference(snap0), afs.reference(snap1)
        op = current[actor]
        out.append({
            'i': i,
            'actor': actor,
            'call': op.split(':')[0].rstrip('b'),
            'api': 'work' if op.split(':')[1] in ('take', 'complete') else op.split(':')[1],
            'label': edge['label'],
            'commands': detail['commands'],
            'stack': detail['stack'],
            'rejected': detail['rejected'],
            'changed': changed,
            'writes': writes,
            'reads': reads,
            'ref_change': _ref_change(ref0, ref1),
            'body_ran': before['effects'] != after['effects'],
            'output': None,
            # Generic atomicity flags, used by move_close='atomic-group': WATCH opens an optimistic
            # transaction, a pipeline (EXEC) that writes or is rejected ends it.
            'tx_open': any(name == 'WATCH' for name, _ in detail['commands']),
            'tx_end': edge['label'].startswith('EXEC['),
        })
        for a, kind, op_id, value in run['events'][i]:
            if kind == 'begin':
                current[a] = op_id
            elif not (isinstance(value, (list, tuple)) and value[-1] == 'taken'):
                out[-1]['output'] = value
    return out


def _ref_change(ref0: Any, ref1: Any) -> list[Any]:
    """Per job: what a user can observe before and after, when it differs."""
    a = {row[0]: row[1:] for row in ref0}
    b = {row[0]: row[1:] for row in ref1}
    return [[j, a.get(j), b.get(j)] for j in sorted(set(a) | set(b)) if a.get(j) != b.get(j)]


def mark_chain(effects: list[dict[str, Any]]) -> None:
    needed: set[str] = set()
    later_chain: set[tuple[str, str]] = set()
    for e in reversed(effects):
        who = (e['actor'], e['call'])
        seed = bool(e['ref_change']) or e['body_ran']
        feeds = bool(set(e['writes']) & needed)
        local = not e['writes'] and who in later_chain
        e['chain'] = seed or feeds or local
        if e['chain']:
            needed.update(e['reads'])
            later_chain.add(who)


MOVE_CLOSE = ('every-write', 'atomic-group', 'write-run')
OUTCOMES = ('object-centric', 'partition')


def build(graph: dict[str, Any], run: dict[str, Any], move_close: str = 'every-write', outcomes: str = 'object-centric') -> dict[str, Any]:
    """The step tree of one run under design B, and the step lists of design A.

    The defaults are the rules of the first experiment. The options are those of
    docs/ladder-strategies.md 5.2: O1 `move_close` and O3 `outcomes`.
    """
    return build_effects(effects_of(graph, run), move_close, outcomes)


def build_effects(effects: list[dict[str, Any]], move_close: str = 'every-write', outcomes: str = 'object-centric') -> dict[str, Any]:
    """The step tree of one run, from its effects: rq's (effects_of) or report-jobs' (rjexp)."""
    assert move_close in MOVE_CLOSE and outcomes in OUTCOMES
    mark_chain(effects)
    activations: list[dict[str, Any]] = []
    goals: list[dict[str, Any]] = []
    open_act: dict[str, dict[str, Any]] = {}
    open_goal: dict[tuple[str, str], dict[str, Any]] = {}
    for e in effects:
        gkey = (e['actor'], e['call'])
        if gkey not in open_goal:
            open_goal[gkey] = {'actor': e['actor'], 'api': e['api'], 'effects': [], 'activations': [], 'output': None}
            goals.append(open_goal[gkey])
            open_act.pop(e['actor'], None)
        goal = open_goal[gkey]
        if e['actor'] not in open_act:
            act = {'actor': e['actor'], 'api': e['api'], 'part': len(goal['activations']), 'effects': [], 'goal': goal}
            open_act[e['actor']] = act
            goal['activations'].append(act)
            activations.append(act)
        act = open_act[e['actor']]
        act['effects'].append(e)
        goal['effects'].append(e)
        e['activation'], e['goal'] = act, goal
        if e['output'] is not None:
            goal['output'] = e['output']
        if e['body_ran'] or e['output'] is not None:
            del open_act[e['actor']]
    moves = []
    for act in activations:
        current: list[dict[str, Any]] = []
        act['moves'] = []
        watching = False
        on_chain = [e for e in act['effects'] if e['chain']]
        for n, e in enumerate(on_chain):
            current.append(e)
            if e['tx_open']:
                watching = True
            closes = bool(e['writes'] or e['rejected'])
            if move_close == 'atomic-group' and watching:
                # A transaction is one move: it ends at its commit attempt, accepted or rejected.
                # Writes made meanwhile outside it (an upload, a heartbeat) join it.
                closes = e['rejected'] or (bool(e['writes']) and e['tx_end'])
            if move_close == 'write-run' and closes and not e['rejected'] and n + 1 < len(on_chain):
                # Consecutive writes to the same key, with nothing read in between, are one move.
                nxt = on_chain[n + 1]
                if nxt['writes'] and not nxt['reads'] and not nxt['rejected'] and set(nxt['writes']) & set(e['writes']):
                    closes = False
            if closes:
                act['moves'].append({'effects': current, 'activation': act})
                current = []
                watching = False
        if current:
            # Reads left at the end of an activation join the move before them. Writes left there (a
            # transaction cut by the activation's end, under atomic-group) are a move of their own.
            if act['moves'] and not any(e['writes'] for e in current):
                act['moves'][-1]['effects'].extend(current)
            else:
                act['moves'].append({'effects': current, 'activation': act})
        moves.extend(act['moves'])
    for m in moves:
        last = [e for e in m['effects'] if e['writes'] or e['rejected']]
        closer = last[-1] if last else m['effects'][-1]
        written = sorted({key_pattern(k) for e in m['effects'] for k in e['writes']})
        # A move is identified by its call, the code that closes it, and what it writes. On rq the closing
        # function is enough; report-jobs closes several moves in one function, so its effects carry
        # the call site (function and line) as `anchor`.
        where = closer.get('anchor') or (closer['stack'][-1] if closer['stack'] else '?')
        m['class'] = [m['activation']['api'], where, 'rejected' if closer['rejected'] else written]
        m['ref_change'] = [c for e in m['effects'] for c in e['ref_change']]
    for act in activations:
        act['class'] = [act['api'], act['part']]
        act['ref_change'] = [c for e in act['effects'] for c in e['ref_change']]
    for goal in goals:
        goal['class'] = [goal['api']]
        goal['ref_change'] = [c for e in goal['effects'] for c in e['ref_change']]
    subjects = sorted({c[0] for g in goals for c in g['ref_change']})
    if outcomes == 'object-centric':
        # A goal is listed under every subject it changes.
        tops = [{'subject': j, 'goals': [g for g in goals if any(c[0] == j for c in g['ref_change'])]} for j in subjects]
    else:
        # Each goal belongs to one outcome: the subject it changes first.
        for g in goals:
            g['primary'] = next((c[0] for e in g['effects'] for c in e['ref_change']), None)
        tops = [{'subject': j, 'goals': [g for g in goals if g.get('primary') == j]} for j in subjects]
    return {'effects': effects, 'moves': moves, 'activations': activations, 'goals': goals, 'outcomes': tops, 'ladder_a': ladder_a(effects)}


def ladder_a(effects: list[dict[str, Any]]) -> dict[str, list[Any]]:
    """Step descriptors of the specification's levels, read onto an rq run."""
    statements, services, components, system, business = [], [], [], [], []
    last_stack: dict[str, list[str]] = {}
    seen_call: set[tuple[str, str]] = set()
    for e in effects:
        who = (e['actor'], e['call'])
        if who not in seen_call:
            seen_call.add(who)
            system.append([e['actor'], 'call ' + e['api']])
        previous = last_stack.get(e['actor'], [])
        common = 0
        while common < min(len(previous), len(e['stack'])) and previous[common] == e['stack'][common]:
            common += 1
        for depth in range(max(common, 1), len(e['stack'])):
            caller, callee = e['stack'][depth - 1], e['stack'][depth]
            if caller.split('.')[0] != callee.split('.')[0]:
                components.append([e['actor'], f'{caller} calls {callee}'])
        last_stack[e['actor']] = e['stack']
        services.append([e['actor'], e['label']])
        components.append([e['actor'], e['label']])
        for name, keys in e['commands']:
            statements.append([e['actor'], name + ' ' + ' '.join(key_pattern(k) for k in keys)])
        if e['body_ran']:
            system.append([e['actor'], 'run the job body'])
        if e['output'] is not None:
            system.append([e['actor'], f'{e["api"]} returns {e["output"]}'])
        for job, a, b in e['ref_change']:
            status0, status1 = (a or ['absent'])[0], (b or ['absent'])[0]
            if status0 != status1:
                business.append([e['actor'], f'job: {status0} to {status1}'])
    return {'business': business, 'system': system, 'services': services, 'components': components, 'statements': statements}


def descriptor(level: str, step: dict[str, Any], with_state: bool) -> str:
    """What a reader sees of one design-B step: its class, and optionally the change it made."""
    if level == 'effect':
        base: Any = [step['api'], step['label'], step['rejected']]
    else:
        base = step['class']
    if with_state:
        change = [[a, b] for _job, a, b in step['ref_change']]
        return digest([base, change, step.get('output')])
    return digest(base)
