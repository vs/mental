"""Build the two LLM prompts and score the answers.

name-step (design B): the step classes are given; the LLM only names them.
ladder (design C): the LLM proposes the levels and cuts one run into them itself.

usage:
  python -I llm_tasks.py prompts <graph dir> <rq checkout> <out dir>
  python -I llm_tasks.py score <out dir> <results dir>
"""

from __future__ import annotations

import itertools
import json
import pathlib
import re
import statistics
import sys
from collections import defaultdict
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from mlexp import analyze, levels  # noqa: E402

from afexp.harness import digest, key_pattern  # noqa: E402

DOCS = ['docs/docs/jobs.md', 'docs/docs/job_registries.md']

NAME_TASK = '''You are naming the steps of a multi-level model of the Python job queue library rq.
The steps were found mechanically; do not regroup them. Three levels, from the bottom:

- move: a few Redis commands of one actor that together make one change (or one rejected attempt).
  Readers: developers working on concurrency. Mechanism words are fine (attempt, commit, claim);
  raw command names and key names are not.
- activation: one actor's uninterrupted part of an API call. Readers: developers.
- goal: one API call of one actor, seen as what that actor got done. Readers: product owners and
  support staff who know rq's user documentation and nothing of its storage. Use only words a
  user of rq would use.

For every step class give a name of at most five words, verb first, and one clause that starts
with "so that" and says what the step is for. Name moves first, then activations using your move
names, then goals. Different classes need different names. The same class must get the name you
would give it again tomorrow: prefer the plainest wording.

Answer with exactly one ```json block: {"moves": {id: {"name": ..., "so_that": ...}},
"activations": {...}, "goals": {...}} with every id from the sheet below.'''

LADDER_TASK = '''You are building a multi-level model of the Python job queue library rq, so that people in
different roles (a manager, a product owner, a developer) can each read one recorded run at the
level that suits them.

Below is one run of 42 steps. Each step is one Redis command or transaction by one actor
(c is a client program, w1 is a worker process), with the rq function that issued it.

1. Propose the levels of description above these steps, from the top down. For each level give
   its name, its readers, the question it answers and what one step is at that level.
2. For each of your levels, cut the run into steps of that level: every one of the 42 steps
   (numbers 0 to 41) must belong to exactly one group per level. Name each group.

Answer with exactly one ```json block:
{"levels": [{"name": ..., "readers": ..., "question": ..., "step_is": ...}, ...],
 "segmentation": {"<level name>": [{"name": ..., "steps": [0, 1, ...]}, ...], ...}}'''


def sheet(result: dict[str, Any]) -> dict[str, Any]:
    moves: dict[str, Any] = {}
    acts: dict[str, dict[str, Any]] = {}
    goals: dict[str, dict[str, Any]] = {}
    for trees in result['_trees'].values():
        for tree in trees:
            for g in tree['goals']:
                gid = 'g:' + g['api']
                entry = goals.setdefault(gid, {'api_call': g['api'], 'activations': set(), 'observable_changes': set(), 'returns': set()})
                entry['returns'].add(json.dumps(g['output']))
                for a in g['activations']:
                    aid = f'a:{a["api"]}:{a["part"]}'
                    entry['activations'].add(aid)
                    aentry = acts.setdefault(aid, {'api_call': a['api'], 'part': a['part'], 'moves': set(), 'observable_changes': set()})
                    for m in a['moves']:
                        mid = 'm:' + digest(m['class'])[:6]
                        aentry['moves'].add(mid)
                        if mid not in moves:
                            closer = m['class']
                            moves[mid] = {
                                'api_call': closer[0],
                                'function': closer[1],
                                'result': 'transaction rejected, nothing written' if closer[2] == 'rejected' else {'writes_key_families': closer[2]},
                                'commands': [[n, [key_pattern(k) for k in keys]] for e in m['effects'] for n, keys in e['commands']][:14],
                            }
                    for c in a['ref_change']:
                        text = f'{c[1]} -> {c[2]}'
                        aentry['observable_changes'].add(text)
                        entry['observable_changes'].add(text)
    for d in list(acts.values()) + list(goals.values()):
        for k, v in d.items():
            if isinstance(v, set):
                d[k] = sorted(v)[:8]
    return {'moves': moves, 'activations': acts, 'goals': goals}


