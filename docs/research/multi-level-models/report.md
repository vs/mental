# A Level Is One Reader's Question, and a Step Ends Where a Goal Is Reached

**Question.** Mental shows one system at several levels. Which levels should exist, how are the transitions of one level mapped onto the transitions of the level above, what is the LLM asked to do, and how does a person see why each level is there? The focus is perception: a person, a developer or a manager, has to understand the domain, the model and a trace in it.

**Short answer.**

1. **A level exists for a reader with a question.** Research on how people describe actions agrees on one structure: going up a level answers *why*, going down answers *how*. A level that cannot say whose question it answers, and what the answer to "why" and "how" is at its edges, should not exist. Each level therefore carries a **charter**: its readers, its question, what one step is, and what it hides.
2. **A step is one goal reached.** People cut activity into events where the goal, the actor, the thing worked on or the causal chain changes. Those points can be found mechanically in Mental's lower model: a message handed over, a wait, a commit, a change in what the level above can observe. So the step map, the record of which lower steps make up each upper step, is computed by **closing rules**, one per kind of level. The LLM names the steps; it does not cut them.
3. **Two axes, not one ladder.** "Why and how" (purpose) and "whole and part" (containment) are different directions. The specification's ladder mixes them: *system*, *services* and *components* describe the same kind of step with different boxes opened. Our own seven-level trial shows the cost: only one of six adjacent pairs of levels gave a real change of grain. The proposal keeps levels for purpose and leaves containment to space zoom.
4. **Levels are admitted by tests, not listed in advance.** A fixed skeleton of seven kinds of level (outcome, stage, goal, exchange, activation, move, effect) is instantiated per project. A candidate level is kept when it changes vocabulary, does not merely restate its neighbour, has a finding or a decision that can only be named there, and reads as a story in its own words.
5. **A trace is shown as a cut, not as a level.** People stay at the highest level that works and go down where something breaks. A trace view should do the same: high everywhere, opened where the anomaly is, with the steps on the causal chain kept and bookkeeping folded.
6. **The LLM gets eight small tasks with typed answers**: name the subjects, the actors, the steps and their goals, write the charters, group lifecycle states into stages, propose intent invariants, and narrate a given cut. Every answer is checked. It is never asked to choose a boundary, drop a step or invent a level's structure.

7. **The experiment supports the core and corrects one rule.** We ran the closing rules on 2,097 explored runs of rq, with the specification's fixed ladder and a free-form LLM ladder beside them (section 8). The rules cut every run without an author. Every one of the 554 anomalous runs shows its symptom at the outcome or goal level, and its cause at the move or activation level in 490 of them, in a cut of at most 6 lines. A stand-in reader answered all manager questions from 2 to 3 lines, all product questions from 3 to 4 lines and all developer questions from 9 to 15 lines; under the fixed ladder the developer needed 24 to 43 lines. A free-form LLM ladder disagreed with itself on where steps end and never reused a step name. The rule that failed is the grain band: on rq most pairs of levels have about two lower steps per upper step or fewer, in both designs. It is replaced by a weaker merge test.

The design in sections 4 to 7 is an inference from the sources. Section 8 reports what was then measured on one library, with an LLM standing in for the human reader; no person has been tested.

**Notes:** [sources, with evidence tags](notes/sources.md). **Walkthrough:** [how one rq run becomes the ladder, step by step, with diagrams](ladder-walkthrough.md). **Builds on:** [abstraction levels](../abstraction-levels/report.md) (how levels are related and checked), [abstraction function](../abstraction-function/report.md) (how upper state is computed and guarded), [trace visualization](../trace-visualization/report.md) (how a trace is drawn).

---

## 1. Each source in one line

| Source | Take for Mental |
|---|---|
| Rasmussen's abstraction hierarchy | Levels are linked by means and ends: up is *why*, down is *how*. Part-whole is a separate dimension. Keep the links between levels visible. |
| Action identification theory | People stay at the highest level that works and drop down when something fails. Open a trace high and descend at the trouble. |
| Event segmentation theory | People cut activity where goals, actors, objects or causes change. Put step boundaries there. |
| Causal network model of stories | Events on the causal chain are remembered; dead ends are not. Show the chain, fold the rest. |
| Basic-level categories | Each reader has a level at which naming is easiest, and experts' level is lower. Give each role a home level. |
| Working memory (Cowan) | About four chunks. One upper step should stand for a handful of lower steps. |
| Expertise reversal effect | Narration that helps a newcomer slows an expert. Make guidance fade. |
| Program comprehension models | Developers switch between a control-flow model, a domain model and a top-down model. Make switching one action. |
| Whyline | Asking "why did" and "why didn't" beats watching. Navigation between levels should be by question. |
| Trace visualization experiments | A trace view beside the code measurably helps (22% less time, 43% more correct). The effect is real and can be measured. |
| Fisheye views | Detail by importance minus distance from the focus. This is how to pick a cut. |
| Cockburn's goal levels | "Sea level" is one actor, one sitting, one goal. This is the home level of a product reader, and a test for it exists. |
| Hierarchical task analysis | Stop expanding when the probability and cost of error are low. A stopping rule for descent. |
| EventStorming | The same notation at three scopes for three audiences. One model, several readers. |
| Domain Storytelling | A story is *pure* (no software) or *digitalized*. Business levels should be pure. |
| APQC process framework | Organisations already name five nested levels of work. Reuse their grain above the software. |
| ArchiMate, ISO 42010 | A viewpoint names its stakeholders and concerns. This is the charter. |
| C4 model | Four zoom levels by containment, each for an audience. It is the part axis, not the purpose axis. |
| Hierarchical code summarization, CodeWiki | Bottom-up LLM summaries lose business context unless grounded in it. Ground every naming task in the glossary. |
| LLM event abstraction | An LLM can label a given group of events. Labelling, not grouping, is the safe task. |
| Mental's rq experiment | LLM-written functions were too fine and their names unstable (overlap 0.54). Keep structure mechanical. |
| Mental's seven-level trial | Each problem has a level where it can be named; switching the whole view between levels was not a design. |
| Lipton's reduction | Right movers, at most one non-mover, left movers make one atomic action. Gives each move a verdict: atomic, guarded or non-atomic. |

## 2. The problem, on one run

The [abstraction-function report](../abstraction-function/report.md) recorded a real bug in rq 2.12 as a run of 42 steps. A client enqueues job `j2`, which depends on job `j1`, while a worker is completing `j1`. The worker's commit is rejected and retried, the retry forgets a cleanup, and `j1` ends finished but still listed as started.

At the lowest level the run reads like this (steps 11 to 25 of 42):

