# Every Level Is a Refinement With a Map, and the Map Is What Gets Checked

Every field that runs or verifies a system at several levels of abstraction does the same three things. It gives each level its own state vocabulary and its own steps. It writes down how lower-level state maps onto upper-level state. It records which lower steps realize each upper step and which ones are invisible above. Fields differ in who writes these three things and in how agreement is established. Verified kernels and compilers (seL4, CertiKOS, IronFleet, CompCert) prove agreement once per pair of adjacent levels. Model checkers (ProB, FDR, TLC) check it by exploring states, with no prover. Hardware flows and translation validators (Formality, Alive2) check each generated output against its input. Architecture tools mostly check structure, and their link-only checks decay in practice; the exceptions are contract checkers such as AGREE and OCRA, and reflexion models.

The sweep answers the three questions directly. Verifiers run per *pair* of adjacent levels, never end to end, and invariants are checked per level on top of that. Predicates are authored in the refinement methods and computed in the analysis tools: CEGAR, abstract interpretation, Daikon and Houdini compute them, and CPAchecker stores them and reuses them across revisions. Invariants are stated at the level whose vocabulary can express them and inherited downward, but only when they have the right shape: seL4's integrity proof carried down to C, while its confidentiality proof did not. Termination of the steps a level hides is either proved with an authored measure (Event-B variants, ASM) or detected automatically as a cycle of hidden steps on a finite graph (FDR, mCRL2, and ProB since 2025).

For Mental, the conclusion is that a level needs more than a vocabulary and a step map. It also needs:

- an explicit abstraction function on state, emitted by the LLM as typed hints that are checked one by one;
- a step map stored as a witness, allowing m lower steps to stand for n upper steps;
- invariants stated in the level's own vocabulary;
- a convergence status on every hidden loop.

The interpreter can then check each pair of levels by exploration, the way ProB and FDR do. A failure is either a model bug or the finding Mental exists for: an upper-level step that the lower level does not implement atomically. The novelty claim is narrower than the first draft said. Event-B Agent, Specula, NeuroAbs and LLM code-to-architecture mappers already have an LLM propose an abstraction that a tool checks. Nobody yet has an LLM propose a named ladder of several levels, each with its own vocabulary, state mapping and behavior, derived from source code and checked by an interpreter.

## How this report was made

Five sourced notes back this report, one per cluster of precedents. Each opens with a one-line-per-system table and tags every claim by evidence: read in the primary source this session, seen only in a search summary, or from training knowledge.

| Notes | Covers |
| --- | --- |
| [formal_refinement_methods.md](notes/formal_refinement_methods.md) | Event-B and Rodin, ProB, classical B, TLA+, ASM and KIV, Z and VDM, CSP and FDR, mCRL2, LLM refinement pipelines |
| [layered_verified_systems.md](notes/layered_verified_systems.md) | seL4, CertiKOS, IronFleet, IronSync and Verus, Verdi, CIVL, Armada, Perennial and Grove, Anvil, AWS practice, LLM provers |
| [compilers_and_computed_abstractions.md](notes/compilers_and_computed_abstractions.md) | CompCert, CakeML, translation validation, Alive2, MLIR, abstract interpretation, CEGAR, IC3, Daikon, Houdini, LLM invariants and abstractions |
| [hardware_simulation_planning.md](notes/hardware_simulation_planning.md) | Equivalence checking, C-to-RTL, ISA-Formal, Sail, riscv-formal, multi-resolution simulation, HTN planning, angelic semantics, Netsem TCP |
| [architecture_and_conformance.md](notes/architecture_and_conformance.md) | C4 and Structurizr, Capella, SysML v2, AADL, AGREE, OCRA, reflexion models, dependency-rule tools, LLM conformance |

Sources the agents could not open are listed in each note. They include Pnueli et al. 1998, SLAM, the lazy-abstraction papers, Abadi–Lamport, the SysML v2 specification PDF, the RAND multi-resolution reports and Cadence's documentation.

## Each solution in brief

### Formal refinement methods

| Solution | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| Event-B / Rodin | Proof-based method for stepwise system models | Authored chain of machines; gluing invariant; fixed list of proof obligations per step; variants for new events | Ready checklist of per-step checks; safety only, so a lower level may do less than promised |
| ProB | Animator and model checker for B and Event-B | Checks trace refinement between two machines by exploring states, without the gluing invariant; failures-divergence since 2025 | Closest precedent for refinement checking by an interpreter |
| Classical B / Atelier B | Proof-based method down to generated code | Abstract machine → refinements → B0 implementation → Ada or C | Industrial chains stay short: 46 intermediate refinements across 233 machines |
| TLA+ / TLC | Specification language and model checker | Authored refinement mapping; upper spec checked as a property of the lower | Abstraction-function check; needs stuttering variables when the lower level takes fewer steps |
| TLA+ trace validation | Checks logs against a spec | Asks whether *some* spec behavior matches a trace; not refinement | Weak evidence alone; LLMs overfit to it |
| ASM / KIV | Ground model plus refinements | m abstract steps matched to n concrete steps, compared only at "states of interest" | Allow m:n step maps; compare at chosen points, not after every step |
| Z / VDM | Model-based specification | Retrieve function; forward and backward simulation per operation | Forward-only checking is incomplete when the upper level decides early |
| CSP / FDR | Process algebra and automatic refinement checker | Lower process with internal events hidden must refine the upper in traces, failures or divergences | Three graded checks; hidden livelock found automatically |
| mCRL2 | Process algebra toolset | Hide actions, then minimise modulo branching bisimulation | Compute "hide" levels instead of authoring them |
| LLM refinement pipelines | Event-B Agent, Specula (2026) | LLM proposes levels, gluing invariants or abstraction choices; ProB, provers and TLC judge | Mental is not alone; copy the loop and its anti-overfitting guard |

