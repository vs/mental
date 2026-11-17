"""Experiments L2 to L5 of docs/ladder-strategies.md on rq.

L2  where a move ends: every-write, atomic-group, write-run, and the `prefer` choice among them
L3  naming from the head: the naming sheet with each step's head effect, and a name-truth check
L4  overlapping outcomes: object-centric against partition
L5  which merge test: band, single-child, span-overlap, against the levels that name an anomaly

usage:
  python -I strategies.py structure <graphs of rq 2.12> <results dir> [<graphs of another version> ...]
  python -I strategies.py name-prompt <graphs of rq 2.12> <rq 2.12 checkout> <out dir>
  python -I strategies.py name-score <out dir> <baseline answers dir> <results dir>
  python -I strategies.py reader-prompts <graphs of rq 2.12> <out dir>
  python -I strategies.py reader-score <out dir> <results dir>

Every rule and threshold below was written before the numbers it decides were computed.
"""

from __future__ import annotations

import itertools
import json
import pathlib
import re
import statistics
import sys
from collections import Counter, defaultdict
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from mlexp import analyze, levels, llm_tasks, reader  # noqa: E402

from afexp.harness import digest, key_pattern  # noqa: E402

# ---------------------------------------------------------------------------
# Thresholds fixed in advance (docs/ladder-strategies.md 5.2, O4).
BAND = (2, 7)            # band: merge when the median fan-in is below 2; look for a level in between above 7
SINGLE_CHILD = 0.8       # single-child: merge when at least this share of upper steps has exactly one lower step
SPAN_OVERLAP = 0.5       # span-overlap: merge when the step-length distributions of two adjacent levels overlap this much
PREFER_TIE = 0.02        # prefer: stability differences below this are ties, broken by purity


def sumstats(values: list[float]) -> dict[str, Any]:
    if not values:
        return {'n': 0}
    return {'n': len(values), 'mean': round(statistics.fmean(values), 3), 'median': statistics.median(values),
            'ones': round(sum(v == 1 for v in values) / len(values), 3), 'max': max(values)}


def total_placement(result: dict[str, Any], kind: str) -> dict[str, int]:
    out: Counter[str] = Counter()
    for per in result['placement'].values():
        out.update(per[kind])
    return dict(out)


def change_kind(changes: list[Any]) -> str:
    """An observable change with the job ids dropped: what kind of change a step makes."""
    return digest(sorted([[(a or ['absent'])[0], (b or ['absent'])[0]] for _j, a, b in changes]))


def purity(result: dict[str, Any]) -> dict[str, float]:
    """How well a move's class predicts the observable change it makes (a proxy for M7)."""
    kinds: dict[str, Counter[str]] = defaultdict(Counter)
    for trees in result['_trees'].values():
        for tree in trees:
            for m in tree['moves']:
                kinds[digest(m['class'])][change_kind(m['ref_change'])] += 1
    single = sum(1 for c in kinds.values() if len(c) == 1) / len(kinds)
    inst = sum(c.most_common(1)[0][1] for c in kinds.values()) / sum(sum(c.values()) for c in kinds.values())
    return {'classes_with_one_change_kind': round(single, 3), 'instances_matching_class_majority': round(inst, 3)}


def stability(base: dict[str, Any], other: dict[str, Any]) -> dict[str, float]:
    full = set(base['classes']['move']), set(other['classes']['move'])
    by_fn = analyze.fn(base), analyze.fn(other)
    return {'move_full': round(len(full[0] & full[1]) / len(full[0] | full[1]), 3),
            'move_by_function': round(len(by_fn[0] & by_fn[1]) / len(by_fn[0] | by_fn[1]), 3)}


# ---------------------------------------------------------------------------
# L2