```
11 w1  HSET rq:job:j1                      18 w1  EXEC[…HSET rq:job:j1; XADD rq:results:j1; ZADD rq:finished; … ZREM rq:wip …]
12 w1  WATCH rq:job:j1:dependents          19 w1  WATCH rq:job:j1:dependents
13 w1  SMEMBERS rq:job:j1:dependents       20 w1  SMEMBERS rq:job:j1:dependents
14 c   WATCH rq:job:j1                     21 w1  EXEC[HGETALL rq:job:j2]
15 c   EXEC[HGETALL rq:job:j1]             22 w1  WATCH rq:job:j2
16 c   EXEC[HSET rq:job:j2; ZADD rq:deferred; SADD rq:job:j1:dependents; …]
17 c   EXEC[SADD rq:queues]                25 w1  EXEC[…RPUSH rq:queue; … ZADD rq:finished; …]   (no ZREM rq:wip)
```

(Labels are shortened, and the job ids are filled in by hand; the recorded labels have `*` in their place.)

Nobody understands the system from this. But there is no single better level either. Four readers want four different things from the same run:

| Reader | Question | What they need to see |
|---|---|---|
| A manager | Did the work get done? | Both jobs finished. One record is wrong. |
| A product owner | What did each party do, and what went wrong for the user? | The client submitted `j2`; the worker ran `j1`, then `j2`. After "run `j1`", the job is finished and also shown as running. |
| A developer | Where is the defect? | Completing `j1` overlapped the enqueue. The first commit was rejected; the second commit did less than the first had prepared. |
| The developer fixing it | What exactly was written? | The transaction in step 25 lacks three commands that the one in step 18 had. |

Each row is a level. Each has a reader, a question, a size of step and a vocabulary, and each answer is the *why* of the row below it and the *how* of the row above. The rest of this report is about producing those rows for any system, and showing a person why each one is there.

**A note on words.** The glossary calls the map on state the *abstraction function* and the map on transitions the *step map*. This report is about the step map. The two are tied by one rule, used throughout: **an upper step ends at the lower step where the upper state changes.** The abstraction function says what the upper level can see; the step map groups the lower steps around each change in it.

## 3. What research says about how people see levels

The claims below are summarised from the [notes](notes/sources.md), which give links and say how strong each one is. Most were checked this session only in abstracts and secondary summaries.

**Up is why, down is how.** Three independent traditions arrive at this. Rasmussen's abstraction hierarchy for process control links its levels by means and ends. Cockburn's use-case levels come with the instruction to ask "why" to go up and "how" to go down. Action identification theory finds that people's own descriptions of what they are doing range from *how* (low) to *why* (high). The experimental support for Rasmussen's hierarchy is modest, but one result is directly useful: Ham and Yoon found that operators did best when information was shown at all levels *with the goal-means links preserved*.

**Purpose and containment are different.** Rasmussen kept means-ends apart from part-whole: a pump is *part of* a cooling loop, and it is *a means to* removing heat, and these are different relations. C4 and Mental's space zoom move along part-whole. Cockburn's levels and Mental's macro steps move along means-ends.

**People choose the highest level that works.** In action identification theory, a familiar action that is going well is identified high ("delivering the report"). When it becomes difficult or fails, identification drops ("writing the status") and attention goes to detail.

**Events have natural boundaries.** In event segmentation theory, people cut continuous activity into events without being asked, coarse events containing fine ones. Boundaries fall where the goal of an actor, the actor, the object, the place or the causal relation changes. What happens at a boundary is remembered better than what happens in the middle.

**The causal chain is what stays.** In studies of story comprehension, statements on the causal chain to the outcome are recalled, summarised and judged important; side events are forgotten.

**Each reader has a home level.** There is a basic level at which people name things fastest, and expertise moves it down: a bird expert says "sparrow" as readily as a novice says "bird". Cockburn's sea level is the same idea for work: one actor, one sitting, one measurable outcome.

**About four things at once.** Working memory holds about four chunks. Notation research asks for explicit hierarchy to keep each picture within that.

**Guidance must fade.** Explanations that help a newcomer slow an expert down (the expertise reversal effect).

**Developers switch levels constantly.** Program comprehension studies observe engineers moving between a control-flow model, a domain-level model and a top-down model during a single task.

**Asking beats watching.** With the Whyline, developers asked "why did" and "why didn't" about a run and debugged several times faster in small studies. The [trace-visualization report](../trace-visualization/report.md) reached the same conclusion from research on animation.

## 4. Design rules that follow

Each rule is an inference from section 3 and from Mental's own trials. The right-hand column says what would make the rule checkable.

| # | Rule | From | Checkable as |
|---|---|---|---|
| **R1** | A level has named readers and one question. Its upward neighbour answers *why*, its downward neighbour *how*. | Rasmussen, Cockburn, action identification, ISO 42010 | every step has a parent that states its purpose; the charter is filled in |
| **R2** | Levels move along purpose only. Containment is space zoom. | Rasmussen's two dimensions; the seven-level trial | two levels with the same closing rule are merged |
| **R3** | A step ends where the goal, the actor, the subject or the causal chain changes. | event segmentation | boundaries come from closing rules, not from an author |
| **R4** | One upper step stands for about two to seven lower steps. | working memory; notation research | median fan-in per pair of levels |
| **R5** | Each role has a home level, and a view opens there. | basic level; sea level; expertise | per-role setting; measured by reader tasks |
| **R6** | A view stays high and descends where something is wrong. Every finding is shown at the highest level that can name it, with the way down. | action identification; F-Z4 | each finding carries the level that names it |
| **R7** | A trace shows the causal chain to the outcome; steps off the chain are folded, never deleted. | causal network model | computed from which steps read what earlier steps wrote |
| **R8** | Links between levels are always one action away, and a view may mix levels. | Ham and Yoon; program comprehension; fisheye views; F-Z9 | a view is a cut through the step tree |
| **R9** | Names come from the project's glossary and stay the same across builds. | basic level; the rq experiment's name overlap of 0.54 | names resolve to terms; agreement between LLM samples |
| **R10** | Levels above the software use no software words. | Domain Storytelling's pure stories | the abstraction barrier, with a vocabulary that excludes system entities |
| **R11** | Guidance fades: narration is on for a newcomer and off for an expert. | expertise reversal | a view setting |
| **R12** | Navigation is by question: why, how, why not, what next. | Whyline; animation research | each step offers these four moves |

Two terms used from here on. A **subject** is the thing a piece of work is about and that has a life of its own: a job, a report, an order. (Process mining calls it a case; the glossary already uses *case* for a path through a behavior.) A **reader** is a role that looks at the model: a developer, a product owner, a manager, a coding agent.

## 5. The architecture

### 5.1 Two axes

