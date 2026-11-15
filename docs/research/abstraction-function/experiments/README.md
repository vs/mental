# Abstraction-function experiments on rq

Research code, not product code: it backs the [report](../report.md) and the [experiment notes](../notes/experiments.md). It runs real [rq](https://github.com/rq/rq) code against an in-memory Redis (fakeredis), with every Redis command or transaction as a scheduling point. It explores interleavings, then scores abstraction functions on the resulting state graphs.

## Layout

| Path | What it does |
|---|---|
| `run.py` | Entry point: `explore`, `observe`, `probe`, `evaluate`, `compare` |
| `afexp/harness.py` | Gate that runs one actor's Redis step at a time; frozen clock; normalised snapshots |
| `afexp/scenarios.py` | Eight scenarios: four for training, four held out |
| `afexp/explore.py` | Preemption-bounded schedule enumeration; merged state graph |
| `afexp/afs.py` | Snapshot-only abstractions: identity, status-only, membership, read-projection, the reference oracle and its documented invariant |
| `afexp/observers.py` | Observer abstraction (the library's own query methods, found and purity-checked automatically) |
| `afexp/probes.py` | Behavioural abstraction (states equal when no probe sequence tells them apart) |
| `afexp/llm.py` | Prompt for, and loader of, LLM-proposed abstraction functions |
| `afexp/own.py` | Our approach: select named observables, then mine the representation invariant |
| `afexp/evaluate.py` | Pair check, oracle, determinism, agreement metrics |
| `afexp/mutants.py` | 20 seeded changes to rq 2.12, labelled harmful or benign in advance |
| `afexp/compare.py` | Visibility of mutants and stability across rq versions |
| `afjobs.py` | Job bodies; counts their runs as the one outside effect |
| `pipeline.sh` | Graphs plus library-backed abstractions for one target |
| `results/` | The JSON outputs that the notes and report cite |

## Reproduce

Pinned in `requirements.txt`. Python 3.12 (fakeredis 2.40 did not need 3.15; we did not try it).

```sh
uv venv --python 3.12 venv && VIRTUAL_ENV=venv uv pip install -r requirements.txt
git clone https://github.com/rq/rq.git src/rq
for t in v2.12 v3.0 v2.0 v1.16.2; do git -C src/rq worktree add --detach ../rq-$t $t; done
PY=venv/bin/python
./pipeline.sh $PY src/rq-v2.12 graphs/v2.12 src/rq-v2.12
$PY -I afexp/mutants.py src/rq-v2.12 src/mut
for m in src/mut/*; do ./pipeline.sh $PY $m graphs/mut/$(basename $m) src/rq-v2.12; done
for v in v3.0 v2.0 v1.16.2; do ./pipeline.sh $PY src/rq-$v graphs/$v src/rq-v2.12 rederive; done
# LLM answers: build prompts with afexp/llm.py build_prompt, run them through `claude -p`
# (flags in notes/experiments.md), store the JSON answers in llm/use/
$PY -I run.py evaluate --graphs graphs/v2.12 --out results/eval-v2.12.json --llm llm/use
$PY -I run.py compare --base graphs/v2.12 --mutants graphs/mut --versions graphs/v3.0 graphs/v2.0 graphs/v1.16.2 --llm llm/use --out results/compare.json
```

Exploration is deterministic: the clock is frozen, ids are derived from actor names, and normalised snapshots drop clock values, so the graphs are byte-stable across runs. The LLM answers are not; the eight answers used are kept in `results/llm-answers/`.

## Shortcuts

- The worker's periodic registry maintenance is skipped (`scenarios.py`); a dedicated scenario covers registry cleanup instead.
- Bounds: two actors per scenario, at most two preemptions. Results hold within these bounds only.
- One queue, one or two jobs. Several queues or many jobs would grow the state space, not change the method.
