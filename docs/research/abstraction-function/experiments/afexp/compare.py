"""Compare what each abstraction shows across code changes.

The visible set of an abstraction on a graph directory is every abstract
transition and final abstract state of every run (sequential and concurrent),
plus, for abstractions with an invariant, every quiescent abstract state that
breaks it. A change to the code is visible through an abstraction when the
visible set differs from the one on unchanged rq 2.12.
"""

from __future__ import annotations

import json
import pathlib
from collections.abc import Callable
from typing import Any

from afexp import afs, evaluate, mutants, own, scenarios

AF = Callable[[dict[str, Any]], Any]


def visible(af: AF, graph_dir: pathlib.Path, invariant: evaluate.Invariant | None) -> set[Any]:
    out: set[Any] = set()
    for bound in (0, 2):
        graphs = evaluate.load(graph_dir, bound)
        alpha = evaluate.node_alpha(af, graphs)
        out |= evaluate.abstract_transitions(alpha, graphs)
        if invariant is None:
            continue
        for name, graph in graphs.items():
            for node_id, node in graph['nodes'].items():
                if not node['mid']:
                    broken = invariant(graph['snaps'][node['snap_id']])
                    if broken:
                        out.add((name, 'broken', alpha[name][node_id], tuple(broken)))
    return out


def arms(base: pathlib.Path, target: pathlib.Path, llm_dir: pathlib.Path | None) -> dict[str, tuple[AF, evaluate.Invariant | None]]:
    """Every abstraction, derived on rq 2.12 (base) and evaluated on the target's snapshots."""
    seq_base = evaluate.load(base, 0)
    out: dict[str, tuple[AF, evaluate.Invariant | None]] = {name: (af, None) for name, af in afs.PURE.items()}
    out['reference+doc-invariant'] = (afs.reference, afs.doc_invariant_violations)
    out['read-projection'] = (afs.read_projection(evaluate.read_set(seq_base)), None)
    for stem in ('observer-semantic@2.12.0', 'behavioral-d2@2.12.0'):
        out[stem] = (evaluate.observer_af(target / f'{stem}.json'), None)
    for name, pair in evaluate.llm_afs(llm_dir).items():
        out[f'llm:{name}'] = (pair['af'], None)
        out[f'llm:{name}+inv'] = (pair['af'], pair['invariant'])
    documented = own.documented_names(pathlib.Path(next(iter(seq_base.values()))['rq_path']))
    train_vars = evaluate.own_variables(base, [seq_base], None)['base']
    target_graphs = [evaluate.load(target, 0), evaluate.load(target, 2)]
    target_vars = evaluate.own_variables(target, target_graphs, None)['base']
    for label, names in (('own', scenarios.TRAIN), ('own@all-seq', scenarios.ALL)):
        spec = own.derive(seq_base, train_vars, documented, names)
        af, invariant = own.instantiate(spec, target_vars)
        out[f'{label}-select-only'] = (af, None)
        out[label] = (af, invariant)
    return out


def run(base: pathlib.Path, mutant_root: pathlib.Path, versions: list[pathlib.Path], llm_dir: pathlib.Path | None, out: pathlib.Path) -> None:
    labels = {name: label for name, label, *_ in mutants.MUTANTS}
    base_visible = {name: visible(af, base, inv) for name, (af, inv) in arms(base, base, llm_dir).items()}
    rows: dict[str, dict[str, Any]] = {name: {'mutants': {}, 'versions': {}} for name in base_visible}
    for mutant in sorted(labels):
        target = mutant_root / mutant
        if not (target / f'{scenarios.ALL[0]}.b2.pkl').exists():
            print(f'skipping {mutant}: no graphs in {target}')
            continue
        for name, (af, inv) in arms(base, target, llm_dir).items():
            seen = visible(af, target, inv)
            diff = len(seen ^ base_visible[name]) / max(1, len(seen | base_visible[name]))
            rows[name]['mutants'][mutant] = {'label': labels[mutant], 'visible': seen != base_visible[name], 'distance': round(diff, 3)}
    for version in versions:
        for name, (af, inv) in arms(base, version, llm_dir).items():
            seen = visible(af, version, inv)
            union = seen | base_visible[name]
            rows[name]['versions'][version.name] = round(len(seen & base_visible[name]) / max(1, len(union)), 3)
    summary = []
    for name, row in rows.items():
        harmful = [m for m in row['mutants'].values() if m['label'] == 'harmful']
        benign = [m for m in row['mutants'].values() if m['label'] == 'benign']
        shown = sum(m['visible'] for m in harmful) / max(1, len(harmful))
        hidden = sum(not m['visible'] for m in benign) / max(1, len(benign))
        summary.append({
            'af': name,
            'harmful_shown': round(shown, 3),
            'benign_hidden': round(hidden, 3),
            'balanced': round((shown + hidden) / 2, 3),
            'versions': row['versions'],
        })
    out.write_text(json.dumps({'summary': summary, 'rows': rows}, indent=1, sort_keys=True))
    print('| AF | harmful shown | benign hidden | balanced | ' + ' | '.join(v.name for v in versions) + ' |')
    print('|' + '---|' * (4 + len(versions)))
    for s in summary:
        print(f"| {s['af']} | {s['harmful_shown']} | {s['benign_hidden']} | {s['balanced']} | " + ' | '.join(str(s['versions'].get(v.name)) for v in versions) + ' |')
