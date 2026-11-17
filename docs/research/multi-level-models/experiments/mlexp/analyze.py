"""Measure the levels that closing rules produce on explored rq runs.

usage: python -I analyze.py <graph dir of rq 2.12> <out dir> [<graph dir of another version> ...]

E1: grain (lower steps per upper step), support share, nesting, candidate kinds.
E4: the highest level at which each anomalous run differs from every clean run,
    by what happened (structure) and by what a user could observe (with state).
E6: how many step classes survive in other rq versions.
"""

from __future__ import annotations

import json
import pathlib
import statistics
import sys
from collections import Counter, defaultdict
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from mlexp import levels  # noqa: E402

from afexp import afs, evaluate, scenarios  # noqa: E402
from afexp.harness import digest  # noqa: E402

B_LEVELS = ['outcome', 'goal', 'activation', 'move', 'effect']
A_LEVELS = ['business', 'system', 'services', 'components', 'statements']
BAND = (2, 7)


def summary(values: list[int]) -> dict[str, Any]:
    if not values:
        return {'n': 0}
    q = statistics.quantiles(values, n=4) if len(values) > 1 else [values[0]] * 3
    return {
        'n': len(values),
        'median': statistics.median(values),
        'mean': round(statistics.fmean(values), 2),
        'q1': q[0],
        'q3': q[2],
        'max': max(values),
        'in_band': round(sum(BAND[0] <= v <= BAND[1] for v in values) / len(values), 3),
        'ones': round(sum(v == 1 for v in values) / len(values), 3),
    }


def span(step: dict[str, Any], level: str) -> tuple[int, int]:
    """First and last effect index of a step, counted over its on-chain effects when it has any."""
    if level == 'effect':
        return step['i'], step['i']
    effects = step['effects']
    on_chain = [e['i'] for e in effects if e['chain']] or [e['i'] for e in effects]
    return on_chain[0], on_chain[-1]


def b_signature(tree: dict[str, Any], level: str, with_state: bool) -> str:
    """What happened at a level, in order, including how steps of different actors overlap."""
    if level == 'outcome':
        rows = []
        for o in tree['outcomes']:
            final = [c[2] for g in o['goals'] for c in g['ref_change'] if c[0] == o['subject']][-1:]
            rows.append([sorted(digest(g['class']) for g in o['goals']), final if with_state else None])
        return digest(sorted(rows, key=str))
    steps = tree[level + 's']
    if level == 'effect':
        steps = [s for s in steps if s['chain']]
    events = []
    for s in steps:
        actor = s['actor'] if 'actor' in s else s['activation']['actor']
        first, last = span(s, level)
        d = levels.descriptor(level, s, with_state)
        events.append((first, 0, actor, d))
        if level != 'effect':
            events.append((last, 1, actor, d))
    return digest([(kind, actor, d) for _, kind, actor, d in sorted(events)])


def a_signature(tree: dict[str, Any], level: str) -> str:
    return digest(tree['ladder_a'][level])


def highest(signatures: dict[str, str], clean: dict[str, set[str]], order: list[str]) -> str:
    for level in order:
        if signatures[level] not in clean[level]:
            return level
    return 'none'


def cut_lines(tree: dict[str, Any], clean_goals: set[str], clean_acts: set[str]) -> int:
    """Lines of the smallest cut that opens only what differs from clean runs, down to moves."""
    lines = 0
    for goal in tree['goals']:
        if goal_sig(goal) in clean_goals:
            lines += 1
            continue
        for act in goal['activations']:
            lines += 1 if act_sig(act) in clean_acts else len(act['moves'])
    return lines


def act_sig(act: dict[str, Any]) -> str:
    return digest([act['class'], [m['class'] for m in act['moves']]])


def goal_sig(goal: dict[str, Any]) -> str:
    return digest([goal['class'], [act_sig(a) for a in goal['activations']]])


def load_trees(graph_dir: pathlib.Path, **options: str) -> tuple[dict[str, list[dict[str, Any]]], dict[str, list[bool]]]:
    """Step trees and anomaly labels per scenario."""
    rj = graph_dir / 'rj-runs.pkl'
    if rj.exists():
        import pickle

        data = pickle.loads(rj.read_bytes())
        return ({name: [levels.build_effects(run['effects'], **options) for run in g['runs']] for name, g in data.items()},
                {name: g['labels'] for name, g in data.items()})
    seq, conc = evaluate.load(graph_dir, 0), evaluate.load(graph_dir, 2)
    ref_seq, ref_conc = evaluate.node_alpha(afs.reference, seq), evaluate.node_alpha(afs.reference, conc)
    labels, _ = evaluate.oracle_labels(ref_seq, ref_conc, seq, conc)
    return {name: [levels.build(conc[name], run, **options) for run in conc[name]['runs']] for name in scenarios.ALL}, labels


