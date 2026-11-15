# How other projects build the abstraction function

Evidence tags: **[S]** seen in a search result or primary page this session (link given); **[K]** background knowledge, not re-verified this session; **[E]** measured in our own experiment (see [experiments.md](experiments.md)).

The earlier [abstraction-levels report](../../abstraction-levels/report.md) surveyed how fields relate levels and check them. This note narrows to one question: **how is the abstraction function itself represented, where does it come from, and how is it trusted?**

## One line per product

| Product | Abstraction function in one line |
|---|---|
| Hoare 1972 / Liskov–Guttag | A hand-written function from representation to abstract value, defined only where a representation invariant holds. |
| TLA+ refinement mapping | A substitution: each upper variable is replaced by an expression over lower variables (`INSTANCE … WITH`). |
| Event-B gluing invariant | A relation between upper and lower variables, discharged as proof obligations per event. |
| Z / VDM retrieve function | A total function from concrete to abstract state, checked per operation by forward simulation. |
| Verus `View` / Creusot `View` | A trait method `view(&self) -> V` mapping each executable type to a mathematical value, written once per type and reused by every spec. |
| Dafny ghost state | Ghost fields (`Contents`, `Repr`) kept in step with real fields, plus a `Valid()` predicate as the representation invariant. |
| IronFleet | Two hand-written mappings: implementation host state → protocol state, protocol state → spec state, each with a per-step obligation. |
| seL4 / AutoCorres | A hand-proved state relation per refinement; AutoCorres computes heap and word abstractions of C automatically, with a proof for each run. |
| Abstract interpretation | A Galois connection α/γ fixed by the chosen abstract domain. |
| Predicate abstraction / CEGAR | The valuation of a set of predicates, grown from spurious counterexamples. |
| TVLA shape analysis | Canonical abstraction: heap cells merged when they agree on every unary predicate. |
| Bisimulation minimisation | The coarsest partition that preserves observable behaviour, computed (Paige–Tarjan, mCRL2). |
| Automata learning (L*, Tomte) | States separated by distinguishing experiments; Tomte refines its input mapper when the learned model turns nondeterministic. |
| ADABU spec mining | The return values of side-effect-free inspector methods, abstracted. |
| Daikon | No function; mined invariants over observed variables. |
| Elle / Jepsen | No state function; a dependency graph inferred from what reads returned. |
| QuickCheck state machines | A model state run in parallel with the real system, related only through postconditions on outputs. |
| Debuggers (DWARF) | Compiler-emitted location expressions from machine state to source variables, partial ("optimized out"). |
| Database views / CQRS projections | A query or a fold over an event log that computes the read model. |
| Architecture recovery, reflexion | A hand-written or clustered map from code entities to components. |
| Process mining event abstraction | Rules or learned models that lift low-level events to activities. |
| LLM pipelines (Specula, ProtocolGPT, SysMoBench) | An LLM writes the model or mapping from code; a tool or trace checks it. |

## Four ways to represent it

