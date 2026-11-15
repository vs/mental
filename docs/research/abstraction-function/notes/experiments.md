# Experiments: building an abstraction function for rq

All numbers come from `experiments/results/*.json`, produced by the code in [`experiments/`](../experiments/README.md) on 2026-11-15.

## The subject

[rq](https://github.com/rq/rq) (Redis Queue), version 2.12 (2026-08-30): a well-known Python job queue of about 13,000 lines, 6,300 in the core (`job.py`, `queue.py`, `registry.py`, `worker/base.py`). It is close to Mental's first target, report-jobs, and it keeps **all** its state in Redis, so the lower level can be observed exactly. A job's business state is spread over several redundant representations:

- the `status` field of the job hash;
- membership in the queue list `rq:queue:<q>` and in six registries (`rq:wip`, `rq:finished`, `rq:failed`, `rq:deferred`, `rq:scheduled`, `rq:canceled`);
- result streams, execution records, an intermediate queue, worker hashes and counters.

The cross-version runs also use rq 3.0 (2026-10-28), 2.0 (2024-10-28) and 1.16.2 (2024-05-01).

## The lower model

Mental will build its lower level by extraction and an LLM. To test the abstraction function on its own, independent of extraction quality, we used the real code as the lower model:

- **Execution.** Real rq code runs in threads against an in-memory Redis (fakeredis 2.40). Every Redis command, and every MULTI/EXEC transaction or Lua script, waits at a gate until a scheduler grants it. One scheduler step is one atomic Redis step, the grain at which Redis serialises work.
- **Lower state.** The whole keyspace, snapshotted after every step. The clock is frozen and execution ids are derived from actor names. Clock values, compressed blobs, base64 blobs, stream ids and tracebacks are normalised. The number of times each job body ran is added as the one outside effect.
- **Exploration.** Schedules are enumerated by re-execution, as stateless model checkers do, with at most two preemptions (switching away from an actor in the middle of an operation). Bound 0 gives exactly the runs in which every business operation is atomic.
- **Upper steps.** The business operations are enqueue, cancel, delete, requeue, registry cleanup and a worker's *take* and *complete*. The worker's job is split at the start of the job body, because a running job is a legitimate business state that other operations may observe. Without the split, a worker legitimately finding the queue empty while another worker runs the job was labelled an anomaly (96 false anomalies in `fail-retry`).

Eight scenarios with two actors each:

| Scenario | Actors | Split |
|---|---|---|
| work-ok | enqueue ∥ worker | train |
| fail-retry | two workers on a failing job with one retry | train |
| requeue-vs-work | requeue a failed job ∥ worker | train |
| dependency | enqueue a dependent job ∥ worker taking two jobs | train |
| cancel-vs-work | cancel ∥ worker | held out |
| cleanup-vs-work | started-registry cleanup (as after a missed heartbeat) ∥ worker | held out |
| two-workers | two workers, one job | held out |
| delete-vs-work | delete ∥ worker | held out |

Bound 2 gives 2,097 runs over the eight scenarios. Derivation algorithms that learn from runs see only the **sequential** (bound-0) runs, and never an anomaly label.

## The oracle

Written from rq's user documentation (`docs/docs/jobs.md`, `docs/docs/job_registries.md` in the rq tree) before any result was seen:

- **Reference abstraction.** Per job: status, which of the queue and six registries list it, and the type of its latest result. Workers, executions, heartbeats, TTLs and the intermediate queue are implementation detail.
- **Documented invariant.** A job is listed in exactly the queue or registry that matches its status, and a deleted job is listed nowhere.
- **Anomaly.** A concurrent run is an anomaly when its final state breaks the documented invariant, or when the final reference state is not the final state of any sequential run of the same scenario (final-state serialisability).

## The arms

| Arm | How the abstraction function is obtained |
|---|---|
| identity | the whole normalised snapshot (baseline) |
| status-only | the enum field that code branches on: Mental's current rule for state variables (spec 4.6) |
| read-projection | cone of influence: only the locations some code path reads (from training runs) |
| membership | shape: for each job, which key families hold it (TVLA-style canonical abstraction) |
| observer | the library's own query API: every public property and zero-argument method of Job, Queue and the registries, kept if it never changes state (ADABU-style), values abstracted |
| behavioural | states equal when every probe sequence of up to two rq operations gives the same outputs and effects (Nerode congruence, as L* approximates it) |
| LLM | an LLM writes `abstract(snap)` and `invariants(snap)` from the rq sources, example snapshots and (in one variant) the user docs: 3 samples of Opus with docs, 2 without, 3 of Sonnet with docs |
| ours | selection of named business observables plus a mined representation invariant, optionally plus LLM intent invariants (below) |

The LLM was called through `claude -p --model <m> --tools "" --strict-mcp-config --setting-sources project`. That keeps the coding agent's own context out of the prompt: 617 tokens of overhead against 167,000 without those flags. The prompts are about 45,000 tokens and the answers 170 to 480 lines of Python. All eight samples together cost $5.30.

### Our approach

1. **Observables first.** The business observables are the observer methods that the library's user documentation names (`get_status`, `is_finished`, `get_job_ids`, `count`, `latest_result`, …): 46 of 113 candidates.
2. **Selection.** Greedily add the named candidate (an observer or a shape variable) that removes the most conflicts per bit of its domain, then prune. A conflict is either nondeterminism of the upper model (same abstract state, same operation, different abstract result or output) or an observable that the chosen variables fail to determine, measured on quiescent states of sequential runs.
3. **Representation invariant.** Every observable or shape variable left out that the chosen variables determine on quiescent sequential states, with at most 8 values, becomes a clause "y = f(chosen)". It is checked at every quiescent state of a concurrent run. The pair check (projection check) runs on the chosen variables.
4. **Intent invariants (optional).** The LLM's invariants are added. A Houdini filter can drop the clauses that sequential runs refute.

The derived abstraction is data (variable names and clause tables), applied unchanged to mutants and to other versions.

### Checks and metrics

- **Pair check.** U is every abstract transition (a, operation, a') of sequential runs. A concurrent run is flagged when it makes an abstract transition outside U, or, for arms with an invariant, passes a quiescent state that breaks it. Precision, recall and F1 are computed against the oracle over runs.
- **det.** The share of (abstract state, operation) pairs in sequential runs with exactly one (abstract result, output): whether the upper model can run on its own.
- **Homogeneity / completeness.** Partition agreement with the reference abstraction over all states.
- **Mutation visibility.** 20 seeded changes to rq 2.12, labelled in advance: 10 harmful (user-visible bugs in job handling) and 10 benign (bookkeeping, TTLs, key layout, extra fields, an atomic reorder, an extra read, a log text). A change is *shown* when the visible set (abstract transitions, final states and invariant-breaking states) differs from the original's.
- **Version stability.** Jaccard similarity of visible sets between rq 2.12 and 3.0, 2.0 and 1.16.2.

## Controls

- **Determinism.** Exploring rq 2.12 twice gives identical node sets in all eight scenarios.
- **Null change.** The `log-text` mutant (changes a log string only) is invisible even through the identity abstraction. An earlier run failed this control: base64-encoded tracebacks in result streams carried file paths, so a copy of the same code in another directory looked different. Base64 blobs are now normalised, and every target was re-explored with the final harness.

## The anomalies in rq

Anomalies per scenario at bound 2 (`results/eval-v2.12.json`, `oracle`):

| Scenario | Runs | Anomalies | Not serialisable | Invariant broken | Sequential runs breaking the invariant |
|---|---|---|---|---|---|
| cancel-vs-work | 410 | 341 | 137 | 331 | 1 of 3 |
| delete-vs-work | 446 | 139 | 139 | 139 | 0 of 3 |
| cleanup-vs-work | 64 | 54 | 28 | 54 | 1 of 3 |
| dependency | 423 | 20 | 20 | 20 | 0 of 4 |
| work-ok, fail-retry, requeue-vs-work, two-workers | 754 | 0 | 0 | 0 | 0 |

What they are:

- **cancel-vs-work.** `Job.cancel` reads the status and the registries outside its transaction. A job canceled while a worker completes it ends in both the canceled and finished registries, or finished with the cancel lost. Canceling a running job at the take/complete boundary breaks the invariant even in a *sequential* run: the job completes anyway and is listed twice.
- **cleanup-vs-work.** Started-registry cleanup after a missed heartbeat marks a running job failed. The worker then finishes it: the job ran once and succeeded, but it is listed as both failed and finished. This also happens sequentially at the boundary.
- **delete-vs-work.** Deleting a job while a worker holds it leaves registry entries for a job hash that no longer exists, or an id in the intermediate queue.
- **dependency (an rq bug).** `handle_job_success` runs inside a WATCH retry loop. If a dependent job is enqueued between WATCH and EXEC, the transaction is retried. But the first attempt already ran `cleanup_execution`, which set `worker.execution = None` in Python, so the retried transaction no longer contains `DEL rq:execution:*`, `ZREM rq:wip:*` or `ZREM rq:executions:*`. The finished job stays listed as started (shortest witness: 42 steps, 2 preemptions). Once its heartbeat score expires, `StartedJobRegistry.cleanup` would treat it as abandoned; that step was read from the code, not run. Present in 2.0 and 2.12; absent in 1.16.2 (no executions) and 3.0 (`cleanup_execution` takes the execution as an argument). Not reported upstream.

The anomalies that sequential runs also produce matter for the design. A check that compares concurrent runs with sequential runs of the same code (the pair check) cannot see them, because the sequential reference already contains them. Only an invariant stated from intent can.

## Results on rq 2.12

Pair check against the oracle, all eight scenarios (2,097 runs, 554 anomalies), from `results/eval-v2.12.md`:

| Arm | Abstract states | Findings | Precision | Recall | F1 | Held-out F1 | det |
|---|---|---|---|---|---|---|---|
| identity | 688 | 602 | 0.31 | 0.79 | 0.44 | 0.55 | 0.94 |
| status-only (Mental's current rule) | 50 | 13 | 0.56 | 0.16 | 0.25 | 0.26 | 0.97 |
| read-projection (cone of influence) | 86 | 22 | 0.45 | 0.17 | 0.25 | 0.27 | 0.96 |
| behavioural, depth 1 | 44 | 14 | 0.55 | 0.15 | 0.23 | 0.26 | 0.88 |
| behavioural, depth 2 | 71 | 36 | 0.64 | 0.35 | 0.45 | 0.45 | 1.00 |
| LLM, 8 samples | 212–502 | 141–392 | 0.32–0.33 | 0.79 | 0.45–0.47 | 0.56–0.58 | 0.94 |
| observer, syntactic purity | 121 | 48 | 0.69 | 0.56 | 0.62 | 0.64 | 0.96 |
| observer, semantic purity | 134 | 65 | 0.72 | 0.79 | 0.75 | 0.78 | 0.96 |
| membership (shape) | 97 | 38 | 0.77 | 0.79 | 0.78 | 0.81 | 0.97 |
| reference (oracle's own view) | 98 | 45 | 0.76 | 0.79 | 0.77 | 0.80 | 0.97 |
| reference + documented invariant | 98 | 59 | 0.80 | 0.98 | 0.88 | 0.91 | 0.97 |
| ours, selection only (all sequential runs) | 57 | 15 | 0.83 | 0.16 | 0.27 | 0.28 | 1.00 |
| ours (selection + mined invariant), train scenarios | 54 | 22 | 0.94 | 0.71 | 0.81 | 0.81 | 1.00 |
| ours, all sequential runs (held-out column not held out: derived from all eight scenarios' sequential runs) | 57 | 24 | 0.94 | 0.76 | 0.84 | 0.84 | 1.00 |
| ours + LLM intent invariants, Houdini on train | 57 | 28–34 | 0.84–0.95 | 0.88–0.98 | 0.91–0.97 | 0.91–0.97 | 1.00 |
| ours + LLM intent invariants, unfiltered | 57 | 28–35 | 0.64–0.95 | 0.88–0.98 | 0.78–0.97 | 0.91–0.97 | 1.00 |

"Findings" is the number of distinct abstract transitions or broken-invariant states a person would have to read.

Our approach's derived abstraction (all sequential runs): 4 chosen variables out of 113 candidates (`Job.get_status`, `Job.results`, `Job.func_name`, `CanceledJobRegistry.count`); conflicts fall from 95 to 3. It also has 39 invariant clauses, over the registries' `get_job_ids` and `count`, `Queue.job_ids`, `Job.is_finished`, `Job.latest_result` and the shape of the queue, the intermediate queue, executions and the six registries.

Ablations:

- **Without the documentation filter** (every observer, 101 of them, counts as an observable; training scenarios), selection picks the undocumented near-identity `Job.to_dict`. F1 rises from 0.81 to 0.84, but the abstraction is no longer in the user's words: 70 abstract states instead of 54, and det falls from 1.00 to 0.96.
- **LLM variables as observables** (318 atomic variables from the eight samples, flattened) make selection fail to converge (conflicts 902 → 413). F1 drops to 0.70, because the LLM's vocabulary includes workers, counters and executions.
- **LLM invariants alone** (no pair check): precision 0.65–1.00, recall 0.37–0.98, F1 0.54–0.84. They are good at spotting inconsistent registries, but vary widely across samples.
- **Houdini filter** (drop every intent clause a sequential run refutes). Filtering on the training scenarios removed one wrong clause, Sonnet's "a queued job is not running (no worker, not ended)": rq keeps `worker_name` and `ended_at` when it requeues a job for retry. That raised that sample from F1 0.78 to 0.91 and changed nothing else. Filtering on *all* scenarios removed the true invariants as well, because cancel-vs-work and cleanup-vs-work break them sequentially. Seven samples fell to recall 0.76 and F1 0.84 (opus-docs-1 to 0.765 and 0.845), the score without any intent invariant.

### LLM proposals in detail

The name-stability and partition-agreement figures come from a one-off computation in the session (flattening each sample's `abstract()` output over all bound-2 snapshots); they are not stored in `results/`.


- **Coverage.** Every sample covered jobs, queues and workers, some also results and executions, with 30 to 50 atomic variables and 16 to 30 invariants. All 8 ran without error on every snapshot.
- **Name stability.** The pairwise Jaccard overlap of variable names between samples has a median of 0.54 (minimum 0.16, maximum 0.79). The induced partitions agree better (pairwise V-measure: median 0.81, minimum 0.79).
- **Agreement with the reference partition** (V-measure): Opus 0.61–0.78, Sonnet 0.79–0.80. Opus wrote longer functions (370–480 lines) with more implementation detail; Sonnet wrote 170–320 lines.
- **Docs or no docs.** With or without the user docs, Opus's pair-check F1 was about the same (0.45–0.47).

## Mutation visibility

20 seeded changes (`results/compare.md`):

| Arm | Harmful shown (of 10) | Benign hidden (of 10) | Harmful missed / benign shown |
|---|---|---|---|
| identity | 10 | 5 | shows execution-key layout, success counter, worker state word, two extra fields |
| status-only | 5 | 10 | misses abandoned-job-not-recorded, cancel/success/requeue/delete registry bugs |
| read-projection | 7 | 10 | misses cancel, success and requeue registry bugs |
| membership | 9 | 10 | misses failure-marks-finished (status only) |
| behavioural, depth 2 | 8 | 9 | misses cancel and success registry bugs; shows execution-key layout |
| observer, semantic | 10 | 9 | shows execution-key layout |
| LLM, Opus | 10 | 7 | shows success counter, worker state word, execution-key layout |
| LLM, Sonnet | 10 | 8 | shows success counter, worker state word |
| ours, selection only (train scenarios; 7 for all sequential runs) | 6 | 10 | misses four registry bugs |
| **ours** | **10** | **10** | none |
| reference (and + invariant) | 10 | 10 | none |

## Stability across versions

The abstraction derived on rq 2.12, applied unchanged to other versions' snapshots. Pair-check F1 against each version's own oracle (`results/versions.md`):

| Arm | 3.0 | 2.0 | 1.16.2 |
|---|---|---|---|
| identity | 0.23 | 0.46 | 0.45 |
| status-only | 0.22 | 0.29 | 0.32 |
| read-projection | 0.19 | 0.28 | 0.31 |
| behavioural, depth 2 | 0.34 | 0.47 | 0.55 |
| observer (2.12 code) | 0.62 | 0.76 | 0.83 |
| membership | 0.64 | 0.78 | 0.82 |
| reference | 0.64 | 0.78 | 0.83 |
| reference + documented invariant | 0.78 | 0.88 | 0.91 |
| ours, train scenarios | 0.83 | 0.85 | 0.82 |
| ours, all sequential runs | 0.78 | 0.88 | 0.83 |

Re-deriving the observers from each version's own code gave the same F1 as the 2.12 observers (0.62, 0.76, 0.83). The Jaccard similarity of visible sets between 2.12 and other versions is in `results/compare.md`. Even the reference sees only 0.55 similarity with 3.0, so business behaviour did change between versions, and that measure mixes real change with abstraction noise. The per-version F1 above is the cleaner signal.

## Threats to validity

- **One library, small bounds.** Two actors, at most two preemptions, one queue, at most two jobs, eight scenarios. Bound 2 misses nothing we know of here, but that is unproven.
- **The oracle is a choice.** Leaving workers out of the business state penalises the LLM arms, which include them. The oracle was written before any result, from the jobs and registries docs.
- **The real code is the lower level.** Mental's lower level will be extracted and partly LLM-written, so its errors add to the abstraction's.
- **Documentation is the intent signal.** Our approach uses rq's user documentation to pick observables. The specification's open question, "How reliable a business level is when the project has no docs", applies directly. The LLM-proposed observables we tried instead did worse.
- **Few samples.** Eight LLM samples and two models.
