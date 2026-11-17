"""Reader test: can a role's questions be answered from one level of one run?

An LLM stands in for the reader. It sees a run rendered at one level of one
design, beside a clean run of the same scenario at the same level, and answers
seven multiple-choice questions, each with a "cannot tell" option. This shows
whether an answer is present at a level, not whether a person would find it.

Three variants:
- B: the proposed levels with LLM-given names and the observable changes of each step.
- A: the specification's levels as they stand; the only upper state is the status field.
- A+: the specification's levels with the same observable changes as B, to separate
  the effect of the ladder from the effect of the observables.

usage:
  python -I reader.py prompts <graph dir> <results dir> <out dir>
  python -I reader.py score <out dir> <results dir>
"""

from __future__ import annotations

import json
import pathlib
import re
import sys
from collections import defaultdict
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from mlexp import analyze, levels  # noqa: E402

from afexp import afs  # noqa: E402
from afexp.harness import digest  # noqa: E402

QUESTIONS = [
    ('manager', 'How many jobs end with the status "finished"?', ['0', '1', '2', 'cannot tell from this description']),
    ('manager', 'At the end of the run, is any job recorded inconsistently (its status and the places that list it disagree)?', ['yes', 'no', 'cannot tell from this description']),
    ('product', 'Did one party start acting on a job while another party\'s action on that same job was still in progress?', ['yes', 'no', 'cannot tell from this description']),
    ('product', 'What did the client get done?', ['submitted a job that waits for another job', 'canceled a job', 'nothing: no client took part', 'cannot tell from this description']),
    ('developer', 'Was any commit rejected and then tried again?', ['yes', 'no', 'cannot tell from this description']),
    ('developer', 'Which part of the run is responsible for an inconsistent record?', ['taking a job', 'completing a job: the commit made after an earlier attempt was rejected', 'a cancel that overlapped the worker\'s handling of the job', 'nothing: no record is inconsistent', 'cannot tell from this description']),
    ('developer', 'Compared with the clean run, what does the last commit that completes the first job fail to do?', ['remove the job from the list of started jobs', 'add the job to the list of finished jobs', 'save the job\'s result', 'nothing: it does the same', 'cannot tell from this description']),
]
# Keys written before any rendering was produced. Questions 1, 2, 3 and 5 are also recomputed from the run.
HAND_KEYS = {'witness': {3: 0, 5: 1, 6: 0}, 'cancel': {3: 1, 5: 2, 6: 3}, 'clean': {3: 0, 5: 3, 6: 3}}
B_LEVELS = ['outcome', 'goal', 'activation', 'move', 'effect']
A_LEVELS = ['business', 'system', 'services', 'components', 'statements']


def show(x: Any) -> str:
    if x is None:
        return 'no record'
    if len(x) == 4 and isinstance(x[1], bool):  # report-jobs: status, has a link, uploads, stranded
        return f'status {x[0]}, {"has a download link" if x[1] else "no download link"}, file uploaded {x[2]} time{"" if x[2] == 1 else "s"}' + (', nothing will pick it up' if x[3] else '')
    result = {None: 'none', '1': 'success', '2': 'failure', '3': 'stopped', '4': 'retried'}.get(x[2], str(x[2]))
    return f'status {x[0]}, listed in {x[1] or "nothing"}, latest result {result}'


def change_text(changes: list[Any]) -> str:
    """The net change per job: its first state before and its last state after."""
    net: dict[str, list[Any]] = {}
    for job, a, b in changes:
        net.setdefault(job, [a, b])[1] = b
    parts = [f'{job}: {show(a)} => {show(b)}' for job, (a, b) in net.items()]
    return ('   -> ' + ' | '.join(parts)) if parts else ''


def final_state(tree: dict[str, Any]) -> dict[str, Any]:
    last: dict[str, Any] = {}
    for e in tree['effects']:
        for job, _a, b in e['ref_change']:
            last[job] = b
    return last


def final_text(tree: dict[str, Any]) -> str:
    return 'At the end: ' + '; '.join(f'{j}: {show(b)}' for j, b in sorted(final_state(tree).items()))


