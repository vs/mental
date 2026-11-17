# Mental: strategies for building the level ladder (proposal)

**Status: Proposal.** This document proposes how `mentalc` builds a model's levels with interchangeable *ladder strategies*, and how the strategies are compared. Nothing here is built. **Date:** 2026-11-17.

The owner asked for two things. First, the ways of building multi-level models found in research should be kept together in one place. Second, `mentalc` should be able to build a ladder in several ways, so that the ways can be tried side by side and the better ones chosen by measurement. The small hooks this needs in the specification, the glossary and the `mentalc` guidelines were applied on the same date and are listed in [section 10](#10-changes-to-accepted-documents).

Sources: the [multi-level models report](research/multi-level-models/report.md) defines closing rules, charters, admission tests, cuts and the rq experiment. The [levels in other domains report](research/levels-in-other-domains/report.md) gives the practices of other fields and proposals N1 to N9. The [abstraction levels report](research/abstraction-levels/report.md) covers how levels are related and checked.

Each section carries a status, as in the specification: **Proposed** (the working answer), **Experiment** (competing designs kept side by side, to be settled by measurement), **Measured** (tried and not chosen; kept for comparison), **Later**, **Open**.

## Contents

1. [Why several strategies](#1-why-several-strategies)
2. [Principles](#2-principles)
3. [What a strategy decides](#3-what-a-strategy-decides)
4. [The building practices, from research](#4-the-building-practices-from-research)
5. [The strategies and their options](#5-the-strategies-and-their-options)
6. [How `mentalc` runs a strategy](#6-how-mentalc-runs-a-strategy)
7. [How strategies are compared](#7-how-strategies-are-compared)
8. [Experiments](#8-experiments)
9. [The plan](#9-the-plan)
10. [Changes to accepted documents](#10-changes-to-accepted-documents)
11. [Open questions](#11-open-questions)

---

## 1. Why several strategies

The specification began with one fixed ladder (5.1): *business, system, services, components, mechanisms, statements*. The rq experiment compared it with two other ways of getting levels:

| Design | Result on rq (2,097 runs, 554 anomalous) |
|---|---|
| **A. Fixed ladder** | 27% of anomalies, the real bug among them, invisible above the raw commands; the developer needed 24 to 43 lines |
| **B. Closing rules** | every run cut with no author; every symptom at the top two levels; the developer answered everything from 9 to 15 lines |
| **C. Ladder proposed by the LLM** | agreement at the top; disagreement below; half the ladders did not nest; no shared step names; one call per run |

B won on rq. That is one library with one store, two actors and one or two jobs. Several of B's rules were set by hand and some are still open:

- the grain band failed and was replaced by a weaker merge test;
- the move rule has two valid readings;
- outcomes overlap in 4.4% of goals.

The survey of other fields added nine more ideas. None of this is settled well enough to build one way only. So the ladder becomes a strategy that `mentalc` takes as a setting, the same way the LLM backend is a setting (6.5). Every strategy is judged by the same checks and measured by the same metrics.

## 2. Principles

**Status: Proposed.**

### S1. One judge for every strategy
A strategy decides levels, boundaries, the step map and names. It does not decide what counts as correct. The pair check, barrier check, coverage, invariants, name truth and admission measurements run the same way whichever strategy built the ladder. This is guideline C4 ("one judge for every backend") applied to ladders.

### S2. A strategy is a pure function of the model and answers
`plan(model) -> tasks` and `apply(model, answers) -> ladder`, as for every LLM phase (C3). A strategy reads nothing else, writes nothing else and keeps no state between builds. Its options are part of its identity.

### S3. Every record says which strategy made it
Each `level` and `step_class` record carries the strategy's ID: its name and canonical options. Two ladders of one commit can therefore sit side by side and be compared record by record.

### S4. The LLM names; it does not cut
In every strategy except the baseline C, boundaries, the step map and support steps are computed. The LLM names step classes, writes charters and groups lifecycle states into stages, and every answer is checked. Strategy C is kept only as a measured baseline (5.1).

### S5. Strategies share LLM work through the cache
The cache key is the task's content (6.8). Two strategies that ask the same naming question about the same step class get the same cached answer. Trying another strategy costs only the tasks that differ.

### S6. One ladder is active; others sit beside it
The model has exactly one active ladder, which `mentalvm` and `mentaldbg` use by default. Other ladders are complete side files that can be shown, measured and made active. A ladder is never partial: if a strategy's tasks cannot be answered, that ladder fails with a clear error (the specification's rule against degraded output).

### S7. Losing strategies stay
A strategy that loses an experiment stays in the code and in this document, marked *Measured*, as long as it costs little to keep. A later subject may reverse the result, and the baseline is what makes a new result meaningful.

## 3. What a strategy decides

**Status: Proposed.**

| # | Decision | Fixed for all strategies |
|---|---|---|
| **D1** | which levels exist, their kinds and order, and each one's charter (readers, question, what a step is, what it hides) | the charter's fields; the `why` and `how` links |
| **D2** | where a step of each level ends | well-formedness: steps nest, each lower step is under exactly one step per level, a run is covered |
| **D3** | the step map: which lower step classes make up each upper step class, and which steps are support | step classes identified from code names (spec P9) |
| **D4** | names and "so that" sentences | names come from the glossary; generated once and stored, never regenerated on rebuild |
| **D5** | admission: which candidate levels are kept, merged or dropped | the admission measurements are computed the same way; a strategy chooses only the thresholds |

The abstraction function on state, the observables and the representation invariant come from the abstraction-function work and are inputs to every strategy, not choices of one. Design A+ of the rq experiment showed why: better observables help every ladder.

## 4. The building practices, from research

**Status: reference.** These are the ways of building and showing multi-level models that recur in research and in other fields. Each row says where it comes from, how Mental uses it, and whether it is already in the design, a strategy option (section 5) or a view or check outside the strategy.

| # | Practice | From | In Mental |
|---|---|---|---|
| **P1** | Up is *why*, down is *how*; every level has readers and a question | Rasmussen, Cockburn, action identification, Minto, Lamport | the charter (design) |
| **P2** | Levels move along purpose; containment is a separate axis | Rasmussen, Gajski–Kuhn Y-chart, capability vs process maps, military levels of war, Marr | design (R2); strategy A does not follow it |
| **P3** | Steps end where goal, actor, subject or causal chain changes | event segmentation, assembly instructions, Simon | the closing rules of strategy B |
| **P4** | Hard rules (what a valid grouping is) apart from preferences (which valid one) | GTTM | option `move_close = prefer` (O1) |
| **P5** | A group is represented by its most important member | GTTM head, RST nucleus, Lamport's step claim | option `naming = head` (O2) |
| **P6** | Side events fold away; the causal chain stays | causal network model, RST deletion test, Fuzzy Miner | support steps (design) |
| **P7** | Cases can overlap; do not force one case notion | object-centric process mining, BPMN's one-instance rule | option `outcomes = object-centric` (O3) |
| **P8** | A level is kept only where it adds words, predicts on its own and names something no other level can | CakeML rule, Wimsatt, admission tests A1–A6 | admission (design); thresholds are option O4 |
| **P9** | Levels with overlapping time spans are one too many | Jaques, hierarchy theory | option `merge_test = span-overlap` (O4) |
| **P10** | Standard words at the top, local words at the bottom | SCOR, APQC, Arcadia, Domain Storytelling | option `top_vocabulary` (O5) |
| **P11** | Steps people do around the software come from documents and are marked as such | runbooks, ISO procedures, work-as-imagined | strategy D (5.1) |
| **P12** | Links between levels are labels or addresses, and they are checked | BPMN end states, IDEF0 arrows, drawing callouts | end-state and boundary checks (6.6); for all strategies |
| **P13** | Read a level with the level above and below on view; restate intent two levels up | Salthe, commander's intent | view: the two-up frame |
| **P14** | Show a mixed cut, opened where something is wrong, not one level at a time | action identification, fisheye views, fragmentation studies | view: the cut (design) |
| **P15** | Open on a simple complete whole; zoom in, then back out | Reigeluth's elaboration theory | view: epitome and zoom back out |
| **P16** | Status per element, across the hierarchy | Lean blueprints | view: status map |
| **P17** | Across many runs is its own direction | Bret Victor's ladder of abstraction | view: one step class over all explored runs |

The views (P13 to P17) do not change how a ladder is built. They are recorded here so that every practice is in one list, and they are measured with people later (L8).

## 5. The strategies and their options

### 5.1 Base strategies

| Strategy | Status | Levels from | Boundaries from | LLM tasks |
|---|---|---|---|---|
| **`fixed`** (design A) | Measured on rq; kept as the reference | the specification's ladder (5.1), read onto the project | each level's own definition | `business` |
| **`closing-rules`** (design B) | Experiment; the working default | seven kinds (outcome, stage, goal, exchange, activation, move, effect), admitted by tests | one closing rule per kind | `subjects`, `actors`, `name-step`, `why-link`, `stages`, `charter` |
| **`llm-ladder`** (design C) | Measured on rq; baseline only | the LLM | the LLM, per step class over the model | `ladder-propose` plus naming |
| **`documented`** | Experiment, needs a project with runbooks | `closing-rules`, plus stages and goals of people taken from documents | as `closing-rules` below the documented levels | as `closing-rules`, plus `process-from-docs` |

`fixed` keeps its levels whatever the admission tests say, so that it stays the reference that it was on rq. A+ of the rq experiment (the fixed ladder with the documented observables) is `fixed` with the abstraction function supplied. That input is shared by all strategies (section 3) and is not an option.

`llm-ladder` cuts *step classes of the model* once per build, not each run. That fixes its cost problem but not its disagreement problem. Whether nesting and agreement improve when the LLM cuts classes instead of runs is a question for L1.

### 5.2 Options of `closing-rules` and `documented`

Each option had a default, the setting used in the first rq experiment, so that every experiment changes one thing at a time. The Default column now shows what L2 to L5 decided on rq and report-jobs (section 8).

| Option | Values | Default | Practice | Experiment |
|---|---|---|---|---|
| **O1 `move_close`** | `every-write`: a move closes after each on-chain write; `atomic-group`: after the evidence of atomicity ends (transaction, lock, conditional update); `write-run`: as `every-write`, but a write followed directly by another write to the same key, with nothing read in between, does not close the move; `prefer`: build the others and keep the one whose move classes are most stable across versions, ties broken by how well a class predicts its observable change | `write-run` (decided; `atomic-group` and `prefer` Measured) | P4 | L2 |
| **O2 `naming`** | `children`: the LLM gets the children's names; `head`: it also gets the head effect, the one that changes the level's observable or closes the step, and name truth is checked against it | `head` for moves, `children` for activations and goals (decided) | P5 | L3 |
| **O3 `outcomes`** | `partition`: each goal under one outcome (the subject it changes first), overlaps reported; `object-centric`: each goal lists every subject it changes and appears under each, marked shared | `object-centric` (decided on rq; not testable on report-jobs) | P7 | L4 |
| **O4 `merge_test`** | `band`: merge below 2 and split above 7 lower steps per step (the original A3); `single-child`: merge when about four in five upper steps have one lower step (the revised A3); `span-overlap`: merge when two adjacent levels' step lengths overlap heavily | `single-child` (decided; `span-overlap` Measured) | P8, P9 | L5 |
| **O5 `top_vocabulary`** | `glossary`: the project glossary only; `framework:<name>`: outcome and stage names prefer the terms of a process framework the organisation uses (APQC, SCOR, its own), mapped to glossary terms | `glossary` | P10 | L6 |
| **O6 `kinds`** | the subset of the seven kinds to try as candidates, and project-specific kinds proposed as a charter plus a closing rule | all seven | P8 | L1 |

`fixed` takes no options. `llm-ladder` takes only the samples and LLMs used, which the ledger records anyway.

### 5.3 A strategy's identity

A strategy's ID is its name followed by its non-default options in a fixed order, for example `closing-rules` or `closing-rules[move_close=prefer,naming=head]`. The ID is stored in every `level` and `step_class` record (S3), is shown with every ladder, and names the side folder of a ladder that is not active (6.3).

## 6. How `mentalc` runs a strategy

**Status: Proposed.**

### 6.1 Configuration and commands

```toml
# mentalc.toml
[levels]
strategy = "closing-rules"          # fixed | closing-rules | llm-ladder | documented
move_close = "every-write"          # options of 5.2; omitted means default
naming = "children"

[[levels.also]]                     # more ladders to build beside the active one
strategy = "fixed"
[[levels.also]]
strategy = "closing-rules"
naming = "head"
```

| Command | Does |
|---|---|
| `mentalc build` | builds the model and the active ladder; then each ladder in `levels.also` |
| `mentalc build --ladder <id>` | uses `<id>` as the active ladder for this build |
| `mentalc ladder build <id>` | builds one ladder over the existing model (phase 11 and its checks only) and stores it beside the model |
| `mentalc ladder list` | the ladders present, with their IDs, status and check results |
| `mentalc ladder use <id>` | makes a stored ladder the active one |
| `mentalc ladder compare <id> <id> …` | the static metrics of 7.1 for each ladder, side by side; `--json` for scripts |
| `mentalvm ladder eval <id> …` | the run-based metrics of 7.1, on runs explored with the configuration given |

Every command has `--json` and follows the exit codes of 6.1 in the specification.

### 6.2 Inside phase 11

Phase 11 (Levels) runs the active strategy, then each strategy in `levels.also`, each as `plan` and `apply` over the same input: the model after phase 10 (Close), the glossary and the abstraction function. Its output per strategy is a **ladder**: `level` records with charters, `step_class` records, and the step map between them. Phase 12 then runs the checks of S1 on every ladder.

```rust
enum Strategy {                       // a closed list, like the record kinds (M4)
    Fixed,
    ClosingRules(RuleOptions),
    LlmLadder(LlmOptions),
    Documented(RuleOptions),
}
fn plan(s: &Strategy, input: &LevelInput) -> Vec<Task>;
fn apply(s: &Strategy, input: &LevelInput, answers: &Answers) -> (Ladder, Vec<Diagnostic>);
```

**Closing rules are evaluated on the model, not on runs.** `mentalc` does not run the model: it does not depend on `mentalvm`. Every closing rule of 5.2 in the multi-level report is stated over things the model already holds:

- an effect is an interpreter rule on a behavior node;
- a move closes at a write or at the end of atomicity evidence;
- an activation ends at a wait or a reply on a port;
- a goal is an external port's request answered;
- a stage is a group of lifecycle states;
- an outcome is a terminal state.

So step classes and the step map can be derived from the behavior graphs and lifecycle machines without running anything. Support steps need a static approximation: a write that no reader anywhere in the model reads. Runs are needed only to *measure* a ladder (grain, placement, determinism). That is `mentalvm ladder eval`. Whether the static derivation gives the same step classes as cutting explored runs is experiment L7. If it does not, phase 11 needs a run sample as input, and that changes the pipeline.

### 6.3 Storage

The active ladder's records live in the model (`levels.jsonl`, `step_classes.jsonl`, as 4.8 and 4.9 of the specification foresee). Each other ladder is stored as

```
.mental/
  ladders/<strategy-id>/        levels.jsonl  step_classes.jsonl  diagnostics.jsonl
```

in the same canonical JSONL. `ladder use` swaps the active ladder's files with a stored one, and the model's other records do not change. LLM answers live in the shared cache and ledger, so `ladder build` for a new strategy calls the LLM only for tasks no earlier ladder asked.

### 6.4 Records

The `level` record of 5.2 in the specification gains:

| Field | Meaning |
|---|---|
| `strategy` | the strategy ID (5.3) |
| `kind` | outcome, stage, goal, exchange, activation, move, effect; a project-specific kind; or a fixed-ladder level name |
| `charter` | readers, question, step_is, closes_when, hides, why, how |
| `admission` | the computed results of the admission tests and the merge test used |

A `step_class` record holds its level, its identity from code names, its parent step class or `support`, its cases as named end states (6.6), its head effect when `naming = head`, its name and "so that" sentence with the call that produced them, and `strategy`.

### 6.5 LLM roles

The ledger's roles (6.6 of the specification) gain the ladder tasks of section 6 of the multi-level report: `subjects`, `actors`, `name-step`, `why-link`, `stages`, `charter`, `intent-invariants`, plus `ladder-propose` for `llm-ladder` and `process-from-docs` for `documented`. The role `business` stays for `fixed`. `narrate` runs at view time, in `mentaldbg`, not in the build.

### 6.6 Checks every ladder gets

These are the checks of 5.3 in the specification (pair, barrier, coverage, invariants) and of the multi-level report (admission, name truth). Two more come from other fields (P12):

- **End states.** Each step class names its cases as end states. The step map above may branch only on those names, and every end state is handled somewhere above or marked terminal.
- **Boundary.** For a step taken from a document (`documented`) or a stage grouped by the LLM: what the document or charter says enters and leaves the step equals what its expansion reads, writes and sends. For computed steps this holds by construction and is skipped.

## 7. How strategies are compared

**Status: Proposed.**

### 7.1 Metrics

| # | Metric | Computed by | From |
|---|---|---|---|
| **M1** | **Cut.** Share of explored runs that the ladder cuts with every well-formedness rule holding; nesting violations | `mentalvm ladder eval` | rq E1 |
| **M2** | **Grain.** Lower steps per upper step; share of upper steps with one child; overlap of step lengths between adjacent levels | `mentalvm ladder eval` | rq E1; P9 |
| **M3** | **Placement.** For each anomalous run, the highest level that shows the symptom and the cause; the length of the smallest explaining cut | `mentalvm ladder eval` | rq E4 |
| **M4** | **Reader.** Lines read and questions answered right per role, by a stand-in LLM reader now and by people later | research harness, from renderings by `mentaldbg` | rq E3 |
| **M5** | **Name stability.** Identical names and shared words across LLM samples; step classes shared with another version of the project | `mentalc ladder compare` | rq E2, E6 |
| **M6** | **Name truth and barrier.** Names whose nouns or claims do not match the step's footprint; names using words of another level | `mentalc` checks | multi-level report 6 |
| **M7** | **Determinism.** How well each upper level predicts its own next step from upper state alone (*det*) | `mentalvm ladder eval` | abstraction-function report |
| **M8** | **Shared steps.** Goals under more than one outcome (convergence); unrelated repetitions under one (divergence) | `mentalc ladder compare` | P7 |
| **M9** | **Cost.** LLM calls, tokens and money per ladder, from the ledger; build time | `mentalc ladder compare` | 6.6 of the specification |

### 7.2 The corpus

| Subject | Why | Available |
|---|---|---|
| rq 2.12, with 2.0 and 3.0 | the measured baseline; one store, a dependency bug | yes, in the research harness |
| report-jobs | the first target; several moves inside one function; Postgres, Redis, S3, two workers | yes; L2 to L5 run on 2026-11-17 with `rjexp/` |
| a system with several services and a subject that changes hands | where stage and exchange should be admitted | to choose |
| a project with runbooks or a documented process | for `documented` and `framework` top vocabulary | to choose |
| a project that is mostly computation, such as a compiler | where outcome and stage have little to work on | to choose |

The three projects still to choose are fixed before any measurement, as in the [resolution plan](resolution.md#the-plan-for-all-six).

## 8. Experiments

### L1. Base strategies on new subjects

The rq result (B over A; C not usable as a default) on more subjects, with C cutting step classes instead of runs.

| | `fixed` | `closing-rules` | `llm-ladder` | `documented` |
|---|---|---|---|---|
| **For** | known to readers; no tests needed | computed, stable, measured best on rq | finds project-specific levels nobody wrote a rule for | brings people's work in, marked as documented |
| **Against** | mixes the part axis into levels; hid 27% of rq's anomalies | rules tuned on one library | disagreed with itself on rq | needs documents; work-as-imagined |

**Decide by:** M1 to M9 on the corpus. The default changes from `closing-rules` only if another strategy wins on M3 and M4 without losing on M1 and M5. Also decide which candidate kinds (O6) are admitted on each subject, and whether stage and exchange are admitted on the multi-service subject.

### L2. Where a move ends

Option O1. On rq the rules closed "mark started" as two moves; the hand grouping saw one. In report-jobs, several moves close inside `generate_report`.

**Decide by:** M3, M4 (developer questions), M5 (move-class stability across rq versions, where 0.50 and 0.86 were measured), M7. `prefer` wins only if its choice beats both fixed rules on at least one subject and loses on none.

**Result on rq (2026-11-17; [`strategies.json`](research/multi-level-models/experiments/results/strategies.json), key `L2`).** All 2,097 runs, three rules, the same observables. M7 was measured by a proxy: how often a move class makes the same kind of observable change (*purity*).

| | `every-write` | `atomic-group` | `write-run` |
|---|---|---|---|
| Moves per run | 7.32 | 7.32 | **5.01** |
| Moves with one effect | 57% | 57% | 36% |
| Move classes | 29 | 29 | 26 |
| Causes named at move / activation (of 554) | 285 / 205 | 285 / 205 | 285 / 205 |
| Longest explaining cut | 6 lines | 6 | 6 |
| Move classes shared with rq 3.0 / 2.0 (call and closing function) | 0.86 / 0.52 | 0.86 / 0.52 | 0.85 / **0.61** |
| Purity: instances with their class's usual change | 0.935 | 0.935 | 0.910 |
| Stand-in reader, developer questions at the move level (3 runs, 2 samples) | 14 of 18, 4 "cannot tell" | 14 of 18, 4 "cannot tell" | **18 of 18** |
| Lines read (witness, cancel, clean) | 15, 9, 14 | 15, 9, 14 | 11, 7, 10 |

- **`atomic-group` is the same as `every-write` on rq.** No on-chain write happens inside an open optimistic transaction except its commit. The job's heartbeat that the first experiment saw as a move of its own comes *before* the WATCH. The rule needs report-jobs, where `generate_report` writes inside a Postgres transaction, to be tested at all.
- **`write-run` is coarser and loses nothing measured.** It merges "record the execution", "mark started" and the `PERSIST` into one move, coarser than the hand grouping (three moves). Every cause stays at the same level, the explaining cut is as short, and the stand-in reader answered every developer question from fewer lines. The four "cannot tell" answers under the other two rules were all one question ("was any commit rejected and retried?") on runs with no rejection. Stability against the older release rose (0.52 to 0.61) and purity fell a little (0.935 to 0.910).
- **`prefer` chooses `write-run`** (mean stability 0.73 against 0.69, outside the tie margin of 0.02).
- **Not decided.** `prefer`'s choice beats the fixed rules on rq, but the rule asks for "at least one subject and loses on none", and rq is one subject. **Proposal:** make `write-run` the working default of `closing-rules`, and repeat L2 on report-jobs, where `atomic-group` can differ. The reader here used mechanical move names (closing function and what it writes), so that only the cut differed between rules; the first experiment's reader used LLM names.

**Result on report-jobs (2026-11-17; [`report-jobs/strategies.json`](research/multi-level-models/experiments/results/report-jobs/strategies.json), key `L2`).** The subject is the real code of `examples/report-jobs`, run by a new harness ([`rjexp/harness.py`](research/multi-level-models/experiments/rjexp/harness.py)):
- one step per SELECT, per session commit, per S3 upload and per Celery message sent or received, on SQLite;
- six two-actor scenarios, explored up to two preemptions: 115 runs, 24 of them anomalous;
- the anomalies are three of the example's four known bugs. Two workers claim one report and upload it twice (14 runs). `requeue_stuck` sets a finished report back to PENDING, so it is delivered again (4). The duplicate idempotency key gives a 500 (6).
- The fourth known bug, a PENDING row nothing will pick up, happens without concurrency, and an oracle that compares with sequential runs cannot see it.

| | `every-write` | `atomic-group` | `write-run` |
|---|---|---|---|
| Moves per run | 3.39 | 3.39 | 3.39 |
| Move classes | 11 | 11 | 11 |
| Causes at outcome / goal / move / activation (of 24) | 14 / 4 / 3 / 1 | the same | the same |
| Longest explaining cut | 3 lines | 3 | 3 |

- **All three rules give the same moves.** In `generate_report` the only write inside an open session transaction is the S3 upload. The upload is a call to an outside service, so it ends the activation before the DONE commit, and a move cannot span two activations. No two consecutive writes go to the same key, so `write-run` joins nothing. The reader test was not run, because the three renderings are identical.
- **How a move is identified (the specification's open question).** Four moves close inside `generate_report`. Identified by their call site (`generate_report:32`, `:48`, `:52`, `:59`), all 11 classes are distinct. But only 5% of them survive a copy of report-jobs that adds comments and one unused line (`move_by_function`). Identified by the function and the *order* of the closing site in it, all 11 survive (1.0) and none collide, with or without the write set. On rq the same identity without the write set scores 0.875 and 0.50 against 3.0 and 2.0, close to the function-only identity (0.86, 0.52).
- **Decided:** `write-run` is the default of O1. It beats the other rules on rq and loses on neither subject, which is the stated rule. `atomic-group` is *Measured*: it changed nothing on either subject. `prefer` is *Measured*: on report-jobs every rule ties. Move classes are identified by call, closing function and the order of the closing site; the write set is a property, not part of the identity.

### L3. Naming from the head

Option O2. Move names agreed exactly in 22% of sample pairs and shared 0.51 of their words. The one wrong name, *record worker heartbeat*, passed every vocabulary check.

**Decide by:** M5 and M6 for moves and activations, with the same six samples (three Opus, three Sonnet). `head` is adopted if move agreement rises clearly and name truth failures do not rise.

**Result on rq (2026-11-17; [`strategies-naming.json`](research/multi-level-models/experiments/results/strategies-naming.json)).** Six new samples (three Opus 5.5, three Sonnet 5.5) on the same 44 step classes, with each move's head (what its closing effect writes, in plain words, and the observable change) and each activation's head move, and the instruction that every thing a move's name mentions must be something its head writes. They are compared with the six samples of the first experiment. Cost $1.45.

| | `children` (first experiment) | `head` |
|---|---|---|
| Moves: identical names / shared words, between two samples | 22% / 0.51 | 22% / 0.56 |
| Activations: identical / shared words | 21% / 0.57 | **6% / 0.37** |
| Goals: identical / shared words | 37% / 0.85 | 34% / 0.80 |
| Move names failing name truth, check as written before the run | 20 of 174 (11.5%) | **6 of 174 (3.4%)** |
| Move names failing name truth, revised check | 5 of 174, among them all three *record worker heartbeat* | **0** |
| Storage words in any name | 0 | 0 |

- **The head makes names true, not stable.** The known wrong name is gone, and the failures fell by about three quarters. Agreement on moves did not rise "clearly": exact matches stayed at 22% and shared words rose from 0.51 to 0.56.
- **Activation names got worse.** Given a head move, the samples named activations after what that move does ("Cancel job across its lists", "Mark job canceled") instead of after the call. Agreement fell from 21% to 6%.
- **The name-truth check as first written was too strict.** Of its 20 failures on the first experiment's names, 15 were not wrong names. "Cancel" named the call a step belongs to; rq's started-jobs list does list executions; a dependent job is the job written. The revision exempts a word that names the step's own call and lets a list stand for what it lists. It was fitted to those failures, so its 0 on the new samples is weaker evidence than the drop from 20 to 6 under the original check.
- **Decision by the stated rule: `head` is not adopted** for stability. **Proposal:** use the head for moves only, as a truth aid. Keep `children` for activations and goals. Keep "generate once, store with the step class" as the answer to instability (it already was). Run the revised name-truth check on report-jobs before relying on it.

**Result on report-jobs (2026-11-17; [`report-jobs/strategies-naming.json`](research/multi-level-models/experiments/results/report-jobs/strategies-naming.json)).** Twelve samples, because report-jobs had no earlier ones: six without the head, six with it, three Opus and three Sonnet each, on 11 move, 5 activation and 4 goal classes. Report-jobs has no user documentation, so the prompt describes the service in a paragraph. The name-truth words and the storage words for report-jobs were written before the samples were seen. Cost $0.69.

| | `children` | `head` |
|---|---|---|
| Moves: identical names / shared words | 9% / 0.43 | **21% / 0.57** |
| Activations: identical / shared words | 12% / 0.49 | 12% / 0.43 |
| Goals: identical / shared words | 25% / 0.67 | 23% / 0.74 |
| Move names failing name truth (of 66) | 2 | 0 |
| Names with a storage word | 0 | **5** ("insert", in two Opus samples) |

- **Here the head raised move agreement clearly**, from 9% to 21% identical. Both "failures" without the head are doubtful: *release claim after failed upload* names the failure as context; *requeue failed report's build task* describes the report.
- **It also let one storage word in.** The sheet lists the commands of a move, and with the head's emphasis two Opus samples called the create step *insert pending report*. The barrier check catches this, and its repair would ask for another word.
- **Activations again did not gain** (shared words fell from 0.49 to 0.43).
- **Decided, over both subjects:** `naming = head` for moves, with the barrier check. Moves on rq became truer, and moves on report-jobs agree more without losing truth. Activations and goals keep `children`. Names are still generated once and stored.

### L4. Overlapping outcomes

Option O3. 4.4% of rq goals change two jobs.

**Decide by:** M3 (does the symptom stay at the top two levels), M4 (manager and product questions on runs with a shared goal), M8. Also check whether divergence marks a wrong subject choice on the multi-service subject.

**Result on rq (2026-11-17; `strategies.json`, key `L4`).** 2,520 outcomes in 2,097 runs.

| | `object-centric` | `partition` |
|---|---|---|
| Goals under two outcomes | 4.4% | 0% |
| Outcomes that miss a change made to their own subject | **0%** | **8.0%** |
| Outcomes whose last change is not the subject's true final state | 0% | 0% |
| Symptoms at outcome / goal (of 554) | 500 / 54 | 500 / 54 |
| Causes at move / activation / outcome | 285 / 205 / 64 | 285 / 205 / 64 |

- **Partition hides changes.** In 8% of outcomes a goal filed under another job changed this job: completing `j1` releases `j2`, and under partition `j2`'s outcome does not show it. Placement is the same both ways, so object-centric costs nothing that was measured.
- **Adopted for rq: `object-centric`** (M3 equal, M8 decides). M4 was not run, because the outcome rendering states the final state of every job, which answers the manager's questions under either rule. Divergence needs the multi-service subject.

**Result on report-jobs.** Not testable: every scenario has one report, so no goal changes two. The two rules give identical outcomes. A scenario with two reports in flight would test it; none of report-jobs' known bugs needs one.

### L5. Which merge test

Option O4. The band failed on rq; `single-child` was fitted to the same data.

**Decide by:** on the corpus, which pairs of levels each test merges, and whether a merged level would have named a finding (test A4 of the multi-level report). The test that never merges a level that names something, and merges the levels that restate a neighbour, wins. On rq, both `single-child` and `span-overlap` should merge stage into goal and exchange into goal.

**Result on rq (2026-11-17; `strategies.json`, key `L5`).** All seven kinds were built as candidates: stage as consecutive goals up to a change of the job's status; exchange as the API call with its answer. Thresholds were fixed before the run: band below 2; single-child at 80%; span overlap at 0.5, measured as the overlap of the two levels' distributions of step length in scheduler steps.

| Pair (upper / lower) | Lower steps per upper step (mean) | Upper steps with one lower step | Length overlap | band | single-child | span-overlap | Anomalies named only at the level dropped |
|---|---|---|---|---|---|---|---|
| outcome / stage | 1.56 | 52% | 0.70 | merge | — | merge | 2 (stage) |
| stage / goal | 1.00 | 99.9% | 0.91 | merge | merge | merge | 2 (stage) |
| goal / exchange | 1.00 | 100% | 1.00 | merge | merge | merge | 0 (exchange) |
| exchange / activation | 1.51 | 49% | 0.48 | — | — | — | 0 |
| activation / move | 2.20 | 30% | 0.42 | — | — | — | 204 (activation) |
| move / effect | 2.01 | 57% | 0.57 | merge | — | merge | **285 (move)** |

- **`single-child` passes.** It merges exactly the two candidates the hand analysis expected (stage into goal, exchange into goal) and keeps activation and move. The stage it drops names one anomalous run of 554: a delete overlapping a work call, whose work left the job's status unchanged. The symptom and the cause of that run are each first visible at stage, which is why it counts twice.
- **`band` and `span-overlap` fail.** Both would merge move into effect, losing the level that names 285 causes, and outcome into stage. Span overlap fails for a plain reason: most moves are one effect long, so the two length distributions coincide. Jaques's test assumes levels whose steps differ in duration, and a level can be needed because of *what* it groups, not how long it lasts.
- **Decided for rq: `single-child`**; `span-overlap` *Measured*, not chosen. `band` was already rejected. Repeat on the multi-service subject, where stage and exchange are expected to be admitted.

**Result on report-jobs (2026-11-17; `report-jobs/strategies.json`, key `L5`).** The same thresholds.

| Pair | Mean fan-in | One child | Length overlap | band | single-child | span-overlap | Anomalies named only at the dropped level |
|---|---|---|---|---|---|---|---|
| outcome / stage | 1.35 | 65% | 0.82 | merge | — | merge | 0 |
| stage / goal | 1.00 | 100% | 0.85 | merge | merge | merge | 0 |
| goal / exchange | 1.00 | 100% | 1.00 | merge | merge | merge | 0 |
| exchange / activation | 1.43 | 57% | 0.63 | merge | — | merge | 0 |
| activation / move | 1.19 | 36% | 0.59 | merge | — | merge | **1 (activation)** |
| move / effect | 1.99 | 33% | 0.33 | — | — | — | 3 (move) |

- **`single-child` passes again.** It merges stage and exchange into goal and drops no level that names an anomaly. `band` and `span-overlap` would also merge activation into move, losing the one cause named at the activation level.
- **A measurement artifact was found and fixed.** The first stage signature included the subject's id. Report-jobs' ids depend on creation order, so a report created as `r2` instead of `r1` made 8 runs look "named at stage". The signature now leaves ids out, like the outcome signature. rq's figures are unchanged by the fix.
- **Decided over both subjects: `single-child`.** `span-overlap` *Measured*, rejected on both.

### L6. Words at the top

Option O5, and strategy `documented`. Needs a project with documents. This experiment is E5 of the multi-level report.

**Decide by:** M4 for manager and product questions; M6 (barrier: no software words at outcome and stage); how many framework terms map to glossary terms with an anchor.

### L7. Static derivation against cuts of runs

The premise of 6.2: closing rules evaluated on the model give the same step classes as cutting explored runs.

**Decide by:** on rq and report-jobs, the share of step classes from runs that the static derivation produces, and the share of statically derived classes that never occur in any explored run. Above about 95% both ways, phase 11 stays static. Otherwise phase 11 takes a run sample as input, and the specification's pipeline changes.

### L8. Views from other fields

P13 to P17: the two-up frame, epitome and zoom back out, the status map, the run-set view, the mixed cut. They do not change a ladder, and they need the viewer and people. **Later**: with the viewer study that the specification leaves open.

## 9. The plan

1. **Now, in the research harness.** L2, L3, L4 and L5 on rq reuse `mlexp/levels.py`, the naming sheet and the reader harness of the [multi-level experiments](research/multi-level-models/experiments/README.md). **Done 2026-11-17** (`mlexp/strategies.py`; $1.65 of LLM calls). Results are under each experiment above and in [findings, section L](findings.md#l-ladder-strategies-on-rq). **Report-jobs done the same day** with a harness for its real code (`rjexp/harness.py`; $0.69 of LLM calls).
2. **When phase 11 is built.** Implement `fixed` and `closing-rules` with every option of 5.2, and `ladder build`, `list`, `use` and `compare`. The golden report-jobs build (C24) covers one ladder per strategy, with answers from the `replay` backend. Run L7 on rq and report-jobs.
3. **When `mentalvm` explores.** `mentalvm ladder eval` replaces the research harness for M1 to M3 and M7.
4. **With the extended corpus.** L1 and L6, on the three subjects of 7.2 still to choose.
5. **Decide.** By the criteria of each experiment. Losing strategies and option values stay, marked *Measured* (S7).

## 10. Changes to accepted documents

Applied on 2026-11-17 at the owner's request, so that `mentalc` takes the ladder as a strategy:

| Document | Change |
|---|---|
| [Specification](specification.md) 5 | Status: the ladder of 5.1 is now strategy `fixed`, one of several under test; the `level` record gains `strategy`, `kind`, `charter` and `admission` (6.4 here) |
| Specification 6.1 | `--ladder` on `build`; the `mentalc ladder` commands |
| Specification 6.2 | Phase 1 reads the ladder strategy from the config; phase 11 runs the configured strategies |
| Specification 6.6 | The ladder roles |
| Specification 13 | The open question on inserting a level points to L5 |
| [Glossary](glossary.md) | *Level* no longer lists one fixed ladder; new terms *ladder* and *ladder strategy* |
| [`mentalc` guidelines](guidelines/mentalc.md) | Module layout: `phase/levels/` with one file per strategy |

Not changed: the specification's ladder table in 5.1 stays as the definition of `fixed`, and the proposals of section 9 of the multi-level report (closing rules, charters, cuts) remain proposals there.

## 11. Open questions

- Whether `documented` is a strategy or an option of `closing-rules`. It is kept separate here because it needs inputs the others do not.
- How a person approves a level that a strategy proposes, and whether that approval is stored as an answer in the cache (so it survives rebuilds) or as configuration.
- Whether more than one ladder should ever be shown at once, for example the fixed ladder's part axis beside the purpose levels.
- How the head effect is chosen when two effects of one step change the observable.
- Where stand-in reader tests live once `mentalc` exists: in the research harness, or in a test crate that renders through `mentaldbg`.
