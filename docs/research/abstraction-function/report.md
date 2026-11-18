# The Abstraction Function Is a Small Selection of Named Observables Plus a Representation Invariant, and the Invariant Is What Sees the Races

The abstraction function is the rule that computes upper-level state from lower-level state. The [earlier report](../abstraction-levels/report.md) concluded that every level needs one, written down and checked. This report asks how to build it. We surveyed how other projects represent, obtain and trust theirs, and then ran an experiment: eight ways of obtaining an abstraction function for the job queue library [rq](https://github.com/rq/rq), scored on 2,097 explored runs of the real code, 554 of them anomalies, on 20 seeded code changes and on three other rq versions.

The result changes what we think the abstraction function is. A business fact in real code is stored several times: an rq job's state is in its `status` field, in its membership of a queue or registry, in its results. A good abstraction function for *running* the upper level picks the smallest set of variables that makes the upper level deterministic, and so it drops the redundant copies. But races are exactly the runs in which the copies disagree, so the minimal function is blind to them. The redundancy has to be kept, as a **representation invariant**: for each copy left out, a clause saying what it must be, given the selected variables. The function projects; the invariant guards. Neither is enough alone.

Main conclusions:

- **Mental's current rule for state variables is blind to most races.** Taking only the enum field that code branches on gives recall 0.16. Taking everything the code reads (cone of influence) gives 0.17.
- **Selection plus a mined representation invariant works.** Four variables chosen from 113 candidates, with 39 invariant clauses, give precision 0.94, recall 0.76 and F1 0.84, with a fully deterministic upper level (det 1.00), all 10 harmful seeded changes shown and all 10 benign ones hidden.
- **Someone must say what is observable, and the user documentation said it best.** With only operation outputs observable, the computed abstraction is deterministic but nearly trivial (recall 0.35). Observables taken from LLM proposals made selection fail.
- **LLM-written abstraction functions are too fine and unstably named, but LLM-written invariants are valuable.** Used directly, the eight LLM functions score F1 0.45–0.47. Their invariants, added to our abstraction and filtered against the training scenarios' sequential runs, give the best results: F1 0.91–0.97 (unfiltered, one of the eight samples drops to 0.78).
- **Some anomalies happen in sequential runs too, and no check against the code's own behaviour can see them.** Only an invariant stated from intent can. Filtering intent invariants against all sequential runs (Houdini) throws exactly those away. A refuted intent invariant must be triaged, never dropped silently.
- **The derived abstraction is data and survives version changes.** Applied unchanged to rq 3.0, 2.0 and 1.16.2 it scores F1 0.82–0.85, against 0.22–0.32 for the status-only rule.
- **The method found a real bug in rq**, not only seeded ones. It has not been reported upstream.

All of this comes from one library within small bounds. The limits are in [Threats to validity](#9-threats-to-validity-and-open-questions).

## How this report was made

| Source | Holds |
|---|---|
| [notes/survey.md](notes/survey.md) | How other projects represent, obtain and trust the abstraction function, with evidence tags and links |
| [notes/experiments.md](notes/experiments.md) | Experiment design, controls and all results |
| [experiments/](experiments/README.md) | The code, and the raw results in `experiments/results/` |

Every number below is taken from `experiments/results/` or from the experiment notes. Design conclusions for Mental are inferences and are marked as such. Terms follow the [glossary](../../glossary.md). New terms are defined where they first appear: *observer*, *business observable*, *representation invariant*, *intent invariant*, *quiescent state*.

## 1. Each project in one line

Condensed from the [survey](notes/survey.md), which has all 22 rows.

| Project | Its abstraction function in one line |
|---|---|
| Hoare 1972, Liskov–Guttag | A hand-written function from representation to abstract value, defined only where a representation invariant holds. |
| TLA+ refinement mapping | A substitution: each upper variable is an expression over lower variables. |
| Event-B gluing invariant | A relation between upper and lower variables, discharged as proof obligations per event. |
| Verus and Creusot `View` | A method per type that maps the executable value to a mathematical one, written once and reused by every spec. |
| Dafny ghost state | Ghost fields kept in step with real fields, plus a `Valid()` predicate as the representation invariant. |
| IronFleet | Two hand-written mappings (implementation to protocol, protocol to spec), each with a per-step obligation. |
| seL4, AutoCorres | A hand-proved state relation per refinement; AutoCorres computes heap and word abstractions with a proof per run. |
| Predicate abstraction, CEGAR | The valuation of a set of predicates, grown from spurious counterexamples. |
| TVLA shape analysis | Canonical abstraction: heap cells merged when they agree on every unary predicate. |
| Bisimulation minimisation | The coarsest partition that preserves observable behaviour, computed. |
| Automata learning (L*, Tomte) | States separated by distinguishing experiments; Tomte refines when the learned model turns nondeterministic. |
| ADABU specification mining | The return values of side-effect-free inspector methods, abstracted. |
| Daikon | No function; mined invariants over observed variables. |
| Elle, Jepsen, QuickCheck state machines | No state function; levels are related through observations (outputs, read results). |
| Debuggers (DWARF) | Compiler-emitted location expressions from machine state to source variables, partial ("optimized out"). |
| Database views, CQRS projections | A query or a fold over an event log that computes the read model. |
| Process mining | Rules or learned models that lift low-level events to activities. |
| LLM pipelines (Specula, ProtocolGPT, SysMoBench) | An LLM writes the model or mapping from code; a tool or trace checks it. |

## 2. What the abstraction function is for Mental, and why it is hard

In Mental, a level is a complete description of the system at one height, and a refinement links two adjacent levels by an abstraction function on state and a step map on steps ([specification](../../specification.md) section 5, P5). The pair check runs the lower level, projects every state upward through the abstraction function, and requires that the projected run is one the upper level allows. Everything the upper level can show, and every race it can report "at the level where the user thinks", passes through this function. If the function is wrong, the business level is either noise or blind.

**The problem, on rq.** rq keeps all its state in Redis. The business state of one job is not in one place. It is spread over redundant representations:

- the `status` field of the job hash (`queued`, `started`, `finished`, `failed`, …);
- membership in the queue list `rq:queue:<q>` and in six registries (`rq:wip`, `rq:finished`, `rq:failed`, `rq:deferred`, `rq:scheduled`, `rq:canceled`);
- result streams, execution records, an intermediate queue, worker hashes and counters.

In a correct sequential run these agree: a finished job has status `finished`, is listed in the finished registry and nowhere else, and has a result. The library's documentation states this as how the system works.

**An example.** One anomalous run of the scenario *cancel ∥ worker*, schematically (`Job.cancel` reads the status and the registries outside its transaction):

| Step | Actor | What happens | `status` | Listed in |
|---|---|---|---|---|
| 1 | worker | takes the job | `started` | started registry |
| 2 | cancel | reads status and registries | `started` | started registry |
| 3 | worker | completes the job | `finished` | finished registry |
| 4 | cancel | writes, from what it read in step 2 | | finished **and** canceled registries |

The job ran and succeeded, and it is also listed as canceled. An abstraction function that keeps only `status` sees a job whose status is a legal one, reached by legal moves. The anomaly is not in any one variable. It is in the disagreement between two representations of the same fact.

**Three tensions.** Any way of building the function has to settle three things.

1. **Too fine against too coarse.** A function that keeps everything (the identity) reports 602 distinct findings at precision 0.31, shows 5 of 10 benign code changes as business changes, and falls to F1 0.23 on the next major version. A function that keeps only `status` reports 13 findings at recall 0.16. Fine means false alarms and instability; coarse means invisible races.
2. **Named against computed.** Computed abstractions (bisimulation quotients, learned automata) are correct for the chosen observations and have no names, so nobody can read them. Named abstractions (written by a person or an LLM) can be read and carry no guarantee.
3. **Derived from the code against derived from intent.** What the code reads and writes is available mechanically, but the code is also the thing under suspicion: if it lists a canceled job as finished, a function derived from the code learns that as normal. Intent (documentation, the glossary, a person) says what should hold, but it is partial and sometimes wrong.

Section 7 proposes one answer to each: select few, guard the rest; choose among named candidates by a computed criterion; take the observables and some invariants from intent, the rest from the code.

## 3. How other projects do it

The [survey](notes/survey.md) has the detail and marks each claim as seen this session, background knowledge, or measured by us. The claims below without a link are background knowledge that was not re-verified.

**Four representations.**

1. **A function in code.** TLA+ mappings, Z retrieve functions, Verus and Creusot [`View`](https://verus-lang.github.io/verus/verusdoc/vstd/view/trait.View.html), [IronFleet](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)'s refinement functions, [ADABU](https://www.st.cs.uni-saarland.de/models/papers/woda-06.php)'s inspectors, database views. It can be evaluated on any lower state, including ones nobody anticipated.
2. **A relation.** Event-B gluing invariants, seL4 state relations, CIVL layer annotations. More expressive, but it cannot be run to produce an upper state; it can only be checked.
3. **A partition computed from behaviour.** Bisimulation quotients, L* congruences, CEGAR predicate sets, [TVLA](https://research.cs.wisc.edu/wpis/abstracts/tr1383r2.abs.html)'s canonical abstraction, [Tomte](https://www.mbsd.cs.ru.nl/publications/papers/fvaan/CEGAR12/FM.pdf). Correct by construction for the chosen observations, and without names.
4. **No state function.** [Elle](https://arxiv.org/abs/2003.10554), Jepsen, QuickCheck and P's monitors relate levels through observations only.

Mental's interpreter must *produce* upper state from lower state to show a run at every level, so it needs the first kind. The experiment borrows the selection criterion from the third.

**Sources and their costs.**

| Source | Examples | Cost and risk |
|---|---|---|
| A person, per type | Liskov, Verus `View`, Dafny, IronFleet | Best names, reused across specs. Expensive: Arm's ISA-Formal function took about 2,500 lines per processor (earlier report). |
| The generator of the lower level | DWARF, Formality guidance files | Free with the output, but partial and buggy: 47 confirmed debug-information bugs in GCC and LLVM ([Li et al.](https://arxiv.org/pdf/2211.09568)). |
| A tool, with a proof | [AutoCorres](https://trustworthy.systems/projects/OLD/autocorres/) | Automatic, but only for fixed patterns (typed heaps, ideal integers). |
| Behaviour | bisimulation, L*, Tomte, CEGAR | No names; preserves exactly what is declared observable. Tomte refines when the abstraction makes the system look [nondeterministic](https://typeset.io/pdf/automata-learning-through-counterexample-guided-abstraction-2a10rgqrzh.pdf). |
| The code's own query API | ADABU, our observer arm | The authors' vocabulary. ADABU's companion study found inspectors in 45% of classes. Purity must be checked. |
| Shape | TVLA, our membership arm | Mechanical; works when entities sit in containers. |
| An LLM | Specula, ProtocolGPT, SysMoBench agents | Fluent names, broad coverage, checking required. [SysMoBench](https://arxiv.org/abs/2509.23130v3) finds agents good on spinlocks and weak on Etcd Raft. [ProtocolGPT](https://arxiv.org/abs/2405.00393) reports over 90% state-machine accuracy on six protocols (a third-party summary, unverified). |

**How it is trusted.** By proof per step, `next_lower(c, c') ⇒ next_upper(AF(c), AF(c'))` or a stutter (seL4, IronFleet, Verus, Event-B; [lecture notes](https://6826.csail.mit.edu/2020/lec/l17.pdf)). By exploration (ProB, FDR, TLC): Mental's pair check belongs here. By validation per output (CompCert's register allocation, Alive2). By conformance testing (ShardStore, QuickCheck).

**Lessons.**

1. Every serious abstraction function ships with a representation invariant. Hoare and Liskov define the function only on representations that satisfy one; Dafny's `Valid()` and seL4's invariants are the same idea.
2. The observables decide the abstraction. Bisimulation, L* and CEGAR all compute the coarsest partition *relative to declared observations*.
3. Author-written query APIs are abstraction functions already. `View` in Verus and inspectors in ADABU say so from verification and from mining.
4. Generator-emitted relations are partial and buggy. An LLM-emitted function should be treated the same way: partial by default, and checked.
5. Computed abstractions have no names, and named ones have no guarantee. Every project surveyed picks one side.
6. Event abstraction is a separate axis. [Process mining](https://link.springer.com/article/10.1007/s41066-020-00226-2) and Elle lift *steps*, not states. In Mental that is the step map.

## 4. The experiment

The design and controls are in [notes/experiments.md](notes/experiments.md); the code is in [experiments/](experiments/README.md).

**Subject.** rq 2.12 (2026-08-30): about 13,000 lines of Python, 6,300 in the core. It is close to Mental's first target, report-jobs, and because all its state is in Redis the lower level can be observed exactly. Cross-version runs use rq 3.0 (2026-10-28), 2.0 (2024-10-28) and 1.16.2 (2024-05-01).

**Lower model.** Mental will build its lower level by extraction and an LLM. To test the abstraction function independently of extraction quality, the real code is the lower model. rq runs in threads against an in-memory Redis (fakeredis 2.40). Every Redis command, MULTI/EXEC transaction or Lua script waits at a gate until a scheduler grants it, so one scheduler step is one atomic Redis step. The lower state is the whole keyspace after each step, with the clock frozen and clock values, blobs, stream ids and tracebacks normalised. The number of times each job body ran is added as the one outside effect. Schedules are enumerated by re-execution with at most two preemptions (switching away from an actor in the middle of an operation). Bound 0 gives exactly the sequential runs, in which every business operation is atomic.

**Upper steps, and the take/complete split.** The business operations are enqueue, cancel, delete, requeue, registry cleanup, and a worker's *take* and *complete*. The worker's operation is split at the start of the job body because a running job is a legitimate business state that other operations may observe. Without the split, a worker that correctly found the queue empty while another worker ran the job was labelled an anomaly: 96 false anomalies in the `fail-retry` scenario. This is a step-map decision, not a state decision, and the oracle could not be written without it.

**Scenarios.** Eight, with two actors each, 2,097 runs at bound 2.

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

Algorithms that learn from runs see only sequential runs, and never an anomaly label.

**Oracle.** Written from rq's user documentation before any result was seen.

- *Reference abstraction:* per job, its status, which of the queue and six registries list it, and the type of its latest result. Workers, executions, heartbeats, TTLs and the intermediate queue are implementation detail.
- *Documented invariant:* a job is listed in exactly the queue or registry that matches its status, and a deleted job is listed nowhere.
- *Anomaly:* a concurrent run whose final state breaks the documented invariant, or whose final reference state is not the final state of any sequential run of the same scenario.

**Arms.**

| Arm | How the abstraction function is obtained |
|---|---|
| identity | the whole normalised snapshot (baseline) |
| status-only | the enum field that code branches on: Mental's current rule for state variables (specification 4.6) |
| read-projection | cone of influence: only the locations some code path reads, from training runs |
| membership | shape: for each job, which key families hold it (as TVLA's canonical abstraction) |
| observer | the library's own query API: every public property and zero-argument method of `Job`, `Queue` and the registries, kept if it never changes state (as ADABU) |
| behavioural | two states are equal when every probe sequence of up to two rq operations gives the same outputs and effects (as L* approximates) |
| LLM | an LLM writes `abstract(snap)` and `invariants(snap)` from the rq sources and example snapshots: 3 samples of Opus with the user docs, 2 of Opus without, 3 of Sonnet with |
| ours | selection of named business observables plus a mined representation invariant, optionally with LLM intent invariants (section 7) |

An **observer** is a public method or property of the library that returns information about state. A **quiescent state** is a state of a run in which no actor is in the middle of an operation.

**Checks and metrics.**

- *Pair check.* U is every abstract transition (a, operation, a') of the sequential runs. A concurrent run is flagged when it makes an abstract transition outside U or, for arms with an invariant, passes a quiescent state that breaks it. Precision, recall and F1 are computed over runs against the oracle.
- *Findings.* The number of distinct abstract transitions or broken-invariant states a person would have to read.
- *det.* The share of (abstract state, operation) pairs in sequential runs with exactly one (abstract result, output). It says whether the upper level can run on its own.
- *Mutation visibility.* 20 seeded changes to rq 2.12, labelled in advance: 10 harmful (user-visible bugs in job handling) and 10 benign (bookkeeping, TTLs, key layout, extra fields, an atomic reorder, an extra read, a log text). A change is *shown* when the set of abstract transitions, final states and invariant-breaking states differs from the original's.
- *Version stability.* The abstraction derived on 2.12 is applied unchanged to another version and scored against that version's own oracle.

**Controls.**

- *Determinism.* Exploring rq 2.12 twice gives identical node sets in all eight scenarios.
- *Null change.* The `log-text` mutant, which changes one log string, is invisible even through the identity abstraction. An earlier run failed this control: base64-encoded tracebacks in result streams carried file paths, so the same code in another directory looked different. Base64 blobs are now normalised and every target was re-explored.

**The anomalies found** (`experiments/results/eval-v2.12.json`, key `oracle`):

| Scenario | Runs | Anomalies | Not serialisable | Invariant broken | Sequential runs breaking the invariant |
|---|---|---|---|---|---|
| cancel-vs-work | 410 | 341 | 137 | 331 | 1 of 3 |
| delete-vs-work | 446 | 139 | 139 | 139 | 0 of 3 |
| cleanup-vs-work | 64 | 54 | 28 | 54 | 1 of 3 |
| dependency | 423 | 20 | 20 | 20 | 0 of 4 |
| the other four | 754 | 0 | 0 | 0 | 0 |

The last column matters later. In cancel-vs-work and cleanup-vs-work, one of the three *sequential* runs already breaks the documented invariant: canceling a running job at the take/complete boundary, or cleaning it up after a missed heartbeat, leaves the job to complete anyway and be listed twice.

## 5. Results

### 5.1 Pair check on rq 2.12

All eight scenarios, 2,097 runs, 554 anomalies (`experiments/results/eval-v2.12.md`).

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
| reference (the oracle's own view) | 98 | 45 | 0.76 | 0.79 | 0.77 | 0.80 | 0.97 |
| reference + documented invariant | 98 | 59 | 0.80 | 0.98 | 0.88 | 0.91 | 0.97 |
| ours, selection only | 57 | 15 | 0.83 | 0.16 | 0.27 | 0.28 | 1.00 |
| ours (selection + mined invariant), train scenarios | 54 | 22 | 0.94 | 0.71 | 0.81 | 0.81 | 1.00 |
| ours, all sequential runs | 57 | 24 | 0.94 | 0.76 | 0.84 | 0.84 | 1.00 |
| ours + LLM intent invariants, Houdini on train | 57 | 28–34 | 0.84–0.95 | 0.88–0.98 | 0.91–0.97 | 0.91–0.97 | 1.00 |

Two notes on reading it. "Ours, all sequential runs" is derived from the sequential runs of all eight scenarios, so its held-out column is not held out; the "train scenarios" row is the one that never saw the held-out scenarios in any form. And no arm that uses the pair check alone exceeds recall 0.79, not even the identity. The arms above that line all carry an invariant from intent (finding F-A6).

What our approach derived, from all sequential runs: 4 variables chosen out of 113 candidates (`Job.get_status`, `Job.results`, `Job.func_name`, `CanceledJobRegistry.count`), with conflicts falling from 95 to 3, and 39 invariant clauses over the registries' `get_job_ids` and `count`, `Queue.job_ids`, `Job.is_finished`, `Job.latest_result` and the shape of the queue, the intermediate queue, executions and the six registries. From the training scenarios alone it chose 3 variables (the same without `CanceledJobRegistry.count`) and 38 clauses, with conflicts falling from 91 to 1 (`eval-v2.12.json`, key `own_specs`).

### 5.2 Ablations

| Change | Result | Reading |
|---|---|---|
| Selection without the invariant | recall 0.16, F1 0.27 | The selected variables alone are as blind as status-only. |
| No documentation filter: every observer counts as an observable (train scenarios) | selection picks the undocumented near-identity `Job.to_dict`; 70 abstract states against 54, det 0.96 against 1.00, F1 0.84 against 0.81 | Detection survives, but the abstraction is no longer in the user's words and is no longer deterministic. |
| LLM variables as observables (318 atomic variables from the eight samples) | selection does not converge (conflicts 902 → 413); F1 0.70 | The LLM vocabulary includes workers, counters and executions. |
| LLM invariants alone, no pair check | precision 0.65–1.00, recall 0.37–0.98, F1 0.54–0.84 | Good at spotting inconsistent registries; vary widely between samples. |
| LLM function with its own invariants | precision 0.35–0.38, recall 0.91–0.98, F1 0.52–0.55 | The invariants raise recall; the too-fine function keeps precision low. |
| Ours + LLM intent invariants, no filter | F1 0.91–0.97 on seven samples, 0.78 on one | One sample has a wrong clause. |
| Houdini filter on the training scenarios | that one sample rises from 0.78 to 0.91; nothing else changes | The filter removed the one wrong clause. |
| Houdini filter on all scenarios | seven samples fall to recall 0.76 and F1 0.84, the eighth to 0.765 and 0.845: the score without intent invariants | The true invariants are removed too, because two scenarios break them sequentially. |

The Houdini filter drops every intent clause that some sequential run refutes. The wrong clause was Sonnet's "a queued job is not running (no worker, not ended)": rq keeps `worker_name` and `ended_at` when it requeues a job for retry.

### 5.3 LLM proposals in detail

- **Coverage.** Every sample covered jobs, queues and workers, some also results and executions, with 30 to 50 atomic variables and 16 to 30 invariants. All 8 ran without error on every snapshot. The answers were 170 to 480 lines of Python.
- **Name stability.** The pairwise Jaccard overlap of variable names between samples has a median of 0.54 (minimum 0.16, maximum 0.79).
- **Partition agreement.** The partitions the samples induce agree better than their names: pairwise V-measure has a median of 0.81 and a minimum of 0.79. Against the reference partition, Opus scores 0.61–0.78 and Sonnet 0.79–0.80. Opus wrote longer functions (370–480 lines) with more implementation detail; Sonnet wrote 170–320 lines.
- **Docs or no docs.** Opus's pair-check F1 was about the same with or without the user docs in the prompt (0.45–0.47).
- **Cost.** All eight samples together cost $5.30. The prompts are about 45,000 tokens.
- **Prompt overhead.** The LLM was called through `claude -p --model <m> --tools "" --strict-mcp-config --setting-sources project`. Those flags keep the coding agent's own context out of the prompt: 617 tokens of overhead, against 167,000 without them.

### 5.4 Mutation visibility

20 seeded changes (`experiments/results/compare.md`).

| Arm | Harmful shown (of 10) | Benign hidden (of 10) | Harmful missed, or benign shown |
|---|---|---|---|
| identity | 10 | 5 | shows execution-key layout, success counter, worker state word, two extra fields |
| status-only | 5 | 10 | misses abandoned-job-not-recorded and the cancel, success, requeue and delete registry bugs |
| read-projection | 7 | 10 | misses the cancel, success and requeue registry bugs |
| membership | 9 | 10 | misses failure-marks-finished (a status-only bug) |
| behavioural, depth 2 | 8 | 9 | misses the cancel and success registry bugs; shows execution-key layout |
| observer, semantic | 10 | 9 | shows execution-key layout |
| LLM, Opus | 10 | 7 | shows success counter, worker state word, execution-key layout |
| LLM, Sonnet | 10 | 8 | shows success counter, worker state word |
| ours, selection only (train scenarios) | 6 | 10 | misses four registry bugs |
| **ours** | **10** | **10** | none |
| reference, with or without the invariant | 10 | 10 | none |

Selection only on all sequential runs shows 7 of 10. Only our approach and the oracle's own reference reach 10 and 10.

### 5.5 Stability across versions

The abstraction derived on rq 2.12, applied unchanged to other versions' snapshots; pair-check F1 against each version's own oracle (`experiments/results/versions.md`).

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

Re-deriving the observers from each version's own code gave the same F1 as the 2.12 observers (0.62, 0.76, 0.83). `compare.md` also holds the Jaccard similarity of visible sets between 2.12 and each version. Even the reference sees only 0.55 similarity with 3.0, so business behaviour did change between versions, and that measure mixes real change with abstraction noise. The per-version F1 is the cleaner signal.

## 6. Findings

Each finding gives its evidence and its consequence for Mental. Consequences are design inferences.

**F-A1. The status-only rule and the cone-of-influence rule are both blind to most races.**
*Evidence.* Status-only, which is the specification's current rule for state variables (4.6: "the part of a store that behavior depends on"), has recall 0.16 and shows 5 of 10 harmful changes. Read-projection has recall 0.17 and shows 7 of 10. Per scenario, status-only flags none of the 139 delete-vs-work anomalies and none of the 20 runs of the real rq bug (`eval-v2.12.json`, `per_scenario`).
*Consequence.* "What the code branches on" and "what the code reads" pick the variable the code treats as the truth, and leave out the copies that the code only writes. For an upper level, 4.6's rule is not sufficient.

**F-A2. Races live in the disagreement between redundant representations, so the abstraction function needs its representation invariant.**
*Evidence.* Selection alone produces a deterministic upper level (det 1.00) with recall 0.16, because a minimal abstraction drops every variable that another determines. Adding the mined invariant raises recall to 0.76 at precision 0.94 and takes harmful changes shown from 7 to 10 of 10, with benign hidden staying at 10. The membership arm, which keeps the redundancy inside the state, reaches recall 0.79 but at precision 0.77 and det 0.97.
*Consequence.* A **representation invariant** is the condition on lower state under which the abstraction function is meaningful. Hoare (1972) and Liskov defined the function only on representations that satisfy one, for sequential data types. Here the same object reappears as a concurrency requirement: correct sequential code keeps the copies in step, and a race is a run that reaches a quiescent state where they are not. A level record with an abstraction function and no representation invariant cannot see such a run.

**F-A3. The observables decide the abstraction, and someone must name them.**
*Evidence.* With only operation outputs observable, the behavioural abstraction at depth 2 is perfectly deterministic (det 1.00) and nearly trivial: recall 0.35, and it misses the cancel and success registry bugs. Declaring the documented observers as observables gives our result. Declaring every observer gives an undocumented near-identity (`Job.to_dict`, 70 states, det 0.96). Declaring the LLM's variables makes selection fail (conflicts 902 → 413, F1 0.70).
*Consequence.* A **business observable** is something the upper level's user can ask about the system and is entitled to rely on. No computation produces the list; it is an input from intent. In our data the user documentation was a sharper source than LLM proposals: 46 of the 113 candidates are observers the documentation names. The level record should store the observables and where each came from.

**F-A4. The library's own query API is an author-written abstraction function, but purity must be judged by effect, not by syntax.**
*Evidence.* The observer arm alone reaches F1 0.75 with no selection at all. But rq's finished and failed registries garbage-collect inside `get_job_ids`. A syntactic purity check (the method issues no write command) rejects them and scores F1 0.62; a semantic check (the state is unchanged after the call) keeps them and scores 0.75.
*Consequence.* Observers are the first source of named candidates, with names that come from the code. The purity check must run the observer and compare state before and after.

**F-A5. LLM-written abstraction functions are too fine and unstably named; LLM-written invariants are valuable as intent.**
*Evidence.* The eight LLM functions have 212–502 abstract states against the reference's 98, precision 0.32–0.33 and F1 0.45–0.47. Opus shows 3 of 10 benign changes and Sonnet 2. Variable names overlap between samples with a median Jaccard of 0.54. The invariants of the same samples, added to our abstraction and filtered on the training scenarios, give F1 0.91–0.97 on all 8 samples, above the oracle's own reference with the documented invariant (0.88).
*Consequence.* This matches the earlier report's note that LLMs drift to code-level granularity. Do not ask the LLM for the function. Ask it for invariants in business words, and possibly for candidates, and let selection decide. The measured name instability also supports P9: identity must come from code names, which the observers supply.

**F-A6. Some anomalies occur in sequential runs, and only an intent invariant can see them.**
*Evidence.* Canceling or cleaning up a running job breaks the documented invariant in 1 of 3 sequential runs of each of those scenarios. The pair check compares concurrent runs with the code's own sequential runs, so its reference already contains the anomaly; no pair-check-only arm exceeds recall 0.79, and an invariant mined from sequential runs has the same limit (0.76). The documented invariant lifts the reference from 0.79 to 0.98; LLM intent invariants lift ours from 0.76 to 0.88–0.98. A Houdini filter over all sequential runs removes exactly these clauses and returns every sample to F1 0.84 (one to 0.845).
*Consequence.* The earlier report recommended proposing invariants generously and dropping every one that exploration refutes. That is right for clauses mined from the code and wrong for clauses that state intent. When a sequential run refutes an **intent invariant** (one stated from documentation, the glossary or a person, not derived from runs), there are two possibilities: the invariant is wrong (Sonnet's "a queued job is not running") or the code is wrong (a canceled job that completes anyway). Telling them apart is a judgment. The clause must be kept as an open finding with its witness until someone decides, never dropped silently.

**F-A7. The derived abstraction, stored as data, is stable across versions.**
*Evidence.* The variable names and clause tables derived on 2.12 (train variant) score F1 0.83, 0.85 and 0.82 on rq 3.0, 2.0 and 1.16.2, releases dated from 2024-05-01 to 2026-10-28. Status-only scores 0.22, 0.29 and 0.32; the identity 0.23, 0.46 and 0.45. On 3.0, ours scores above the unchanged reference abstraction without its invariant (0.64).
*Consequence.* The derivation does not have to be re-run on each build. The stored result can be re-applied and re-checked, as CPAchecker reuses stored precisions (earlier report, item 9).

**F-A8. The harness found a real rq bug.**
*Evidence.* `handle_job_success` runs inside a WATCH retry loop. If a dependent job is enqueued between WATCH and EXEC, the transaction is retried. The first attempt already ran `cleanup_execution`, which set `worker.execution = None` in Python, so the retried transaction no longer contains `DEL rq:execution:*`, `ZREM rq:wip:*` or `ZREM rq:executions:*`. The finished job stays listed as started. The shortest witness has 42 steps and 2 preemptions. The bug is present in 2.0 and 2.12 and absent in 1.16.2 (no executions) and 3.0 (`cleanup_execution` takes the execution as an argument). The consequence that `StartedJobRegistry.cleanup` would later treat the job as abandoned was read from the code, not run. Status-only and selection-only flag 0 of these 20 runs; ours flags all 20.
*Consequence.* The method finds real defects, not only seeded ones, and it finds this one only through the representation invariant. **The bug has not been reported upstream.**

**F-A9. The step map came first: the oracle could not be written until the worker's operation was split.**
*Evidence.* Treating the worker's whole job as one upper step produced 96 false anomalies in `fail-retry`. Splitting it into *take* and *complete* removed them. Quiescent states, where the invariant is checked, are defined by operation boundaries too.
*Consequence.* The abstraction function is half of a refinement. What counts as an upper step fixes where upper state is defined and what "sequential" means. This experiment chose the steps by hand.

**F-A10. Named candidates chosen by a computed criterion give both names and a guarantee.**
*Evidence.* Our abstraction has 57 abstract states and 24 findings to read, against 98 and 59 for the reference with its invariant and 134 and 65 for the observers. Its variables are four method names from rq's public API. Its det is 1.00 where the reference, membership, observer and LLM arms are at 0.94–0.97.
*Consequence.* The named-against-computed tension has a workable answer: people and code supply the names, and the computation supplies the choice.

## 7. The proposed algorithm: select-and-guard

The name says what it does: *select* a few named variables as the abstraction function, and *guard* the rest with a representation invariant. It is what the "ours" arm implements (`experiments/afexp/own.py`), with step 5 changed in the light of F-A6. Steps 1 to 4 and 6 were measured; the triage in step 5 is a proposal.

**Inputs.** The lower level as something that can be run; the sequential runs of a set of scenarios, cut into upper steps; the project's documentation or glossary.

1. **Collect named candidates.** From three sources:
   - *observers*: every public property and zero-argument method of the entity classes, evaluated on each lower state and kept only if it is semantically pure (the state after the call equals the state before);
   - *store shape*: for each entity, which containers hold it;
   - *LLM proposals*: optional, and only as candidates, never as observables.

   *Guards against:* unnamed abstractions, and names invented per run (F-A5: name Jaccard 0.54). The semantic purity test guards against losing observers that do housekeeping (F-A4: 0.75 against 0.62).

2. **Choose the business observables from intent.** An observer is a business observable when the user documentation names its method. When a project has no documentation, the glossary's terms and their anchors are the natural substitute. That substitute is untested.

   *Guards against:* an abstraction in the implementation's words. Without this step selection chose `Job.to_dict`; with LLM variables as observables it did not converge (F-A3).

3. **Select greedily.** A *conflict* is either nondeterminism of the upper level (two sequential operation runs that start in the same abstract state with the same operation and end in different abstract states or outputs) or an observable that the chosen variables fail to determine on some quiescent sequential state. Start from no variables. Repeatedly add the candidate that removes the most conflicts per bit of its domain, that is, conflicts removed divided by log2(1 + number of distinct values). Break ties toward fewer remaining conflicts, then toward observers over shape. Stop when no candidate helps. Then drop every chosen variable whose removal does not raise the conflict count.

   *Guards against:* too fine (the per-bit cost and the pruning keep the state small: 57 states against the identity's 688) and too coarse for running (det 1.00). It does not guard against missing races; step 4 does.

4. **Mine the representation invariant.** For every observable or shape variable left out, with at most 8 distinct values, that the chosen variables determine on every quiescent sequential state and that is not constant: record the clause "y = f(chosen)" as a table. Check every clause at every quiescent state of every explored run. A state whose chosen variables have a value the table never saw is not judged.

   *Guards against:* blindness to races between redundant representations (F-A2: recall 0.16 → 0.76, harmful shown 7 → 10).

5. **Add intent invariants, and triage instead of filtering.** Ask the LLM for business invariants in the level's words, and take any the documentation states. Check each against the sequential runs. A clause no sequential run refutes is active. A clause some sequential run refutes is *not dropped*: it becomes an open item with the refuting run as witness, to be resolved as "invariant wrong" (retire it, with the reason) or "code wrong" (a finding).

   *Guards against:* anomalies the code commits even sequentially (F-A6). In the experiment the choice of filter was the whole difference: filtering on the training scenarios kept F1 at 0.91–0.97; filtering on all scenarios returned it to 0.84. Triage is the version of the filter that does not depend on which scenarios happen to be in the set.

6. **Store the result as data.** The chosen variable names, the clause tables, the observables with their sources, and the intent invariants with their status. On rebuild, re-apply and re-check; re-derive only when the check fails or a named variable no longer exists. A variable the new code never produces reads as absent.

   *Guards against:* instability across builds (F-A7: F1 0.82–0.85 on three other versions, unchanged).

**The checks that use the result.** The pair check runs on the chosen variables. The invariant check runs at quiescent states on the mined clauses and the active intent invariants. A run is a finding when either fails.

## 8. What Mental should change

These are recommendations for the owner to decide. Nothing in the specification has been changed. Section numbers refer to the [specification](../../specification.md).

**Proposal 1: extend the `level` record (5.2).**

| Field | Today | Proposed |
|---|---|---|
| `observables` | absent | the business observables of this level, each with its source (a documentation anchor, a glossary term, a person) |
| `abstraction` | one expression per upper state variable, over lower state | the *selected named variables*: each an observer or shape variable of the lower level, identified by a code name (P9), with the selection trace as its justification |
| `representation_invariant` | absent | the mined clauses "y = f(selected)" as tables, with the scenarios and bounds they were mined from |
| `invariants` | properties in this level's words | unchanged in meaning, but each carries its origin (mined or intent) and, for intent invariants, a triage status |
| quiescent states | absent (the earlier report proposed `states_of_interest`) | where the upper state is defined and the invariants are checked: no upper step half done |

**Proposal 2: state that 4.6's rule is for lifecycle machines, not for upper levels.** "The part of a store that behavior depends on" selects `status` and nothing else in rq, and the glossary adds that a column nothing branches on "is data, not a state variable". F-A1 shows that this rule, used as the upper level's state, gives recall 0.16. The rule can stay for what it was written for (which writes are legal moves). The redundant representations it leaves out should not be discarded as data: they are what the representation invariant is made of.

**Proposal 3: give the pair check a companion (5.3).** Add an *invariant check at quiescent states* beside the pair check, covering the representation invariant and the active intent invariants. The pair check alone, even through the oracle's own reference abstraction, reaches recall 0.79; with the invariant, 0.98. The two checks fail for different reasons and should be reported separately: "the lower level made an upper move the upper level does not have" and "the lower level reached a resting state in which two descriptions of one fact disagree".

**Proposal 4: record what happens to a refuted intent invariant (P8).** An intent invariant refuted by a sequential run needs a recorded state, because both outcomes are valuable and neither may be silent. One way, within the existing provenance values: an invariant anchored in documentation is `attested`; one proposed by the LLM without such an anchor is `assumed`; either carries a status of *active*, *refuted, untriaged* (with the witness run), *retired: invariant wrong* (with the reason) or *finding: code wrong*. An untriaged refutation is shown as a finding, not as a fact and not hidden.

**Proposal 5: divide the work by P7.**

| Actor | Does |
|---|---|
| Extractors | list observer candidates (public properties and zero-argument methods of entity types); list store shape (which containers hold which entity); find which names the documentation mentions |
| Interpreter | produce the sequential runs; run each observer for the semantic purity test; produce the explored runs |
| Checkers | count conflicts, select, prune, mine the clauses, run the pair check and the invariant check; all deterministic |
| LLM | propose intent invariants in business words; propose display names for selected variables and mined clauses; propose observables from the glossary when documentation is missing; suggest a triage verdict with its reasons |
| Person | confirm the observables when asked; decide triage |

The LLM never writes the abstraction function. This keeps to P7's "never decide what is feasible", and it is also what the data supports (F-A5).

**Proposal 6: revise item 6 of the earlier report.** "Propose invariants generously and keep only what survives" should apply to mined clauses only. For intent invariants the rule is F-A6's triage.

One dependency these proposals carry: in the experiment an observer is real code run against real state. In Mental an observer would be a compiled behavior of a read-only port, run by the interpreter on the lower level's state. Whether compiled observers are accurate enough for selection and mining is not tested here.

## 9. Threats to validity and open questions

**Limits of the evidence.**

- **One library.** rq is small, keeps all state in one store, has a rich query API and good user documentation. Each of those helped. A system whose entities are not in containers, or whose API has few observers, may give few candidates.
- **Small bounds.** Two actors, at most two preemptions, one queue, at most two jobs, eight scenarios. Bound 2 misses nothing we know of here, but that is unproven. The worker's periodic registry maintenance is skipped; one scenario covers cleanup instead.
- **The oracle is a choice.** It was written before any result, from the jobs and registries documentation, and it leaves workers out of the business state. That penalises the LLM arms, which include them. A different reading of the documentation would move every number.
- **The oracle and our approach share a source.** Both take their vocabulary from rq's user documentation. The oracle was hand-written and the selection is mechanical, but an abstraction built from the same documentation as its judge is favoured by construction.
- **The real code is the lower level.** Mental's lower level will be extracted and partly LLM-written, so its errors will add to the abstraction's.
- **The documentation dependency.** Step 2 uses documentation as the intent signal. The specification's open question, "How reliable a business level is when the project has no docs", applies directly. The one alternative we tried, LLM-proposed observables, did worse.
- **Few LLM samples.** Eight samples from two models, one prompt. The ranges reported are ranges over eight values, not distributions.
- **Mined clauses are tables.** They reproduce what sequential runs did, and they do not judge states whose selected variables take a value never seen. More scenarios make them stronger and can also teach them a sequential anomaly as normal, which is why F-A6 needs intent.
- **The triage step was not run.** The experiment measured the two filters it replaces. How often triage is needed, and how well a person or an LLM does it, is unmeasured.

**What to try next.**

1. **A second library without good documentation**, using the glossary as the source of observables. This tests the weakest link directly.
2. **Mental's own extracted lower model** in place of the real code, on rq or on report-jobs, to measure how much extraction error costs.
3. **Larger bounds**: three actors, three preemptions, several queues, and the worker's maintenance turned on.
4. **Naming the mined clauses.** A clause table is exact and unreadable. Ask the LLM to state each clause in business words ("a finished job is in the finished registry and no other") and check the statement against the table.
5. **The step map.** This report fixed the upper steps by hand (F-A9). Deriving the step map, including splits like take/complete, is the other half of the refinement and probably the next research question. Process mining's event abstraction and Tomte's refinement on nondeterminism are the nearest precedents.
6. **Triage in practice.** Give the refuted clauses from this experiment (Sonnet's wrong one, and the true ones refuted by cancel and cleanup) to an LLM and to a person, and see whether each is classified correctly.
7. **Name stability across commits.** The earlier report asked how stable LLM-proposed names are. We now have one measurement between samples (median Jaccard 0.54) and none across commits when seeded with the stored set.
8. **Report the rq bug upstream**, after confirming on a real Redis that the retried transaction loses the cleanup.

## 10. Sources

### Our own files

- [notes/experiments.md](notes/experiments.md): design, controls, results.
- [notes/survey.md](notes/survey.md): the survey, with evidence tags.
- [experiments/README.md](experiments/README.md): code layout and how to reproduce.
- `experiments/results/eval-v2.12.md` and `eval-v2.12.json`: the pair-check table, per-scenario counts, the oracle counts and the derived specifications.
- `experiments/results/compare.md` and `compare.json`: mutation visibility and visible-set similarity across versions.
- `experiments/results/versions.md`, with `eval-v3.0.json`, `eval-v2.0.json` and `eval-v1.16.2.json`: per-version scores.
- `experiments/results/llm-answers/`: the eight LLM answers used.
- `experiments/afexp/own.py`, `evaluate.py`, `observers.py`: the algorithm, the checks and the purity test.
- [The abstraction-levels report](../abstraction-levels/report.md), the [specification](../../specification.md) (sections 2, 4.6, 5 and 13) and the [glossary](../../glossary.md).

**Outside sources** (links as in the survey)

- rq: <https://github.com/rq/rq>
- Verus `View`: <https://verus-lang.github.io/verus/verusdoc/vstd/view/trait.View.html>
- IronFleet: <https://6826.csail.mit.edu/2019/papers/ironfleet.pdf>
- Refinement proof per step, MIT 6.826 lecture 17: <https://6826.csail.mit.edu/2020/lec/l17.pdf>
- ADABU and its companion study: <https://www.st.cs.uni-saarland.de/models/papers/woda-06.php>
- TVLA canonical abstraction: <https://research.cs.wisc.edu/wpis/abstracts/tr1383r2.abs.html>
- Tomte: <https://www.mbsd.cs.ru.nl/publications/papers/fvaan/CEGAR12/FM.pdf> and <https://typeset.io/pdf/automata-learning-through-counterexample-guided-abstraction-2a10rgqrzh.pdf>
- Elle: <https://arxiv.org/abs/2003.10554>
- Debug-information bugs (Li et al., PLDI 2020, as cited in the survey): <https://arxiv.org/pdf/2211.09568>
- AutoCorres: <https://trustworthy.systems/projects/OLD/autocorres/>
- SysMoBench: <https://arxiv.org/abs/2509.23130v3>
- ProtocolGPT: <https://arxiv.org/abs/2405.00393>
- Event abstraction in process mining: <https://link.springer.com/article/10.1007/s41066-020-00226-2>

Hoare 1972, Liskov–Guttag, TLA+, Event-B, Z and VDM, Dafny, seL4, CEGAR, bisimulation minimisation, Daikon, QuickCheck, DWARF and CQRS are cited from background knowledge, as the survey marks them; they were not re-read for this report.