def analyse(graph_dir: pathlib.Path, **options: str) -> dict[str, Any]:
    """`options` are passed to levels.build (the strategy options O1 and O3).

    `graph_dir` holds either rq's explored graphs or report-jobs' runs (`rj-runs.pkl`, from rjexp/harness.py).
    """
    trees, labels = load_trees(graph_dir, **options)
    fan: dict[str, list[int]] = defaultdict(list)
    counts_b: dict[str, list[int]] = defaultdict(list)
    counts_a: dict[str, list[int]] = defaultdict(list)
    support = chain = 0
    shared_goals = total_goals = 0
    classes: dict[str, set[str]] = defaultdict(set)
    class_examples: dict[str, dict[str, Any]] = defaultdict(dict)
    placement: dict[str, dict[str, Counter[str]]] = {}
    cut: dict[str, list[int]] = defaultdict(list)
    for name, built in trees.items():
        clean_b = {(lv, st): set() for lv in B_LEVELS for st in (False, True)}
        clean_a: dict[str, set[str]] = {lv: set() for lv in A_LEVELS}
        clean_goals: set[str] = set()
        clean_acts: set[str] = set()
        sigs = []
        for tree, anomalous in zip(built, labels[name]):
            sb = {(lv, st): b_signature(tree, lv, st) for lv in B_LEVELS for st in (False, True)}
            sa = {lv: a_signature(tree, lv) for lv in A_LEVELS}
            sigs.append((sb, sa))
            if not anomalous:
                for key, value in sb.items():
                    clean_b[key].add(value)
                for lv, value in sa.items():
                    clean_a[lv].add(value)
                clean_goals.update(goal_sig(g) for g in tree['goals'])
                clean_acts.update(act_sig(a) for g in tree['goals'] for a in g['activations'])
        place = {'b_structure': Counter(), 'b_with_state': Counter(), 'a': Counter()}
        for tree, anomalous, (sb, sa) in zip(built, labels[name], sigs):
            for m in tree['moves']:
                fan['effect per move'].append(len(m['effects']))
                classes['move'].add(digest(m['class']))
                class_examples['move'].setdefault(digest(m['class']), {'class': m['class'], 'effects': [e['label'] for e in m['effects']], 'stack': m['effects'][-1]['stack'][-3:]})
            for a in tree['activations']:
                fan['move per activation'].append(len(a['moves']))
                fan['status change per activation'].append(sum(1 for c in a['ref_change'] if (c[1] or [None])[0] != (c[2] or [None])[0]))
                classes['activation'].add(digest(a['class']))
            for g in tree['goals']:
                fan['activation per goal'].append(len(g['activations']))
                fan['status change per goal'].append(sum(1 for c in g['ref_change'] if (c[1] or [None])[0] != (c[2] or [None])[0]))
                classes['goal'].add(digest(g['class']))
                owners = sum(1 for o in tree['outcomes'] if g in o['goals'])
                total_goals += 1
                shared_goals += owners > 1
            for o in tree['outcomes']:
                fan['goal per outcome'].append(len(o['goals']))
            chain += sum(e['chain'] for e in tree['effects'])
            support += sum(not e['chain'] for e in tree['effects'])
            counts_b['outcome'].append(len(tree['outcomes']))
            counts_b['goal'].append(len(tree['goals']))
            counts_b['activation'].append(len(tree['activations']))
            counts_b['move'].append(len(tree['moves']))
            counts_b['effect (on chain)'].append(sum(e['chain'] for e in tree['effects']))
            counts_b['effect (all)'].append(len(tree['effects']))
            for lv in A_LEVELS:
                counts_a[lv].append(len(tree['ladder_a'][lv]))
            if anomalous:
                place['b_structure'][highest({lv: sb[(lv, False)] for lv in B_LEVELS}, {lv: clean_b[(lv, False)] for lv in B_LEVELS}, B_LEVELS)] += 1
                place['b_with_state'][highest({lv: sb[(lv, True)] for lv in B_LEVELS}, {lv: clean_b[(lv, True)] for lv in B_LEVELS}, B_LEVELS)] += 1
                place['a'][highest(sa, clean_a, A_LEVELS)] += 1
                cut[name].append(cut_lines(tree, clean_goals, clean_acts))
        placement[name] = {k: dict(v) for k, v in place.items()}
    return {
        'runs': sum(len(v) for v in trees.values()),
        'fan_in': {k: summary(v) for k, v in fan.items()},
        'steps_per_run_b': {k: round(statistics.fmean(v), 2) for k, v in counts_b.items()},
        'steps_per_run_a': {k: round(statistics.fmean(v), 2) for k, v in counts_a.items()},
        'support_share': round(support / (support + chain), 3),
        'goals_in_two_outcomes': round(shared_goals / total_goals, 3),
        'placement': placement,
        'cut_lines': {k: summary(v) for k, v in cut.items() if v},
        'classes': {k: sorted(v) for k, v in classes.items()},
        'move_classes': class_examples['move'],
        '_trees': trees,
        '_fan': fan,
        '_labels': labels,
    }