1. **As a function in code.** TLA+ mappings, Z retrieve functions, Verus/Creusot `View`, IronFleet's refinement functions, ADABU's inspectors, database views. It is executable, so it can be evaluated on any lower state, including states no one anticipated. [K; Verus [S](https://verus-lang.github.io/verus/verusdoc/vstd/view/trait.View.html), IronFleet [S](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf), ADABU [S](https://www.st.cs.uni-saarland.de/models/papers/woda-06.php)]
2. **As a relation.** Event-B gluing invariants, seL4 state relations, CIVL layer annotations. More expressive (one lower state can relate to several upper ones), but it cannot be "run" to produce an upper state; it can only be checked. [K]
3. **As a partition computed from behaviour.** Bisimulation quotients, L* congruences, CEGAR predicate sets, TVLA canonical abstraction. It is correct by construction for the chosen observations, and it has no names. [K; TVLA [S](https://research.cs.wisc.edu/wpis/abstracts/tr1383r2.abs.html), Tomte [S](https://www.mbsd.cs.ru.nl/publications/papers/fvaan/CEGAR12/FM.pdf)]
4. **As no state function at all.** Elle, QuickCheck, Jepsen and P's spec monitors relate levels through *observations* (outputs, read results, events), never through a map on state. [K; Elle [S](https://arxiv.org/abs/2003.10554)]

## Where it comes from

| Source | Examples | Cost and risk |
|---|---|---|
| Written by a person, per type | Liskov, Verus `View`, Dafny ghost state, IronFleet | The best names; written once per type and reused. Arm's ISA-Formal abstraction function took about 2,500 lines per processor (earlier report). |
| Written by the generator of the lower level | DWARF location lists, Formality guidance files, CompCert | The relation comes for free with the output, but it is partial and has bugs: Li et al. (PLDI 2020) found 47 confirmed debug-information bugs in GCC and LLVM [S](https://arxiv.org/pdf/2211.09568). |
| Computed by a tool with a proof | AutoCorres heap and word abstraction | Automatic and proof-producing, but only for fixed abstraction patterns (typed heaps, ideal integers) [S](https://trustworthy.systems/projects/OLD/autocorres/). |
| Computed from behaviour | bisimulation, L*, Tomte, CEGAR | No names; preserves exactly what is declared observable. Tomte refines when the abstraction makes the system look nondeterministic [S](https://typeset.io/pdf/automata-learning-through-counterexample-guided-abstraction-2a10rgqrzh.pdf). |
| Mined from the code's own query API | ADABU (inspectors), our observer arm | The authors' own vocabulary. ADABU's companion study found inspectors in 45% of classes [S](https://www.st.cs.uni-saarland.de/models/papers/woda-06.php). Purity must be checked. |
| Mined from shape | TVLA canonical abstraction, our membership arm | Mechanical; works when entities sit in containers. |
| Proposed by an LLM | Specula, Event-B Agent, ProtocolGPT, SysMoBench agents | Fluent names and broad coverage, but checking is required: SysMoBench finds agents good on spinlocks and weak on Etcd Raft [S](https://arxiv.org/abs/2509.23130v3); ProtocolGPT reports over 90% state-machine accuracy on six protocols (third-party summary, unverified) [S](https://arxiv.org/abs/2405.00393). |

## How it is trusted

- **Proof per step** (seL4, IronFleet, Verus, Event-B): `next_lower(c, c') ⇒ next_upper(AF(c), AF(c'))`, or a stutter [S](https://6826.csail.mit.edu/2020/lec/l17.pdf).
- **Exploration** (ProB, FDR, TLC): run the lower level and check the projected trace. Mental's pair check belongs here.
- **Validation per output** (CompCert register allocation, Alive2, debug-info validators).
- **Conformance testing** (ShardStore, QuickCheck): compare outputs of the model and the real system on random sequences.

## Lessons that bear on Mental

1. **Every serious AF ships with a representation invariant.** Hoare and Liskov define the AF only on representations satisfying an invariant; Dafny's `Valid()` and seL4's invariants (80% of the first refinement proof) are the same idea. [K] Our experiment shows why in a concurrent setting: races are exactly the states where redundant representations disagree, so an AF without its invariant cannot see them [E].
2. **The observables decide the abstraction.** Bisimulation, L*, and CEGAR all compute the coarsest partition *relative to declared observations*. With only operation outputs observable, rq collapses to a nearly trivial abstraction [E]. Someone must say what is observable, and in our data the user documentation was a sharper signal for that than the LLM's own proposals [E].
3. **Author-written query APIs are abstraction functions already.** Verus's `View`, Creusot's `View` and ADABU's inspectors say the same thing from verification and mining. rq's `get_status`, `is_finished` and the registries' `get_job_ids` are its authors' abstraction function. But "query" methods are not always pure. rq's finished and failed registries garbage-collect inside `get_job_ids`, which a naive purity check rejects [E].
4. **Generator-emitted relations are partial and buggy.** DWARF's "optimized out" is the canonical partial AF, and the debug-information studies count real bugs in it [S]. An LLM-emitted AF should be treated the same way: partial by default, and checked.
5. **Computed abstractions have no names, and named ones have no guarantee.** Every project picks one side. Our proposal combines them: named candidates, chosen and checked by a computed criterion [E].
6. **Event-level abstraction is a separate axis.** Process mining's event abstraction [S](https://link.springer.com/article/10.1007/s41066-020-00226-2), and Elle's inference of structure from observations, lift *steps* rather than states. Mental's step map is this axis. The take/complete split we needed in the oracle [E] is an event-abstraction decision.