def render_b(tree: dict[str, Any], level: str, names: dict[str, dict[str, str]]) -> list[str]:
    if level == 'outcome':
        out = []
        for o in tree['outcomes']:
            goals = ', then '.join(f'{g["actor"]}: {names["goals"]["g:" + g["api"]]}' for g in o['goals'])
            out.append(f'job {o["subject"]}: {goals}')
        return out + [final_text(tree)]
    if level == 'effect':
        rows = [(e['i'], e['i'], e['actor'], '; '.join(f'{n} {" ".join(k)}' for n, k in e['commands']) + (' (rejected)' if e['rejected'] else ''), e['ref_change']) for e in tree['effects']]
    else:
        rows = []
        for s in tree[level + 's']:
            first, last = analyze.span(s, level)
            if level == 'goal':
                name, actor = names['goals']['g:' + s['api']], s['actor']
            elif level == 'activation':
                name, actor = names['activations'][f'a:{s["api"]}:{s["part"]}'], s['actor']
            else:
                name, actor = names['moves']['m:' + digest(s['class'])[:6]], s['activation']['actor']
            rows.append((first, last, actor, name, s['ref_change']))
    lines = [f'[{a}-{b}] {actor}: {name}{change_text(ch)}' if a != b else f'[{a}] {actor}: {name}{change_text(ch)}' for a, b, actor, name, ch in sorted(rows)]
    if level in ('move', 'effect'):
        folded = sum(not e['chain'] for e in tree['effects'])
        if level == 'move':
            lines.append(f'({folded} bookkeeping steps are folded)')
    return lines + [final_text(tree)]


def render_a(tree: dict[str, Any], level: str, with_observables: bool) -> list[str]:
    lines = [f'{actor}: {text}' for actor, text in tree['ladder_a'][level]]
    if with_observables:
        lines.append(final_text(tree))
    else:
        lines.append('At the end: ' + '; '.join(f'{j}: status {b[0] if b else "no record"}' for j, b in sorted(final_state(tree).items())))
    return lines


def pick_runs(result: dict[str, Any]) -> dict[str, tuple[dict[str, Any], dict[str, Any]]]:
    """Three runs, each with a clean sequential run of its scenario as reference."""
    def clean_reference(name: str) -> dict[str, Any]:
        ok = [t for t, bad in zip(result['_trees'][name], result['_labels'][name]) if not bad and _sequential(t)]
        return max(ok, key=lambda t: len(t['effects']))

    dep = list(zip(result['_trees']['dependency'], result['_labels']['dependency']))
    witness = min((t for t, bad in dep if bad), key=lambda t: len(t['effects']))
    clean = max((t for t, bad in dep if not bad and not _sequential(t) and not any(e['rejected'] for e in t['effects'])), key=lambda t: len(t['effects']))
    can = list(zip(result['_trees']['cancel-vs-work'], result['_labels']['cancel-vs-work']))
    cancel = min((t for t, bad in can if bad and not _sequential(t) and _final_places(t) == ['canceled', 'finished']), key=lambda t: len(t['effects']))
    return {'witness': (witness, clean_reference('dependency')), 'cancel': (cancel, clean_reference('cancel-vs-work')), 'clean': (clean, clean_reference('dependency'))}


def _sequential(tree: dict[str, Any]) -> bool:
    spans = sorted(analyze.span(g, 'goal') for g in tree['goals'])
    return all(a[1] < b[0] for a, b in zip(spans, spans[1:]))


def _final_places(tree: dict[str, Any]) -> Any:
    last = final_state(tree).get('j1')
    return last[1] if last else None


def computed_keys(tree: dict[str, Any]) -> dict[int, int]:
    last = final_state(tree)
    finished = sum(1 for b in last.values() if b and b[0] == 'finished')
    inconsistent = any(b is not None and b[1] != afs.EXPECTED_LOCATION.get(b[0], b[1]) for b in last.values())
    return {0: finished, 1: 0 if inconsistent else 1, 2: 1 if _sequential(tree) else 0, 4: 0 if any(e['rejected'] for e in tree['effects']) else 1}