### Layered verified systems

| Solution | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| seL4 | Verified OS microkernel | Abstract spec → executable spec → C → binary; one simulation proof per pair | 80% of the first refinement was invariants; only Hoare-style properties carry down |
| CertiKOS | Verified hypervisor kernel | 37 thin layers, each a small proved refinement | Thin layers ease each proof but duplicate invariant proofs on every change |
| IronFleet | Verified Paxos and key-value store | Spec → protocol → implementation; host-step atomicity argued by reduction on paper | Check a cheap syntactic rule (receives before sends) instead of proving atomicity |
| IronSync / Verus / Atmosphere | SMT-based verification of concurrent systems | Two levels, decomposed by ownership instead of by more layers | Recent practice uses fewer levels, not more |
| Verdi | Verified distributed systems | Levels are fault models; a transformer lifts a system and its proofs to a harsher network | "Fault model" is a level pattern of its own |
| CIVL | Verifier for layered concurrent programs | Layer numbers on one program; movers make atomic actions | Exact per-run rule: one fragment does the upper step, all others stutter |
| Armada | Refinement prover for concurrent code | Each pair linked by a recipe naming one of eight strategies; proof generated | Closed list of level patterns; hard patterns need invariants as parameters |
| Perennial / Grove | Crash-safe and distributed Go systems | A crash plus recovery must look like one atomic upper step | Crash-and-recover is an upper step to check |
| Anvil | Verified Kubernetes controllers | One code step = one model step; liveness proved on the model | Check liveness at the model level, under stated fairness |
| AWS (ShardStore, Cedar, authorization engine) | Production systems | Reference model about 1% of code size; conformance by property-based or differential testing | A tiny model checked per run works in production |
| Kondo, LLM provers | Tools writing invariants and proofs | Kondo generates the asynchronous level and routine invariants; LLMs fill proofs under a human spec | LLMs succeed only with a checker; state-machine obligations stay hard |

### Compilers and computed abstractions

| Solution | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| CompCert | Verified C compiler | 8 intermediate languages, 14–20 passes, one simulation proof each; register allocation validated per run | Accept output only if its validator says yes |
| CakeML | Verified ML compiler | A new intermediate language whenever a feature is compiled away | Add a level only when vocabulary appears or disappears |
| Translation validation (Necula) | Check each run of an unverified translator | Infers the relation between code before and after a pass | Inferring the relation causes false alarms; have the generator emit it |
| Alive2 | Bounded validator for LLVM | SMT check of function pairs; loops unrolled | Five outcomes, bounds always printed; found 47 LLVM bugs |
| MLIR | Multi-level compiler infrastructure | Dialects mixed in one program; lowering toward a declared legal set | Declare each level's legal vocabulary; structure checks are not semantic checks |
| Abstract interpretation (Astrée, IKOS) | Sound static analysis | Lattice of domains; invariants computed as fixed points | Start cheap, refine where false findings appear |
| CEGAR / CPAchecker | Software model checkers | Predicates grown from spurious counterexamples; stored and reused | Refine predicates from failed replays; store them per behavior (over 4× faster) |
| IC3 / PDR | SAT-based model checking | Learns small inductive lemmas one counterexample at a time | Invariants can be built incrementally |
| Daikon / Houdini | Invariant mining and filtering | Propose many candidates, drop every refuted one | Cheapest way to use LLM proposals |
| LLM + checker (Lemur, Loopy, LORIS) | LLM proposes invariants, verifier decides | Filter, repair, backtrack | Generation works (78–93%); repair does not (16%) |
| LLM abstractions (NeuroAbs, PolyVer) | LLM proposes abstraction or contract, solver checks | Refined by counterexample | Prior art for "LLM proposes, checker validates" |

### Hardware, simulation and planning

| Solution | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| Gate vs RTL (Formality, Conformal) | Combinational equivalence checking | Matched registers; synthesis tool emits hints that the checker proves or rejects | Have the LLM emit its level map as typed hints |
| C-to-RTL (DPV, SLEC, Jasper) | Transaction equivalence | Outputs compared only at transaction boundaries | Works per block with human case splits; the checkers had soundness bugs |
| Arm ISA-Formal | Processor vs architecture spec | Hand-written abstraction function, one check per instruction class | The abstraction function is the costly, bug-prone part |
| Sail / ASL | Executable ISA specifications | One authored top level, everything else generated | Generate all checks from one executable artifact |
| riscv-formal | Open RISC-V checking framework | Standard per-instruction trace record; generic checks on it | One observation record per level; checks written once |
| Multi-resolution modeling | Simulation families at several resolutions | Commuting diagram: aggregate-then-run vs run-then-aggregate | Same as Mental's projection check |
| Downward refinement | Planning theory | Refinement probability measured per level pair; weak levels merged | Count how often upper steps refine; merge weak levels |
| Angelic semantics | Semantics of high-level actions | Upper and lower bounds on reachable states | Use the upper bound for actors the system does not control |
| HTN plan verification | Checking plans against task hierarchies | NP-complete without the decomposition | Store the step map; never reconstruct it |
| ChatHTN, Ada | LLM-proposed decompositions with a checker | Proposals kept only if their effects hold below | Score each proposed upper step by success rate |
| Netsem TCP | TCP specified at protocol and service level | Testable abstraction function checked on captured traces | Precedent for projection checks on real protocols |

