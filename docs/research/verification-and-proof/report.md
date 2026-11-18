# Proof needs a sound model, not a stronger checker

**Date:** 2026-11-17. **Question:** can Mental prove that code has no bugs, and how close can it come to tools such as TLA+? **Scope:** what existing tools prove, how they tie that result to real code, and what Mental can borrow. How levels are related by refinement (Event-B, TLA+ refinement mappings, seL4, IronFleet) is covered in the [abstraction levels](../abstraction-levels/report.md) survey and not repeated here.

Evidence labels used throughout: **read** means the page was read in full; **partly read** means about the first third was read word for word and the rest only through a summary of the full text; **snippet-only** means the claim comes from search-result summaries and is a lead, not a confirmed fact; **unverified** means prior knowledge that was not checked this session.

## Per-product summary

Model checkers and provers for hand-written specs:

| Product | What it is | What it can claim | Take for Mental |
|---|---|---|---|
| **TLA+ / TLC** | explicit-state model checker for TLA+ specs | every state reachable with the chosen constants satisfies the property; nothing about the code | Mental's bounded-exhaustive search already plays this part |
| **Apalache** | symbolic (SMT) checker for TLA+ | bounded runs, and proofs that an invariant is inductive for **fixed** parameter values | an inductive check is cheaper than search and covers runs of any length |
| **TLAPS** | interactive prover for TLA+ | unbounded proofs, with invariants written by a person | the route to "for any number of workers"; costly by hand |
| **Ivy** | verifier in a decidable logic (EPR) | unbounded safety once the user supplies an inductive invariant; shows a counterexample to induction otherwise | a counterexample to induction is a good task for the LLM to repair |
| **P** (AWS) | state-machine language with a systematic tester | bugs within explored schedules | Mental's step semantics already follows P |

Bridges from a spec to code:

| Product | How it ties the spec to code | What it can claim | Take for Mental |
|---|---|---|---|
| **TLA+ trace validation** (CCF, NSDI 2025) | logs from tests are checked against the spec by TLC | some spec behavior explains each recorded trace; not a proof | evidence that the model is faithful, per trace |
| **PObserve** (AWS) | production logs checked against P specs | the same, on real traffic | the "conformance" strategy of §8.4 |
| **Quint connect** | traces generated from the spec drive the code through a driver; states compared after each step | the code follows the spec on those traces | generate tests from witnesses |
| **Specula** (2026) | an LLM agent writes TLA+ from code, validates it against code traces, repairs it | no false positives, because each violation is reproduced in the code; no soundness claim | the closest precedent to Mental; copy reproduction |
| **Stateright** | the checked actor code is the code that runs | properties of the checked code under its network model | not applicable: Mental does not own the code |

Verifiers that start from code:

| Product | How it gets a model | What it can claim | Take for Mental |
|---|---|---|---|
| **SLAM / Static Driver Verifier** | predicate abstraction turns C into a boolean program that **over-approximates** it; spurious paths refine the predicates | "the rule holds on all executions", within its assumptions | Mental's predicates are the same idea; a proof needs them unconstrained |
| **Bandera, Java PathFinder** | extract finite models from Java by slicing and abstraction | "safe" models by construction (snippet-only) | old precedent for compiling code into a checkable model |
| **Kani** | bounded model checking of Rust through CBMC | sound **only if every unwinding assertion passes**; sequential code only; library models are assumptions | list every assumption a proof rests on |
| **Astrée** | sound abstract interpretation of C | proves absence of run-time errors when it ends with zero alarms; not sound for concurrent code | "zero alarms" is a proof only for the class of programs it was tuned for |

LLM-written models and properties:

| Product | What it measures | Result | Take for Mental |
|---|---|---|---|
| **SysMoBench** (ICLR 2026) | LLM-written TLA+ of 11 real systems | syntax near 100%; conformance to the code about 46%; invariants about 41% | an LLM-written model errs in both directions |
| **IC3Syn** (2026) | LLM proposes blocking clauses inside an IC3 loop; TLC and TLAPS check | inductive invariants for 29 of 29 protocols, proven unbounded in TLAPS (self-reported, snippet-only) | the LLM proposes pieces of an invariant, a prover judges |
| **nl2postcond, NL2Contract** | LLM-written postconditions and contracts | catch real bugs; faithfulness to intent is hard to check | properties need a person's approval |

Practice:

| Source | Finding |
|---|---|
| **AWS 2015** (Newcombe et al.) | TLA+ found subtle design bugs in S3, DynamoDB and EBS; the model was "exhaustively testable pseudo-code"; they avoided the words "formal", "verification" and "proof" |
| **AWS 2025** (Brooker, Desai) | P, PObserve, property-based testing, fault injection and deterministic simulation side by side (snippet-only) |

## Summary

No tool proves that code has no bugs. Each one proves **a stated property** of **a model** under **stated conditions**, and the useful tools say exactly which. TLC's verdict holds for the constants it was given. Apalache proves an invariant inductive for fixed parameters. TLAPS and Ivy prove properties without bounds, but a person writes the invariants. Kani's "successful" counts only if every unwinding assertion passed, and its library models are assumptions. Astrée is sound for sequential C and says so; it is not sound for concurrent code.

The gap that matters is between the model and the code, not between bounded and unbounded checking. For hand-written TLA+ the gap is closed, partly, by trace validation: CCF's authors say high-level specs alone "would not provide any guarantees about the production code itself". Tools that start from code close it by construction: SLAM's predicate abstraction gives a model that allows every behavior of the C program, and more. An LLM-written model closes it in neither way. SysMoBench shows LLM-written TLA+ failing in both directions, with states the code never reaches and states the code reaches that the spec cannot.