| Axis | Moving along it | In the glossary | Examples |
|---|---|---|---|
| **Purpose** | why ↔ how; one step stands for several | time zoom; macro step | "run `j1`" expands into "take" and "complete" |
| **Part** | whole ↔ part; one entity contains several | space zoom; the logical and physical facets | the system opens into services, a service into components |

The specification's ladder (5.1) is *business, system, services, components, mechanisms, statements*. Read with these two axes, *system*, *services* and *components* differ in which entities are open. At all three, a step is a message or a call between the entities that are visible. They are positions on the part axis.

The seven-level trial (findings, section Z) shows this in its step counts for one run:

| Level | Steps | Lower steps per step of this level |
|---|---|---|
| Business | 4 | 1.5 |
| System | 6 | 3.5 |
| Services | 21 | 1.2 |
| Components | 26 | 1.0 |
| Mechanisms | 26 | 1.2 |
| Statements | 30 | 1.1 |
| Postgres internals | 34 | — |

Only system → services changes the grain by more than a factor of two. Four of the six moves down change almost nothing a reader would notice as a new chunking. This is one hand-written run, and a level that adds detail only where its vocabulary applies will always pass many steps through (F-Z2), so the figures overstate the problem. But they agree with the rule: a move along the part axis reveals who talks inside a box, and does not change what a step is.

**Proposal.** A *level* is a position on the purpose axis. At any level, space zoom chooses which entities are open, as it does today. "Services" and "components" stop being levels and stay as what they already are in the logical facet.

### 5.2 Seven kinds of level, each with a closing rule

A **closing rule** says when one step of a level ends. It is stated over things the lower model already has, so it can be evaluated on any run without an author.

| Kind | One step is | It closes when | Mechanical source | Typical reader | Outside analogues |
|---|---|---|---|---|---|
| **Outcome** | a subject's whole life: value delivered or not | the subject reaches a terminal state | lifecycle machine; representation invariant | executive, manager | APQC process; Cockburn cloud |
| **Stage** | a subject moves to its next stage, often changing hands | the subject's stage changes | groups of lifecycle states; who acts next | manager, analyst, domain expert | APQC activity; EventStorming process level; Cockburn kite |
| **Goal** | one actor gets one thing done | the actor's request is answered and the subject is at rest (a quiescent state) | external ports; observables | product, support, QA | Cockburn sea level; APQC task; a use case |
| **Exchange** | one request between two visible parties, with its answer | the answer or acknowledgement arrives | wires and their replies | architect, operations | a span in distributed tracing; C4 arrows |
| **Activation** | one entity handles one message | it must wait | already in the glossary | developer | a handler run |
| **Move** | effects that together make one move of a mechanism: a claim, an attempt, a commit | the mechanism's state changes, or the atomic group ends | atomicity evidence; mechanisms | developer on a concurrency problem | Mental's mechanisms level |
| **Effect** | one read, write or send | the interpreter rule fires | the interpreter | developer, coding agent | a statement; a Redis command |

Below *effect*, a pack's own levels attach as a branch for that entity only (F-Z5).

Three properties of this skeleton matter.

- **Each rule is a change that people perceive as a boundary.** A terminal state and a stage change are changes of *goal*. A request answered is a change of *actor*. A wait is a break in the *causal chain*. This is R3 turned into definitions.
- **The top three are pure.** Outcome, stage and goal are stated over subjects, actors and observables, with no service, queue or table in them (R10). A manager's level never says "Redis".
- **The rule ties the step map to the abstraction function.** For outcome, stage and goal, "closes when" is a change in upper state, as computed by the selected observables of the abstraction-function report. The rq experiment found this by necessity: its oracle could not be written until the worker's operation was split into *take* and *complete* at exactly such a boundary (F-A9).

### 5.3 From runs to the model: step classes

Closing rules cut a *run* into a tree of steps, like the span tree of a distributed trace. The model needs more than that, because any run must be showable at any level without new work (F-Z8). So each cut is generalised to a **step class**: a kind of upper step, identified by names from the code and the glossary (P9).

| Level kind | A step class is identified by |
|---|---|
| Effect, move | the node or nodes of a behavior, and the mechanism |
| Activation | the port, and the case that ran |
| Exchange | the wire, and the outcome of the answer |
| Goal | the actor, the external port, the subject's kind, and the change in observables |
| Stage | the subject's kind, and the stage it enters |
| Outcome | the subject's kind, and the terminal state |

Each lower step class names the upper step class it belongs to, or is marked as support (section 5.6). That is the step map, stored in the model. A step class can have several possible expansions (a commit that succeeds at once, or after a retry); these are its cases at the upper level.

Two lower steps of different upper steps can interleave in time. The mapping is therefore by identity, never by position (F-Z1), and an upper level's order of steps is a partial order (F-Z7).

**A verdict on each move (proposed, not run).** The closing rule decides where a move ends; it does not say whether the move is safe. Lipton's reduction gives that from the move's own effects: a move whose reads and write can be reordered into one atomic block is **atomic**; one whose write is conditional on what it read, inside the store (`UPDATE … WHERE status = 'PENDING'`, a Redis `WATCH`), is **guarded**; one that reads, decides in the program and then writes unconditionally is **non-atomic**. The verdict is computed from footprints and pack rules, is stored with the move class, and is what the degree of interest (5.7) counts as "a non-atomic group". On report-jobs the claim in `generate_report` (`tasks.py:28-32`: read the row, test `PENDING`, write `RUNNING`) is one move with the verdict non-atomic, which is where the duplicate-claim bug lives. Whether the verdict is a useful pointer for the interpreter's search has not been measured.

### 5.4 Admission: why each level exists

The skeleton has seven kinds. A project does not have to use seven. A candidate level is **admitted** when it passes these tests, and the results are stored in its charter so that a person can read why the level is there.

