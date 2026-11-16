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
| `results/llm-answers/` | The twelve raw answers |

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

The level analysis is deterministic. The LLM parts are not; the answers used are kept.

## Shortcuts

- A move closes at every write. Closing at the end of an atomic group is the alternative, not tried.
- An outside effect is detected through the job-body counter of the harness; in Mental it would be a send on an external port.
- Design A is one reading of the specification's ladder onto a library with one store; its *mechanisms* level is not implemented.
- The reader is an LLM. It tests whether an answer is present at a level, not whether a person finds it.