def l2(base_dir: pathlib.Path, others: list[pathlib.Path]) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for variant in levels.MOVE_CLOSE:
        r = analyze.analyse(base_dir, move_close=variant)
        r['witness'] = analyze.witness(r)
        per_version = {o.name: stability(r, analyze.analyse(o, move_close=variant)) for o in others}
        out[variant] = {
            'moves_per_run': r['steps_per_run_b']['move'],
            'effects_per_move': sumstats(_values(r, 'effect per move')),
            'moves_per_activation': sumstats(_values(r, 'move per activation')),
            'move_classes': len(r['classes']['move']),
            'cause_placement': total_placement(r, 'b_structure'),
            'symptom_placement': total_placement(r, 'b_with_state'),
            'explaining_cut': {k: {'median': v['median'], 'max': v['max']} for k, v in r['cut_lines'].items()},
            'explaining_cut_max': max(v['max'] for v in r['cut_lines'].values()),
            'purity': purity(r),
            'stability': per_version,
            'witness_moves': r['witness']['moves'],
            'witness_tree': r['witness']['tree'],
        }
    out['prefer'] = prefer_choice(out)
    return out


def _values(result: dict[str, Any], key: str) -> list[int]:
    return result['_fan'][key]


def prefer_choice(out: dict[str, Any]) -> dict[str, Any]:
    """The `prefer` rule, stated before the run: keep the variant whose move classes are most stable
    across versions (identified by call and closing function); a difference below PREFER_TIE is a tie,
    broken by purity. A variant that places fewer causes at activation or move than every-write is excluded."""
    base_named = sum(out['every-write']['cause_placement'].get(lv, 0) for lv in ('activation', 'move'))
    scores = {}
    for v in levels.MOVE_CLOSE:
        named = sum(out[v]['cause_placement'].get(lv, 0) for lv in ('activation', 'move'))
        if named < base_named:
            continue
        st = statistics.fmean(s['move_by_function'] for s in out[v]['stability'].values())
        scores[v] = (st, out[v]['purity']['instances_matching_class_majority'])
    best = max(scores, key=lambda v: scores[v][0])
    ties = [v for v in scores if scores[best][0] - scores[v][0] < PREFER_TIE]
    chosen = max(ties, key=lambda v: scores[v][1])
    return {'scores': {v: {'stability_by_function': round(s[0], 3), 'purity': s[1]} for v, s in scores.items()}, 'chosen': chosen}


# ---------------------------------------------------------------------------
# L4


def l4(base_dir: pathlib.Path) -> dict[str, Any]:
    out = {}
    for variant in levels.OUTCOMES:
        r = analyze.analyse(base_dir, outcomes=variant)
        outcomes = lost = wrong_final = shared = goals = 0
        for trees in r['_trees'].values():
            for tree in trees:
                final = reader.final_state(tree)
                for g in tree['goals']:
                    goals += 1
                    shared += sum(1 for o in tree['outcomes'] if g in o['goals']) > 1
                for o in tree['outcomes']:
                    outcomes += 1
                    j = o['subject']
                    mine = {id(g) for g in o['goals']}
                    if any(c[0] == j for g in tree['goals'] if id(g) not in mine for c in g['ref_change']):
                        lost += 1
                    # Goals interleave, so the outcome's last change is the latest by step index, not by goal order.
                    own = sorted((e['i'], c[2]) for g in o['goals'] for e in g['effects'] for c in e['ref_change'] if c[0] == j)
                    if (own[-1][1] if own else None) != final.get(j):
                        wrong_final += 1
        out[variant] = {
            'outcomes': outcomes,
            'goals_under_two_outcomes': round(shared / goals, 3),
            'outcomes_missing_a_change_to_their_subject': round(lost / outcomes, 3),
            'outcomes_whose_final_state_is_wrong': round(wrong_final / outcomes, 3),
            'symptom_placement': total_placement(r, 'b_with_state'),
            'cause_placement': total_placement(r, 'b_structure'),
        }
    return out


# ---------------------------------------------------------------------------
# L5

SEVEN = ['outcome', 'stage', 'goal', 'exchange', 'activation', 'move', 'effect']


def stages_of(tree: dict[str, Any]) -> list[dict[str, Any]]:
    """Per subject, consecutive goals up to and including the one that changes its status."""
    out = []
    for o in tree['outcomes']:
        j, current = o['subject'], []
        for g in o['goals']:
            current.append(g)
            moved = [c for c in g['ref_change'] if c[0] == j and (c[1] or [None])[0] != (c[2] or [None])[0]]
            if moved:
                out.append({'subject': j, 'goals': current, 'enters': (moved[-1][2] or ['absent'])[0], 'outcome': o})
                current = []
        if current:
            out.append({'subject': j, 'goals': current, 'enters': None, 'outcome': o})
    return out