def witness(result: dict[str, Any]) -> dict[str, Any]:
    """The shortest dependency run that ends with the finished job still listed as started."""
    best = None
    name = 'dependency' if 'dependency' in result['_trees'] else 'requeue-vs-work'  # rq, or report-jobs
    for tree, anomalous in zip(result['_trees'][name], result['_labels'][name]):
        if anomalous and (best is None or len(tree['effects']) < len(best['effects'])):
            best = tree
    assert best is not None
    lines = []
    for goal in best['goals']:
        lines.append(f"GOAL {goal['actor']} {goal['api']} ({len(goal['effects'])} effects, {sum(not e['chain'] for e in goal['effects'])} support)")
        for act in goal['activations']:
            lines.append(f"  ACTIVATION part {act['part']} ({len(act['effects'])} effects)")
            for m in act['moves']:
                steps = ','.join(str(e['i']) for e in m['effects'])
                lines.append(f"    MOVE [{steps}] {m['class'][1]} -> {m['class'][2]}")
    return {
        'effects': len(best['effects']),
        'support': sum(not e['chain'] for e in best['effects']),
        'support_steps': [e['i'] for e in best['effects'] if not e['chain']],
        'moves': len(best['moves']),
        'activations': len(best['activations']),
        'goals': len(best['goals']),
        'outcomes': len(best['outcomes']),
        'tree': lines,
        'listing': [[e['i'], e['actor'], e['commands'], e['stack'][-2:], e['rejected'], e['chain']] for e in best['effects']],
    }


def fn(result: dict[str, Any]) -> set[str]:
    """Move classes identified by API call and closing function only."""
    return {str(v['class'][:2]) for v in result['move_classes'].values()}


def main() -> None:
    base = pathlib.Path(sys.argv[1])
    out = pathlib.Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    result = analyse(base)
    result['witness'] = witness(result)
    stability = {}
    for other in sys.argv[3:]:
        r = analyse(pathlib.Path(other))
        stability[pathlib.Path(other).name] = {
            kind: round(len(set(result['classes'][kind]) & set(r['classes'][kind])) / len(set(result['classes'][kind]) | set(r['classes'][kind])), 3)
            for kind in ('goal', 'activation', 'move')
        } | {'move_classes': len(r['classes']['move']), 'move_by_function': round(len(fn(result) & fn(r)) / len(fn(result) | fn(r)), 3)}
    result['class_stability'] = stability
    public = {k: v for k, v in result.items() if not k.startswith('_')}
    (out / 'levels.json').write_text(json.dumps(public, indent=1, sort_keys=True, default=str))
    for key in ('runs', 'fan_in', 'steps_per_run_b', 'steps_per_run_a', 'support_share', 'goals_in_two_outcomes', 'placement', 'cut_lines', 'class_stability'):
        print(key, json.dumps(public[key], default=str))
    print('classes', {k: len(v) for k, v in public['classes'].items()})
    w = public['witness']
    print('witness', {k: w[k] for k in ('effects', 'support', 'moves', 'activations', 'goals', 'outcomes', 'support_steps')})
    print('\n'.join(w['tree']))


if __name__ == '__main__':
    main()
