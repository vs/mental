"""Score abstraction functions on explored state graphs.

Terms used below:
- U, the sequential reference: every abstract transition (a, op, a') seen in
  bound-0 runs, where each operation runs atomically. It plays the upper level.
- pair check: a concurrent run is flagged when one of its steps makes an
  abstract transition that is not in U (the projection check of spec 5.3).
- oracle: a concurrent run is a real anomaly when its final state, seen
  through the reference abstraction, is not the final state of any sequential
  run of the same scenario (final-state serialisability at the business level).
"""

from __future__ import annotations

import json
import pathlib
import pickle
from collections import defaultdict
from collections.abc import Callable, Iterator
from typing import Any

from afexp import afs, scenarios
from afexp.harness import digest

Graphs = dict[str, dict[str, Any]]


def load(directory: pathlib.Path, bound: int) -> Graphs:
    out = {}
    for name in scenarios.ALL:
        path = directory / f'{name}.b{bound}.pkl'
        if path.exists():
            with open(path, 'rb') as fh:
                out[name] = pickle.load(fh)
    return out


def steps(run: dict[str, Any]) -> Iterator[tuple[int, str, str, list[Any]]]:
    """(index, actor, operation name, events) for each step of a run."""
    current = {a: op for a, kind, op, _ in run['initial_events'] if kind == 'begin'}
    for index, events in enumerate(run['events']):
        actor = run['schedule'][index]
        op = current.get(actor, '?')
        for a, kind, op_id, _ in events:
            if kind == 'begin':
                current[a] = op_id
        yield index, actor, op.split(':')[-1], events


def op_instances(run: dict[str, Any]) -> Iterator[tuple[str, int, int, Any]]:
    """(operation name, index of the node before its first step, index after its last, output)."""
    current = {a: op for a, kind, op, _ in run['initial_events'] if kind == 'begin'}
    first: dict[str, int] = {}
    for index, events in enumerate(run['events']):
        op = current.get(run['schedule'][index])
        if op is not None and op not in first:
            first[op] = index
        for actor, kind, op_id, value in events:
            if kind == 'begin':
                current[actor] = op_id
            else:
                yield op_id.split(':')[-1], first.get(op_id, index), index + 1, value


def node_alpha(af: Callable[[dict[str, Any]], Any], graphs: Graphs) -> dict[str, dict[str, str]]:
    """Abstract value of every node, per scenario."""
    out: dict[str, dict[str, str]] = {}
    for name, graph in graphs.items():
        by_snap: dict[str, str] = {}
        values = {}
        for node_id, node in graph['nodes'].items():
            snap_id = node['snap_id']
            if snap_id not in by_snap:
                by_snap[snap_id] = digest(af(graph['snaps'][snap_id]))
            values[node_id] = digest([by_snap[snap_id], node['effects']])
        out[name] = values
    return out


def sequential_reference(alpha: dict[str, dict[str, str]], seq: Graphs) -> set[tuple[str, str, str]]:
    u = set()
    for name, graph in seq.items():
        a = alpha[name]
        for run in graph['runs']:
            path = run['path']
            for index, _actor, op, _events in steps(run):
                src, dst = a[path[index]], a[path[index + 1]]
                if src != dst:
                    u.add((src, op, dst))
    return u


def final_snap(graph: dict[str, Any], run: dict[str, Any]) -> dict[str, Any]:
    return graph['snaps'][graph['nodes'][run['path'][-1]]['snap_id']]


def oracle_labels(
    ref_seq: dict[str, dict[str, str]], ref_conc: dict[str, dict[str, str]], seq: Graphs, conc: Graphs
) -> tuple[dict[str, list[bool]], dict[str, Any]]:
    """A run is an anomaly when its final state breaks the documented invariant,
    or is not the final state of any sequential run (final-state serialisability)."""
    labels = {}
    counts: dict[str, Any] = {}
    for name, graph in conc.items():
        finals = {ref_seq[name][run['path'][-1]] for run in seq[name]['runs']}
        seq_broken = sum(1 for run in seq[name]['runs'] if afs.doc_invariant_violations(final_snap(seq[name], run)))
        unserialisable = [ref_conc[name][run['path'][-1]] not in finals for run in graph['runs']]
        broken = [bool(afs.doc_invariant_violations(final_snap(graph, run))) for run in graph['runs']]
        labels[name] = [u or b for u, b in zip(unserialisable, broken)]
        counts[name] = {
            'runs': len(graph['runs']),
            'unserialisable': sum(unserialisable),
            'invariant_broken': sum(broken),
            'anomalies': sum(labels[name]),
            'sequential_runs_breaking_invariant': f'{seq_broken}/{len(seq[name]["runs"])}',
        }
    return labels, counts


