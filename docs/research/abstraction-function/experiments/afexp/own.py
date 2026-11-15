"""Our approach: select a small named abstraction, then mine its representation invariant.

1. Candidates. Named variables from three sources: the library's observers
   (author-written abstraction functions), the shape of the store (which key
   families hold each job), and the variables an LLM proposed.
2. Selection (counterexample-guided, like CEGAR on the upper model). Start
   from nothing and repeatedly add the candidate that removes the most
   nondeterminism from the upper model on sequential training runs: two
   operation runs that start in the same abstract state and do the same
   operation must end in the same abstract state with the same output. Ties go
   to the coarser variable, then to the source a person wrote. Then drop every
   variable whose removal keeps the model deterministic.
3. Representation invariant (Hoare 1972; Liskov and Guttag). Every candidate
   left out that the selected variables determine on quiescent training
   states, with few values, becomes a clause "y = f(selected)". Correct
   sequential code keeps redundant representations of one fact in step; races
   are where they part, so the clauses are checked at every quiescent state.

The derived abstraction is stored as data (variable names and clause tables),
so it can be applied unchanged to another version or a mutant of the code.
"""

from __future__ import annotations

import json
import math
import pathlib
import re
from collections import defaultdict
from collections.abc import Callable
from typing import Any

from afexp import afs, scenarios
from afexp.evaluate import Graphs, op_instances
from afexp.harness import digest, key_pattern

Variables = dict[str, Callable[[str, dict[str, Any]], Any]]
SOURCE_RANK = {'observer': 0, 'llm': 1, 'shape': 2}
MAX_CLAUSE_VALUES = 8


def family(var: str) -> str:
    return re.sub(r'\[[^\]]+\]', '[*]', var)


def observer_variables(path: pathlib.Path) -> Variables:
    data = json.loads(path.read_text())
    by_snap: dict[str, dict[str, Any]] = {}
    for values in data['values'].values():
        by_snap.update(values)
    families = sorted({family(v) for vals in by_snap.values() for v in vals})
    out: Variables = {}
    for fam in families:
        def get(snap_id: str, snap: dict[str, Any], fam: str = fam) -> Any:
            vals = by_snap[snap_id]
            return sorted([v, vals[v]] for v in vals if family(v) == fam)
        out[f'observer:{fam}'] = get
    return out


def shape_variable(fam: str) -> Callable[[str, dict[str, Any]], Any]:
    def get(snap_id: str, snap: dict[str, Any]) -> Any:
        return [
            [j, any(key_pattern(k) == fam and afs.holds(v, j) for k, v in snap.items() if v['t'] in afs.CONTAINERS)]
            for j in afs.mentioned_jobs(snap)
        ]

    return get


def shape_variables(snaps: list[dict[str, Any]]) -> Variables:
    families = sorted({key_pattern(k) for s in snaps for k, v in s.items() if v['t'] in afs.CONTAINERS})
    out: Variables = {f'shape:{fam}': shape_variable(fam) for fam in families}
    out['shape:job-hash-exists'] = lambda snap_id, snap: [[j, f'rq:job:{j}' in snap] for j in afs.mentioned_jobs(snap)]
    return out


def flatten(value: Any, path: str = '') -> list[tuple[str, str, Any]]:
    """Nested dicts become (family, instance, leaf value); job ids in a path become *."""
    if isinstance(value, dict) and value:
        out = []
        for k, v in value.items():
            out.extend(flatten(v, f'{path}.{k}' if path else str(k)))
        return out
    return [(re.sub(r'(^|\.)j\d+(?=\.|$)', r'\1*', path), path, value)]


def llm_variables(name: str, abstract: Callable[[dict[str, Any]], dict[str, Any]], snaps: list[dict[str, Any]]) -> Variables:
    """One atomic variable per flattened path family of the LLM's abstract state."""

    def leaves(snap: dict[str, Any]) -> list[tuple[str, str, Any]]:
        try:
            return flatten(abstract(snap))
        except Exception as exc:
            return [('error', 'error', f'exc:{type(exc).__name__}')]

    families = sorted({fam for snap in snaps for fam, _, _ in leaves(snap)})
    out: Variables = {}
    for fam in families:
        def get(snap_id: str, snap: dict[str, Any], fam: str = fam) -> Any:
            return sorted([inst, v] for f, inst, v in leaves(snap) if f == fam)
        out[f'llm:{name}:{fam}'] = get
    return out


class Table:
    """Values of every candidate on every training snapshot, computed once."""

    def __init__(self, variables: Variables, snaps: dict[str, dict[str, Any]]) -> None:
        self.names = sorted(variables)
        self.values = {n: {sid: digest(variables[n](sid, s)) for sid, s in snaps.items()} for n in self.names}
        self.domain = {n: len(set(v.values())) for n, v in self.values.items()}

    def key(self, chosen: list[str], snap_id: str) -> str:
        return digest([self.values[n][snap_id] for n in chosen])


def training_instances(seq: Graphs, names: list[str]) -> list[tuple[str, str, str, str, Any, Any]]:
    """(op, start snap, end snap, output, start effects, end effects) from sequential training runs."""
    out = []
    for name in names:
        graph = seq[name]
        for run in graph['runs']:
            path = run['path']
            for op, start, end, value in op_instances(run):
                a, b = graph['nodes'][path[start]], graph['nodes'][path[end]]
                out.append((op, a['snap_id'], b['snap_id'], digest(value), a['effects'], b['effects']))
    return out


Instances = list[tuple[str, str, str, str, Any, Any]]