def witness_listing(result: dict[str, Any]) -> str:
    lines = []
    for i, actor, commands, stack, rejected, _chain in result['witness']['listing']:
        text = '; '.join(f'{n} {" ".join(keys)}' for n, keys in commands)
        lines.append(f'{i:2d} {actor:2s} {stack[-1] if stack else "?"}: {text}' + ('   (transaction rejected)' if rejected else ''))
    return '\n'.join(lines)


def prompts(graph_dir: pathlib.Path, rq_root: pathlib.Path, out: pathlib.Path) -> None:
    from afexp import llm

    result = analyze.analyse(graph_dir)
    result['witness'] = analyze.witness(result)
    out.mkdir(parents=True, exist_ok=True)
    naming = sheet(result)
    (out / 'naming-sheet.json').write_text(json.dumps(naming, indent=1, sort_keys=True))
    docs = '\n\n'.join(f'### {rel}\n' + (rq_root / rel).read_text() for rel in DOCS)
    observable = ('An observable change is written as [status, places listing the job, type of latest result] '
                  'before -> after; None means the job has no record.')
    (out / 'prompt-name.md').write_text(
        f'{NAME_TASK}\n\n{observable}\n\n## Sheet of step classes\n\n```json\n{json.dumps(naming, indent=1, sort_keys=True)}\n```\n\n## rq user documentation\n\n{docs}\n'
    )
    code = llm.build_prompt(rq_root, graph_dir / 'cancel-vs-work.b0.pkl', True)
    code = code[code.index('## Source code of rq'):]
    listing = witness_listing(result)
    (out / 'witness.txt').write_text(listing + '\n')
    (out / 'prompt-ladder.md').write_text(f'{LADDER_TASK}\n\n## The run\n\n```\n{listing}\n```\n\n{code}\n')
    b_partitions = {}
    best = min((t for t, bad in zip(result['_trees']['dependency'], result['_labels']['dependency']) if bad), key=lambda t: len(t['effects']))
    for level in ('goal', 'activation'):
        b_partitions[level] = [[e['i'] for e in s['effects']] for s in best[level + 's']]
    b_partitions['move'] = [[e['i'] for e in m['effects']] for m in best['moves']] + [[e['i']] for e in best['effects'] if not e['chain']]
    (out / 'b-partitions.json').write_text(json.dumps(b_partitions))
    print({k: len(v) for k, v in naming.items()}, 'prompt sizes', len((out / 'prompt-name.md').read_text()), len((out / 'prompt-ladder.md').read_text()))


BLOCK = re.compile(r'```json\n(.*?)```', re.S)
STORAGE_WORDS = re.compile(r'\b(redis|hash|hset|zset|zadd|zrem|lmove|sadd|key|keys|pipeline|exec|watch|rq:|sorted set|multi)\b', re.I)
MECHANISM_WORDS = re.compile(r'\b(transaction|commit|attempt|atomic|lock|retry loop|optimistic)\b', re.I)


def answer(path: pathlib.Path) -> Any:
    match = BLOCK.search(json.loads(path.read_text())['result'])
    if match is None:
        raise ValueError(f'{path}: no json block in the answer; re-run that sample')
    return json.loads(match.group(1))


def tokens(name: str) -> set[str]:
    return set(re.findall(r'[a-z0-9]+', name.lower())) - {'the', 'a', 'an', 'to', 'of', 'for', 'and', 'in', 'on'}


def rand_index(a: list[int], b: list[int]) -> float:
    from sklearn.metrics import adjusted_rand_score

    return float(adjusted_rand_score(a, b))


def labels_of(groups: list[list[int]], n: int) -> list[int] | None:
    out = [-1] * n
    for g, steps in enumerate(groups):
        for s in steps:
            if not 0 <= s < n or out[s] != -1:
                return None
            out[s] = g
    return None if -1 in out else out