### Architecture models and conformance

| Solution | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| C4 / Structurizr | Architecture model and tool | Four fixed views of one model; upper edges implied by lower ones | Derive upper wires on read; materializing all of them explodes |
| Arcadia / Capella | Systems-engineering method | Five fixed levels joined by realization links; link rules | Link-only rules decayed; maintainers removed several |
| SysML v2 | OMG modeling standard (Sept 2025) | Decomposition, satisfy and verify; "refinement" is only a tag | Copy the requirement shape (assumptions imply constraints) |
| AADL / OSATE | Avionics architecture language | Type vs implementation; software bound to hardware; latency along flows | Precedent for logical and physical sides with allocation |
| AGREE | Contract checker for AADL | One proof per hierarchy layer; realizability check | Check that a summary is implementable at all |
| OCRA | Contract refinement checker | n + 1 obligations per decomposition, including an environment-side one | Check both directions between parent and children |
| Reflexion models | Code vs drawn architecture | Lift dependencies through a map; report convergences, divergences, absences | The structural abstraction-barrier check; build first |
| Hierarchical reflexion, HuGMe, InMap | Extensions | Hierarchy, incremental recompute, tool-proposed maps | Needed once the ladder has more than one upper level |
| ArchUnit, jQAssistant, import-linter, SonarQube, Lattix | Dependency-rule tools | Layers as patterns or queries; inherited rules; baselines of known violations | Baselines, transitive paths, "unmapped code" as a finding |
| LLM conformance (RAMD, SemRef, 2025–26) | Research prototypes | LLM proposes maps; best results need a deterministic loop | LLMs drift to code-level granularity |

## Three different things are called a level

| Kind | What changes between levels | Who decides the levels | Relation between levels | Examples |
| --- | --- | --- | --- | --- |
| Refinement level | Detail of state and steps of one system | People, sometimes a generator | Simulation or refinement mapping | Event-B, TLA+, seL4, CompCert passes, TLM vs RTL |
| Precision level | How much of one program's state an analysis tracks | The analysis, by computation | Galois connection, or a predicate set refined on demand | Abstract interpretation, CEGAR |
| Structural or view level | Which containers are open | A method or notation | Containment; upper edges implied by lower ones | C4, Capella, AADL, MLIR dialects (partly) |

Mental's ladder mixes the first and third kinds. Space zoom is structural: a closed entity's behavior is exactly the product of its interior (see the [DEVS note](../model-interpreter-semantics/notes/devs_formalisms.md)), so structural levels need only structural checks. Logical levels with new vocabulary are refinement levels and make claims the code may contradict. These include mechanisms, the business level, a project's domain ladder, and the summary level of a behavior. Predicates and input classes are precision-level machinery used inside each level.

## What a refinement level is made of

| Ingredient | Event-B | TLA+ | seL4, CertiKOS | Mental today |
| --- | --- | --- | --- | --- |
| Vocabulary | machine variables and events | variables and actions | abstract state and primitives of a layer | vocabulary per level |
| Abstraction from lower to upper state | gluing invariant | refinement mapping, plus history, prophecy or stuttering variables | state relation of the simulation | implicit |
| Step correspondence | concrete event refines one or more abstract events, or `skip` | step or stutter; one lower step may need several upper steps | each primitive simulates its spec | upper step to anchored lower steps |
| Invariants per level | machine invariants | invariants per spec | invariants over abstract state; most of the proof effort | none |
| Hidden-step termination | variants on convergent and anticipated events | fairness | total-correctness proofs | none |

## Findings by cluster

### Formal refinement methods: refinement can be checked by exploration, but only safety transfers