Invariant = Callable[[dict[str, Any]], list[str]]


def pair_check(
    alpha: dict[str, dict[str, str]], u: set[tuple[str, str, str]], conc: Graphs, invariant: Invariant | None = None
) -> tuple[dict[str, list[bool]], set[Any]]:
    """Flag runs with an abstract transition outside U, or, when an invariant is
    given, a quiescent state (no operation half done) that breaks it."""
    flags = {}
    novel: set[Any] = set()
    for name, graph in conc.items():
        a = alpha[name]
        broken: dict[str, list[str]] = {}
        out = []
        for run in graph['runs']:
            path = run['path']
            hit = False
            for index, _actor, op, _events in steps(run):
                t = (a[path[index]], op, a[path[index + 1]])
                if t[0] != t[2] and t not in u:
                    hit = True
                    novel.add(t)
            if invariant is not None:
                for node_id in path:
                    node = graph['nodes'][node_id]
                    if node['mid']:
                        continue
                    sid = node['snap_id']
                    if sid not in broken:
                        broken[sid] = invariant(graph['snaps'][sid])
                    if broken[sid]:
                        hit = True
                        novel.add(('invariant', a[node_id], tuple(broken[sid])))
            out.append(hit)
        flags[name] = out
    return flags, novel


def determinism(alpha: dict[str, dict[str, str]], seq: Graphs) -> dict[str, float]:
    outcomes: dict[tuple[str, str], set[str]] = defaultdict(set)
    outputs: dict[tuple[str, str], set[str]] = defaultdict(set)
    changes = []
    for name, graph in seq.items():
        a = alpha[name]
        for run in graph['runs']:
            path = run['path']
            for op, start, end, value in op_instances(run):
                key = (a[path[start]], op)
                outcomes[key].add(digest([a[path[end]], value]))
                outputs[key].add(digest(value))
                changes.append(sum(1 for i in range(start, end) if a[path[i]] != a[path[i + 1]]))
    keys = len(outcomes)
    return {
        'det_state': round(sum(1 for v in outcomes.values() if len(v) == 1) / keys, 3) if keys else 1.0,
        'det_output': round(sum(1 for v in outputs.values() if len(v) == 1) / keys, 3) if keys else 1.0,
        'changes_per_op': round(sum(changes) / len(changes), 2) if changes else 0.0,
    }


def prf(flags: dict[str, list[bool]], labels: dict[str, list[bool]], names: list[str]) -> dict[str, Any]:
    tp = fp = fn = tn = 0
    for name in names:
        for f, l in zip(flags[name], labels[name]):
            tp += f and l
            fp += f and not l
            fn += (not f) and l
            tn += (not f) and (not l)
    precision = tp / (tp + fp) if tp + fp else 1.0
    recall = tp / (tp + fn) if tp + fn else 1.0
    f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
    return {'tp': tp, 'fp': fp, 'fn': fn, 'tn': tn, 'precision': round(precision, 3), 'recall': round(recall, 3), 'f1': round(f1, 3)}


def agreement(alpha: dict[str, dict[str, str]], ref: dict[str, dict[str, str]], conc: Graphs) -> dict[str, float]:
    from sklearn.metrics import homogeneity_completeness_v_measure

    truth, pred = [], []
    for name, graph in conc.items():
        for node_id in graph['nodes']:
            truth.append(ref[name][node_id])
            pred.append(alpha[name][node_id])
    h, c, v = homogeneity_completeness_v_measure(truth, pred)
    return {'homogeneity': round(h, 3), 'completeness': round(c, 3), 'v_measure': round(v, 3)}


def abstract_transitions(alpha: dict[str, dict[str, str]], conc: Graphs) -> set[tuple[str, str, str, str]]:
    """Everything the upper level can see: abstract transitions and final states, per scenario."""
    out = set()
    for name, graph in conc.items():
        a = alpha[name]
        for run in graph['runs']:
            path = run['path']
            for index, _actor, op, _events in steps(run):
                if a[path[index]] != a[path[index + 1]]:
                    out.add((name, a[path[index]], op, a[path[index + 1]]))
            out.add((name, 'final', '', a[path[-1]]))
    return out