This decides what Mental can claim. **A "no violation" result carries over to the code only if every record it rests on allows at least the code's behavior.** Mental's mechanically extracted records do; its LLM-written summaries may not. **A violation carries over to the code only if its steps are possible in the code**, which is what Specula gets by reproducing every violation in the code. So the proposal is: label each record with the direction in which it may differ from the code; report every verdict with its extent (bounded or unbounded) and its basis (sound or not, with the records that make it unsound); keep proofs on the mechanical records, where Mental's predicates work like SLAM's; and get unbounded results from existing provers through export before writing one. The proposal is written into the [specification, section 8.11](../../specification.md#125-what-a-result-may-claim), as an Experiment.

## 1. What each tool means by "verified"

The claims differ along two axes: **how far** the result reaches (these constants, these bounds, any size), and **what** it is about (a hand-written spec, a model derived from code, the code itself).

| Tool | Reach | About | Conditions it states |
|---|---|---|---|
| TLC | the chosen constants; the finite state space they give | the spec | none about the code |
| Apalache | runs of bounded length; or any length, if the invariant is inductive, for fixed constants | the spec | constants must be fixed (snippet-only) |
| TLAPS | any size | the spec | the proof is checked; invariants are written by people |
| Ivy | any size | the spec, in a decidable logic | the user supplies the inductive invariant |
| Kani | inputs within assumptions, loops within the unwinding bound | the Rust code | "Verification results are sound only if all unwinding assertions pass"; "Kani verifies sequential code only"; library models are assumptions (read) |
| Astrée | all executions | the C code | zero alarms needed; not sound for concurrency (snippet-only) |
| SLAM | all executions | the C code against one API rule | the abstraction over-approximates; CEGAR may not end (snippet-only) |

Two things stand out.

**Unbounded results need an inductive invariant.** Every tool that reaches "any size" does so the same way: find a property that holds initially and that every step preserves. Search cannot do this, however large the bounds. The invariant is written by a person (TLAPS, Ivy), inferred by a tool (IC3PO, DistAI, Endive, snippet-only), or proposed by an LLM and checked (IC3Syn, snippet-only). IC3Syn's split is instructive: the LLM is asked for one blocking clause for one bad state, not for a whole invariant, and the solver decides (snippet-only).

**Fixed constants can sometimes be lifted by a cutoff.** For some classes of parameterized systems, a theorem says that checking a small instance settles every size: Emerson and Kahlon's guarded protocols have cutoffs that depend on the process template, and arbitrarily many caches in a coherence protocol reduce to at most 7 (snippet-only). A cutoff holds only for the class of systems its theorem covers. None was found for systems shaped like report-jobs (workers, a queue, rows), so Mental cannot assume one.

## 2. The gap between the model and the code

### 2.1 Which direction of error breaks which result

A model can differ from the code in two directions:

| The model | Example | A "no violation" result | A violation |
|---|---|---|---|
| allows **more** than the code (over-approximates) | a predicate left free where the code correlates it with another | still true of the code | may be spurious |
| allows **less** than the code (under-approximates) | two code steps fused into one, so nothing can run between them | may be false of the code | real, if its steps exist in the code |

Proofs need the first direction; bug reports need the second. A model that may err in both directions gives neither. That is the plain reason Mental's specification says it "does not prove correctness" (§1).

### 2.2 Three ways tools close the gap

**By construction.** SLAM turns C into a boolean program in which each predicate is a boolean variable and data variables are gone. The abstraction is built to over-approximate. When the checker finds an error path, SLAM tests whether the path is feasible in the C program; if not, it adds predicates and tries again (CEGAR). SLAM2 is reported to cut false alarms on WDM drivers from 25.7% to under 4% (snippet-only). Bandera did the same for Java with slicing and abstract interpretation (snippet-only). Astrée does it with abstract interpretation and accepts false alarms as the price of soundness (snippet-only).

**By sharing the code.** Stateright checks actor code that also runs on a real network, so there is no second artifact to drift (snippet-only). This needs the code to be written in the checker's framework, which Mental cannot ask for.

**By checking traces.** CCF's team wrote a TLA+ "Trace" spec that reuses the spec's actions and enables each one only when the next logged event matches; TLC then asks whether some behavior of the spec explains the trace (partly read). It found six bugs in consensus "before they could impact production" (partly read). Its costs, from the summary of the full text: about a day for the driver and logging, about two engineer-months over four months for the consensus trace spec, and one model-checking run of 48 hours on 128 cores. A rejected trace may mean a bug in the code, the spec or the logging, and the tool cannot say which. PObserve does the same with P specs and production logs (snippet-only). Quint connect runs it the other way: traces generated from the spec drive the code through a driver, and states are compared after each step (snippet-only).

Trace checking is evidence, not proof. It covers only what the tests exercise; SysMoBench reports a lock path that received no trace windows at all because nothing traced it (read).

### 2.3 Specula: the closest precedent

Specula is an LLM agent that writes TLA+ from a repository, then validates the model against traces from the instrumented code and repairs the model or the instrumentation until they agree (partly read). It writes invariants at two levels, protocol and code, each citing its evidence in code, tests, issues or commits (partly read). It reports 249 bugs in 48 projects; of 89 reported so far, 68 are confirmed and 24 fixed (partly read). Its claim about correctness is narrow and well chosen: "Specula reports no false positive as all the bugs are reproduced at the code level" (partly read). It does not claim soundness for the pipeline, and says a model "can therefore remain inconsistent with the implementation in ways we did not catch" (partly read, from the summary).

What Specula does not have and Mental does: anchored records with provenance, mechanical extraction of every branch and effect, and atomicity computed from evidence in the code. What Specula has and Mental lacks: reproduction of every violation in the real code.

### 2.4 How LLM-written models go wrong

SysMoBench asks LLMs to write TLA+ for 11 real systems and checks four phases (read):

| Phase | Checks | Result |
|---|---|---|
| syntax | SANY parses it | near 100% for most LLMs |
| runtime | TLC runs it | 30% to 92% |
| conformance | windows of (state, action, state) from code traces fit the spec | about 46% for the leading LLMs |
| invariant | safety and liveness properties hold | about 41%; the weakest LLM 16%, Gemini 3.1 81% |

The two failure modes it describes map exactly onto the two directions of 2.1 (read):

- **Extra states.** A ZooKeeper election spec kept received votes as a set union; the code overwrites a peer's old vote. The spec followed "a formalization template" rather than the code.
- **Missing states.** The LLM fused two code steps into one guard; the real code first raises the logical clock, then processes the message. "The LLM knows what Raft and ZAB look like as protocols, but it does not know how Etcd or ZooKeeper splits a particular action."

The second failure is the wrong grain of atomicity that F-R5 already names as the best-documented failure of LLM-written models. Mental's answer, computing atomicity from evidence in the code (§8.7), is aimed at exactly this. The first failure is what Mental's anchors and coverage checks are for, but they check that each branch appears, not that each summarized effect is right.

## 3. Properties: who writes them

"No bugs" is not a property. Every tool above checks properties someone stated. Three sources exist.

**Properties that need nobody.** Some properties follow from the model's form: no deadlock, no message nothing handles, every write is a legal move of its lifecycle. TLC checks deadlock by default (unverified); Mental's interpreter already stops on a model gap, a deadlock, an illegal write and a zero-time cycle (§8.5). In report-jobs, the duplicate claim is an illegal write and the stranded row is a stranded state, so two of the four bugs are found without anyone writing a property.

**Properties written by people.** TLA+ and Ivy users write their own. AWS found this was where the value lay, and avoided the word "proof" when selling it (read, through a summary of the paper).

**Properties proposed by an LLM.** nl2postcond turns comments into postconditions; generated postconditions caught 64 historical bugs in Defects4J, and the authors note that two readings of one comment can both be literal while only one matches intent (snippet-only). NL2Contract reports that verifiers given LLM-written postconditions alone often raise false alarms, and fewer with full contracts (snippet-only). Specula makes every invariant cite its evidence (partly read). The lesson for Mental: an LLM may propose a property, with anchors to where the intent is stated; a person accepts it before a proof of it is called a proof. This matches F-R7: LLMs asked to write an upper level make it permissive, and a permissive property is easy to prove.

**Liveness needs fairness.** A property such as "every requested report ends `DONE` or `FAILED`" fails trivially if the scheduler may starve a worker forever. TLA+ states fairness on actions: weak fairness (an action continuously enabled eventually happens) is the usual default; strong fairness (an action enabled infinitely often eventually happens) is needed for a receiver that is disabled each time a message is lost (snippet-only). Mental's scheduler has no fairness yet, so it can check safety but not liveness.

## 4. What this means for Mental

Mental already has the parts of a model checker: a transition system (P1), a step function generated on demand (P2), bounded-exhaustive search with witnesses (§8.4), invariants per level and refinement checks between levels (§5). What it lacks is the bookkeeping that says **of what** a result is true. Six proposals follow; the specification takes them up in [section 8.11](../../specification.md#125-what-a-result-may-claim).

1. **A direction on every record.** Besides provenance, each record says how it may differ from the code: `exact`, `over` (allows more), `under` (may allow less) or `unknown`. Mechanically extracted full-detail behaviors are `over` or `exact`; a predicate left unbound is `over`, as in SLAM. An LLM-written summary case is `unknown` until its summary-agreement check passes against a mechanical full graph. A pack rule is an assumption, listed by name, as Kani lists its library models.
2. **Two axes on every "holds" verdict.** *Extent:* `bounded` (with the bounds) or `unbounded` (with the prover and the invariant). *Basis:* `sound` (every record the property depends on is `exact` or `over`, and the pack rules used are listed as assumptions) or `unsound` (with the records that make it so). Only `holds, unbounded, sound` may be called a proof, and always "under these assumptions".
3. **A reproduction label on every violation.** A witness is `model-only` until it is replayed in the code (a generated test, or conformance), then `reproduced`. This is Specula's rule, and it is how a violation from an over-approximating model is told apart from a spurious one.
4. **A proof mode on mechanical records.** For a proof, the interpreter runs the full-detail behaviors from the extractor, with predicates the extractor cannot relate left as free choices. Spurious witnesses are expected; each one is a repair task: add the relation between predicates that the code establishes, anchored, as SLAM adds predicates. F-S12 suggests this is feasible on report-jobs, since three of its four bugs follow from mechanical facts alone.
5. **A `property` record.** Safety or liveness, written in one level's vocabulary from a small set of patterns (an invariant, a step that must never happen, at most one per key, leads-to), with fairness for liveness and an approval: proposed by the LLM with anchors to stated intent, accepted by a person.
6. **Unbounded results by export before invention.** Generate TLA+ (for TLC, Apalache and TLAPS) or Ivy from the model, and adopt a backend only after it agrees with `mentalvm` on the same bounds. The LLM proposes invariant pieces from counterexamples to induction, IC3Syn style; the prover decides.

Experiments that would settle these are listed in the specification. Each one waits for the component it needs: none can be measured before the reference model (milestone 1) and `mentalvm` (milestone 4) exist.

## 5. What was not verified

- The CACM article on AWS practice (2025) could not be read (HTTP 403); everything said about it is snippet-only.
- The Specula and CCF papers were read only in part; their effort figures, the Specula cost of $19 to $168 per system and the false-positive counts of its baselines come from a summary of the full text.
- SLAM's false-alarm figures, Astrée's scope, Apalache's requirement of fixed constants, the cutoff results and IC3Syn's 29 of 29 are snippet-only.
- No source was found that measures how often a model derived from code over-approximates it, for any tool. Mental would be the first to measure it.

## Sources

- Konnov, Kuppe, Merz, "Specification and Verification with the TLA+ Trifecta: TLC, Apalache, and TLAPS", [arXiv 2211.07216](https://arxiv.org/pdf/2211.07216) (snippet-only); TLA+ mailing-list threads on Apalache and fixed constants, [msg04561](https://discuss.tlapl.us/msg04561.html) (snippet-only).
- Howard, Kuppe, Ashton, Chamayou, Crooks, "Smart Casual Verification of the Confidential Consortium Framework", NSDI 2025, [arXiv 2406.17455](https://arxiv.org/html/2406.17455v2) (partly read).
- Cheng, Xu, Huang et al., "Specula: Scaling formal specifications for autonomous model checking of system code", [arXiv 2607.25333](https://arxiv.org/abs/2607.25333) (partly read); [github.com/specula-org/specula](https://github.com/specula-org/specula) (snippet-only).
- "Can LLMs model real-world systems in TLA+?", [SIGOPS blog, May 2026](https://sigops.org/2026/can-llms-model-real-world-systems-in-tla) (read); SysMoBench, [arXiv 2509.23130](https://arxiv.org/pdf/2509.23130v3) (snippet-only).
- Kani, [soundness page](https://model-checking.github.io/kani/soundness.html) (read); "Kani: A Model Checker for Rust", [arXiv 2607.01504](https://arxiv.org/abs/2607.01504) (snippet-only).
- Ball, Levin, Rajamani, "A Decade of Software Model Checking with SLAM", [CACM 2011](https://cacm.acm.org/magazines/2011/7/109893-a-decade-of-software-model-checking-with-slam/fulltext) (snippet-only); SLAM2, [FMCAD 2010](https://www.dcc.fc.up.pt/~nam/resources/vfs12/exame/slam2_fmcad2010_final.pdf) (snippet-only).
- Corbett et al., "Bandera: extracting finite-state models from Java source code", ICSE 2000, [doi 10.1145/337180.337234](https://doi.org/10.1145%2F337180.337234) (snippet-only); Visser et al., "Model Checking Programs" ([PDF](https://sos-vo.org/system/files/sos_files/Model_Checking_Programs.pdf), snippet-only).
- [The Astrée Static Analyzer](https://www.astree.ens.fr/) and the AstréeA work on concurrency ([HAL](https://hal.sorbonne-universite.fr/hal-01312246v1/document)) (snippet-only).
- McMillan, Padon et al., Ivy ([PDF](https://ilyasergey.net/CS6213/_static/papers/ivy.pdf)) and follow-on invariant inference ([CAV 2019](https://www.cs.tau.ac.il/research/yotam.feldman/papers/cav19/cav19.pdf)) (snippet-only).
- Cao et al., "Synthesizing Inductive Invariants for Distributed Protocols via IC3 and Large Language Models", [arXiv 2605.24619](https://arxiv.org/html/2605.24619v1) (snippet-only).
- Emerson, Kahlon, "Reducing Model Checking of the Many to the Few" and later cutoff work, [arXiv 1707.01369](https://ar5iv.arxiv.org/html/1707.01369) (snippet-only).
- Livshits et al., "In Defense of Soundiness: A Manifesto", [CACM 2015](https://cacm.acm.org/magazines/2015/2/182650-in-defense-of-soundiness/fulltext) (snippet-only).
- Endres et al., nl2postcond, [arXiv 2310.01831](https://arxiv.org/html/2310.01831v2); Clover ([summary](https://sos-vo.org/node/107139)); Richter, Wehrheim, NL2Contract ([summary](https://aitopics.org/doc/arxivorg:BC01FC6F)) (all snippet-only).
- Newcombe et al., "How Amazon Web Services Uses Formal Methods", CACM 2015, through [Adrian Colyer's summary](https://blog.acolyer.org/2014/11/24/use-of-formal-methods-at-amazon-web-services/) (read).
- Brooker, Desai, "Systems Correctness Practices at Amazon Web Services", [CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services) (snippet-only; the page refused access); [P language](https://p-org.github.io/P/) (snippet-only).
- [Quint connect](https://quint-lang.org/posts/quint_connect_emerald) (snippet-only); [Stateright](https://docs.rs/crate/stateright/latest) (snippet-only).
- Fairness in TLA+: [Lamport's tutorial](https://lamport.azurewebsites.net/tla/tutorial/session11-2.html), [Hillel Wayne on fairness](https://hillelwayne.com/post/fairness/) (snippet-only).