Event-B generates a fixed list of proof obligations per refinement step: invariant preservation including the gluing invariant, guard strengthening, action simulation, witness feasibility, and variant obligations. A *convergent* new event must decrease the variant; an *anticipated* one may postpone the proof to a later refinement ([Rodin proof obligation names](https://wiki.event-b.org/index.php/Proof_Obligation_Names_(Rodin_User_Manual)); [Rodin Handbook](https://stups.hhu-hosting.de/handbook/rodin/current/html/machine.html)). The standard obligations cover safety only. Rodin generates no deadlock-freedom obligation, so a lower level can do less than the upper level promised. In 2025, Stock, Leuschel and Mashkoor added failures-divergence refinement to ProB. It treats a cycle of new events on a finite state space as divergence, and it found a bug in a published interlocking model ([arXiv 2505.19069](https://arxiv.org/abs/2505.19069)). ProB also checks trace refinement between two machines by exploring states, without using the gluing invariant at all ([ProB refinement checking](https://prob.hhu.de/w/index.php/Refinement_Checking)). FDR does the same for CSP, with shortest counterexamples and over ten billion states in a few hours ([FDR3](https://www.cs.ox.ac.uk/files/6237/Document.pdf)). These two tools are the precedent for Mental's "refinement checked by an interpreter".

Proof automation is high but not complete. One evaluation over 3,641 Rodin obligations left about 16% unproved with the Atelier B provers and 4% with SMT solvers added ([ADVANCE D3.3](https://wiki.event-b.org/index.php/ADVANCE_D3.3_Improvement_of_automated_proof)). Industrial chains stay short: a typical B train-protection project has 233 machines but only 46 intermediate refinements ([arXiv 2005.07190](https://arxiv.org/abs/2005.07190)).

The TLA+ refinement mapping is the abstraction function Mental needs. Lamport and Merz show when it fails to exist. The upper level may remember more than the lower keeps (add a history variable), decide earlier (prophecy variable), or take steps the lower lacks (stuttering variable) ([arXiv 1703.05121](https://arxiv.org/abs/1703.05121)). TLA+ trace validation is not a refinement check: its authors say it can accept wrong traces when logs are imprecise ([arXiv 2404.16075](https://arxiv.org/abs/2404.16075)), and MongoDB gave it up ([MongoDB](https://www.mongodb.com/blog/post/engineering/conformance-checking-at-mongodb-testing-our-code-matches-our-tla-specs)). ASM allows m abstract steps to match n concrete ones, compared only at chosen "states of interest". Both points show that "every lower step is an upper step or a stutter" is too narrow.

LLMs have entered the field. Event-B Agent has an LLM plan refinement steps and gluing invariants, validated by ProB and then by provers ([arXiv 2605.17475](https://arxiv.org/abs/2605.17475)). Specula has agents choose abstraction levels from code and validates with traces and model checking. It also reports a direct warning: the agents overfit trace validation by weakening guards and adding permissive transitions ([arXiv 2607.25333](https://arxiv.org/abs/2607.25333)).

### Layered verified systems: invariants dominate the cost, and fewer levels is the recent trend

seL4 links an abstract spec, an executable spec and C code by two forward-simulation proofs. In the first refinement, 80% of the effort went to invariants; the C refinement added none ([CACM 2010](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)). Integrity carried down to C, but confidentiality "is not preserved by refinement without additional conditions". It needed a deterministic abstract spec and about 18.5 person-months of repair before 20.4 person-months of confidentiality proofs ([TOCS 2014](https://trustworthy.systems/publications/nicta_full_text/7371.pdf)). Only properties of the form "every step preserves P" carry down for free.

CertiKOS built mCertiKOS from 37 thin layers in under a person-year ([POPL 2015](https://www.cs.columbia.edu/~rgu/publications/popl15-gu.pdf)). Its concurrent successor reports that at least a third of its proofs are redundant, because invariant proofs are duplicated across layers on any change ([OSDI 2016](https://www.usenix.org/system/files/conference/osdi16/osdi16-gu.pdf)). IronSync, Verus, Atmosphere, Perennial and Grove use only a spec and the code, and decompose by ownership instead.

On atomicity across levels, the evidence divides three ways:

- **IronFleet** argues reduction on paper only; Dafny checks just a syntactic rule that every host step does all its receives before its sends ([SOSP 2015](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)).
- **CIVL** states the exact per-run rule Mental wants: the refined action's effect "happens in exactly one yield-to-yield fragment", and every other fragment leaves upper state unchanged ([FMCAD 2021](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)).
- **Armada** links levels by recipes naming one of eight strategies. Easy steps take 4–7 lines, while reduction and rely-guarantee steps take about 100 lines plus invariant files of up to 528 lines ([PLDI 2020](https://6826.csail.mit.edu/2020/papers/armada.pdf)).

Liveness is not untouched. CertiKOS proves starvation freedom under a fair scheduler, and Anvil proves liveness on a model linked to the code one step at a time ([OSDI 2024](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)).

AWS's ShardStore keeps reference models at about 1% of the implementation's size and checks them by property-based testing. Concurrent crash runs are not checked, and one bug was missed because tests never reached its code path ([SOSP 2021](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)). Kondo generates the asynchronous protocol level and its routine invariants from a synchronous one ([OSDI 2024](https://www.usenix.org/system/files/osdi24-zhang-nuda.pdf)). LLM provers reach 81% on 849 system-level Verus tasks, but only 37% on Anvil's controller tasks, where the spec contains a whole state machine ([VeruSAGE](https://arxiv.org/pdf/2512.18436)).

### Compilers and computed abstractions: validate each output, and keep predicates between runs

CompCert has 8 intermediate languages and 14–20 passes, each with a simulation proof. Only register allocation is validated per run, and that validator is itself proved, so it gives the same guarantee as a proof. An unverified validator, like Mental's would be, gives less. Necula's validator for GCC cost about one compiler pass to build, slowed compilation fourfold, and raised false alarms on up to 10% of functions in some passes, because it had to infer the relation between input and output ([PLDI 2000](https://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf)). Alive2 aims for zero false alarms by bounding loops. It found 47 LLVM bugs by re-running existing unit tests, and it reports separate outcomes for validated, violated, timeout and unsupported ([PLDI 2021](https://users.cs.utah.edu/~regehr/alive2-pldi21.pdf)). CakeML adds an intermediate language only when a language feature is compiled away ([JFP 2019](https://cakeml.org/jfp19.pdf)).

MLIR is the closest structural precedent. Levels are open-ended dialects that can be mixed in one program, and each conversion targets a declared set of legal operations, with partial or full conversion ([MLIR dialect conversion](https://mlir.llvm.org/docs/DialectConversion/)). Its verifier checks structure only. Semantic checking arrived in 2025, by lowering each dialect to SMT, and found five upstream miscompilations ([PLDI 2025](https://pldi25.sigplan.org/details/pldi-2025-papers/60/First-Class-Verification-Dialects-for-MLIR)).

Predicates are computed and maintained in this family. CEGAR grows the predicate set from spurious counterexamples. CPAchecker stores those sets ("precisions") and reuses them across revisions, more than four times faster over 1,119 revisions of 62 Linux drivers. Precisions attached to a function survive edits; those attached to a program location do not ([FSE 2013](https://www.sosy-lab.org/research/pub/2013-FSE.Precision_Reuse_for_Efficient_Regression_Verification.pdf)). Daikon mines candidate invariants from runs ([Daikon manual](https://plse.cs.washington.edu/daikon/download/doc/daikon.html)), and Houdini keeps the largest subset of candidates that survives checking.

LLM invariant generation works on small programs when a checker is in the loop:

- Lemur solved 107 of 133 Code2Inv tasks ([arXiv 2310.04870](https://arxiv.org/html/2310.04870)).
- Loopy went from 293 to 398 of 469 with filtering and repair. 44 of its 71 failures needed disjunctions ([arXiv 2311.07948](https://arxiv.org/html/2311.07948)).
- First-attempt generation reaches about 78%, but repairing a bad invariant succeeds only about 16% of the time.

Only 38.2% of one LLM tool's vectorized loops validated under Alive2 ([arXiv 2406.04693](https://arxiv.org/abs/2406.04693)). Deterministic scaffolding raised the number of proofs and cut cost by 88% ([Trivet](https://cs.uwaterloo.ca/~cnsun/public/preprint/trivet/)). NeuroAbs and PolyVer already have an LLM propose an abstraction or contract that a solver checks and counterexamples refine ([arXiv 2608.17304](https://arxiv.org/abs/2608.17304); [arXiv 2503.03207](https://arxiv.org/abs/2503.03207)).

One point is new for Mental. CEGAR's abstraction is sound by construction and can only be too coarse. An LLM-proposed level can also be *unsound*, dropping behavior the code has. Mental must check both directions and report them differently.

### Hardware, simulation and planning: checks are routine when the generator says what it did

Gate-versus-RTL checking is routine because registers can be matched, and because the synthesis tool writes a guidance file of its transformations; Formality proves or rejects each entry ([Synopsys white paper](https://www.synopsys.com/content/dam/synopsys/implementation&signoff/white-papers/higher-completion-formality-wp.pdf)). That is the template for LLM-proposed level maps. C-to-RTL transaction equivalence is commercial practice for blocks. The checkers themselves had bugs: fuzzing the three commercial tools found 16 bugs, 9 of them cases where the tool wrongly claimed equivalence ([DVCon 2024](https://www.doc.ic.ac.uk/~afd/papers/2024/DVCon.pdf)).

Arm's ISA-Formal checks pipelines against the architecture spec with bounded model checking, one instruction class at a time. It excludes fetch, the memory system, floating point and most exceptions. The hand-written abstraction function is about 2,500 lines per processor, and Arm calls writing it "a difficult task" ([CAV 2016](https://alastairreid.github.io/papers/CAV_16); [limitations](https://alastairreid.github.io/isa-formal-limitations)). riscv-formal defines a standard record per retired instruction and runs generic checks on it: per-step checks, plus sequence checks that nothing is lost, duplicated or reordered ([RVFI](https://yosyshq.readthedocs.io/projects/riscv-formal/en/latest/rvfi.html)).

Multi-resolution simulation defines consistency as a commuting diagram, aggregate-then-run against run-then-aggregate, with only the evaluation approximate. The RAND primary texts were unreachable; see the notes. In planning, checking a plan against a task hierarchy is NP-complete without the decomposition ([Behnke et al., ICAPS 2015](https://gki.informatik.uni-freiburg.de/papers/behnke-etal-icaps15.pdf)), so the step map has to be stored. Bacchus and Yang measured a refinement probability per level pair and merged levels below 0.5 ([tech report](https://cs.uwaterloo.ca/research/tr/1992/45/drp.pdf)).

Angelic semantics bounds what a high-level action can reach from above and below. The lower bound is what the planner can *force* by choosing a refinement ([ICAPS 2007](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf); [ICAPS 2008](https://aima.eecs.berkeley.edu/~russell/papers/icaps08-hla.pdf)). For actors the system does not control, such as a courier, only the upper bound is meaningful. LLM planners now score proposed abstract operators by successes over uses ([Ada](https://arxiv.org/abs/2312.08566)) or verify each LLM decomposition ([ChatHTN](https://arxiv.org/abs/2505.11814)). Netsem specified TCP at both protocol and service level, with a testable abstraction function checked on captured traces. It took 11 person-years and no full proof ([Bishop et al.](https://www.trustworthy.systems/publications/full_text/Bishop_FMNRSSW_19.pdf)).

### Architecture models and conformance: structure is checkable, and detection alone fixes nothing

Structurizr derives upper-level relationships from lower ones. That is on by default in the DSL and off in the Java library, whose eager strategy is documented as causing an "explosion of relationships" ([Structurizr](https://docs.structurizr.com/java/implied-relationships)). Capella's maintainers removed several validation rules that required elements to trace up to the level above, calling them useless ([Capella forum](https://forum.mbse-capella.org/t/how-to-get-rid-of-validation-warnings-from-rules-tj-la-02-capability-refinement-and-tc-dc-11-logical-component-realization/4844)). A rule requiring every lower element to trace upward does not survive real models. SysML v2, a formal standard since September 2025, treats "refinement" as a metadata tag with no semantics.

AGREE checks contracts one hierarchy layer at a time and orders siblings to avoid unsound circular reasoning ([arXiv 1502.03343](https://ar5iv.labs.arxiv.org/html/1502.03343)). OCRA gives n + 1 obligations per decomposition. They include an environment-side obligation: the parent's environment plus the siblings must satisfy each child's assumptions ([ASE 2013](https://es-static.fbk.eu/people/tonetta/publications/ase13.pdf)). Mental's checks so far point downward only.

Reflexion models remain the cheapest cross-level check. They scaled to Excel's 1.2 million lines with a 971-line map. Field studies found that detection did not lead to removal and that the effort of building the mapping is the main barrier ([Rosik et al. 2011](https://pure.ul.ie/en/publications/assessing-architectural-drift-in-commercial-software-development-/); [Ali et al. 2018](https://bura.brunel.ac.uk/bitstream/2438/14795/4/Fulltext.pdf)). SonarQube now ships an intended-architecture editor with inherited rules ([SonarQube](https://docs.sonarsource.com/sonarqube-server/architecture-analysis/project-architecture)). LLM mapping works with a deterministic loop: RAMD reaches 85.8% with a compliance check in the loop ([arXiv 2611.14606](https://arxiv.org/abs/2611.14606)), and doc-to-code links reach F1 0.86 ([Fuchß et al., ICSA 2025](https://fuchss.org/conferences/icsa25/)). Without one, LLM-generated architecture views show consistent "granularity mismatches", working at code level instead of architectural level.

## Answers to the three questions

**Do they run verifiers at each level?** They run them per pair of adjacent levels, and they check invariants per level on top. Event-B, seL4, CertiKOS, CIVL, Armada, CompCert, AGREE, OCRA and the hardware flow each split one big check into one check per adjacent pair, each with the method that fits that pair. Nobody runs one verifier from the top level to the bottom. Alive2 is the one partial exception: it can check any before-and-after pair, adjacent or not.

**How are predicates computed and maintained?** In refinement methods they are authored. The upper level's state is defined as a function of the lower one (TLA+ mapping) or related to it (Event-B gluing invariant), so upper predicates are derived. In analysis tools they are computed: CEGAR from spurious counterexamples, abstract interpretation as fixed points, Daikon from runs, Houdini by filtering, IC3 one lemma at a time, and LLM pipelines by proposing candidates for a checker. Maintenance across change is solved in places. CPAchecker reuses stored predicate sets, and Goblint re-analyses incrementally. Elsewhere it is expensive: seL4's reply-capability change touched under 5% of the code and cost about a person-year to re-verify.

**How are invariants and variants maintained?** Invariants are stated at the highest level whose vocabulary can express them and inherited by refinements, but only properties of the "every step preserves P" kind; information-flow and liveness properties need more. Each level adds the invariants its own simulation needs, and these dominate cost. Termination of hidden steps is either proved with an authored measure (Event-B variants; ASM's well-founded orders on one-sided diagrams) or detected as a cycle of hidden steps on a finite graph (FDR, mCRL2, ProB). Event-B adds a useful three-way classification of new events: ordinary (may repeat forever), anticipated (bound postponed) and convergent (bound proved).

## What Mental should take

Everything in this section is a design inference for Mental, assembled from the precedents above. The items are ordered by when to build them.

**1. Build the structural barrier check first, as hierarchical reflexion with MLIR-style legal sets.** Each level declares its legal vocabulary. Lifting every extracted dependency, wire and data flow through the mapping classifies each upper edge as convergent, divergent or absent. The check needs four features:

- rules inherited down the containment tree;
- transitive paths counted;
- a committed baseline of accepted leaks, so that only new ones fail;
- unmapped code reported as its own category, with a switch that turns it into a failure.

Derive upper wires on read and mark them as derived, never materialize all pairs. This needs no interpreter. Findings should be few, tied to a diff, and dismissible into the baseline with a reason, because the field studies show that undismissable findings are ignored.

**2. Make the LLM emit the relation with the level, as typed hints.** Following Formality's guidance files and Necula's lesson about inferred relations, the LLM proposes the abstraction function and the step map along with the level, as hints of fixed kinds: rename, group, hide, merge or split state, map a lower segment to one or more upper steps, declare a lower step invisible. Each hint is accepted or rejected separately, and an upper element whose hint is rejected is marked unverified. The step map is stored as a witness and never reconstructed from a trace. The abstraction function is the riskiest artifact, so debug each new one against two or three simple scenarios a person has confirmed.

**3. Check each pair of levels by exploration, with CIVL's rule and m:n steps.** Cut a lower run into fragments at points where other actors may run, and compare only at states of interest, such as no mechanism mid-operation. Exactly one fragment may change the projected upper state, and it must do so by the claimed upper step or sequence of upper steps; every other fragment must leave upper state unchanged. When another actor's interleaved fragment is what breaks the rule, report a race at the level where the user thinks. When no mapping exists without a forbidden trace, classify the cause from Lamport and Merz: the upper level remembers too much, decides too early, or has steps the code lacks. Report "not shown" rather than "violated" in those cases.

**4. Report graded findings in both directions.** From CSP, run three passes, cheapest first:

- *trace violation*: the lower level does something the upper level forbids;
- *refusal*: an upper step is enabled but has no lower realization;
- *divergence*: there is a reachable cycle of hidden lower steps.

From the compiler and CEGAR evidence, name the two directions separately. *Missing behavior* means the summary is unsound and must be fixed; *extra behavior* means it is imprecise and should be refined. From OCRA, add the environment-side check: each child is driven only within the guards its summary assumes. From riscv-formal, add sequence checks: no upper step is lost, duplicated or reordered. Each verdict carries its bounds, using Alive2's outcomes: validated within bounds, refuted with a run, timeout, unsupported, or checker error.

**5. Give hidden loops a convergence status.** When the divergence pass finds a cycle of hidden steps, classify it as Event-B does: *convergent* if code or a pack names a bound (a retry counter, a queue's maximum delivery count), *anticipated* if the bound is assumed and recorded as an assumption, *ordinary* if the loop is meant to run forever, such as a poll loop. Only an unclassified cycle is a *hidden livelock* finding.

**6. Propose invariants generously and keep only what survives.** Ask the LLM for many candidate invariants per level, in that level's vocabulary, and add candidates mined from interpreter runs. Drop every candidate that bounded exploration falsifies (Houdini, Daikon). Derive routine invariants about messages and monotonic state mechanically from wires and stores (Kondo); ask the LLM only for protocol-level and business-level ones. Record each invariant's shape, because only "every step preserves P" invariants are inherited by a lower level that passes the pair check. Attach each invariant to the level that owns its state, and track dependencies so that a code change re-checks only what it touches.

**7. Guard against vacuous levels.** Specula's agents passed trace checks by making the upper level permissive. Four countermeasures:

- *Mutation:* mutate lower runs, for example by reordering two conflicting steps or dropping a write, and require that some mutants fail.
- *Realizability:* require every summary case to be witnessed by at least one lower run, and every admitted input to fire some case.
- *Granularity:* reject levels that restate the level below. Measure the ratio of upper to lower entities and the share of lower identifiers in the upper vocabulary.
- *Tie-break:* when two proposals both pass, prefer the smaller and more deterministic one.

**8. Name the level patterns and add a level only when vocabulary changes.** A level transition should be one of a closed list of patterns, each with its own check:

- *hide*: compute this level by hiding and reducing (mCRL2); ask the LLM only for the hide set and names;
- *group*: an upper entity is a set of lower entities;
- *atomize*: Armada's reduction, with IronFleet's syntactic rule as a cheap first check;
- *summarize*: summary cases checked against the full graph;
- *fault model*: Verdi's ideal network refined into a faulty one;
- *crash-recovery*: Perennial's crash plus recovery as one upper step;
- *assume*: no code below the level.

Following CakeML, accept an LLM-inserted level only if some vocabulary appears or disappears at it. There is no general answer to how many levels to have. CertiKOS used 37, B projects keep refinement chains short, and recent Verus projects use two. Thin levels make each check easy but duplicate invariant work on change.

**9. Keep results and predicates between builds.** Store the predicate set of each behavior, keyed by port or function rather than by statement position, start each rebuild from it, and re-validate (CPAchecker). Cache results per pair of levels with dependency tracking, in the Git note next to the model. Keep refinability counters per upper step (uses and successful realizations), and propose merging an inserted level whose ratio stays low (Bacchus and Yang, Ada).

**10. Describe uncontrolled actors with an upper bound.** For people and external systems, such as a courier, a customer or an operator, store what they *may* do and require invariants to hold over all of it. A lower bound, what can be *forced*, only makes sense for choices the modeled system itself makes.

**11. Test the checkers.** Generate level pairs that are equal by construction and pairs with one seeded mismatch, and confirm that the checker accepts the first and rejects the second. Commercial equivalence checkers failed this test through interactions between features, not through any single feature.

### Proposed fields for the `level` record

| Field | Meaning | Checked by |
| --- | --- | --- |
| `vocabulary` | legal entities, state variables, predicates and step kinds | anchoring; full-conversion check of the barrier |
| `below` | the level or levels it refines (a DAG) | — |
| `pattern` | hide, group, atomize, summarize, fault model, crash-recovery or assume, with parameters | that pattern's checker |
| `abstraction` | typed hints: one expression per upper variable or predicate over lower state | per-hint acceptance; evaluated at states of interest |
| `states_of_interest` | where the upper state is defined, such as no mechanism mid-operation | — |
| `step_map` | lower segment to one or more upper steps, or invisible; stored as a witness | CIVL-style fragment rule; sequence checks |
| `invariants` | properties in this level's vocabulary, each with its shape and source | bounded exploration on projected states |
| `hidden_loops` | each cycle of invisible steps, with convergent, anticipated or ordinary status and its bound | divergence pass |
| `actors` | for each uncontrolled actor, `may` (upper bound) and optional `can_force` | used by the interpreter; shown as assumptions |
| `verdicts` | outcome per pair check, with bounds, coverage and refinability counters | — |

## Where Mental is alone, after the sweep

- **LLM-proposed levels are no longer unprecedented, but a full ladder is.** Event-B Agent, Specula, NeuroAbs, PolyVer, RAMD and SemRef have an LLM propose a refinement step, abstraction or mapping that a tool checks. No system found has an LLM propose a ladder of several named levels from source code, each with its own vocabulary, abstraction function, step map and behavior, checked by an interpreter.
- **No refinement evidence comes from ordinary web services.** All of it comes from kernels, compilers, consensus protocols, storage engines, hardware, avionics and railway software. None comes from services wired through HTTP, queues and databases.
- **Bounded checks prove less than they look.** TLC, Alive2, ISA-Formal and Mental's projection check all hold only within their bounds, and the bounds must be shown with every result.
- **Liveness across levels is mostly unaddressed in practice.** CertiKOS, IronFleet and Anvil prove liveness at an upper level under explicit fairness assumptions. CIVL, Armada, Verdi, Perennial and IronSync cover safety only. Mental should report liveness as bounded and state its assumptions.
- **Nobody has measured how LLMs assign code to levels of a ladder.** Benchmarks exist for components and documents, not levels. Git's porcelain and plumbing command lists are a possible ground truth for the domain-ladder experiment.

## Corrections to the first draft

The first version of this report was written from background knowledge. The sweep corrected it in these places:

- **Variants are not unique to Event-B.** ASM, FDR, mCRL2 and ProB handle hidden-step termination too.
- **"Every lower step is an upper step or a stutter" is too narrow.** Steps map m:n, compared at states of interest.
- **TLA+ trace validation is weaker than a refinement check.** It is not the same thing as the projection check.
- **seL4's confidentiality proof did not carry down for free.** The 80% invariant share applies to the first refinement only.
- **IronFleet's reduction argument is on paper, not machine-checked.** Its liveness proofs are machine-checked.
- **"Many thin levels beat few thick ones" is not general.** CertiKOS also reports duplicated invariant proofs, and B chains and recent Verus projects are short.
- **CompCert has 8 intermediate languages, not about twenty.** Its one validated pass uses a proved validator.
- **Predicate maintenance is not always re-run from scratch.** CPAchecker reuses stored predicate sets across revisions.
- **ShardStore's reference models are about 1% of the implementation.** The draft said "a few times smaller".
- **Angelic semantics was misread.** Its lower bound is what the planner can force, not what an action is guaranteed to do.
- **Hardware checks are less settled than the draft said.**
  - *DPV:* the claim that it needs no correspondence between internal state could not be confirmed.
  - *Checker bugs:* the commercial C-to-RTL checkers had soundness bugs.
  - *TLM vs RTL:* "open" applies to loosely timed virtual platforms, not to untimed blocks.
- **Several architecture claims were wrong.**
  - *AGREE:* the sources do not state "N + 1" or "k-induction", or that the unrealizable contracts were at leaf components.
  - *OSATE:* no source supports the claim that it checks flow implementations against flow specifications.
  - *OCRA:* it has an environment-side obligation the draft missed.
- **Schellhorn's ASM work was mis-cited.** It is in JUCS 2001, with the completeness result in ENTCS 2008, not TCS 2005.
- **Missing systems are now included:** ProB, FDR, mCRL2, MLIR, CakeML, Alive2, Houdini, Verdi, Perennial, Anvil, Kondo, riscv-formal, Netsem, SysML v2, SonarQube and the LLM pipelines.

## Open questions

- Can states of interest be found mechanically (quiescent mechanisms, transaction boundaries, lock release), or must the LLM propose them per level, and how is a bad choice detected?
- Is the "exactly one fragment" rule too strict for upper steps that span several commits, such as a saga? CIVL would use pending asynchronous calls; Mental may need an equivalent pattern.
- Does the refusal check flood ordinary code with findings, when many upper steps are legitimately disabled by lower conditions the upper vocabulary cannot name? Event-B allows guard strengthening and CSP's failures model does not. Which default suits users who do not write specifications?
- How large is the abstraction function plus step map for a real service, relative to the upper level? CCF's trace spec was about a third of its main spec and changed often.
- What false-alarm rate will users accept from the pair check? Necula accepted 10% on some passes; Alive2 chose zero and accepted missed bugs.
- For a domain ladder like Git's, is the link between rungs closer to data refinement (a retrieve function from storage to plumbing objects) than to event refinement? If so, VDM's rules may be the better template.
- How stable are LLM-proposed predicate names and abstraction functions across commits when seeded with the stored set? No published measurement exists.