def score(
    name: str,
    af: Callable[[dict[str, Any]], Any],
    seq: Graphs,
    conc: Graphs,
    ref_alpha: dict[str, dict[str, str]],
    labels: dict[str, list[bool]],
    invariant: Invariant | None = None,
) -> dict[str, Any]:
    alpha_seq = node_alpha(af, seq)
    alpha_conc = node_alpha(af, conc)
    u = sequential_reference(alpha_seq, seq)
    flags, novel = pair_check(alpha_conc, u, conc, invariant)
    distinct = sum(len(set(v.values())) for v in alpha_conc.values())
    concrete = sum(len({digest([n['snap_id'], n['effects']]) for n in g['nodes'].values()}) for g in conc.values())
    result: dict[str, Any] = {
        'af': name,
        'abstract_states': distinct,
        'concrete_states': concrete,
        'findings': len(novel),
        'all': prf(flags, labels, list(conc)),
        'held_out': prf(flags, labels, [n for n in conc if n in scenarios.HELD_OUT]),
        'per_scenario': {n: prf(flags, labels, [n]) for n in conc},
    }
    result.update(determinism(alpha_seq, seq))
    result.update(agreement(alpha_conc, ref_alpha, conc))
    return result


def read_set(seq: Graphs) -> set[tuple[str, str]]:
    out: set[tuple[str, str]] = set()
    for name in scenarios.TRAIN:
        for edge in seq[name]['edges']:
            out.update(afs.reads_of(edge['label']))
    return out


def observer_af(path: pathlib.Path) -> Callable[[dict[str, Any]], Any]:
    data = json.loads(path.read_text())
    by_snap: dict[str, Any] = {}
    for values in data['values'].values():
        by_snap.update(values)

    def af(snap: dict[str, Any]) -> Any:
        return by_snap[digest(snap)]

    return af


def write_results(results: list[dict[str, Any]], out: pathlib.Path) -> None:
    out.write_text(json.dumps(results, indent=1, sort_keys=True))


def candidate_afs(graph_dir: pathlib.Path, seq: Graphs) -> dict[str, Callable[[dict[str, Any]], Any]]:
    out: dict[str, Callable[[dict[str, Any]], Any]] = dict(afs.PURE)
    out['read-projection'] = afs.read_projection(read_set(seq))
    for path in sorted(graph_dir.glob('*@*.json')):
        out[path.stem] = observer_af(path)
    return out


def table(results: list[dict[str, Any]]) -> str:
    head = '| AF | abs states | findings | P | R | F1 | held-out F1 | det | changes/op | homog | compl |'
    rows = [head, '|' + '---|' * 11]
    for r in results:
        rows.append(
            f"| {r['af']} | {r['abstract_states']} | {r['findings']} | {r['all']['precision']} | {r['all']['recall']} | "
            f"{r['all']['f1']} | {r['held_out']['f1']} | {r['det_state']} | {r['changes_per_op']} | {r['homogeneity']} | {r['completeness']} |"
        )
    return '\n'.join(rows)


OWN_OBSERVERS = 'observer-semantic@2.12.0.json'


def own_variables(graph_dir: pathlib.Path, graphs: list[Graphs], llm_dir: pathlib.Path | None) -> dict[str, Any]:
    from afexp import llm, own

    snaps = [s for g in graphs for graph in g.values() for s in graph['snaps'].values()]
    variables = dict(own.observer_variables(graph_dir / OWN_OBSERVERS))
    variables.update(own.shape_variables(snaps))
    with_llm = dict(variables)
    if llm_dir is not None:
        for path in sorted(llm_dir.glob('*.json')):
            with_llm.update(own.llm_variables(path.stem, llm.load_answer(path)['abstract'], snaps))
    return {'base': variables, 'llm': with_llm}


