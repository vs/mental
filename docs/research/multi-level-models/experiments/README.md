# Multi-level model experiments on rq

Research code behind section 8 of the [report](../report.md). It reuses the harness, scenarios and oracle of the [abstraction-function experiments](../../abstraction-function/experiments/README.md), whose harness now also records, for every step, the concrete keys it touches, the rq functions on the call stack and whether a transaction was rejected.

## Layout

| Path | What it does |
|---|---|
| `mlexp/levels.py` | Cuts a run into effect, move, activation, goal and outcome by closing rules; marks the causal chain; reads the specification's ladder onto the same run |
| `mlexp/analyze.py` | Grain, support share, nesting, placement of anomalies, explaining cut, step-class stability across versions |
| `mlexp/llm_tasks.py` | Builds the `name-step` and free-form ladder prompts; scores the answers |
| `mlexp/reader.py` | Renders runs per level and variant, builds the stand-in reader prompts, scores the answers |
| `results/levels.json` | Output of `analyze.py` (E1, E4, E6, the witness) |
| `results/llm.json` | Naming stability and free-form ladders (E2, design C) |
| `results/reader.json` | Reader test (E3), with every answer |
| `results/naming-sheet.json`, `prompt-name.md`, `witness.txt`, `b-partitions.json` | Inputs given to the LLM, and design B's cut of the witness |
| `results/llm-answers/` | The raw answers: twelve of the first experiment, six `name-head-*` of L3 |
| `mlexp/strategies.py` | Experiments L2 to L5 of the [ladder strategies](../../../ladder-strategies.md#8-experiments): move rules, naming from the head, object-centric outcomes, merge tests |
| `results/strategies.json` | L2, L4 and L5 |
| `results/strategies-naming.json`, `naming-sheet-head.json` | L3: the sheet with heads, agreement and name-truth failures for both naming variants |
| `results/strategies-reader.json` | L2's stand-in reader at the move level under each move rule, with every answer |

## Reproduce

Set up the environment and the rq checkouts as in the abstraction-function README, then:

```sh
PY=venv/bin/python
AF=../../abstraction-function/experiments
for v in v2.12 v3.0 v2.0; do for b in 0 2; do $PY -I $AF/run.py explore --rq src/rq-$v --out graphs/$v --bound $b; done; done
$PY -I mlexp/analyze.py graphs/v2.12 results graphs/v3.0 graphs/v2.0
$PY -I mlexp/llm_tasks.py prompts graphs/v2.12 src/rq-v2.12 llm
# run llm/prompt-name.md and llm/prompt-ladder.md through `claude -p` (flags as in the
# abstraction-function notes), 3 samples each for opus and sonnet, into llm/answers/{name,ladder}-<model>-<n>.json
$PY -I mlexp/llm_tasks.py score llm results
$PY -I mlexp/reader.py prompts graphs/v2.12 results reader
# run every reader/*.md through `claude -p --model sonnet` twice, into reader/answers/<name>.<n>.json
$PY -I mlexp/reader.py score reader results
```

For the ladder strategy experiments (2026-11-17):

```sh
$PY -I mlexp/strategies.py structure graphs/v2.12 results graphs/v3.0 graphs/v2.0      # L2, L4, L5
$PY -I mlexp/strategies.py name-prompt graphs/v2.12 src/rq-v2.12 l3
# run l3/prompt-name-head.md through `claude -p --output-format json` (flags as above), 3 samples each
# for opus and sonnet, into l3/answers/name-head-<model>-<n>.json
$PY -I mlexp/strategies.py name-score l3 results/llm-answers results                    # L3
$PY -I mlexp/strategies.py reader-prompts graphs/v2.12 l2reader
# run every l2reader/*.md through `claude -p --model sonnet` twice, into l2reader/answers/<name>.<n>.json
$PY -I mlexp/strategies.py reader-score l2reader results
```

`levels.build` and `analyze.analyse` take the options `move_close` and `outcomes`; their defaults give the results of the first experiment byte for byte.

The level analysis is deterministic. The LLM parts are not; the answers used are kept.

## Shortcuts

- A move closes at every write by default. The alternatives `atomic-group` and `write-run` are options since L2. The harness records every pipeline as `EXEC[…]`, transactional or not, so `atomic-group` ends an optimistic transaction at the first pipeline that writes.
- An outside effect is detected through the job-body counter of the harness; in Mental it would be a send on an external port.
- Design A is one reading of the specification's ladder onto a library with one store; its *mechanisms* level is not implemented.
- The reader is an LLM. It tests whether an answer is present at a level, not whether a person finds it.