def documented_names(rq_root: pathlib.Path) -> set[str]:
    """Method and attribute names that the library's user documentation mentions."""
    text = ' '.join(p.read_text() for p in sorted((rq_root / 'docs' / 'docs').glob('*.md')))
    return set(re.findall(r'\.([a-z_][a-z0-9_]*)\b', text)) | set(re.findall(r'\b([a-z_][a-z0-9_]*)\(', text))


def business_observables(table_names: list[str], documented: set[str] | None) -> list[str]:
    """Observer variables whose method the documentation names (all observers when documented is None),
    plus every LLM-proposed variable."""
    out = []
    for name in table_names:
        source = name.split(':')[0]
        if source == 'llm' or (source == 'observer' and (documented is None or name.split('.')[-1] in documented)):
            out.append(name)
    return out


def conflicts(table: Table, chosen: list[str], instances: Instances, observables: list[str], quiescent: list[str]) -> int:
    """Nondeterminism of the upper model plus observables the chosen variables fail to determine."""
    groups: dict[str, set[str]] = defaultdict(set)
    for op, start, end, output, e0, e1 in instances:
        groups[digest([table.key(chosen, start), e0, op])].add(digest([table.key(chosen, end), e1, output]))
    total = sum(len(v) - 1 for v in groups.values())
    keys = {sid: table.key(chosen, sid) for sid in quiescent}
    for name in observables:
        if name in chosen:
            continue
        seen: dict[str, set[str]] = defaultdict(set)
        for sid in quiescent:
            seen[keys[sid]].add(table.values[name][sid])
        total += sum(len(v) - 1 for v in seen.values())
    return total


def select(
    table: Table, instances: Instances, observables: list[str], quiescent: list[str], pool: list[str]
) -> tuple[list[str], list[dict[str, Any]]]:
    """Greedy cover: add the candidate with the most conflicts removed per bit of its domain."""
    chosen: list[str] = []
    trace = []
    current = conflicts(table, chosen, instances, observables, quiescent)
    while current > 0:
        best: tuple[float, int, int, str] | None = None
        for name in pool:
            if name in chosen:
                continue
            c = conflicts(table, chosen + [name], instances, observables, quiescent)
            if c >= current:
                continue
            efficiency = (current - c) / math.log2(1 + table.domain[name])
            rank = (-efficiency, c, SOURCE_RANK[name.split(':')[0]], name)
            if best is None or rank < best:
                best = (rank[0], c, rank[2], name)
        if best is None:
            break
        chosen.append(best[3])
        current = best[1]
        trace.append({'add': best[3], 'conflicts': current})
    for name in list(chosen):
        rest = [n for n in chosen if n != name]
        if conflicts(table, rest, instances, observables, quiescent) <= current:
            chosen = rest
            trace.append({'drop': name, 'conflicts': current})
    return chosen, trace


def mine_invariant(table: Table, chosen: list[str], quiescent: list[str], guarded: list[str]) -> list[dict[str, Any]]:
    clauses = []
    for name in guarded:
        if name in chosen or table.domain[name] > MAX_CLAUSE_VALUES:
            continue
        mapping: dict[str, str] = {}
        functional = True
        for sid in quiescent:
            k = table.key(chosen, sid)
            v = table.values[name][sid]
            if mapping.setdefault(k, v) != v:
                functional = False
                break
        if functional and len(set(table.values[name][s] for s in quiescent)) > 1:
            clauses.append({'var': name, 'map': mapping})
    return clauses


def derive(seq: Graphs, variables: Variables, documented: set[str] | None, names: list[str]) -> dict[str, Any]:
    """Derive from the sequential runs of the named scenarios; no anomaly label is ever read."""
    snaps = {sid: s for name in names for sid, s in seq[name]['snaps'].items()}
    table = Table(variables, snaps)
    instances = training_instances(seq, names)
    quiescent = sorted({n['snap_id'] for name in names for n in seq[name]['nodes'].values() if not n['mid']})
    observables = business_observables(table.names, documented)
    shapes = [n for n in table.names if n.startswith('shape:')]
    pool = table.names if documented is None else observables + shapes
    chosen, trace = select(table, instances, observables, quiescent, pool)
    guarded = observables + shapes
    clauses = mine_invariant(table, chosen, quiescent, guarded)
    return {
        'candidates': len(table.names),
        'observables': observables,
        'conflicts_before': conflicts(table, [], instances, observables, quiescent),
        'conflicts_after': conflicts(table, chosen, instances, observables, quiescent),
        'selected': chosen,
        'trace': trace,
        'clauses': clauses,
    }


def instantiate(spec: dict[str, Any], variables: Variables) -> tuple[Callable[[dict[str, Any]], Any], Callable[[dict[str, Any]], list[str]]]:
    """The abstraction function and the invariant check of a derived spec, on any snapshots.

    A variable the target never produces (a key family or an observer value that
    the changed code no longer writes) reads as absent rather than failing."""
    variables = dict(variables)
    for name in spec['selected'] + [c['var'] for c in spec['clauses']]:
        if name not in variables:
            variables[name] = shape_variable(name[len('shape:'):]) if name.startswith('shape:') else (lambda sid, snap: [])

    def af(snap: dict[str, Any]) -> Any:
        sid = digest(snap)
        return [variables[n](sid, snap) for n in spec['selected']]

    def violations(snap: dict[str, Any]) -> list[str]:
        sid = digest(snap)
        key = digest([digest(variables[n](sid, snap)) for n in spec['selected']])
        out = []
        for clause in spec['clauses']:
            expected = clause['map'].get(key)
            if expected is not None and digest(variables[clause['var']](sid, snap)) != expected:
                out.append(clause['var'])
        return out

    return af, violations