| Test | Passes when | Otherwise |
|---|---|---|
| **A1. Reader and question** | the charter names at least one role and one question | not a level |
| **A2. New words** | the level adds or hides vocabulary against its neighbours (the earlier report's CakeML rule) | merge with the neighbour |
| **A3. Grain** | on explored runs, the median upper step stands for about two to seven lower steps | near 1: merge. Far above 7: look for a level in between, usually a project-specific one |
| **A4. Named only here** | at least one finding, invariant or decision can be stated at this level and not at the one above (F-Z4) | keep it available but out of the default ladder |
| **A5. Stands alone** | a trace at this level reads as a causal chain in its own words: barrier check passes, and the upper level is deterministic enough to run (the *det* measure of the abstraction-function report) | fix the vocabulary or the observables |
| **A6. Why and how closed** | every step class has a parent that states its purpose, and every expansion is recorded | incomplete; report coverage |

A3 answers an open question of the specification: "when an extra level should be inserted because one is too crowded". Crowding is measurable, and the fix is a level whose closing rule falls between its neighbours'. Project-specific levels, such as Git's porcelain and plumbing, enter here: the LLM proposes a charter and a closing rule in terms of the project's own entities, and the same tests decide.

The band "two to seven" is a starting value taken from working-memory figures of three to five chunks and the older seven. It is a parameter to tune by experiment, not a law.

**After the experiment (section 8.2): A3 as written does not hold.** On rq the mean number of lower steps per upper step is 2.0, 2.2, 1.5 and 1.6 going up design B, and three of the fixed ladder's four figures are below two as well. A subject's life simply has few goals, and a goal few activations. What the data does separate cleanly is a level that *restates* its neighbour: 84% of activations contain exactly one status change, so a stage level would repeat the activation level, and an exchange is the goal again in every case. The revised test: **merge two levels when at least about four in five upper steps have exactly one lower step; look for a level in between when the typical upper step has more than seven.** Below that, a small grain is not a reason to drop a level; test A4 decides.

### 5.5 The level charter

```json
{"id": "level:goal",
 "kind": "goal",
 "readers": ["product", "support", "qa"],
 "question": "What did each party get done, and did it work?",
 "step_is": "one actor gets one thing done",
 "closes_when": "the actor's request is answered and the subject is at rest",
 "hides": ["messages between services", "retries", "worker bookkeeping"],
 "why": "level:stage",
 "how": "level:exchange",
 "admission": {"median_fan_in": 2, "named_only_here": ["finding:finished-job-listed-as-started"]},
 "provenance": "inferred"}
```

The `readers` and `question` are written by the LLM in the project's words and can be edited by a person. Everything under `admission` is computed. A viewer shows the charter when a person asks "why is this level here?", and it shows the `why` and `how` neighbours as the two ways out.

### 5.6 Support steps and the causal chain

Not every lower step serves the upper step it happens inside. In the rq run, 12 of the 42 steps write only the worker's own bookkeeping hash (its state, its heartbeat). They are real, and no later step on the way to the outcome reads them.

A step is **on the causal chain** when something it wrote, sent or decided is used by a later step that is on the chain, starting from the steps that change the level's observables. Everything else is a **support step**. This is computed from what steps read and write (the footprint), with no judgment involved. Support steps are folded behind a count ("12 bookkeeping steps"), and following the glossary's rule for collapse they are never deleted and never hide an error, an assumption or a non-atomic group.

### 5.7 A trace is a cut

The throwaway viewer of the seven-level trial switched the whole view from one level to another, and that was judged not a design (F-Z9). The research says why: people stay high and go down only where there is trouble.

The steps of a run form a tree under the step map. A **cut** is a set of steps that covers the run once: every effect is under exactly one step of the cut. A view shows a cut. "The goal level" is the cut that takes every goal. The useful cuts are mixed.

The cut is chosen by a degree of interest, after Furnas: a step's importance minus its distance from the focus. Importance is computed from the model: a step under which a finding is named, a step where an invariant breaks, a non-atomic group, a step on the causal chain. Distance is counted in the step tree from the current focus. Steps above a threshold are opened one level; the rest stay closed. This is a *lens* in the glossary's sense, and the cut is *view state*: it never changes the model or the run.

### 5.8 The worked example

The 42-step witness, grouped by hand for this report by applying the closing rules of 5.2. No tool produced this grouping; it shows what the architecture is meant to yield.

```
OUTCOME  both jobs finished                                              ⚠ cause below
│
├─ GOAL  client: submit j2, to run after j1                              (5 effects)
│   └─ ACTIVATION  enqueue-dependent
│       ├─ MOVE  register j2 as waiting on j1    (optimistic transaction, 4 effects)
│       └─ MOVE  register the queue              (1 effect)
│
├─ GOAL  worker: run j1                                                  ⚠ j1 finished, still listed as started
│   ├─ ACTIVATION  take j1                       (10 effects, 5 of them bookkeeping)
│   │   ├─ MOVE  pop the job
│   │   ├─ MOVE  record the execution
│   │   └─ MOVE  mark started
│   └─ ACTIVATION  complete j1                   (12 effects, 1 bookkeeping)   ⚠ overlaps the client's goal
│       ├─ MOVE  attempt 1: commit rejected      ← the client wrote j1's dependents in between
│       └─ MOVE  attempt 2: commit accepted      ⚠ commits less than attempt 1 prepared
│           └─ EFFECT 25  EXEC[…]                ⚠ lacks DEL rq:execution, ZREM rq:wip, ZREM rq:executions
│
└─ GOAL  worker: run j2                                                  (15 effects, 6 bookkeeping)
    ├─ ACTIVATION  take j2
    └─ ACTIVATION  complete j2
```

| Level | Steps | Lower steps per step | Where the problem can be named |
|---|---|---|---|
| Outcome | 1 | 3 | only as a marker: a record disagrees |
| Goal | 3 | 1.7 | the symptom: finished, and still listed as started |
| Activation | 5 | 2.2 | the overlap: completing `j1` ran across the client's enqueue |
| Move | 11 | 2.7 (of the 30 effects on the chain) | **the cause: the retry committed less than the first attempt prepared** |
| Effect | 42 (30 on the chain, 12 support) | — | the evidence: three commands missing from one transaction |

What the example shows:

- **The four readers of section 2 each have their row.** The manager reads one line with a marker. The product owner reads three goals and one symptom. The developer reads the overlap and the two attempts.
- **The cause has a level.** "The retry committed less than the first attempt prepared" can be said at the move level and nowhere above it. Below it, it is three missing commands that must be noticed by comparing two long lines. This is F-Z4 again, and it is test A4 for the move level.
- **The cut that explains the bug is mixed.** It has 2 closed goals, 1 closed activation and 2 moves: five lines for 42 steps. Effect 25 is shown under the second move as its evidence, on demand. The tree above is that cut, with the closed parts opened one level for orientation.
- **No stage or exchange level appears.** One queue and one store give them nothing to add in this run: stage would restate goal, and exchange would restate activation. Tests A2 and A3 would merge them. A system with several services, or a subject that changes hands between people, would admit them.
- **Some fan-ins are below two.** With one or two jobs the upper levels are small. The experiment confirmed this for all runs, and test A3 was revised (5.4).

**The same run, cut by the implemented rules** (`experiments/results/levels.json`, key `witness`):

| | By hand | By the rules |
|---|---|---|
| Goals | 3 | 3, the same |
| Activations | 5 | 5, the same |
| Moves | 11 | 13 |
| Support steps | 12 | 14 |

The rules differ from the hand grouping in four places, all at the bottom. They close a move at every write, so "mark started" became two moves (the status write, then a `PERSIST`), and the job's heartbeat before the first commit attempt became a move of its own. They fold two more steps as support: registering the queue name, which nothing in the run reads, and a heartbeat on `j2`. The two attempts, the rejected one and the one that commits less, come out exactly as drawn. The hand version read intent into the code ("mark started" is one thing); the rule sees two writes. Whether a move should close at every write or at the end of an atomic group is a choice the experiment exposes and does not settle.

### 5.9 Processes around the software

The same skeleton extends upward and sideways to what people do: a customer asks for a report, support requeues a failed job, a reviewer approves a change, the on-call engineer restarts a worker.

| What | Where it comes from | How it enters the model | Provenance |
|---|---|---|---|
| Steps people take through the software | external ports: routes, commands, schedules | goals, with the person's role as the actor | `inferred` |
| Steps people take around the software, written down | runbooks, contribution guides, pipeline definitions, ownership files, issue templates | stages and goals of an actor, with the document as anchor | `attested` |
| Steps people take that nothing records | an interview, or the LLM's reading of the docs | what the actor *may* do | `assumed`, always shown as such |

Three points keep this honest.

- **A written process is work as imagined.** Code and traces are work as done. Where a documented stage has software steps under it, the pair check applies and a disagreement is a finding. Where it has none, the level uses the specification's `assume` pattern and nothing is checked. The specification already lists "how business steps that are done by people or by physical processes are checked" as open; this report does not close it.
- **Organisations already have level names.** The APQC framework's process, activity and task line up with outcome, stage and goal. When an organisation uses such a framework, its names should be the level's vocabulary.
- **Pure first.** Following Domain Storytelling, the stage level is written without software words. The same stage can then be shown *digitalized* by descending one level, and that is what makes it readable to a manager and still checkable.

### 5.10 The pipeline

Each step says who does it, following the specification's division of labour (P7).

| # | Step | Who | Output |
|---|---|---|---|
| 1 | Find candidate subjects (entities with identity and a lifecycle) and actors (triggers of external ports; roles in documents) | extractors | candidates |
| 2 | Choose and name subjects and actors | LLM, checked | terms in the glossary |
| 3 | Select observables and derive the representation invariant, per the abstraction-function report | checkers | upper state |
| 4 | Explore sequential runs of each entry point | interpreter | runs |
| 5 | Apply the closing rules; build the step tree of each run; mark support steps | checkers | step trees |
| 6 | Generalise to step classes; write the step map into the model | checkers | step classes |
| 7 | Group lifecycle states into stages; add documented people steps | LLM, checked | stages |
| 8 | Run the admission tests; merge, split or drop candidate levels | checkers | the project's ladder |
| 9 | Name each step class and state its goal; write each charter | LLM, checked | names, charters |
| 10 | Propose intent invariants per level | LLM, triaged | invariants |
| 11 | Check: coverage, barrier, pair, invariants, admission | checkers, interpreter | verdicts and findings, each with the level that names it |
| 12 | Set home levels per role | defaults, then the person | view defaults |

At view time: the viewer or the agent picks a cut by degree of interest, and the LLM may narrate that cut for a reader.

## 6. What the LLM is asked

Eight task types. Each has a small input, a typed answer and a check, and each is one entry in the call ledger. The inputs follow the finding that bottom-up LLM summaries lose business context unless grounded: every naming task receives the glossary and the names of the level above.

| Task | The LLM is given | It answers | Checked by |
|---|---|---|---|
| **subjects** | candidate entities with their lifecycles; glossary; documentation extracts | which candidates are subjects, with a term for each | the entity has identity and a lifecycle; the term is anchored in code or docs |
| **actors** | external ports and triggers; documented procedures | a role term for each | anchored to a route, command, schedule or document |
| **name-step** | one step class: its children's names, its actor, its subject, the change in observables, the level's vocabulary | a verb-and-object name and one sentence "so that …" | only the level's words (barrier); two classes with different results get different names; agreement across samples; **name truth**: every noun of the name maps to something the step writes, sends or decides, and every claim it makes (a backoff, a retry count, an owner) matches the effects |
| **why-link** | a step class and its computed parent | the sentence that says how it serves the parent | mentions the parent's subject or goal terms; the link itself is computed, not chosen |
| **stages** | a subject's lifecycle states and the goals that move it | named stages, each a set of states | the stages partition the states; each is contiguous in every explored run |
| **charter** | an admitted level, its steps' names, its computed admission results | readers and question, in the project's words | fields complete; a person may edit |
| **intent-invariants** | a level's vocabulary and step names; documentation | invariants in the level's words | triage against sequential runs, per the abstraction-function report |
| **narrate** | a cut of one trace, a reader's role, the charters | one sentence per step of the cut | every sentence cites step identifiers on the cut; counts and totals are inserted by the tool |

What the LLM is not asked, and why:

- **To place a boundary or group steps.** Closing rules do it. LLMs drift to code-level granularity, and on rq their own state functions were two to five times too fine.
- **To decide that a step does not matter.** Support steps are computed from reads and writes. The specification forbids the LLM to remove a branch (F-R3); the same holds for a step.
- **To invent the ladder.** It may propose a project-specific level as a charter plus a closing rule; admission tests decide.
- **To write a trace or a number.** Traces come from the interpreter. Aggregates such as "deliveries: 2" are counts of lower steps (F-Z6) and are computed.

**The prompt for `name-step`, in outline.** "This is one kind of step at the *goal* level of this system. A goal step is one actor getting one thing done. Actor: *worker*. Subject: *job*. Before: status `queued`, listed in the queue. After: status `finished`, listed as finished, one result. It consists of: *take the job*, *complete the job*. The level above calls the surrounding step: *job done*. Words you may use: (glossary terms of this level). Give a name of at most four words, verb first, and one sentence beginning 'so that'." The answer "run the job, so that its result is available to the client" passes; "execute perform_job and update Redis" fails the barrier check.

**The name-truth check** was added after the experiment (8.4), where the one wrong name, *record worker heartbeat* for a write of the job's heartbeat, passed every vocabulary check. It compares the name's nouns and claims with the step's footprint: *worker* is not the key the step writes, so the name fails. On report-jobs it would reject *retry with backoff* for `generate_report`'s fixed 30-second delay. Nouns are matched through the glossary's anchors, so the check is mechanical; it has not been run.

## 7. What a person sees

These follow from the rules of section 4 and are proposals for the viewer study that the specification leaves open.

- **Opening.** A view opens at the reader's home level: outcome or stage for a manager, goal for product and support, activation for a developer. Experts can set it lower.
- **Why is this level here?** Each level's name opens its charter: readers, question, what a step is, what is hidden, and the measured grain.
- **Four moves on every step.** *Why* goes to the parent and shows its goal sentence. *How* opens the step one level. *Why not* is offered when an expected step did not happen, and leads to the guard or the missing message. *What next* follows the causal chain.
- **Findings come with a route.** A finding is listed at the highest level that names its symptom, with a marker "cause below" and one action that opens the cut down to the level that names the cause.
- **Folded, with a count.** Support steps and unchanged steps show as one line with a number.
- **Narration that fades.** A newcomer gets a sentence per step; an expert gets names only.
- **One vocabulary per line.** A line of a trace uses the words of its level. A mixed cut therefore shows, by its wording, which level each line is at.
- **For the coding agent.** The focus sent to the agent carries the step, its charter's question, its parent's goal sentence and its children's names: the why and the how, in a few lines.

## 8. The experiment

Run on 2026-11-16. Code and results are in [`experiments/`](experiments/README.md). The subject is rq 2.12, with the eight two-actor scenarios and the oracle of the [abstraction-function report](../abstraction-function/report.md): 2,097 runs explored up to two preemptions, 554 of them anomalous. The harness was extended to record, for every step, the concrete keys it touches, the rq functions on the call stack and whether a transaction was rejected, and all runs were explored again.

### 8.1 Three designs

| Design | Who decides the levels | Who decides the boundaries | How it was run |
|---|---|---|---|
| **A. Fixed ladder** (the specification today) | the specification | each level's own definition | read onto rq: *business* is a change of the status field, *system* a call across the library's boundary, *services* a round trip to Redis, *components* the same plus calls between rq classes, *statements* single commands. *Mechanisms* renames steps without regrouping them (as in the seven-level trial) and was not run. |
| **A+** | as A | as A | A, with each step annotated by the documented observables instead of the status field alone. It separates the effect of the ladder from the effect of the observables. |
| **B. Closing rules** (this report) | kinds, admitted by tests | closing rules, computed | implemented in `mlexp/levels.py`: effect, move, activation, goal, outcome; stage and exchange as candidates |
| **C. LLM-proposed ladder** | the LLM | the LLM | 3 samples of Opus and 3 of Sonnet, given the rq sources and the 42-step witness, asked for levels and for a cut of the run at each |

How B's rules were made concrete. An *effect* is one scheduler step. A *move* closes after each on-chain effect that writes, or after a rejected transaction; reads join the move they feed. An *activation* closes where the API call returns or calls the job body. A *goal* is one API call. An *outcome* collects, per job, the goals that change what the documentation lets a user observe about it. An effect is on the causal chain when it changes an observable, writes a key that a later on-chain effect reads, or is a read of an actor that then acts on the chain in the same call.

### 8.2 E1: do the rules cut runs, and at what grain?

They do. All 2,097 runs were cut with no author and no LLM (`results/levels.json`).

| Design B | Steps per run (mean) | Lower steps per step (mean) | Upper steps with exactly one lower step |
|---|---|---|---|
| Outcome | 1.2 | 1.6 goals | 52% |
| Goal | 2.2 | 1.5 activations | 49% |
| Activation | 3.3 | 2.2 moves | 30% |
| Move | 7.3 | 2.0 effects on the chain | 57% |
| Effect, on the chain | 14.7 | | |
| Effect, all | 25.7 | | |

| Design A | Steps per run (mean) | Lower steps per step |
|---|---|---|
| Business | 3.0 | 1.8 |
| System | 5.5 | 4.6 |
| Services | 25.7 | 1.5 |
| Components | 38.0 | 1.4 |
| Statements | 53.9 | |

- **Support steps.** 43% of all effects are off the causal chain and fold away: worker state, heartbeats, registrations nothing reads. The move level therefore shows 7.3 lines where the raw run has 25.7.
- **The grain band fails.** No pair of levels in B reaches a mean of 2.5, and A has one large jump (services to system, 4.6) and nothing else above 1.8. Test A3 was revised (5.4).
- **Candidates that restate a neighbour.** 84% of activations contain exactly one change of a job's status, so a *stage* level would repeat the activation level. An *exchange* is the API call, that is the goal, in every case. Both are merged, as the hand analysis expected.
- **The top is not a tree.** 4.4% of goals change two jobs (completing `j1` also releases `j2`) and so belong to two outcomes. Outcomes overlap; the step map above the goal is a relation, not a partition.
- **A's levels do not order by purpose.** From services down, design A only adds lines (25.7, 38.0, 53.9) without regrouping. This is the part axis, as section 5.1 argued from the earlier trial.

### 8.3 E4: at which level does each anomaly show?

For each of the 554 anomalous runs, the highest level at which the run differs from every clean run of its scenario (`results/levels.json`, key `placement`). *Symptom* compares what happened together with the observable change of each step. *Cause* compares only which steps happened and how the steps of different actors overlap.

| | Outcome | Goal | Activation | Move | Effect |
|---|---|---|---|---|---|
| **B, symptom** | 500 | 54 | 0 | 0 | 0 |
| **B, cause** | 64 | 0 | 205 | 285 | 0 |

| | Business | System | Services | Components | Statements |
|---|---|---|---|---|---|
| **A** (status field only) | 407 | 0 | 147 | 0 | 0 |

- **Every symptom is at the top two levels of B.** A manager's or a product owner's level shows that something is wrong in all 554 runs.
- **Causes sit two or three levels down.** 285 at the move level (a rejected attempt, a commit with a different write set) and 205 at the activation level (which activations overlapped). The 64 at the outcome level are cleanup runs, where the set of goals that touched the job already differs. This is F-Z4 measured: each problem has a level where it can be named, and it is not the level of its symptom.
- **Under A, 147 anomalies (27%) are invisible above the raw commands.** They include all 20 runs of the real rq bug. The business level of A is the status field, and the status is right in those runs. The first level of A that differs is *services*, at 25.7 lines per run.
- **The explaining cut is short.** The smallest cut that opens only what differs from clean runs has a median of 6 lines for the dependency bug, 3 for cancel, and 2 for cleanup and delete; the maximum over all 554 is 6.
- **The activation level earns its place by test A4.** Its grain is small (1.5 per goal), but 205 causes are named there and nowhere above.

### 8.4 E2: how stable are names when the LLM only names?

Six samples (3 Opus, 3 Sonnet) each named all 29 move classes, 8 activation classes and 7 goal classes from one sheet (`results/naming-sheet.json`, `results/llm.json`).

| Level | Classes | Pairs of samples with the identical name | Shared words between two samples' names | Storage words used | Two classes given one name |
|---|---|---|---|---|---|
| Goal | 7 | 37% | 0.85 | 0 of 42 | 0 |
| Activation | 8 | 21% | 0.57 | 0 of 48 | 0 |
| Move | 29 | 22% | 0.51 | 0 of 174 | 0 |

- **The barrier holds.** No name at any level used a Redis command, key or storage word, and no sample gave two classes the same name.
- **Wording is not stable.** Two samples pick the identical name for a goal about one time in three, and for a move one time in five. Goal names share most of their words ("enqueue a job", "enqueue job"); move names share half. The plan's pass mark, agreement well above the 0.54 of free-form names, is met for goals and not for moves.
- **Consequence.** A name must be generated once and stored with its step class, never regenerated on rebuild. Because step classes are identified from code (8.7), the stored name stays attached. The majority names read well: goals are *enqueue a job*, *run next job*, *cancel a job*, *requeue a failed job*; the two activations of a worker are *pick up and start job* and *record job outcome*; the two moves of the bug are *attempt success commit, rejected* and *finish job and release dependents*.
- **One wrong name.** The majority called a write of the job's heartbeat *record worker heartbeat*. Names are checked for vocabulary, not for truth; a check of a name against what its step writes is missing. It is now proposed in section 6 (name truth).

### 8.5 Design C: what a free-form LLM ladder looks like

Six samples proposed levels and cut the 42-step witness themselves (`results/llm.json`, key `ladder`).

| Measure | Result |
|---|---|
| Number of levels | 3, 3, 4, 3, 4, 3 |
| Level names | every sample has an outcome-like top level ("Outcome", "Business outcome", "Business story") and a "Job lifecycle" level; below that: "rq operation", "Protocol phase", "Redis transaction protocol", "Redis mechanisms", "rq call" |
| Valid cuts (every step in exactly one group per level) | all levels of all samples |
| Levels nest (each lower group inside one upper group) | in 3 of 6 samples; broken in the other 3 |
| Agreement between samples on boundaries (best-matching levels, adjusted Rand) | mean 0.75, minimum 0.28 |
| Step names shared between two samples | 0.01 |
| Agreement with B: goal level | 0.83 to 1.0 |
| Agreement with B: activation level | 0.54 to 0.88 |
| Agreement with B: move level | 0.42 to 0.66 |
| Cost | $0.30 to $0.65 per sample, for one run |

- **The LLMs find the same top as the rules.** Two or three groups at the top, matching B's three goals closely, and a "Job lifecycle" level in all six that matches B's activation level. That is independent support for those two levels, and "job lifecycle" is the name a reader would expect for the activation level in a system like rq.
- **Below the top they disagree with each other.** Between 7 and 26 groups at the lowest level, half of the ladders not nested, and no step name in common.
- **C must be asked again for every run.** B's rules cut all 2,097 runs in seconds; C's cut exists for one run per call.
- **What C is good for.** One Opus sample named the defect in a step name: "execution cleanup skipped, x1w1 left in wip". An LLM that reads the code can explain. That is the `narrate` task, on a cut the rules made.

### 8.6 E3: can each reader's questions be answered at their level?

A stand-in reader (Sonnet, with no code and no other context) was given one run rendered at one level, a clean run of the same scenario at the same level, and seven multiple-choice questions, each with a "cannot tell" option. The questions and their keys were written before any rendering existed: two for a manager (how many jobs finished; is any record inconsistent), two for a product owner (did two parties' actions on a job overlap; what did the client get done), three for a developer (was a commit rejected and retried; which part is responsible; what does the last commit fail to do). Three runs: the bug witness, a cancel anomaly, and a clean run with overlap. Each prompt was run twice: 90 calls (`results/reader.json`).

| Variant and level | Lines to read (three runs) | Manager right | Product right | Developer right | Wrong answers |
|---|---|---|---|---|---|
| **B** outcome | 3, 2, 3 | **12 of 12** | 6 of 12 | 6 of 18 | 0 |
| **B** goal | 4, 3, 4 | 12 | **12 of 12** | 9 | 0 |
| **B** activation | 6, 4, 6 | 12 | 10 | 13 | 2 |
| **B** move | 15, 9, 14 | 12 | 8 | **18 of 18** | 4 |
| **B** effect | 43, 24, 40 | 12 | 12 | 18 | 0 |
| **A** business | 7, 4, 7 | 6 | 6 | 1 | 3 |
| **A** system | 9, 6, 9 | 6 | 9 | 0 | 0 |
| **A** services | 43, 24, 40 | 11 | 12 | 16 | 3 |
| **A** components | 65, 36, 61 | 11 | 12 | 16 | 3 |
| **A** statements | 111, 50, 98 | 10 | 12 | 14 | 6 |
| **A+** business | 7, 4, 7 | **12 of 12** | 6 | 7 | 3 |
| **A+** system | 9, 6, 9 | 12 | **12 of 12** | 9 | 0 |
| **A+** services | 43, 24, 40 | 12 | 12 | **18 of 18** | 0 |
| **A+** components | 65, 36, 61 | 12 | 12 | 18 | 0 |
| **A+** statements | 111, 50, 98 | 12 | 12 | 18 | 0 |

- **Each reader has a level in B where everything is answered.** The manager at outcome (2 to 3 lines), the product owner at goal (3 to 4 lines), the developer at move (9 to 15 lines). Below a reader's level the answers cost more lines and, apart from one ambiguous question (last point), stay right; above it the reader says "cannot tell" and does not guess.
- **The ladder's gain is the developer's level.** With the same observables (A+), manager and product questions are answered from as few lines as in B. The developer's are not: A+ has nothing between *system* (9 of 18) and *services* (18 of 18, at 24 to 43 lines). B's move level answers all 18 from 9 to 15 lines. The activation level answers 13.
- **The observables' gain is the manager's level.** Design A as it stands answers half of the manager's questions at its business level and never all of them, because the status field does not show an inconsistent record. This repeats the abstraction-function report's finding from the reader's side.
- **More detail made the reader worse under A.** At the statements level (50 to 111 lines) the reader gave 6 wrong answers, more than at any other level of any variant.
- **B's six wrong answers are one question.** "Did one party start acting on a job while another party's action on that same job was still in progress?" The key says yes whenever two goals overlap in time. At the move and activation levels the reader saw that the client was writing `j2` while the worker handled `j1` and answered no. The question was ambiguous about "the same job"; it was fixed in advance and is reported as scored.

### 8.7 E6: do step classes survive a new version?

Step classes computed on rq 2.12 compared with those on 3.0 and 2.0 (`results/levels.json`, key `class_stability`).

| Step class, identified by | Shared with rq 3.0 | Shared with rq 2.0 |
|---|---|---|
| Goal (API call) | 1.00 | 1.00 |
| Activation (API call and part) | 1.00 | 1.00 |
| Move (call, closing function, set of key families written) | 0.50 | 0.46 |
| Move (call and closing function only) | 0.86 | 0.52 |

The upper classes are unchanged across two years of releases. Move classes are stable only when identified by the function that closes them; the set of keys a move writes changes with every storage change. A move's identity should come from code names, as P9 already says, and its write set is a property, not part of the name.

### 8.8 What the data supports

| Question | Answer from the data |
|---|---|
| Can the step map be computed by closing rules, with no author? | Yes, on every run. The grouping matches the hand-made one at goal and activation level and differs slightly at the move level. |
| Do the rules give a tree? | Up to the goal, yes. Outcomes overlap in 4.4% of goals. |
| Is the grain band a usable admission test? | No. Replaced by merging levels that restate each other. |
| Does each problem have a level? | Yes: symptoms at outcome and goal, causes at activation and move, in cuts of at most 6 lines. |
| Does each reader have a level? | For the stand-in reader, yes: 2–3, 3–4 and 9–15 lines for manager, product and developer. |
| What does the ladder add beyond better observables? | The developer's level: 9–15 lines against 24–43. Nothing measurable for manager and product. |
| Should the LLM propose the ladder and the cuts? | No. Its ladders agree at the top, disagree below, half do not nest, share no names, and cost a call per run. |
| Are LLM names stable enough to regenerate? | No. Generate once, store with the step class. |
| Which levels does rq end up with? | Five: outcome, goal, activation (which readers would call the job lifecycle), move, effect. Stage and exchange merge away. |

**Not run.**

- **E3 with people.** The reader was an LLM. It measures whether an answer is present at a level and can be read off; it does not measure a person's time or errors.
- **E5, processes from documents.** rq has no runbooks or pipeline that describe people's work. This needs another project.
- **Other subjects.** One library with one store. Stage and exchange were merged here for reasons specific to it; a system with several services and a subject that changes hands is where they would be tested.
- **Design A's mechanisms level**, which renames steps without regrouping.

**Costs.** The naming and ladder samples cost $3.93, the 90 reader calls $1.34.

## 9. Where this touches the specification

Recommendations for the owner to decide. Nothing has been changed.

| Section | Today | Proposed |
|---|---|---|
| 5.1 The ladder | six fixed levels on one ladder, with a physical side | levels on the purpose axis only, instantiated from seven kinds; *system*, *services*, *components* become positions of space zoom |
| 5.2 The `level` record | vocabulary, below, pattern, abstraction, step map, invariants, actors | add the charter: `kind`, `readers`, `question`, `step_is`, `closes_when`, `hides`, `why`, `how`, `admission` |
| 5.2 `step_map` | which lower steps make up each upper step | computed by the closing rule; stored as step classes; lower steps are *part of*, or *support for*, an upper step |
| 5.3 Checks | pair, barrier, coverage, invariants | add the admission tests and the name-truth check; coverage counts support steps as covered |
| 6.3 Vocabulary | terms with anchors and levels | add subjects and roles as kinds of term |
| 6.5 LLM roles | vocabulary, port-behavior, lifecycle, mechanism, business, repair | replace *business* by the task types of section 6 |
| 9 Debugger and viewer | zoom, focus, lens | a view shows a cut; the default lens is degree of interest; four moves per step; home level per role |
| 13 Open questions | "when an extra level should be inserted", "whether LLMs reliably tell a mechanism from an ordinary local decision" | the first becomes test A3; the second narrows to naming a move that atomicity evidence already found |
| Glossary | *level*: "a complete way of describing the system at one height" | add: reader, subject, closing rule, step class, support step, charter, cut, home level; note that time zoom is the purpose axis |

## 10. What is evidence and what is inference

**Evidence, from others.** How people describe actions (why and how), where they place event boundaries, what they remember of a story, the basic level and its shift with expertise, the capacity of working memory, the expertise reversal effect, the gains from question-asking and from trace views. Most of this was checked only in abstracts and secondary summaries this session; the notes say which.

**Evidence, our own.** The step counts of the seven-level trial; findings F-Z1 to F-Z9; the measured instability of free LLM naming (0.54) in the rq abstraction-function experiment; and section 8 of this report: closing rules on 2,097 runs, placement of 554 anomalies, six naming samples, six free-form ladders, 90 stand-in reader calls, and step classes on three rq versions.

**Inference.** That the perceptual findings transfer from physical activity, stories and process control to software traces; the seven kinds as a general skeleton; the charter; the degree-of-interest cut; the processes around the software; everything about the viewer. Of the seven kinds, five were run and two were merged on rq.

**Known weak points.**

- **Transfer.** Event segmentation was studied on films of everyday activity, and the causal network on short stories. No study we found tests them on program traces.
- **The band.** Tested and rejected in its first form (8.2). The replacement, merging levels that restate each other, was fitted to the same data and needs another subject.
- **The reader was an LLM.** It reads every line with equal attention and has no working-memory limit of four. The line counts are the nearest thing to a human cost in this experiment, and they are a proxy.
- **Design A is a reading.** The specification's ladder was written for services and handlers, not for a library with one store. Another reading of *business* or *components* onto rq would move A's numbers.
- **Bounds.** Two actors, two preemptions, one or two jobs. Clean runs within these bounds define what counts as different.
- **The top of the ladder.** Outcome, stage and goal need subjects with lifecycles. A system that is mostly computation, such as a compiler, has few; its upper levels may have to come from its own domain ladder, through project-specific levels.
- **Closing rules need a good lower model.** They read waits, replies, atomic groups and observables. Errors in extraction move the boundaries.
- **The hand-built example.** The implemented rules reproduced it at goal and activation level and differed at the move level (5.8).
- **Moves inside one function.** The stable move identity of 8.7 (the call and the closing function) works on rq because rq's moves tend to close at function edges. In report-jobs, `generate_report` closes four to six moves inside one function body: the claim, the failure or release, and the completion all close at `s.commit()`, so the call and closing function name three different moves alike. One candidate is the port plus the anchor statement that closes the move plus its order in the function; it is untested, and the first target needs an answer before step classes can be stored.
- **People.** Whether the result is easier for a person was not measured. It is the experiment that matters most, and the hardest to run without a budget.

## 11. Sources

All outside sources, with links and evidence tags, are in [notes/sources.md](notes/sources.md). Mental's own: the [specification](../../specification.md) (sections 2, 5, 6, 9, 13), the [glossary](../../glossary.md), the [findings](../../findings.md) (section Z), and the reports on [abstraction levels](../abstraction-levels/report.md), the [abstraction function](../abstraction-function/report.md) and [trace visualization](../trace-visualization/report.md).
