# Layered verified systems: how levels are defined, linked, checked and maintained

Scope: systems that were verified as a stack of levels, from an abstract specification down to running code: seL4, CertiKOS and its concurrent layer toolkit (CCAL), IronFleet and its successors (IronSync, Verus, Atmosphere), Verdi, CIVL, Armada, Perennial and Grove, Anvil, and the lighter refinement-style checking used at AWS (ShardStore, Cedar, the Dafny authorization engine, s2n). A final section covers 2023–2026 work that uses tools or LLMs to write invariants and proofs for such systems. For each system the notes answer: who writes the levels, what links adjacent levels, what is checked per level or per pair and at what cost, how invariants and liveness are handled, how atomicity is handled, and what Mental (an interpreter with LLM-proposed levels and no theorem prover) can borrow.

Reading notes for the writer:

- Evidence tiers. **[V]** = the primary source (paper PDF or official page) was downloaded and its text read in this session (13 November 2026); numbers and quotes come from that text. **[S]** = the claim was seen only in a search-result summary or abstract page in this session; the linked source was not read in full. **[B]** = training knowledge; not re-fetched. Treat [S] and [B] items as needing a spot check before quoting.
- PDF text was extracted with a script, so quotes may differ from the original in ligatures and line breaks, not in wording.
- "py" = person-year, "pm" = person-month. "Proof-to-code ratio" = lines of proof or annotation per line of executable code, as each paper defines it; definitions differ between papers, so compare ratios loosely.
- Section 12 lists every point where these sources contradict or sharpen the draft report (`../report.md`).

## At a glance

| System | What it is | How it handles levels | Take for Mental |
|---|---|---|---|
| seL4 | Verified OS microkernel (Isabelle/HOL) | Abstract spec → executable spec (auto-translated from a Haskell prototype) → hand-written C → binary; one forward-simulation proof per pair | Invariants live at the middle level; only Hoare-style properties carry down for free |
| CertiKOS / CCAL | Verified (concurrent) OS kernel (Coq) | 37 thin layers (sequential kernel); each layer = interface + module + simulation proof; concurrency as logs and environment contexts | Thin levels make variants cheap but duplicate invariant work on every change |
| IronFleet | Verified Paxos library and key-value store (Dafny) | Spec → protocol → implementation; refinement proved per pair; atomicity of a host step argued by reduction on paper | Check a cheap syntactic obligation (receives before sends) instead of proving reduction |
| IronSync / Verus / Atmosphere | SMT-based verification of concurrent Rust/Dafny systems | Two levels (spec, code), split by concern: a "sharded" transition system carries the protocol, ownership types tie code to it | Recent practice has fewer levels, not more; decompose state by ownership |
| Verdi | Verified distributed systems (Coq) | Levels are fault models; a verified system transformer lifts a system and its proofs to a harsher network | Treat "add retries / dedup / replication" as a level change with a property-transfer rule |
| CIVL | Verifier for layered concurrent programs (Boogie) | One program text with layer numbers; each layer turns a procedure into an atomic action using mover types and yield invariants | Its refinement check is exactly "one fragment does the upper step, the rest stutter" |
| Armada | Refinement prover for low-level concurrent code (Dafny) | Chain of programs; each pair linked by a recipe naming one of 8 strategies; proof text is generated | A closed list of level patterns, each with its own generated check |
| Perennial / Grove | Crash-safe and distributed systems in Go (Coq, Iris) | Code vs. one spec; a crash plus recovery must look like one atomic crash step above | "Crash and recover" is an upper step too; add a recovery-refinement check |
| Anvil | Verified Kubernetes controllers (Verus) | Code → controller model (1 code step = 1 model step) → liveness proved on the model with an environment model | Prove or explore liveness at the model level, under stated fairness assumptions |
| AWS: ShardStore, Cedar, AuthV2, s2n | Production systems with refinement-style checks | Small executable reference model; conformance by property-based or differential testing; proofs only on the model or in CI | Closest precedent: a tiny model in the same repo, checked per run, kept current by engineers |
| Kondo, LLM provers | Tools that write invariants or proofs | Kondo generates the asynchronous level and its routine invariants from a synchronous one; LLM agents fill proofs under a human spec | LLMs succeed when a checker judges them; state-machine and temporal obligations stay hard |

## 1. seL4

### Takeaway

seL4 has three authored levels in Isabelle/HOL plus the compiled binary, linked by two interactive forward-simulation proofs and one automatic translation-validation step. The cost sits in the first refinement (abstract to executable), and 80% of that went to invariants, not to the correspondence itself. Properties proved at the top carry down only when they have the right shape: integrity did, confidentiality did not without changing the abstract spec. Proof maintenance cost follows how cross-cutting a change is, not how large it is.

### Cited Findings