def length(effects: list[dict[str, Any]]) -> int:
    """A step's length in scheduler steps, first to last on-chain effect (its time span)."""
    idx = [e['i'] for e in effects if e['chain']] or [e['i'] for e in effects]
    return idx[-1] - idx[0] + 1


def overlap(a: list[int], b: list[int]) -> float:
    pa, pb = Counter(a), Counter(b)
    return sum(min(pa[k] / len(a), pb[k] / len(b)) for k in set(pa) | set(pb))


def l5(base_dir: pathlib.Path) -> dict[str, Any]:
    r = analyze.analyse(base_dir)
    fan: dict[tuple[str, str], list[int]] = defaultdict(list)
    spans: dict[str, list[int]] = defaultdict(list)
    for trees in r['_trees'].values():
        for tree in trees:
            st = stages_of(tree)
            for o in tree['outcomes']:
                fan[('outcome', 'stage')].append(sum(1 for s in st if s['outcome'] is o))
                spans['outcome'].append(length([e for g in o['goals'] for e in g['effects']]))
            for s in st:
                fan[('stage', 'goal')].append(len(s['goals']))
                spans['stage'].append(length([e for g in s['goals'] for e in g['effects']]))
            for g in tree['goals']:
                fan[('goal', 'exchange')].append(1)
                fan[('exchange', 'activation')].append(len(g['activations']))
                spans['goal'].append(length(g['effects']))
                spans['exchange'].append(length(g['effects']))
            for a in tree['activations']:
                fan[('activation', 'move')].append(len(a['moves']))
                spans['activation'].append(length(a['effects']))
            for m in tree['moves']:
                fan[('move', 'effect')].append(len(m['effects']))
                spans['move'].append(length(m['effects']))
            spans['effect'].extend(1 for e in tree['effects'] if e['chain'])
    named = placement_seven(r)
    pairs = {}
    for upper, lower in zip(SEVEN, SEVEN[1:]):
        f = fan[(upper, lower)]
        med = statistics.median(f)
        ones = sum(v == 1 for v in f) / len(f)
        ov = overlap(spans[upper], spans[lower])
        dropped = 'stage' if 'stage' in (upper, lower) else 'exchange' if 'exchange' in (upper, lower) else upper
        pairs[f'{upper}/{lower}'] = {
            'fan_in': sumstats(f), 'span_overlap': round(ov, 3),
            'merge': {'band': med < BAND[0], 'single-child': ones >= SINGLE_CHILD, 'span-overlap': ov >= SPAN_OVERLAP},
            'split_hint_band': med > BAND[1],
            'would_drop': dropped,
            'anomalies_named_only_at_dropped_level': named['symptom'].get(dropped, 0) + named['cause'].get(dropped, 0),
        }
    verdict = {}
    for test in ('band', 'single-child', 'span-overlap'):
        merged = [p for p, v in pairs.items() if v['merge'][test]]
        verdict[test] = {'merges': merged,
                         'drops_a_level_that_names_an_anomaly': [p for p in merged if pairs[p]['anomalies_named_only_at_dropped_level'] > 0],
                         'merges_stage': any('stage' in p for p in merged), 'merges_exchange': any('exchange' in p for p in merged)}
    return {'pairs': pairs, 'verdict': verdict, 'named_at': named, 'span_median': {k: statistics.median(v) for k, v in spans.items()}}