def prompts(graph_dir: pathlib.Path, results: pathlib.Path, out: pathlib.Path) -> None:
    result = analyze.analyse(graph_dir)
    names = {k: v['majority'] for k, v in json.loads((results / 'llm.json').read_text())['name_step']['levels'].items()}
    out.mkdir(parents=True, exist_ok=True)
    runs = pick_runs(result)
    keys = {}
    sizes = {}
    questions = '\n'.join(f'Q{i + 1}. {q}\n' + '\n'.join(f'   {chr(65 + j)}. {o}' for j, o in enumerate(opts)) for i, (_role, q, opts) in enumerate(QUESTIONS))
    for run_id, (tree, reference) in runs.items():
        keys[run_id] = {**computed_keys(tree), **HAND_KEYS[run_id]}
        for variant, level_names in (('B', B_LEVELS), ('A', A_LEVELS), ('A+', A_LEVELS)):
            for level in level_names:
                if variant == 'B':
                    shown, ref = render_b(tree, level, names), render_b(reference, level, names)
                else:
                    shown, ref = render_a(tree, level, variant == 'A+'), render_a(reference, level, variant == 'A+')
                sizes[f'{variant}|{level}|{run_id}'] = len(shown)
                text = (
                    'You are reading a description of one run of a job queue system. A client program submits or cancels jobs; '
                    'worker processes take jobs and complete them. Answer the questions about THE RUN using only what is written '
                    'below. If the description does not contain the answer, choose the "cannot tell" option; do not guess.\n\n'
                    f'## The run, described at the "{level}" level\n\n' + '\n'.join(shown) +
                    '\n\n## For comparison: a clean run of the same scenario at the same level\n\n' + '\n'.join(ref) +
                    f'\n\n## Questions about THE RUN\n\n{questions}\n\n'
                    'Answer with exactly one line of JSON: {"Q1": "A", "Q2": "B", ...} for all seven questions.'
                )
                (out / f'{variant.replace("+", "plus")}--{level}--{run_id}.md').write_text(text)
    (out / 'keys.json').write_text(json.dumps({'keys': keys, 'sizes': sizes}, indent=1))
    print(keys)
    print({k: v for k, v in sizes.items() if k.endswith('witness')})


def score(out: pathlib.Path, results: pathlib.Path) -> None:
    meta = json.loads((out / 'keys.json').read_text())
    table: dict[str, dict[str, dict[str, list[int]]]] = defaultdict(lambda: defaultdict(lambda: defaultdict(lambda: [0, 0, 0])))
    raw = {}
    for path in sorted((out / 'answers').glob('*.json')):
        variant, level, rest = path.stem.split('--')
        run_id = rest.split('.')[0]
        try:
            text = json.loads(path.read_text())['result']
            given = json.loads(re.search(r'\{[^{}]*\}', text, re.S).group(0))  # type: ignore[union-attr]
        except Exception:
            continue
        raw[path.stem] = given
        for i, (role, _q, opts) in enumerate(QUESTIONS):
            letter = str(given.get(f'Q{i + 1}', '')).strip()[:1].upper()
            choice = ord(letter) - 65 if letter else -1
            key = meta['keys'][run_id][str(i)]
            cell = table[variant.replace('plus', '+')][level][role]
            cell[0 if choice == key else 1 if choice == len(opts) - 1 else 2] += 1
    summary = {v: {lv: {r: {'right': c[0], 'cannot_tell': c[1], 'wrong': c[2]} for r, c in roles.items()} for lv, roles in lvls.items()} for v, lvls in table.items()}
    (results / 'reader.json').write_text(json.dumps({'summary': summary, 'sizes': meta['sizes'], 'keys': meta['keys'], 'answers': raw}, indent=1, sort_keys=True))
    for variant, order in (('B', B_LEVELS), ('A', A_LEVELS), ('A+', A_LEVELS)):
        for level in order:
            cells = summary.get(variant, {}).get(level, {})
            lines = [meta['sizes'][f'{variant}|{level}|{r}'] for r in ('witness', 'cancel', 'clean')]
            print(f'{variant:2s} {level:11s} lines {lines}', ' '.join(f'{r}: {c["right"]}/{c["right"] + c["cannot_tell"] + c["wrong"]} right, {c["wrong"]} wrong' for r, c in sorted(cells.items())))


if __name__ == '__main__':
    if sys.argv[1] == 'prompts':
        prompts(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), pathlib.Path(sys.argv[4]))
    else:
        score(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