- [V] The levels: an abstract specification written by the verification team in Isabelle/HOL; an executable specification obtained by automatic translation of a Haskell prototype kernel into Isabelle; and the C implementation, which was "translated ... manually" from the Haskell prototype and then parsed into Isabelle. The translations are not trusted because the proof is about the C code as parsed. — [Klein et al., CACM 2010, §3.1, §4, Fig. 2](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] The abstract level uses non-determinism to leave choices open. The scheduler at the top is "picking any runnable thread"; the executable level fixes priority round-robin with explicit queues. — [same, §4.1–4.2](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] The link is forward simulation with one state relation R per pair: for each concrete transition there is an abstract transition such that R holds again afterwards; R is equality on externally visible state. Two theorems (executable refines abstract, C refines executable) compose by transitivity. The machine model and the model of user programs are the same at all levels; only kernel state and behavior change. — [same, §4.4, Fig. 6](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] Size and effort (Table 1): abstract spec 4.9 kloc Isabelle, about 75 invariants; executable spec 5.7 kloc Haskell / 13 kloc Isabelle, about 80 invariants, 8 py and 110 kloc of proof; C 8.7 kloc, 0 additional invariants, 3 py and 55 kloc of proof. Total proof 200,000 lines; 20 py overall, of which 9 py was reusable frameworks and 11 py seL4-specific. The authors estimate 6 py to repeat for a new kernel. — [same, §5.1](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf); [project numbers page](https://trustworthy.systems/projects/OLD/l4.verified/numbers)
- [V] "80 % of the effort in the first refinement went into establishing invariants, only 20 % into the actual correspondence proof." Invariants are expensive because each must be proved "for the whole kernel", not only for the functions that touch the data structure. — [same, §3.2, §5.1](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] What each refinement found: the first led to about 300 changes in the abstract spec and 200 in the executable spec, about half of them real algorithm or design bugs. The second (to C) found 160 bugs, mostly typos and missed updates, and no deep algorithmic bugs. — [same, §5.1](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] Property transfer: refinement "ensures that all Hoare logic properties of the abstract model also hold for the refined model"; the paper adds in parentheses that not all security properties can be stated that way. — [same, §4](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] Integrity and authority confinement were proved on the abstract spec in 7.4 pm and 10,500 lines, and hold for the C code by composition with refinement. "The confidentiality statement is not preserved by refinement without additional conditions": the team had to make the abstract spec deterministic and repair the functional-correctness proofs (about 18.5 pm), change the scheduler (1.8 pm), then prove confidentiality (20.4 pm). — [Klein et al., TOCS 2014, §5.2.3, §7.2](https://trustworthy.systems/publications/nicta_full_text/7371.pdf)
- [V] C to binary is "an automatic proof of refinement between the semantics of the kernel binary after compilation/linking and the C source code semantics" (translation validation, about 2 py to build). — [TOCS 2014, §1, §4.7, Fig. 19](https://trustworthy.systems/publications/nicta_full_text/7371.pdf)
- [V] Concurrency is avoided by design: a single-processor, event-based kernel with one kernel stack and a "mostly atomic" API; long operations poll for interrupts at fixed points and restart. "As a result, we completely avoid concurrent execution in the kernel." Termination of every kernel call is proved. — [CACM 2010, §2, §3.2](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] Cost of change: a local optimisation costs effort "roughly proportional to the size of the change"; a new independent system call took one day to implement and under one person-week to re-prove; adding interrupts, ARM page tables and address spaces cost 1.5–2 py of re-verification (new invariants had to be preserved "over the whole kernel"); reply capabilities touched under 5% of the code but cost about 1 py, 17% of the original proof effort, because they broke key invariants. — [same, §5.2](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)
- [V] After 2009 the two refinement proofs entered a maintenance phase; every change to kernel or models requires re-establishing all affected proofs, and all proofs run in an automated checking suite like a regression test. Total proof files were about 500,000 lines by 2015. — [TOCS 2014, §7.1](https://trustworthy.systems/publications/nicta_full_text/7371.pdf); [numbers page](https://trustworthy.systems/projects/OLD/l4.verified/numbers)
- [V] Today proofs exist for Arm, RISC-V and Intel configurations, but "Not all properties are available on all platforms and all configurations"; multicore is unverified. Supporting more platforms through proof branches makes "maintenance cost increase". — [seL4 proofs page](https://sel4.systems/Verification/proofs.html); [Klein, seL4 Summit 2023 slides](https://sel4.systems/Summit/2023/slides/day1-04-klein.pdf)

### Inferences

- The middle level is where invariants belong. seL4 proved about 80 invariants on the executable spec "such that they directly transfer to the data structures used in the C program", and needed none new at C. For Mental this argues for stating data-structure invariants at the mechanism or component level, where the vocabulary is close to the code, and checking business invariants separately at the top.
- "Prove at the top, carry down" needs a qualifier. It holds for properties of the form "every upper step preserves P". It fails for properties that non-determinism at the top can hide (information flow, and by the same argument some fairness and timing properties). Mental's upper levels are full of LLM-introduced non-determinism, so the report should say which invariant shapes are inherited and which are not.
- The first refinement is a design review: half of its 500 spec changes were real bugs. The second is a transcription check. Mental's upper pairs (business to system, system to services) should be expected to find model and design errors; its lowest pair (statements to mechanisms) should find mostly anchoring mistakes.
- Proof cost tracks invariant breakage. A Mental analogue: when code changes, re-run only the level pairs whose abstraction expressions or invariants mention the touched state, and report "this change breaks invariant X at level L" as the unit of maintenance.

### Gaps

- The binary translation-validation paper (Sewell et al., PLDI 2013) was not opened; its description here comes from the TOCS 2014 summary.
- No current, primary figure for total proof size after 2015 was found; search results mention "over a million lines" [S] without a primary page.
- Quantitative studies of seL4 proof effort versus size (Staples et al. 2014, Matichuk et al. 2015) appeared only in search summaries [S] and are not cited above.

## 2. CertiKOS and certified (concurrent) abstraction layers

### Takeaway

CertiKOS builds a kernel as many thin layers in Coq. Each layer has an interface (abstract state plus primitive specifications), a module written against the layer below, and a simulation proof that implies contextual refinement; a layer calculus composes them. The thin-layer design made kernel variants cheap, but the authors report that invariant proofs are duplicated across layers whenever the invariant set changes. The concurrent version replaces shared state with a global log of events and treats the scheduler and other threads as an "environment context" constrained by rely conditions.

### Cited Findings

- [V] The most realistic sequential kernel "consists of 37 abstraction layers, took less than one person year to develop, and can boot a version of Linux as a guest". The breakdown: 9.5 pm for mCertiKOS plus 2 pm for linking and code extraction; the hypervisor variant then took 1.5 pm and two further variants half a person-month each. The hypervisor kernel is about 3,000 lines of C and assembly. — [Gu et al., POPL 2015, abstract and §7](https://www.cs.columbia.edu/~rgu/publications/popl15-gu.pdf)
- [V] Layers are authored by the kernel developers, "layer by layer", in two languages (ClightX and LAsm); when a step is too hard they "insert an intermediate layer ... to divide the difficult task into two steps". — [same, §6–7](https://www.cs.columbia.edu/~rgu/publications/popl15-gu.pdf)
- [V] A layer is a triple (L1, M, L2): module M, running on underlay interface L1, implements overlay interface L2 under a simulation relation R. Layers with matching interfaces compose vertically; layers with the same state compose horizontally. — [Gu et al., CACM 2019, Fig. 1 caption and §2](https://www.cs.columbia.edu/~rgu/publications/cacm19-gu.pdf)
- [V] The concurrent kernel mC2 is 6,500 lines of C and x86 assembly with fine-grained locking. "The entire proof effort for supporting concurrency took less than two person-years." Trusted specifications are small: 450 lines of Coq for the system-call interface (top layer) and 943 for the hardware model (bottom layer). — [CACM 2019, §1–2](https://www.cs.columbia.edu/~rgu/publications/cacm19-gu.pdf)
- [V] Outside the trusted base there are 5,249 lines of further specifications, about 40K lines of auxiliary definitions, lemmas and invariants, and 50K lines of proof scripts for the concurrency features. "At least one third of these auxiliary definitions and proof scripts are redundant and semi-automatically generated ... many invariant proofs get duplicated across the layers whenever there is a minor change to the entire set of invariants." — [Gu et al., OSDI 2016, §6](https://www.usenix.org/system/files/conference/osdi16/osdi16-gu.pdf)
- [V] Concurrency model: the multicore machine allows arbitrary interleaving at assembly-instruction level; a chain of machine models (hardware scheduler, local copies of shared memory with push/pull, CPU-local) is proved to refine one another so that code can be verified per CPU as if sequential. Push/pull on shared blocks makes a program get stuck on a data race, so safe programs are race-free. — [OSDI 2016, §3](https://www.usenix.org/system/files/conference/osdi16/osdi16-gu.pdf); [CACM 2019, §3](https://www.cs.columbia.edu/~rgu/publications/cacm19-gu.pdf)
- [V] In CCAL, shared state is a global log of events; "functions that reconstruct the current shared state from the log are called replay functions". Each layer interface carries rely and guarantee conditions, "simply expressed as invariants over the global log"; an environment context supplies the moves of the scheduler and other threads. Parallel composition requires each side's guarantee to imply the other's rely. — [Gu et al., PLDI 2018, §2–3](https://www.cs.columbia.edu/~rgu/publications/pldi18-gu.pdf)
- [V] Liveness is in scope: refinement is termination-sensitive, and lock layers prove starvation freedom by putting scheduler fairness into the rely condition. The starvation-freedom proof is about 1,500 lines for each of the ticket and MCS locks. — [PLDI 2018, §4, §6](https://www.cs.columbia.edu/~rgu/publications/pldi18-gu.pdf)
- [V] Cost per component (Table 2): ticket lock, 74 lines of source, 615 of specification, 1,080 of invariant proof, 1,173 of code proof, 2,296 of simulation proof; MCS lock, 287 lines of source against 1,569 / 2,299 / 1,899 / 3,049. The toolkit itself is about 52,500 lines of Coq. The authors state that the refinement proofs "still require quite a bit of manual proof". — [PLDI 2018, §6](https://www.cs.columbia.edu/~rgu/publications/pldi18-gu.pdf)
- [V] Payoff of the interface: ticket and MCS locks share one atomic specification, so "the lock implementations can be freely interchanged without affecting any proof in the higher-level modules". — [PLDI 2018, §6](https://www.cs.columbia.edu/~rgu/publications/pldi18-gu.pdf)
- [V] Verification found an overflow bug in the ticket lock loop that "passed all our tests". — [OSDI 2016, §6](https://www.usenix.org/system/files/conference/osdi16/osdi16-gu.pdf)

### Inferences

- The thin-layer argument has two sides. In favor: once the stack existed, a variant kernel cost 0.5–1.5 pm, and a lock implementation could be swapped under an unchanged interface. Against: a change to the invariant set ripples through every layer. For Mental, thin inserted levels are cheap only if invariants are attached to the level that owns the state and are not restated at each level above.
- The log-and-replay idea fits an interpreter well. If Mental records each level's events in one global trace and defines each upper-level state variable as a replay function over that trace, then the abstraction function is a fold over events, and "rely" conditions on other actors become predicates over the same trace.
- "Environment context" is a precise name for what Mental calls assumed behavior of people, couriers and third-party services: a strategy for everything outside the focus set, constrained by a rely condition. The focus set can grow until the environment is only the scheduler.
- A layer interface that stays fixed while its implementation changes is the unit of reuse. Mental's packs for known mechanisms (lock, pool, retry) play this role: the pack is the overlay interface, the recognized code is the module.

### Gaps

- The exact number of layers in the concurrent mC2 kernel was not found in the sources read.
- The "new layer calculus to minimize redundant definitions and proofs" announced in OSDI 2016 was not followed up (later work such as CompCertO and refinement-based game semantics was not searched).

## 3. IronFleet

### Takeaway

IronFleet fixes three levels (spec, protocol, implementation), all written by the developers in Dafny, and proves refinement per adjacent pair. The atomicity of a host's step is not proved by the tool: the implementation is made to satisfy a simple checked obligation (all receives before all sends in a step), and a paper argument shows that this obligation permits reduction. Liveness is proved at the protocol level by embedding TLA in Dafny.

### Cited Findings

- [V] The method: "we write a simple spec ... We then write an abstract distributed protocol layer and use TLA-style techniques to prove that it refines the spec layer. Then we write an imperative implementation layer to run on each host and prove that ... the implementation correctly refines the protocol layer." — [Hawblitzel et al., SOSP 2015, §3](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)
- [V] Each level is a state machine given as `Init` and `Next` predicates; the link is a relation between implementation or protocol state and spec state (`SpecRelation` in the toy lock example, Fig. 4). — [same, §3.1–3.3](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)
- [V] Atomicity: the proofs "assume that every implementation step performs an atomic protocol step". The real handler is not atomic, so a reduction argument reorders events. "A machine-checked proof that this obligation enables reduction is future work; instead, we sketch an informal argument here." What Dafny enforces is the reduction-enabling obligation: in any step, a host performs "all of its receives before all its sends", with at most one time-dependent operation between them. — [same, §3.6](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)
- [V] Liveness: TLA is embedded in Dafny; Lamport's WF1 fairness rule is proved as a lemma and used repeatedly; "always-enabled actions" (an action that does nothing when it has nothing to do) simplify fairness reasoning. Assumptions are explicit: each host's main loop runs infinitely often and the network is fair. — [same, §2.5, §4](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)
- [V] Size (Fig. 12): 1,400 lines of trusted spec, 5,114 of implementation, 39,253 of proof. The IronRSL liveness proof alone is 7,869 lines and takes 115 minutes to verify; protocol refinement is 3,379 lines. Implementation-layer ratio of proof annotation to executable code is 3.6 to 1. Total effort about 3.7 person-years for the method and two systems. A full serial build takes about six hours; with a cloud build the developer "rarely waits more than 6–8 minutes". — [same, §7.1](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)
- [V] Invariants at the implementation level are kept as one predicate over concrete state that every method assumes on entry and re-proves on return. — [same, §5](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)

### Inferences

- The obligation trick is the most reusable idea here for a tool without a prover. Instead of proving that a lower-level sequence is atomic, state a cheap structural condition that makes it so, and check that condition on the extracted code: all inputs before all outputs, at most one clock read, no shared-state access after the first send. Mental's footprints can check this statically per handler and report violations as "this handler cannot be shown atomic".
- The protocol level is where both the inductive invariant and the liveness argument live. Mental's services or components level plays the same role: it is the level at which whole-system invariants should be stated and where bounded "eventually" checks should run.
- Three fixed levels were enough for two real systems. The split was by kind of difficulty (what, how distributed, how implemented), not by a rule that thinner is better.

### Gaps

- No report on maintaining the IronFleet proofs after code change was found.
- Whether the reduction argument was later mechanized (for example in Armada's library, which mentions Cohen–Lamport reduction) was not confirmed.

## 4. IronSync, Verus and Atmosphere

### Takeaway

The successors to IronFleet moved to shared-memory concurrency and to Rust. They do not add levels. They keep a spec and the code, and split the proof by concern: an abstract transition system whose state is cut into "shards", and an ownership type system that lets each thread hold shards as ghost values and reason locally. Reported costs are a proof-to-code ratio of 3–8 and a few person-years per system. Liveness and termination are out of scope.

### Cited Findings

- [V] IronSync "scales to complex systems by splitting system-wide proofs into isolated concerns such that each can be substantially automated". The ownership type system proves data safety and thread-local correctness; a Localized Transition System (LTS) "connects the correctness of local actions to the correctness of the entire system". — [Hance et al., OSDI 2023, abstract](https://www.usenix.org/system/files/osdi23-hance.pdf)
- [V] An LTS transition names the shards it consumes and produces (pre-state and post-state shards); other shards are irrelevant, so the exchange is atomic by construction. Code holds shards as ghost owned values, often inside a lock, with an invariant tying the ghost shard to the concrete field. The developer chooses the shard granularity. — [same, §3](https://www.usenix.org/system/files/osdi23-hance.pdf)
- [V] Correctness is still stated as refinement of a trusted spec: "every execution" of the implementation must match the spec. IronSync "does not verify liveness, termination, or deadlock-freedom". — [same, §2, §4.1](https://www.usenix.org/system/files/osdi23-hance.pdf)
- [V] Cost: across the case studies (a NUMA node-replication library and a production page cache) the overall proof-to-code ratio is 7.5; inside implementation files it is about 4 to 1. — [same, §7, Fig. 15](https://www.usenix.org/system/files/osdi23-hance.pdf)
- [V] Verus carries the idea into Rust as VerusSync, where "transitions [are] the central object for reasoning" and Verus generates the ghost-token API from the declared transitions. Its case studies total 6.1K lines of implementation and 31K lines of proof. — [Lattuada et al., SOSP 2024, §1, §3.4, §4](https://www.andrew.cmu.edu/user/bparno/papers/verus-sys.pdf)
- [V] Atmosphere, a microkernel in Rust verified with Verus, proves one refinement from implementation to a high-level spec. It reports 6K lines of executable code, under 2.5 person-years in total (1.5 on verification), a proof-to-code ratio of 3.32 to 1, and full verification "in less than 20 seconds on a modern laptop". It relies on a big lock to avoid kernel concurrency and on a "flat" map of pointer permissions to keep invariants non-recursive. — [Chen et al., SOSP 2025, abstract, §1, §4.1](https://mars-research.github.io/doc/2025-sosp-atmo.pdf)

### Inferences

- The trend from 2015 to 2025 in SMT-based systems verification is toward two levels with a richer link, not toward tall stacks. This is evidence against reading CertiKOS as "more levels are always better". Levels are added when a step is too hard to check in one piece, and removed when the checker gets stronger.
- Sharding suggests a different axis of decomposition for Mental: by owner of state instead of by level. If each upper-level transition lists the entities whose state it consumes and produces, two upper transitions with disjoint shards commute by construction, and a race is a pair of transitions that claim the same shard without a common guard. This is the footprint analysis, stated at the upper level.
- Atmosphere's "flat" design shows that the shape of the state decides the cost of invariants. Models with flat maps keyed by id are cheaper to check than nested structures; Mental's compiled model can prefer flat stores.

### Gaps

- IronSync's and Verus's own descriptions of proof maintenance under code change were not found.
- Atmosphere's figures are self-reported in the paper; the paper says the code was released in November 2025 [S] and no independent assessment was found.

## 5. Verdi

### Takeaway

Verdi's levels are fault models, not levels of detail. The developer writes and proves a system against an idealized network, then applies a verified system transformer that produces a system for a harsher network together with a rule for translating the proved properties. The Raft proof shows the price of the invariants underneath: 50,000 lines of proof for 530 lines of code, with proof maintenance named as the main difficulty.

### Cited Findings

- [V] A system transformer "is a function whose input is an implementation of a system and whose output is a new system implementation"; a verified one includes a proof that the output satisfies analogous properties under the new fault model. There are transmission transformers (packet duplication and drops) and replication transformers (node crashes). — [Wilcox et al., PLDI 2015, §1, §2.4, §4](https://homes.cs.washington.edu/~ztatlock/pubs/verdi-wilcox-pldi15.pdf)
- [V] Each transformer T provides a `transfer` function with the guarantee: if property Φ holds of system S under semantics 1, then transfer(Φ) holds of T(S) under semantics 2. For the sequence-numbering transformer transfer is the identity on externally visible traces, proved by backward simulation with an `unwrap` function from transformed state to underlying state. — [same, §4.2](https://homes.cs.washington.edu/~ztatlock/pubs/verdi-wilcox-pldi15.pdf)
- [V] Evidence that transfer saves work: stating and proving a property of the key-value store with the primary-backup transformer applied took 19 additional lines. Verdi proves safety only; liveness is left to future work. — [same, §1, §8.1, Table 2](https://homes.cs.washington.edu/~ztatlock/pubs/verdi-wilcox-pldi15.pdf)
- [V] The complete Raft proof: "530 lines of code and 50,000 lines of proof", 90 system invariants found iteratively, developed over roughly 18 months. "The primary challenge we faced during the verification process was proof maintenance, since proving one invariant often required strengthening and updating other parts of our proof." — [Woos et al., CPP 2016, abstract, §2, §8](https://homes.cs.washington.edu/~mernst/pubs/raft-proof-cpp2016.pdf)
- [V] Their remedies are software-engineering ones: separate theorem statements (an interface) from proofs (an implementation) so that changing one proof does not force rechecking its dependents, and factor recurring invariant-strengthening patterns into reusable principles. Before these were applied the team "spent the majority of our time reworking proofs in response to changes". — [same, §7.2, §8](https://homes.cs.washington.edu/~mernst/pubs/raft-proof-cpp2016.pdf)

### Inferences

- A fault model is a level axis the draft report does not list. In Mental terms: the same service modeled over "reliable calls", then over "calls may be retried", then over "process may crash". A pack for a retry policy, an idempotency key or a replicated store is a transformer, and it should come with a transfer rule saying which upper-level properties survive (for example "exactly-once effects survive retries only if the handler is idempotent").
- Verdi's `unwrap` is an abstraction function that strips the fault-tolerance wrapper. Mental can use the same shape to hide retries and sequence numbers at the services level.
- Invariants depend on each other. The interface/implementation split for proofs maps to Mental as: store each invariant's statement separately from the evidence for it, and record which invariants were assumed while checking which, so that a change invalidates only dependents.

### Gaps

- Later Verdi-family work (for example on liveness or on proof repair tools) was not searched.

## 6. CIVL and layered concurrent programs

### Takeaway

CIVL writes all levels as one program. Variables and procedures carry layer numbers; at its "disappearing layer" a procedure is replaced by an atomic action. Each replacement is justified by mover types and by a refinement check that is very close to what Mental proposes: in every execution of the procedure, exactly one yield-to-yield fragment performs the upper action and all other fragments leave upper-layer state unchanged. Invariants are named, parameterized yield invariants checked for interference freedom. Checks are automatic (Boogie and an SMT solver) and take seconds.

### Cited Findings

- [V] A layered concurrent program "specifies a sequence of connected concurrent programs, from most concrete to most abstract, such that common parts of different programs are written exactly once". Earlier versions required one file per layer, "leading to code duplication". — [Kragl and Qadeer, CAV 2018, abstract and §1](https://my.eng.utah.edu/~cs7931/papers/paper-kragl-2018-LayeredConcurrentPrograms.pdf)
- [V] The user writes the layers: "Variables and atomic actions have a layer range"; a variable is introduced at one layer and hidden at another; a yielding procedure's layer annotation is "its disappearing layer", above which it "is replaced with its refined atomic action". — [Kragl and Qadeer, FMCAD 2021, §III](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] Atomicity by movers: "every yield-to-yield fragment is checked to be a sequence of right movers, followed by at most one non-mover, followed by a sequence of left movers". Acquiring a lock is a right mover, releasing it a left mover, protected accesses are both. — [same, §I, §III](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] Refinement check: "Civl checks that in every execution of the procedure, the effect of the refined action happens in exactly one yield-to-yield fragment and that other yield-to-yield fragments leave the layer-(n + 1) state unchanged." — [same, §III](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] The link is hidden from the user: "the simulation relation justifying the refinement step in Civl is computed but never revealed to the programmer who focuses only on the program layers and the connecting invariants". Refinement is treated "as a tactic to simplify verification of safety properties". — [same, §II](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] Yield invariants are "named and parameterized invariants that can be reused and suitably instantiated across multiple program locations where interference may happen", checked to be interference-free across all yield-to-yield fragments. Permissions (linear variables) make those checks local. — [same, §II–III](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] Scope: "Civl supports the verification of arbitrary safety properties"; correctness means the lowest-layer program is free of assertion failures. Liveness is not covered. — [same, §II](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] Case study: a concurrent garbage collector proved "via a sequence of 6 program transformations connecting 7 program layers", from individual CPU operations up to atomic allocate/read/write. The proof dates from 2015, "has been maintained and improved" through a rewrite of the tool, and "is 2031 LOC and verifies in 25s". Layers also let several developers work in parallel on agreed interfaces. — [same, §V](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)
- [V] The project remains active: its publication list includes "Reduction for structured concurrent programs" (ESOP 2026) and inductive sequentialization for asynchronous programs (PLDI 2020). — [Civl site](https://civl-verifier.github.io/)

### Inferences

- CIVL's refinement check is the mechanical definition Mental needs for "an upper step maps to lower steps": split the lower run at yield points; exactly one fragment may change the projected upper state, by exactly the upper action; every other fragment must stutter. The draft report's per-run projection check should adopt this wording, including "exactly one".
- A race then has a precise form: a lower-level procedure whose fragments cannot be classified as right movers, one non-mover, left movers, so that another thread's step between two fragments is visible above. Mental's footprint-based commutation is the unproved, per-run version of mover types.
- "Written exactly once" matters for LLM-generated levels too. If the LLM writes each level as a full separate model, levels drift apart. If levels are annotations on one model (this variable exists from level 2 to 4; this procedure disappears at level 3), then only differences are authored and consistency is structural.
- Yield invariants show how to keep invariants small: name them, parameterize them, and instantiate them at each point where interference can occur, instead of one global inductive invariant.

### Gaps

- The CAV 2018 paper was only skimmed for definitions; the formal account of how checker programs are extracted was not read in detail.
- No person-effort figures for CIVL proofs were found beyond the case-study descriptions.

## 7. Armada

### Takeaway

Armada links a chain of program levels with short recipes, each naming one of eight proof strategies; the tool generates the Dafny refinement proof for the pair. The developer writes the levels and the recipes, and for the hard pairs also the invariants and rely-guarantee predicates. Generated proofs are large (tens of thousands of lines for programs under 100 lines), and Armada proves safety only.

### Cited Findings

- [V] Developers "define a series of levels, each of which abstracts the [one below]", and for each adjacent pair supply "a short proof recipe" naming the strategy; Armada "analyzes both levels" and generates a lemma that the refinement relation R holds. Adjacent levels "should be similar enough" for one strategy to apply. — [Lorch et al., PLDI 2020, §1–2](https://6826.csail.mit.edu/2020/papers/armada.pdf)
- [V] "A strategy is a proof generator designed for a particular type of correspondence between a low-level and a high-level program." The eight strategies: reduction, rely-guarantee reasoning, TSO elimination, weakening, non-deterministic weakening, combining, variable introduction, variable hiding. — [same, §4, §4.2.1–4.2.8](https://6826.csail.mit.edu/2020/papers/armada.pdf)
- [V] The reduction strategy uses the classic pattern: right movers R1…Rn, one non-mover N, left movers L1…Lm are replaced by one atomic action above. — [same, §4.2.1](https://6826.csail.mit.edu/2020/papers/armada.pdf)
- [V] Extensibility is sound because generated lemmas and library lemmas are all checked by Dafny, so a wrong strategy implementation cannot produce an accepted proof. — [same, §4](https://6826.csail.mit.edu/2020/papers/armada.pdf)
- [V] Effort and size: for one 57-line example an author took about 3 days to write the levels; the first level uses 5 lines of recipe and yields 3,649 generated proof lines; the next uses 102 lines of recipe "mostly for invariants and rely-guarantee predicates" plus 114 lines of customization and yields 46,404 generated lines. For the 70-line lock-free queue: about 6 person-days, eight transformations, one of which needs a 92-line recipe and 528 lines of invariant and lemmas; 24,540 generated lines in total. The MCS lock uses six levels with 4-line recipes for the easy ones, against "3.2K LOC" in CertiKOS for the same lock. — [same, §6](https://6826.csail.mit.edu/2020/papers/armada.pdf)
- [V] Limitation: because the proof covers finite behaviors, "it can prove safety but not liveness properties". — [same, §8](https://6826.csail.mit.edu/2020/papers/armada.pdf)

### Inferences

- The draft's recommendation of a closed set of level patterns is supported, with one correction to expectations: the pattern name is enough for the easy pairs (hiding or introducing a variable, weakening a statement), while the pairs that carry the real argument (reduction, rely-guarantee) need invariants and ownership predicates supplied with the recipe. For Mental, a level transition record needs a `pattern` field and a pattern-specific `parameters` field (the ownership predicate, the invariant), not only a name.
- Variable introduction followed by variable hiding is how Armada changes representation: introduce the abstract queue, show the two agree, hide the concrete one. Mental's inserted levels can follow the same three-step shape, which keeps each step checkable.
- Armada's developers reported that the first recipe drafts "had many mistakes" and that verifier output guided the fixes. That is the loop an LLM would run: propose pattern and parameters, read the checker's counterexample, repair.

### Gaps

- No follow-up on Armada after 2020 (maintenance, larger case studies, liveness support) was found in this session.

## 8. Perennial and Grove

### Takeaway

Perennial and Grove verify Go systems against a single specification using concurrent separation logic in Coq; they do not stack levels. Their contribution to the level question is the treatment of crashes: a crash followed by recovery, possibly crashing again during recovery, must appear at the spec level as one atomic crash step. Both prove safety only.

### Cited Findings

- [V] Concurrent recovery refinement: every sequence of code transitions must correspond to some interleaving of spec transitions with the same external I/O, and "whenever the implementation crashes (followed by recovery and perhaps some number of crashes during recovery), the whole sequence should simulate a single atomic crash step in the specification". — [Chajed et al., SOSP 2019, §3](https://pdos.csail.mit.edu/papers/perennial:sosp19.pdf)
- [V] Crash invariants state what holds on durable state at every instant, and must imply the precondition of recovery; recovery leases and "recovery helping" let recovery finish an operation another thread started. — [same, §1, §5](https://pdos.csail.mit.edu/papers/perennial:sosp19.pdf)
- [V] "Perennial does not currently support composing layers of abstraction"; proofs "only cover safety properties and not liveness". — [same, §1](https://pdos.csail.mit.edu/papers/perennial:sosp19.pdf)
- [V] Effort: the framework and the Go-to-Coq translator "took two people 5 months to develop", and the Mailboat mail server "took one person 2 weeks to verify". — [same, §9.4](https://pdos.csail.mit.edu/papers/perennial:sosp19.pdf)
- [V] Grove extends the approach to distributed systems with leases (time-bounded invariants), reconfiguration and crash recovery; components are verified one by one against specifications used by the components above. "The proof-to-code ratio is about 12×." "Grove is only able to verify safety" and cannot verify liveness. — [Sharma et al., SOSP 2023, §1](https://pdos.csail.mit.edu/papers/grove:sosp23.pdf)

### Inferences

- Crash is a step the upper level must name. For every durable entity in a Mental model, the upper level should have a `crash` transition stating what may be lost, and the per-run refinement check should treat "lower-level crash, then recovery to quiescence" as one fragment that must project onto that transition.
- A crash invariant is an invariant checked at every lower-level step, not only at upper-level step boundaries. This is a different check point from ordinary invariants and catches "torn" intermediate states.
- Leases need a notion of time in the invariant ("this holds until T"). Mental's business level already has durations; lease-like guards can be modeled as predicates that expire.

### Gaps

- Grove's specification style and its component-by-component structure were read only in the introduction; the logic itself (time-bounded invariants) was not studied.
- Later Perennial work that adds layering (for example the journaling file-system proofs) was not searched.

## 9. Anvil

### Takeaway

Anvil verifies Kubernetes controllers in Verus with three artifacts: the implementation, a controller model with the same structure, and an environment model. Conformance is one implementation step to exactly one model step. The correctness property, eventually stable reconciliation, is a liveness property proved on the model with an embedded TLA and explicit fairness assumptions. It shows that liveness can be handled across levels when the upper level is a state machine and the lower pair is a simple conformance check.

### Cited Findings

- [V] The developer "first provides a controller model (an abstract state machine) and then proves two theorems": the implementation conforms to the model, and the model together with "a model of the environment (e.g., the network, other controllers, faults)" satisfies the specification. — [Sun et al., OSDI 2024, §4](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Conformance: the model has "the same structure as the implementation state machine", and developers prove "that each step in the implementation corresponds to exactly one step in the model" by ordinary pre/postcondition reasoning. — [same, §4](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Eventually stable reconciliation (ESR) is "a liveness property" written as a TLA formula: if the desired state stops changing, the cluster eventually matches it and stays that way. Verus has no temporal logic, so Anvil provides a TLA embedding (85 lines) and reusable liveness lemmas. — [same, §3, §4.2, §7.1](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Size (Table 1): for three controllers, 3,940 lines of executable code and 23,975 lines of proof; the liveness proofs dominate (for ZooKeeper, 7,245 lines of liveness proof against 172 of conformance). Proof-to-code ratio 4.5 to 7.4. Each controller verifies in under 3 minutes. — [same, §7.1](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Effort: about two person-months for ESR on the first controller, about two person-weeks each for the next two "using the same proof strategy and similar invariants"; around 2.5 person-months per controller to implement and verify. — [same, §6, §7.1](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Maintenance data: 28 features were added to one verified controller; "on average, implementing a feature took less than a day and 47 lines of changes, including 19 lines in the proof"; the largest needed 403 lines, 211 of them proof. — [same, §7.1](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)
- [V] Trusted parts are sizeable: the framework has 7,817 lines of trusted code, including the environment model (1,846 lines) and the Kubernetes integration. The authors list the quality of the trusted environment model as a limitation. — [same, §7.1, §8](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)

### Inferences

- Anvil is a precedent for the split Mental already has: deterministic conformance between code and a same-shaped model, with the hard property examined on the model. Mental's statements-to-mechanisms pair is the conformance step; scenarios and exploration run above it.
- ESR is a ready-made property template for business and system levels: "if the inputs stop changing, the system reaches the desired state and stays there". In Mental it can only be checked in bounded form (reached within N steps, stable for M further steps under a fair scheduler), and the fairness assumption must be printed with the result, as Anvil does with its fairness premises.
- The environment model is trusted and large. Mental's packs for databases, queues and HTTP are its environment model; their fidelity limits every finding, and the report should say so.
- The maintenance numbers (19 proof lines per feature) show that when the upper property and proof strategy are stable, incremental change is cheap. The first proof is the expensive one.

### Gaps

- The proof strategy for ESR (§5 of the paper) was not read in detail.
- Follow-up work on Anvil after 2024 was not searched.

## 10. AWS practice: ShardStore, Cedar, the Dafny authorization engine, s2n

### Takeaway

AWS's published practice is two levels checked per run: a small executable reference model in the same repository, and conformance by property-based or differential testing, with proofs applied only where cheap (on the model, or automatically in continuous integration). The reported numbers are concrete: ShardStore's reference models are about 1% of the implementation and the checking harness about 12% of the code base; the authorization engine was compared against 10^15 production samples before launch. The published limits are just as concrete: checks are unsound, coverage must be measured, and concurrent crash behavior is not checked at all.

### Cited Findings

- [V] ShardStore: "Reference models are written in the same language as the implementation and embedded in its code base, allowing them to be written and maintained by the engineering team". They are "small executable specifications (1% of the implementation code)"; the model of a log-structured merge tree "is a hash map". Models double as mocks in unit tests, which forces engineers to update them. — [Bornholt et al., SOSP 2021, §1, §3.2](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)
- [V] Conformance is framed as refinement and split three ways: sequential crash-free runs are checked for equivalence by property-based testing on random operation sequences; sequential crashing runs use a model extended to say which data may be lost; concurrent crash-free runs use separate reference models and a linearizability check by stateless model checking. "We do not currently check properties of concurrent crashing executions because we have not found an effective automated approach." — [same, §1, §3](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)
- [V] Concurrency tools: Loom "soundly check[s] all interleavings of small, correctness-critical code"; for end-to-end tests with tens of thousands to over a million atomic steps the team wrote Shuttle, which samples interleavings randomly. — [same, §6](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)
- [V] Cost and results: implementation over 40,000 lines of Rust; property definitions and harnesses are 12% of the code base; the checks "prevented 16 issues from reaching production"; 18% of model and harness code was written by engineers who are not formal-methods experts. The checks are "pay-as-you-go": run longer to find more. — [same, §1](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)
- [V] Limits: the checks "can miss bugs". One bug was missed because the cache in tests was configured so large that the cache-miss path was never reached; this led to coverage metrics for the tests. Modeling was done per component once APIs were stable, to avoid churn; a single model of only the public interface was judged the wrong choice. — [same, §8.3–8.4](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)
- [V] Cedar (authorization language): an executable Lean model of each component (1,673 lines) with 5,714 lines of proofs about the model, against 15,693 lines of production Rust. Differential random testing runs "millions of inputs" through model and production code and compares outputs. Proofs found 4 bugs; differential and property-based testing found 21 more. — [Disselkoen et al., "How We Built Cedar", 2024, §1, §3, Table 1](https://arxiv.org/pdf/2407.01688)
- [V] AWS authorization engine: a new engine was written in Dafny over four years, proved against a Dafny specification, compiled to Java, and required to be behaviorally equivalent to the legacy Java engine. Spec and proofs are 11,570 lines against 2,751 of implementation, a 5 to 1 ratio. Equivalence to the legacy engine was established by testing, not proof: shadow testing through 2023 with 8 iterations, the last of which "agreed with AuthV1 on 10^15 production authorization samples". Shadow testing exposed cases where the specification "had not modeled all the subtleties" of the legacy engine. — [Chakarov et al., ICSE 2025, §I, §III](https://cdn.amazon.science/bb/40/22ac44f84f6d8eb625ac9666a00f/formally-verified-cloud-scale-authorization.pdf)
- [S] s2n (TLS library): the HMAC implementation in C is proved equivalent to a Cryptol specification with the SAW tool, and the proof runs in continuous integration on every code change, because new code does not inherit old proofs. — [Chudnov et al., CAV 2018 (Springer)](https://link.springer.com/chapter/10.1007/978-3-319-96142-2_26)
- [S] AWS's own 2025 overview lists TLA+, the P language, property-based testing and deterministic simulation among its correctness practices. — [Brooker and Desai, CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services)

### Inferences

- ShardStore's decomposition is a checklist for Mental's conformance modes: (1) sequential runs, exact agreement; (2) runs with crashes, agreement up to a stated loss rule; (3) concurrent runs, linearizability against the sequential model. Each mode has its own upper-level statement of what is allowed. Mental's bounded exhaustive mode corresponds to Loom (sound for small scopes) and its random scenario mode to Shuttle.
- The reference model is far smaller than the draft assumed: about 1% of the code. An LLM-proposed upper level that is a tenth of the code is still much more detailed than what S3 found sufficient. Aim upper levels at "a hash map" simplicity.
- The missed-bug story generalizes: a per-run check is only as good as the paths the runs reach. Mental should report, per level pair, which lower-level branches and which upper-level transitions the checked runs covered.
- The authorization engine shows the same triangle Mental faces with LLM-written upper levels: the specification itself can be wrong about the existing code, and only differential runs against the real system reveal it. Proof established code-against-spec; testing established spec-against-reality.
- "Model doubles as mock" is a maintenance mechanism: the model stays current because something else depends on it. Mental's upper-level behaviors could double as test stubs or simulation stand-ins for services.

### Gaps

- The s2n paper download failed; its claims are from search summaries only.
- The CACM 2025 article was not read in full.
- No public figure for the ongoing maintenance cost of ShardStore's models after 2021 was found.

## 11. Tools and LLMs that write invariants, levels and proofs (2023–2026)

### Takeaway

Two lines of work reduce the human cost. Kondo derives the asynchronous level of a protocol and its routine invariants from a synchronous level that a person proves. LLM agents now complete most function-level and many system-level proof obligations when a verifier judges each attempt: about 81% of 849 tasks from real Verus systems for the best 2025–2026 agent, and from 27% to above 50% of sampled seL4 lemmas over two years. In every case found, a person wrote the specification and the levels; the LLM filled in proofs or invariants. Obligations stated over whole state machines remain the weakest area.

### Cited Findings

- [V] Kondo splits protocol invariants into "Regular Invariants, which have one of a few simple low-level structures, and Protocol Invariants, which capture the higher-level host relationships". The developer proves "a synchronous version of the protocol" in which "sends and receives are replaced with atomic variable assignments"; the tool "automatically generates the asynchronous protocol description, Regular Invariants, and proofs that the Regular Invariants are inductive", producing a draft proof that "may then require a small amount of user effort to complete". Regular invariants include message invariants and monotonicity invariants. — [Zhang et al., OSDI 2024, abstract and §1](https://www.usenix.org/system/files/osdi24-zhang-nuda.pdf)
- [V] AutoVerus (LLM agents for Verus proofs, three phases: generate, refine, debug from verifier errors) proves 137 of 150 function-level tasks; direct prompting of GPT-4o proves 67 of 150. The tasks are mostly loop invariants and assertions for small programs. — [Yang et al., OOPSLA 2025, abstract and §1](https://arxiv.org/pdf/2409.13082)
- [V] VeruSAGE builds 849 proof tasks from eight verified systems (including Anvil, IronKV, a page table, Atmosphere). Earlier agents managed "20% or less" on such system tasks. The best model and agent setting reaches 81% overall, with 7.2 minutes per task on average: 100% on Anvil's lemma library but 37% on Anvil controller tasks, where the specification averages 2,037 lines and "often ... includes an entire system state machine". Functions using Verus's state-machine macros were excluded from the benchmark. A "cheating checker" rejects proofs that use `assume` or `admit`. The agents also proved 33 tasks that human experts had left unfinished in Atmosphere. — [Yang et al., arXiv 2512.18436, §1, §3, Table 5](https://arxiv.org/pdf/2512.18436)
- [S] On seL4's Isabelle proofs: Selene (2024) reports GPT-4 proving about 27% of sampled lemmas; AutoReal (February 2026) reports 51.67% on 660 theorems with a fine-tuned 7B model; a neuro-symbolic search system (March 2026) reports "up to 77.6%" on a different seL4 benchmark. The theorem sets differ, so the numbers are not directly comparable. — [Selene](https://arxiv.org/abs/2401.07663); [AutoReal](https://arxiv.org/abs/2602.08384); [He et al.](https://arxiv.org/abs/2603.19715)
- [V] IC3Syn combines an IC3-style symbolic loop over TLA+ states with an LLM that proposes blocking invariants. It "discovers candidate invariants for all 29 protocols" tested, including an industrial Raft-based reconfiguration protocol that four compared tools did not solve; the invariants found on finite instances "are shown in TLAPS to be inductive for the full unbounded protocol". — [arXiv 2605.24619, abstract](https://arxiv.org/pdf/2605.24619)
- [S] A FMCAD 2025 study on 8 distributed systems found a strong LLM used alone to be less effective at generating invariants than a symbolic synthesis method, and more useful when it suggests predicates from counterexamples. — [Xia et al., FMCAD 2025](https://www.isi.edu/results/publications/50873/guiding-likely-invariant-synthesis-on-distributed-systems-with-large-language-models)

### Inferences

- The pattern that works is "LLM proposes, a mechanical checker disposes, errors feed back". Mental has the same shape with the interpreter as checker. The published success rates are for proof text under a fixed human specification; they say nothing yet about LLM-proposed specifications or levels, which is Mental's case. The draft's claim that Mental is alone there stands as far as this sweep could see.
- The cheating checker is a necessary part of the loop. The Mental analogue of `assume` is an abstraction function that maps everything to a constant, an upper level so non-deterministic that it allows everything, or an invariant weakened to true. The checker must reject vacuous levels: require that the upper level forbids something (some mutated lower run must fail the check).
- Kondo shows a generator writing a level. Its direction is the reverse of Mental's (upper to lower), but its taxonomy is directly usable: message invariants ("every message in transit was sent by a step that justified it") and monotonicity invariants ("this counter never decreases") can be derived mechanically from the model's wires and stores, leaving the LLM to propose only the protocol-level ones.
- Kondo's synchronous version is a concrete definition of a level above message passing: replace send/receive pairs by atomic assignments. That is Mental's services-to-system summarization, and the asynchronous-versus-synchronous difference is exactly where its races appear.
- The VeruSAGE gap (37% on state-machine tasks) and the FMCAD 2025 result suggest that whole-system invariants should come from a loop with counterexamples from bounded exploration, not from one LLM call.

### Gaps

- Only abstracts were read for Selene, AutoReal and the neuro-symbolic system.
- No paper was found in which an LLM writes the intermediate levels or refinement mappings of a layered proof (CertiKOS-style layer interfaces, IronFleet-style protocol levels, Armada recipes). The search covered Verus, Dafny, Isabelle and TLA+ terms; Coq-specific and Lean-specific work was not searched separately.
- No study of LLM-assisted proof maintenance after code change in these systems was found.

## 12. Corrections and additions to the draft report

Each item names the draft claim and what the sources say.

1. **seL4, "security properties (integrity, confidentiality) transfer to the C code without being re-proved".** True for integrity and authority confinement (7.4 pm on the abstract spec). False for confidentiality as stated: it "is not preserved by refinement without additional conditions", and required making the abstract spec deterministic and repairing the refinement proofs (about 18.5 pm) before 20.4 pm of confidentiality proofs ([TOCS 2014](https://trustworthy.systems/publications/nicta_full_text/7371.pdf)). The rule is "Hoare-style properties carry down".
2. **seL4, "most of the proof was ... the invariants".** The 80/20 split is reported for the first refinement only. The second refinement (to C) added no invariants and cost 3 py against 8 py. The executable spec is not written directly: it is generated from a Haskell prototype; the C is hand-written ([CACM 2010](https://www.read.seas.harvard.edu/~kohler/class/cs260r-s17/klein10sel4.pdf)).
3. **CertiKOS, "many small refinements ... made each proof tractable".** Confirmed for 37 layers and under one person-year, about 3,000 lines of C and assembly ([POPL 2015](https://www.cs.columbia.edu/~rgu/publications/popl15-gu.pdf)). Add the cost: at least a third of the concurrent kernel's auxiliary definitions and proofs are redundant, because "many invariant proofs get duplicated across the layers" on any change ([OSDI 2016](https://www.usenix.org/system/files/conference/osdi16/osdi16-gu.pdf)). The concurrent kernel is a different artifact: 6,500 lines, under 2 py for concurrency.
4. **IronFleet, "the implementation-to-protocol refinement uses a reduction argument".** The reduction argument is on paper only. Dafny checks a "reduction-enabling obligation" (receives before sends); a machine-checked proof that this suffices "is future work" ([SOSP 2015](https://6826.csail.mit.edu/2019/papers/ironfleet.pdf)).
5. **"IronFleet needed hand proofs" for liveness.** The liveness proofs are hand-written but machine-checked in Dafny through an embedded TLA (7,869 lines for IronRSL). The unchecked part of IronFleet is the reduction step, not liveness.
6. **"Liveness across levels is unaddressed."** Too strong. CertiKOS proves termination-sensitive refinement and starvation freedom under a fair-scheduler rely condition ([PLDI 2018](https://www.cs.columbia.edu/~rgu/publications/pldi18-gu.pdf)); IronFleet proves liveness at the protocol level; Anvil proves a liveness property on the model and links code by one-step conformance ([OSDI 2024](https://www.usenix.org/system/files/osdi24-sun-xudong.pdf)). What holds: CIVL, Armada, Verdi, Perennial, Grove and IronSync cover safety only, and every liveness result is stated at an upper level under explicit fairness assumptions.
7. **"Many thin levels beat few thick ones" (CertiKOS row and recommendation 7).** Not a general finding. IronFleet uses three levels; IronSync, Verus, Atmosphere, Perennial and Grove use a spec and the code, and decompose by ownership or by component instead. Perennial states that it "does not currently support composing layers of abstraction". Levels are inserted when one check is too hard, as CertiKOS and Armada both say.
8. **"Always per pair of adjacent levels, never per level alone."** Holds for seL4, CertiKOS, IronFleet, CIVL, Armada, Anvil. Add that invariants are per level and often dominate the cost (seL4: 80% of the first refinement; Armada: the long recipes are the ones carrying invariants; Verdi's Raft: 90 invariants).
9. **Armada, "a short recipe".** Short for hiding, introducing and weakening steps (4–7 lines). The reduction and rely-guarantee steps needed about 100 lines of recipe plus separate invariant files of up to 528 lines. Generated proofs are large: up to 46,404 lines for one level of a 57-line program ([PLDI 2020](https://6826.csail.mit.edu/2020/papers/armada.pdf)). The eight strategy names in the draft are correct; the paper's names for the last two are "variable introduction" and "variable hiding".
10. **ShardStore, reference models "a few times smaller than the implementation".** They are about 1% of the implementation code; the harnesses and property definitions are 12% of the code base. Add three facts: concurrency is checked against separate reference models for linearizability; concurrent crashing executions are not checked; one bug was missed because tests never reached a code path ([SOSP 2021](https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf)).
11. **CIVL.** The description is correct. Add the exact refinement condition ("the effect of the refined action happens in exactly one yield-to-yield fragment" and others leave upper state unchanged) and that the simulation relation is "computed but never revealed" to the user ([FMCAD 2021](https://repositum.tuwien.at/bitstream/20.500.12708/18644/1/Kragl-2021-The%20Civl%20Verifier-vor.pdf)).
12. **Missing systems.** Verdi (levels as fault models with a property-transfer function), Perennial/Grove (crash and recovery as one upper step), Anvil (liveness on a model plus one-step conformance), Kondo (a generator that writes the asynchronous level from the synchronous one), Cedar and the AWS authorization engine (differential testing of model against production code).
13. **"Every precedent's levels are authored."** Still true for LLMs. One exception by a tool: Kondo generates the asynchronous protocol level and its regular invariants ([OSDI 2024](https://www.usenix.org/system/files/osdi24-zhang-nuda.pdf)); seL4's executable spec is generated by translation from Haskell.

## Recommendations for Mental

1. **Adopt CIVL's refinement condition for the per-run check.** Cut the lower run into fragments at points where other actors may run. Exactly one fragment may change the projected upper state, and it must do so by the claimed upper step; all other fragments must leave every upper variable unchanged. Report a violation as a race when an interleaved fragment of another actor is what makes it fail.
2. **Check a reduction-enabling obligation statically.** Before any exploration, classify each handler or mechanism step: inputs before outputs, at most one clock read, shared-state accesses bracketed by the same guard (right movers, one non-mover, left movers). Handlers that pass can be treated as atomic above; handlers that fail are the candidates for race findings. This is IronFleet's and CIVL's shared idea, without a prover.
3. **Say which properties are inherited.** An upper-level invariant of the form "every step preserves P" is inherited by a lower level that passes the refinement check. Properties that depend on what the upper level's non-determinism hides (information flow, fairness, timing) are not. Record the shape with each invariant.
4. **Put data-structure invariants at the middle levels and attach each to the level that owns the state.** Do not restate them above. Track which invariants were assumed when checking which, so a code change invalidates only dependents (seL4's cost-of-change evidence; Verdi's interface/proof split; CertiKOS's duplication warning).
5. **Add fault model as a level axis.** Model a service over an ideal network first, then apply pack-provided transformers (retry, deduplication, replication, crash with recovery), each with a stated transfer rule for properties and an `unwrap` abstraction. Add a named `crash` transition at the upper level for every durable entity and check recovery refinement (Verdi, Perennial).
6. **Use three conformance modes with separate upper-level statements:** sequential exact agreement, crash runs up to a loss rule, concurrent runs as linearizability against the sequential upper model. Use bounded exhaustive exploration for small critical mechanisms and random scheduling for end-to-end scenarios (ShardStore's Loom and Shuttle split).
7. **Keep upper levels very small and write them once.** Target a reference-model size near 1% of the code. Represent levels as layer ranges on one model (this variable exists from level i to j; this procedure becomes an atomic step at level k) so that only differences are authored (CIVL, ShardStore).
8. **Make the level-transition record carry a pattern and its parameters.** Names from Armada cover most cases: reduction, rely-guarantee, weakening, non-deterministic weakening, combining, variable introduction, variable hiding, plus crash-recovery and fault-model transformer. Hard patterns need parameters (ownership predicate, invariant, rely condition); expect the LLM to supply them and the checker to return counterexamples.
9. **Guard against vacuous levels.** Reject an upper level that forbids nothing: mutate lower runs (reorder two conflicting steps, drop a write) and require that at least some mutants fail the refinement check. Report coverage of lower branches and upper transitions per level pair. This is the analogue of VeruSAGE's cheating checker and ShardStore's coverage metrics.
10. **Derive routine invariants mechanically; ask the LLM only for protocol-level ones.** Message invariants and monotonicity invariants follow from wires and stores (Kondo's regular invariants). Propose whole-system invariants in a loop with counterexamples from bounded exploration, since single-shot LLM invariants are the weak point in the published results.
11. **Treat liveness as a bounded, assumption-labelled check at an upper level.** Use Anvil's template (if inputs stop changing, the desired state is reached and stays) and print the fairness and environment assumptions with each result.
12. **Describe the environment explicitly.** Everything outside the focus set (people, third parties, the scheduler) is an environment context with a rely condition over the event trace (CCAL). Define upper state as replay functions over that trace where possible.

## Open questions

- Is the per-run "exactly one fragment" check too strict for upper steps that legitimately span several lower-level commits, such as a saga? CIVL would model these with pending asynchronous calls and inductive sequentialization; whether Mental needs an equivalent pattern is undecided.
- How should Mental pick yield points when the code has no explicit ones? Candidates are every access to shared state, every I/O call, or only points where the footprint analysis finds a possible conflict.
- Can an LLM produce Armada-style parameters (ownership predicates, rely conditions) reliably? No published success rate exists for this task; a small experiment on the worked examples would give the first data.
- What is the right size and number of levels for an ordinary web service? The verified-systems evidence ranges from 2 levels (Verus projects) to 37 (CertiKOS), and none of it comes from HTTP, queue and database systems.
- How much of seL4's and Verdi's proof-maintenance pain carries over to a tool that re-checks by running? Re-running is cheap, but re-proposing a level after an invariant breaks may not be; the cost model for "LLM repairs a level" is unknown.
- Which fairness assumptions are realistic for bounded liveness checks at the business level, where actors are people with durations?
- Does the environment model (packs for databases, queues, HTTP) need its own conformance check against the real components, as the AWS authorization engine's shadow testing suggests?