def placement_seven(r: dict[str, Any]) -> dict[str, dict[str, int]]:
    """The highest of the seven candidate levels at which each anomalous run differs from every clean run."""
    out = {'symptom': Counter(), 'cause': Counter()}
    for name, trees in r['_trees'].items():
        for with_state, key in ((True, 'symptom'), (False, 'cause')):
            sigs = []
            for tree in trees:
                s = {lv: analyze.b_signature(tree, lv, with_state) for lv in analyze.B_LEVELS}
                s['exchange'] = s['goal']
                s['stage'] = digest(sorted([[x['subject'], [digest(g['class']) for g in x['goals']], x['enters'] if with_state else None] for x in stages_of(tree)], key=str))
                sigs.append(s)
            clean = {lv: {s[lv] for s, bad in zip(sigs, r['_labels'][name]) if not bad} for lv in SEVEN}
            for s, bad in zip(sigs, r['_labels'][name]):
                if bad:
                    out[key][analyze.highest(s, clean, SEVEN)] += 1
    return {k: dict(v) for k, v in out.items()}


# ---------------------------------------------------------------------------
# L3

OWNERS = [  # key family prefix -> what a reader would call it, checked in order
    ('rq:job:*:dependents', 'the dependents of a job'), ('rq:job::*:dependencies', 'the dependencies of a job'),
    ('rq:execution:*:*', 'an execution record'), ('rq:queue:*:intermediate', "a queue's in-progress list"), ('rq:job:*', "the job's record"), ('rq:results:*', "the job's results"),
    ('rq:execution:*', 'an execution record'), ('rq:executions:*', "the job's executions"), ('rq:worker:*', "a worker's record"),
    ('rq:workers', 'the set of workers'), ('rq:queues', 'the set of queues'), ('rq:queue:*', 'a queue'),
    ('rq:wip:*', 'the started-jobs list'), ('rq:finished:*', 'the finished-jobs list'), ('rq:failed:*', 'the failed-jobs list'),
    ('rq:deferred:*', 'the deferred-jobs list'), ('rq:canceled:*', 'the canceled-jobs list'), ('rq:scheduled:*', 'the scheduled-jobs list'),
]


def owner(pattern: str) -> str:
    for prefix, name in OWNERS:
        if pattern == prefix or (prefix.endswith('*') and pattern.startswith(prefix[:-1]) and pattern.count(':') == prefix.count(':')):
            return name
    return pattern


def head_of_move(m: dict[str, Any]) -> dict[str, Any]:
    closer = ([e for e in m['effects'] if e['writes'] or e['rejected']] or m['effects'])[-1]
    return {'function': closer['stack'][-1] if closer['stack'] else '?',
            'result': 'commit attempt rejected, nothing written' if closer['rejected'] else 'writes: ' + ', '.join(sorted({owner(key_pattern(k)) for k in closer['writes']})),
            'observable_change': sorted({f'{a} -> {b}' for _j, a, b in closer['ref_change']})[:4]}


def head_sheet(result: dict[str, Any]) -> dict[str, Any]:
    sheet = llm_tasks.sheet(result)
    heads_m: dict[str, Any] = {}
    heads_a: dict[str, Any] = {}
    for trees in result['_trees'].values():
        for tree in trees:
            for a in tree['activations']:
                aid = f'a:{a["api"]}:{a["part"]}'
                for m in a['moves']:
                    heads_m.setdefault('m:' + digest(m['class'])[:6], head_of_move(m))
                if a['moves']:
                    key = next((m for m in a['moves'] if m['ref_change']), a['moves'][-1])
                    heads_a.setdefault(aid, {'head_move': 'm:' + digest(key['class'])[:6]})
    for mid, h in heads_m.items():
        sheet['moves'][mid]['head'] = h
    for aid, h in heads_a.items():
        sheet['activations'][aid]['head'] = h
    return sheet


HEAD_NOTE = '''Each move and activation also has a "head": for a move, the effect that closes it (what it
writes, in plain words, and the observable change it makes); for an activation, the move that
changes what a user can observe. Name each step after what its head does. Every thing named in
a move's name (a job, a worker, a queue, a list, an execution, a result) must be something its
head writes.'''