def llm_afs(llm_dir: pathlib.Path | None) -> dict[str, dict[str, Callable[[dict[str, Any]], Any]]]:
    from afexp import llm

    out = {}
    for path in sorted(llm_dir.glob('*.json')) if llm_dir else []:
        loaded = llm.load_answer(path)

        def af(snap: dict[str, Any], f: Callable[[dict[str, Any]], Any] = loaded['abstract']) -> Any:
            try:
                return f(snap)
            except Exception as exc:
                return f'exc:{type(exc).__name__}'

        def inv(snap: dict[str, Any], f: Callable[[dict[str, Any]], Any] = loaded['invariants']) -> list[str]:
            try:
                return sorted(k for k, ok in f(snap).items() if not ok)
            except Exception as exc:
                return [f'exc:{type(exc).__name__}']

        out[path.stem] = {'af': af, 'invariant': inv}
    return out


def houdini(invariant: Invariant, seq: Graphs, names: list[str]) -> Invariant:
    """Drop every clause that some quiescent state of a sequential run breaks (Houdini's filter)."""
    refuted: set[str] = set()
    for name in names:
        graph = seq[name]
        for node in graph['nodes'].values():
            if not node['mid']:
                refuted.update(invariant(graph['snaps'][node['snap_id']]))
    return lambda snap: [c for c in invariant(snap) if c not in refuted]


def both(first: Invariant, second: Invariant) -> Invariant:
    return lambda snap: first(snap) + second(snap)


def constant(snap: dict[str, Any]) -> Any:
    return None


def run_evaluation(
    graph_dir: pathlib.Path, out: pathlib.Path, spec_dir: pathlib.Path | None = None, llm_dir: pathlib.Path | None = None
) -> list[dict[str, Any]]:
    from afexp import own

    seq, conc = load(graph_dir, 0), load(graph_dir, 2)
    ref_seq, ref_conc = node_alpha(afs.reference, seq), node_alpha(afs.reference, conc)
    labels, counts = oracle_labels(ref_seq, ref_conc, seq, conc)
    candidates = candidate_afs(graph_dir, seq)
    results = [score(name, af, seq, conc, ref_conc, labels) for name, af in candidates.items()]
    results.append(score('reference+doc-invariant', afs.reference, seq, conc, ref_conc, labels, afs.doc_invariant_violations))
    for name, pair in llm_afs(llm_dir).items():
        results.append(score(f'llm:{name}', pair['af'], seq, conc, ref_conc, labels))
        results.append(score(f'llm:{name}+inv', pair['af'], seq, conc, ref_conc, labels, pair['invariant']))
        results.append(score(f'llm:{name}:inv-only', constant, seq, conc, ref_conc, labels, pair['invariant']))
    spec_root = spec_dir or graph_dir
    spec_seq = seq if spec_root == graph_dir else load(spec_root, 0)
    train_vars = own_variables(spec_root, [spec_seq], llm_dir)
    here_vars = own_variables(graph_dir, [seq, conc], llm_dir)
    documented = own.documented_names(pathlib.Path(next(iter(spec_seq.values()))['rq_path']))
    variants = [
        ('own', 'base', documented, scenarios.TRAIN),
        ('own@all-seq', 'base', documented, scenarios.ALL),
        ('own-all-observables', 'base', None, scenarios.TRAIN),
    ]
    if llm_dir:
        variants.append(('own+llm', 'llm', documented, scenarios.TRAIN))
    specs = {}
    for label, kind, docs, names in variants:
        spec = own.derive(spec_seq, train_vars[kind], docs, names)
        specs[label] = spec
        af, invariant = own.instantiate(spec, here_vars[kind])
        results.append(score(f'{label}-select-only', af, seq, conc, ref_conc, labels))
        results.append(score(label, af, seq, conc, ref_conc, labels, invariant))
        if label == 'own@all-seq':
            for name, pair in llm_afs(llm_dir).items():
                combined = both(invariant, pair['invariant'])
                results.append(score(f'{label}+intent:{name}', af, seq, conc, ref_conc, labels, combined))
                for filter_label, names in (('houdini-train', scenarios.TRAIN), ('houdini-all', scenarios.ALL)):
                    kept = houdini(pair['invariant'], seq, names)
                    results.append(score(f'{label}+intent-{filter_label}:{name}', af, seq, conc, ref_conc, labels, both(invariant, kept)))
    payload = {'graph_dir': str(graph_dir), 'oracle': counts, 'results': results, 'own_specs': specs}
    out.write_text(json.dumps(payload, indent=1, sort_keys=True))
    for name, c in counts.items():
        print(f'  {name}: {c}')
    print(table(results))
    return results