def score(out: pathlib.Path, results: pathlib.Path) -> None:
    report: dict[str, Any] = {}
    names = {p.stem: answer(p) for p in sorted((out / 'answers').glob('name-*.json'))}
    naming: dict[str, Any] = {}
    for level, barrier in (('moves', STORAGE_WORDS), ('activations', STORAGE_WORDS), ('goals', re.compile(STORAGE_WORDS.pattern + '|' + MECHANISM_WORDS.pattern, re.I))):
        ids = sorted(set.intersection(*(set(a[level]) for a in names.values())))
        exact, jacc, clash, breaks = [], [], [], 0
        majority = {}
        for cid in ids:
            given = [a[level][cid]['name'].strip().lower() for a in names.values()]
            for x, y in itertools.combinations(given, 2):
                exact.append(x == y)
                tx, ty = tokens(x), tokens(y)
                jacc.append(len(tx & ty) / max(1, len(tx | ty)))
            breaks += sum(bool(barrier.search(g)) for g in given)
            majority[cid] = max(set(given), key=given.count)
        for a in names.values():
            used = [a[level][c]['name'].strip().lower() for c in ids]
            clash.append(len(used) - len(set(used)))
        naming[level] = {
            'classes': len(ids),
            'pairwise_exact': round(statistics.fmean(exact), 3),
            'pairwise_token_jaccard': round(statistics.fmean(jacc), 3),
            'barrier_breaks': breaks,
            'names_given': len(ids) * len(names),
            'duplicate_names_per_sample': clash,
            'majority': majority,
        }
    report['name_step'] = {'samples': sorted(names), 'levels': naming}

    ladders = {}
    for p in sorted((out / 'answers').glob('ladder-*.json')):
        try:
            ladders[p.stem] = answer(p)
        except (ValueError, json.JSONDecodeError) as exc:
            ladders[p.stem] = {'error': str(exc)}
    b_parts = {k: labels_of(v, 42) for k, v in json.loads((out / 'b-partitions.json').read_text()).items()}
    per_sample: dict[str, Any] = {}
    partitions: dict[str, dict[str, list[int]]] = {}
    for sid, lad in ladders.items():
        if 'error' in lad:
            per_sample[sid] = lad
            continue
        level_names = [lv['name'] for lv in lad['levels']]
        parts = {}
        invalid = []
        for name in level_names:
            groups = lad['segmentation'].get(name)
            lab = labels_of([g['steps'] for g in groups], 42) if groups else None
            if lab is None:
                invalid.append(name)
            else:
                parts[name] = lab
        ordered = [n for n in level_names if n in parts]
        nested = []
        for upper, lower in zip(ordered, ordered[1:]):
            owner: dict[int, set[int]] = defaultdict(set)
            for u, l in zip(parts[upper], parts[lower]):
                owner[l].add(u)
            nested.append(all(len(v) == 1 for v in owner.values()))
        sizes = [len(set(parts[n])) for n in ordered]
        per_sample[sid] = {
            'levels': level_names,
            'groups_per_level': sizes,
            'invalid_levels': invalid,
            'nested_pairs': nested,
            'fan_in': [round(b / a, 2) for a, b in zip(sizes, sizes[1:])] + ([round(42 / sizes[-1], 2)] if sizes else []),
            'best_match_to_b': {k: round(max((rand_index(v, p) for p in parts.values()), default=0.0), 3) for k, v in b_parts.items() if v},
            'step_names': {n: [g['name'] for g in lad['segmentation'][n]] for n in ordered},
        }
        partitions[sid] = parts
    agree = []
    name_overlap = []
    for x, y in itertools.combinations(sorted(partitions), 2):
        for part in partitions[x].values():
            agree.append(max(rand_index(part, q) for q in partitions[y].values()))
        nx = {n.lower() for v in per_sample[x]['step_names'].values() for n in v}
        ny = {n.lower() for v in per_sample[y]['step_names'].values() for n in v}
        name_overlap.append(len(nx & ny) / max(1, len(nx | ny)))
    level_sets = [frozenset(n.lower() for n in s['levels']) for s in per_sample.values() if 'levels' in s]
    report['ladder'] = {
        'samples': per_sample,
        'level_counts': [len(s['levels']) for s in per_sample.values() if 'levels' in s],
        'identical_level_name_sets': len(level_sets) - len(set(level_sets)) + 1 if level_sets else 0,
        'cross_sample_best_match_rand': {'mean': round(statistics.fmean(agree), 3), 'min': round(min(agree), 3)} if agree else None,
        'cross_sample_step_name_jaccard': round(statistics.fmean(name_overlap), 3) if name_overlap else None,
    }
    (results / 'llm.json').write_text(json.dumps(report, indent=1, sort_keys=True))
    print(json.dumps({k: {kk: vv for kk, vv in v.items() if kk != 'majority'} for k, v in naming.items()}))
    for sid, s in per_sample.items():
        print(sid, {k: v for k, v in s.items() if k != 'step_names'})
    print({k: v for k, v in report['ladder'].items() if k != 'samples'})


if __name__ == '__main__':
    if sys.argv[1] == 'prompts':
        prompts(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), pathlib.Path(sys.argv[4]))
    else:
        score(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