def name_prompt(graph_dir: pathlib.Path, rq_root: pathlib.Path, out: pathlib.Path) -> None:
    result = analyze.analyse(graph_dir)
    sheet = head_sheet(result)
    out.mkdir(parents=True, exist_ok=True)
    (out / 'naming-sheet-head.json').write_text(json.dumps(sheet, indent=1, sort_keys=True))
    docs = '\n\n'.join(f'### {rel}\n' + (rq_root / rel).read_text() for rel in llm_tasks.DOCS)
    observable = ('An observable change is written as [status, places listing the job, type of latest result] '
                  'before -> after; None means the job has no record.')
    (out / 'prompt-name-head.md').write_text(
        f'{llm_tasks.NAME_TASK}\n\n{HEAD_NOTE}\n\n{observable}\n\n## Sheet of step classes\n\n```json\n{json.dumps(sheet, indent=1, sort_keys=True)}\n```\n\n## rq user documentation\n\n{docs}\n')
    print('moves', len(sheet['moves']), 'activations', len(sheet['activations']), 'goals', len(sheet['goals']), 'prompt chars', len((out / 'prompt-name-head.md').read_text()))


# Nouns that a move's name may use only if the move's head writes the matching thing. `queue` as a noun
# only (not "enqueue", "requeue"); "list" words map to the registries.
TRUTH = [
    (re.compile(r'\bworkers?\b'), ("a worker's record", 'the set of workers')),
    (re.compile(r'\bqueues?\b'), ('a queue', 'the set of queues')),
    (re.compile(r'\bexecutions?\b'), ('an execution record', "the job's executions")),
    (re.compile(r'\bresults?\b'), ("the job's results",)),
    (re.compile(r'\bdependents?\b|\bdependencies\b'), ('the dependents of a job', 'the dependencies of a job', 'the deferred-jobs list')),
    (re.compile(r'\bstarted\b'), ('the started-jobs list', "the job's record")),
    (re.compile(r'\bfinished\b'), ('the finished-jobs list', "the job's record")),
    (re.compile(r'\bfailed\b'), ('the failed-jobs list', "the job's record")),
    (re.compile(r'\bcancel(?:ed|led)?\b'), ('the canceled-jobs list', "the job's record")),
]


def name_truth(name: str, head: dict[str, Any]) -> list[str]:
    """Words of a move's name that its head does not write. Rejected commits write nothing and are skipped."""
    if not head['result'].startswith('writes: '):
        return []
    written = set(head['result'][len('writes: '):].split(', '))
    return [rx.pattern for rx, owners in TRUTH if rx.search(name.lower()) and not written & set(owners)]


# Revision written after reading the failures of the check above on the first experiment's names
# (so fitted to them): a word that names the step's own API call is context, not a claim; and a list
# counts as holding what the glossary says it lists (rq's started-jobs list lists executions).
CONTAINS = {'the started-jobs list': ("the job's executions", 'an execution record')}


def _stem(word: str) -> str:
    for suffix in ('led', 'ed', 's'):
        if word.endswith(suffix) and len(word) - len(suffix) >= 4:
            return word[: -len(suffix)]
    return word


def name_truth_revised(name: str, head: dict[str, Any], api_call: str) -> list[str]:
    if not head['result'].startswith('writes: '):
        return []
    written = set(head['result'][len('writes: '):].split(', '))
    for holder, held in CONTAINS.items():
        if holder in written:
            written |= set(held)
    call_words = set(re.findall(r'[a-z]+', api_call.lower()))
    bad = []
    for rx, owners in TRUTH:
        words = set(rx.findall(name.lower()))
        context = all(_stem(x) in call_words for x in words)
        if words and not context and not written & set(owners):
            bad.append(rx.pattern)
    return bad


def name_score(out: pathlib.Path, baseline: pathlib.Path, results: pathlib.Path) -> None:
    sheet = json.loads((out / 'naming-sheet-head.json').read_text())
    first = [p for p in sorted(baseline.glob('name-*.json')) if not p.stem.startswith('name-head')]
    variants = {'children': first, 'head': sorted((out / 'answers').glob('name-head-*.json'))}
    report: dict[str, Any] = {}
    for variant, paths in variants.items():
        answers = {p.stem: llm_tasks.answer(p) for p in paths}
        per_level = {}
        for level in ('moves', 'activations', 'goals'):
            ids = sorted(set.intersection(*(set(a[level]) for a in answers.values())))
            exact, jacc = [], []
            for cid in ids:
                given = [a[level][cid]['name'].strip().lower() for a in answers.values()]
                for x, y in itertools.combinations(given, 2):
                    exact.append(x == y)
                    tx, ty = llm_tasks.tokens(x), llm_tasks.tokens(y)
                    jacc.append(len(tx & ty) / max(1, len(tx | ty)))
            per_level[level] = {'classes': len(ids), 'pairwise_exact': round(statistics.fmean(exact), 3), 'pairwise_token_jaccard': round(statistics.fmean(jacc), 3)}
        failures = []
        for sid, a in answers.items():
            for mid, entry in a['moves'].items():
                bad = name_truth(entry['name'], sheet['moves'][mid]['head']) if mid in sheet['moves'] else []
                if bad:
                    failures.append({'sample': sid, 'move': mid, 'name': entry['name'], 'words': bad, 'head': sheet['moves'][mid]['head']['result']})
        revised = [f for f in failures if name_truth_revised(f['name'], sheet['moves'][f['move']]['head'], sheet['moves'][f['move']]['api_call'])]
        names_given = sum(len(a['moves']) for a in answers.values())
        storage = sum(bool(llm_tasks.STORAGE_WORDS.search(a[lv][c]['name'])) for a in answers.values() for lv in ('moves', 'activations', 'goals') for c in a[lv])
        majority = {}
        for mid in sheet['moves']:
            given = [a['moves'][mid]['name'].strip().lower() for a in answers.values() if mid in a['moves']]
            if given:
                majority[mid] = max(set(given), key=given.count)
        report[variant] = {'samples': sorted(answers), 'levels': per_level, 'name_truth_failures': len(failures), 'move_names_given': names_given,
                           'failure_share': round(len(failures) / names_given, 3), 'storage_words': storage,
                           'name_truth_failures_revised': len(revised), 'revised_failures': [[f['sample'], f['name']] for f in revised],
                           'majority_failures': [{'move': m, 'name': n} for m, n in majority.items() if name_truth(n, sheet['moves'][m]['head'])],
                           'failures': failures, 'majority_moves': majority}
    (results / 'strategies-naming.json').write_text(json.dumps(report, indent=1, sort_keys=True))
    for v, r in report.items():
        print(v, json.dumps({k: r[k] for k in ('levels', 'name_truth_failures', 'name_truth_failures_revised', 'move_names_given', 'failure_share', 'storage_words', 'majority_failures', 'revised_failures')}))


# ---------------------------------------------------------------------------
# Reader for L2: the same three runs at the move level under each move rule. Move names are
# mechanical (closing function and what it writes) so that only the cut differs between variants.


def mech_name(m: dict[str, Any]) -> str:
    h = head_of_move(m)
    return f'{h["function"]}: {h["result"]}'


def render_moves(tree: dict[str, Any]) -> list[str]:
    rows = []
    for m in tree['moves']:
        first, last = analyze.span(m, 'move')
        rows.append((first, last, m['activation']['actor'], mech_name(m), m['ref_change']))
    lines = [f'[{a}-{b}] {actor}: {name}{reader.change_text(ch)}' if a != b else f'[{a}] {actor}: {name}{reader.change_text(ch)}' for a, b, actor, name, ch in sorted(rows)]
    lines.append(f'({sum(not e["chain"] for e in tree["effects"])} bookkeeping steps are folded)')
    return lines + [reader.final_text(tree)]


def reader_prompts(graph_dir: pathlib.Path, out: pathlib.Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    questions = '\n'.join(f'Q{i + 1}. {q}\n' + '\n'.join(f'   {chr(65 + j)}. {o}' for j, o in enumerate(opts)) for i, (_role, q, opts) in enumerate(reader.QUESTIONS))
    keys, sizes = {}, {}
    for variant in levels.MOVE_CLOSE:
        r = analyze.analyse(graph_dir, move_close=variant)
        for run_id, (tree, ref) in reader.pick_runs(r).items():
            keys[run_id] = {**reader.computed_keys(tree), **reader.HAND_KEYS[run_id]}
            shown, base = render_moves(tree), render_moves(ref)
            sizes[f'{variant}|move|{run_id}'] = len(shown)
            text = (
                'You are reading a description of one run of a job queue system. A client program submits or cancels jobs; '
                'worker processes take jobs and complete them. Answer the questions about THE RUN using only what is written '
                'below. If the description does not contain the answer, choose the "cannot tell" option; do not guess.\n\n'
                '## The run, described at the "move" level\n\n' + '\n'.join(shown) +
                '\n\n## For comparison: a clean run of the same scenario at the same level\n\n' + '\n'.join(base) +
                f'\n\n## Questions about THE RUN\n\n{questions}\n\n'
                'Answer with exactly one line of JSON: {"Q1": "A", "Q2": "B", ...} for all seven questions.'
            )
            (out / f'{variant}--move--{run_id}.md').write_text(text)
    (out / 'keys.json').write_text(json.dumps({'keys': keys, 'sizes': sizes}, indent=1))
    print(sizes)


def reader_score(out: pathlib.Path, results: pathlib.Path) -> None:
    meta = json.loads((out / 'keys.json').read_text())
    table: dict[str, dict[str, list[int]]] = defaultdict(lambda: defaultdict(lambda: [0, 0, 0]))
    raw = {}
    for path in sorted((out / 'answers').glob('*.json')):
        variant, _level, rest = path.stem.split('--')
        run_id = rest.split('.')[0]
        text = json.loads(path.read_text())['result']
        match = re.search(r'\{[^{}]*\}', text, re.S)
        if match is None:
            raise ValueError(f'{path}: no JSON answer; re-run that call')
        given = json.loads(match.group(0))
        raw[path.stem] = given
        for i, (role, _q, opts) in enumerate(reader.QUESTIONS):
            letter = str(given.get(f'Q{i + 1}', '')).strip()[:1].upper()
            choice = ord(letter) - 65 if letter else -1
            key = meta['keys'][run_id][str(i)]
            table[variant][role][0 if choice == key else 1 if choice == len(opts) - 1 else 2] += 1
    summary = {v: {r: {'right': c[0], 'cannot_tell': c[1], 'wrong': c[2]} for r, c in roles.items()} for v, roles in table.items()}
    (results / 'strategies-reader.json').write_text(json.dumps({'summary': summary, 'sizes': meta['sizes'], 'answers': raw}, indent=1, sort_keys=True))
    for v in levels.MOVE_CLOSE:
        lines = [meta['sizes'][f'{v}|move|{r}'] for r in ('witness', 'cancel', 'clean')]
        print(f'{v:13s} lines {lines}', json.dumps(summary.get(v, {})))


def main() -> None:
    cmd = sys.argv[1]
    if cmd == 'structure':
        base, results = pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
        others = [pathlib.Path(p) for p in sys.argv[4:]]
        report = {'L2': l2(base, others), 'L4': l4(base), 'L5': l5(base)}
        results.mkdir(parents=True, exist_ok=True)
        (results / 'strategies.json').write_text(json.dumps(report, indent=1, sort_keys=True, default=str))
        for k, v in report['L2'].items():
            print('L2', k, json.dumps({kk: vv for kk, vv in v.items() if kk != 'witness_tree'} if isinstance(v, dict) else v, default=str))
        print('L4', json.dumps(report['L4']))
        print('L5 pairs', json.dumps({p: {k: v[k] for k in ('fan_in', 'span_overlap', 'merge', 'would_drop', 'anomalies_named_only_at_dropped_level')} for p, v in report['L5']['pairs'].items()}))
        print('L5 verdict', json.dumps(report['L5']['verdict']))
        print('L5 named', json.dumps(report['L5']['named_at']), json.dumps(report['L5']['span_median']))
    elif cmd == 'name-prompt':
        name_prompt(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), pathlib.Path(sys.argv[4]))
    elif cmd == 'name-score':
        name_score(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), pathlib.Path(sys.argv[4]))
    elif cmd == 'reader-prompts':
        reader_prompts(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
    elif cmd == 'reader-score':
        reader_score(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
    else:
        raise SystemExit(__doc__)


if __name__ == '__main__':
    main()
