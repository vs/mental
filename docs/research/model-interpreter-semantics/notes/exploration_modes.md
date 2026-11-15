# Exploration modes for a model interpreter: concrete, symbolic and exhaustive runs, and the traces they produce

Scope: techniques for running a model of a software system (entities, ports, wires, state variables, predicates, guarded transitions) in three modes (concrete, symbolic, exhaustive), how each mode scales, and how each generates, minimizes, explains and replays traces. State as of November 2026. Classic results (1990s to 2010s) are marked as such and still hold unless noted. Project context: `docs/brainstorm/checkpoint-01.md` ("Framing: a model interpreter") and `docs/research/static-analysis-llm-system-models/report.md`.

Source-quality note: links to classic papers point to DOI or publisher pages I am confident of. Facts about 2024 to 2026 tools were checked against pages fetched for this note: TigerBeetle 2026 blog, NSDI '26 Fest abstract, P docs, Coyote docs, Antithesis blog, TLC docs and the 2026 TLA+ mailing list. Claims taken only from search-result summaries are flagged.

## 1. Explicit-state model checking (SPIN, TLC, P, Murphi): reductions, bounding, swarm, practical sizes

### Takeaway
Explicit-state checkers enumerate reachable states breadth- or depth-first. They dedupe states by storing 64-bit fingerprints or bitstate hashes instead of full states, and shrink the space with symmetry and partial-order reduction. For our interpreter the standard design works: BFS for shortest counterexamples, fingerprint dedup, symmetry over identical replicas and requests, POR over independent transitions, and small bounds (2 to 3 concurrent requests, a few replicas). Tools that target industrial models (P, Coyote) default to randomized, bounded schedule sampling rather than full enumeration, because full enumeration does not scale to real designs.

### Cited Findings
- **TLC dedups by fingerprint, not by full state.** It stores only 64-bit hashes, so two distinct states can collide and some reachable states may be missed. It prints collision-probability estimates at the end of a run, and a fingerprint seed index selects one of 64 hash functions so a run can be repeated with another one — [TLC options page](https://nightly.tlapl.us/doc/model/tlc-options-page.html).
- **TLC symmetry reduction is implemented in fingerprinting.** TLC permutes a state and keeps the smallest fingerprint, but computes successors from the actual state, so every state it examines is reachable — [TLA+ codebase architecture](https://docs.tlapl.us/codebase:architecture); [Lamport on the TLA+ list](https://discuss.tlapl.us/msg00388.html). A 2015 user report says permutation overhead sometimes cancelled the gain from reduction — [tlaplus Google group](https://groups.google.com/g/tlaplus/c/LlUYtAG8OyE/m/PJttvGBiDAAJ).
- **TLC simulation mode samples behaviors instead of enumerating states.** It generates random behaviors up to a maximum length, and the only statistic reported is States Found — [TLC advanced options (older Toolbox doc)](https://course.ccs.neu.edu/cs3650/parent/tla+/toolbox/plugins/org.lamport.tla.toolbox.doc_1.5.4.201710061831/html/model/advanced-page.html). A 2026 list post says the simulator estimates distinct states with a per-worker HyperLogLog, and that exact counts need `-Dtlc2.tool.Simulator.extendedStatistics.naive=true` (single forum post) — [TLA+ list, 2026](https://discuss.tlapl.us/msg06757.html).
- **Symmetry reduction originates in Murphi (classic, 1996).** Ip and Dill's scalarsets let a modeler declare interchangeable identifiers (replicas, processes), and the checker explores one canonical representative per orbit — [Ip & Dill, "Better verification through symmetry", FMSD 1996](https://link.springer.com/article/10.1007/BF00625969).
- **SPIN brought explicit-state checking with POR and bitstate hashing to protocol models (classic, 1997)** — [Holzmann, "The model checker SPIN", IEEE TSE 1997](https://ieeexplore.ieee.org/document/588521).
- **Swarm verification** splits one big search into many small, independent, differently configured bounded searches (different hash functions, search orders and depth limits) run in parallel. It aims to find bugs in spaces too large to exhaust rather than to prove absence — [Swarm tool page, spinroot.com](http://spinroot.com/swarm/).
- **Partial-order reduction (classic, 1996).** Persistent (stubborn) sets explore only a subset of enabled transitions that is sufficient for safety properties. Sleep sets avoid re-exploring transitions already covered by a sibling branch — [Godefroid, "Partial-Order Methods for the Verification of Concurrent Systems", LNCS 1032](https://link.springer.com/book/10.1007/3-540-60761-7).
- **DPOR (classic, 2005).** It computes persistent sets on the fly from the dependencies actually observed during a run: two operations on the same shared object, at least one a write. It then backtracks only where a reordering of dependent operations is possible — [Flanagan & Godefroid, POPL 2005](https://dl.acm.org/doi/10.1145/1040305.1040315).
- **The P checker's defaults.** By default `p check` runs a seeded `random` strategy for a requested number of schedules (`-s 100`) and reports the fraction of buggy schedules. For each bug it writes a readable `.txt` trace, a `.schedule` file for replay, and `.trace.json` / `.dgml` artifacts — [P docs: debugging errors](https://p-org.github.io/P/advanced/debuggingerror).
- **P's exhaustive checkers.** P ships a separate checker, PSym, which compiles the model to a symbolically instrumented Java IR and "guarantees to never repeat an already-explored execution". It reports safe, buggy or partially-safe plus estimated coverage. The docs warn that exhaustive exploration "is generally not feasible for large models" and advise few replicas and few `choose(*)` options. The navigation also lists "PEx (Exhaustive Checking)" and "PVerifier" — [P docs: PSym](https://p-org.github.io/P/advanced/psym/whatisPSym).
- **Fest (NSDI '26, CMU and AWS) replaces random sampling of P schedules with grey-box-fuzzing-style, feedback-guided adaptive schedule generation.** On 94 model configurations (5 open-source models, including DeepSeek's distributed file system, and 14 AWS industrial models) it reports up to 41x (1.5x average) more behavioral coverage, up to 278x (15x average) more scenario coverage, and 33% more bugs than existing methods (authors' own evaluation) — [Li, Padhye, Desai, NSDI '26](https://www.usenix.org/conference/nsdi26/presentation/li).

### Inferences
- **Our state is a configuration:** state variables plus messages in flight per port or queue plus the position of each lane. It should get a canonical, order-independent encoding, so that fingerprinting and symmetry work: replicas `×3` and N identical concurrent requests are scalarsets.
- **Default exhaustive mode should be BFS with fingerprint dedup**, because BFS gives shortest counterexamples, which are the easiest scripts to watch. For large models, fall back to swarm-style parallel bounded searches or a Fest-style coverage-guided sampler, and say so in the UI ("explored N of about M states, no proof").
- **Industrial practice favours randomized sampling over enumeration once models are large** (P's default, Fest's NSDI '26 result). Our "exhaustive" mode should really be two modes: exhaustive within explicit bounds, and coverage-guided sampling without a guarantee.

### Gaps
- I found no primary 2025–2026 benchmark giving practical explicit-state sizes. Folklore says TLC handles about 10^8 to 10^10 distinct states on one large machine, but I found no citable primary source; treat it as unverified.
- The exact current relationship between PSym and PEx (rename or separate tool) is not stated on the PSym page.
- I could not open the Fest PDF (binary fetch), so its feedback metric ("behavioral coverage" definition) is unverified.

## 2. Stateless model checking, controlled concurrency testing and deterministic simulation

### Takeaway
Stateless model checkers (CHESS, Nidhugg, GenMC) re-execute the program from the start under a controlled scheduler and store no visited states. They rely on DPOR (optimal variants explore one execution per equivalence class) and on bounds on preemptions or delays. Controlled concurrency testing (Coyote, P, Shuttle) samples schedules with randomized strategies: PCT gives a probabilistic guarantee per depth-d bug. Deterministic simulation testing (FoundationDB, TigerBeetle VOPR, Antithesis, Turmoil and madsim) applies the same idea to whole systems: every source of nondeterminism comes from one seeded generator. All of them reproduce a bug from (seed or schedule, code version). This is exactly the "(binding, schedule)" determinism the checkpoint asks for.

### Cited Findings
- **CHESS (classic, 2008).** It systematically tests multithreaded programs by taking control of thread scheduling, and introduced iterative context bounding to make the search tractable — [Musuvathi et al., OSDI 2008](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi.pdf).
- **Iterative context bounding (classic, 2007).** It explores executions in order of increasing preemption count. The number of executions is polynomial in program size for a fixed bound, and the paper reports that many bugs show up with very few (≤2) preemptions — [Musuvathi & Qadeer, PLDI 2007](https://dl.acm.org/doi/10.1145/1250734.1250785).
- **PCT (classic, 2010).** It assigns random thread priorities and d−1 random priority-change points. For a program with n threads and k steps, it finds a bug of depth d with probability at least 1/(n·k^(d−1)) per run — [Burckhardt et al., ASPLOS 2010](https://dl.acm.org/doi/10.1145/1736020.1736040) ([PDF](https://www.cse.iitd.ac.in/~sbansal/csl862-soft/readings/pct.pdf)).
- **PCT is still a controlled technique:** every scheduling choice is decided by the algorithm, so each run is replayable. The 2016 empirical study (SCTBench) compares controlled schedulers, PCT and schedule bounding on real benchmarks — [Thomson, Donaldson, Betts, ACM TOPC 2016](https://www.doc.ic.ac.uk/~afd/papers/2016/TOPC.pdf).
- **Optimal DPOR (classic, 2014).** Source sets and wakeup trees explore exactly one execution per Mazurkiewicz trace, and Nidhugg implements it for C/pthreads — [Abdulla et al., POPL 2014](https://dl.acm.org/doi/10.1145/2535838.2535845); [Nidhugg](https://github.com/nidhugg/nidhugg).
- **TruSt (POPL 2022)** is optimal DPOR with polynomial (linear) memory. It works for any memory model meeting basic assumptions (SC, TSO, PSO, RC11), its correctness is proven in Coq, and it explores execution graphs rather than interleavings — [Kokologiannakis et al., POPL 2022](https://plv.mpi-sws.org/wmc/popl2022-trust-full.pdf). GenMC implements TruSt — [GenMC TACAS 2024 paper](https://plv.mpi-sws.org/wmc/tacas2024-gater.pdf); [GenMC](https://github.com/MPI-SWS/genmc). A preemption-bounded TruSt variant exists — [Springer chapter](https://link.springer.com/chapter/10.1007/978-3-031-30823-9_5).
- **Parsimonious Optimal DPOR (POP, 2024)** is a sequence-based, polynomial-space, optimal algorithm for sequential consistency. Its authors report similar performance to TruSt on small benchmarks and much better scaling on programs with long executions (authors' claim) — [arXiv 2405.11128](https://arxiv.org/abs/2405.11128).
- **Coyote** offers pluggable strategies: random, `prioritization` (PCT-like, with a maximum number of priority switch points, e.g. `-s prioritization -sv 10`), `fair-prioritization` (prioritization for a prefix, then fair random, for liveness) and `portfolio`. It saves a trace for each bug, and `coyote replay` reproduces it with a debugger attached, because Coyote controls every nondeterministic choice — [Coyote failure-detector sample](https://microsoft.github.io/coyote/samples/actors/failure-detector); [Coyote liveness how-to](https://microsoft.github.io/coyote/how-to/liveness-checking); [Coyote concurrency unit testing](https://microsoft.github.io/coyote/concepts/concurrency-unit-testing). For non-actor C# code, Coyote rewrites binaries to insert scheduling points — [Binary rewriting](https://microsoft.github.io/coyote/concepts/binary-rewriting). Tool paper: [TACAS 2023, "Industrial-Strength Controlled Concurrency Testing for Programs With Coyote"](https://www.microsoft.com/en-us/research/uploads/prod/2023/04/978-3-031-30820-8_26.pdf).
- **P#, Coyote's predecessor, ran portfolio testing** across several randomized strategies and had a replayer driven by a schedule file — [P# testing methodology](https://github.com/p-org/PSharp/blob/master/Docs/Testing/TestingMethodology.md).
- **Shuttle** (AWS, Rust) implements random and PCT schedulers for controlled concurrency testing — [shuttle PCT test source](https://docs.rs/crate/shuttle/latest/source/tests/basic/pct.rs).
- **Lockstep** (Elixir) lists a PCT strategy and states that strategies are deterministic given a seed, so a bug-finding iteration is reproducible — [Lockstep.Strategy](https://hexdocs.pm/lockstep/Lockstep.Strategy.md).
- **FoundationDB** simulates an entire cluster deterministically inside one single-threaded process — [FoundationDB: Simulation and Testing](https://apple.github.io/foundationdb/testing.html); [SIGMOD 2021 paper](https://dl.acm.org/doi/10.1145/3448016.3457559). BUGGIFY sites are enabled or disabled once per run, then fire with some probability, about 25% per a secondary source — [transactional.blog on BUGGIFY](https://hn.jakechampion.name/reader/https://transactional.blog/simulation/buggify). Sources conflict on whether BUGGIFY code ships in production binaries; unresolved.
- **TigerBeetle VOPR.** A run is determined by seed plus Git commit, so failures replay locally. The seed tunes fault-injection parameters (packet loss, disk corruption), and clock, network and disk are stubbed — [VOPR docs](https://docs.tigerbeetle.com/about/vopr); [HACKING.md](https://github.com/tigerbeetle/tigerbeetle-history-archive/blob/main/docs/HACKING.md). It ran on 1,000 cores with time accelerated about 700x, "nearly two millennia of simulated runtime per day" (Nov 2024) — [TigerBeetle newsletter](https://tigerbeetle.com/newsletters/2024-11-07-october-in-tigerland).
- **TigerBeetle protocol-aware DST (Aug 2026).** The simulator "has complete visibility into each replica's consensus and storage-level state", so it checks per-replica safety and liveness invariants, not only API-level ones as Jepsen-style or hypervisor-level testing does. Hand-crafted "what happens if…" scenarios take 30–40 lines and answer within milliseconds — [TigerBeetle blog, 2026-08-20](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst).
- **Antithesis** runs the whole system (clients and servers) in one deterministic VM on a hypervisor forked from FreeBSD bhyve. Guest time is computed by the hypervisor from deterministic state, each VM is pinned to one core, and parallelism comes from many VMs. Input ingestion points "become possible branch points for future execution", so exploration is a tree of paths branched from saved states ("We are not replaying every single execution path from the beginning"). Determinism enables time-travel debugging and destructive analysis of buggy states — [Antithesis deterministic hypervisor blog](https://antithesis.com/blog/deterministic_hypervisor/); [FreeBSD Foundation](https://freebsdfoundation.org/antithesis-pioneering-deterministic-hypervisors-with-freebsd-and-bhyve/).
- **Rust DST frameworks.** Turmoil (tokio) simulates hosts, network and time in one thread — [tokio-rs/turmoil](https://github.com/tokio-rs/turmoil). madsim replaces tokio and std APIs with a deterministic simulator — [madsim-rs/madsim](https://github.com/madsim-rs/madsim).
- **Themis (NSDI '26)** detects distributed concurrency bugs by RPC-driven, race-directed test generation and fuzzing. It reports 198 new violations corresponding to 52 new bugs across eight systems (per search summary of the abstract) — [Themis, NSDI '26](https://www.usenix.org/conference/nsdi26/presentation/cao).

### Inferences
- **Our interpreter is in the best possible position for stateless exploration.** The model is the program, so every scheduling point is explicit: a scheduler picks an enabled transition. We need no binary rewriting or hypervisor. A DST-style "one seeded chooser for all nondeterminism" interface gives concrete runs, random sampling, PCT and DPOR as strategies behind one `choose(enabled)` hook.
- **Pick strategies by goal:**
  - DPOR when the step count is small and we want "all distinct orderings of the conflicting DB accesses".
  - PCT or prioritization with depth 2–3 for quick bug-finding.
  - Preemption bounding (0, 1, 2) as the user-facing "how adversarial is the scheduler" knob, because the bound is explainable ("this race needs one preemption").
- **Branch from saved configurations, as Antithesis does.** It matches the checkpoint's "reverse stepping: states can be stored or recomputed" and makes "flip an assumption mid-run, continue" cheap.

### Gaps
- No public 2026 head-to-head benchmark compares DPOR and PCT-style sampling on actor or message-passing models of web systems. The TOPC 2016 study is on C/pthreads.
- How Antithesis guides exploration (its scoring and fuzzing heuristics) is not described in the hypervisor post.

## 3. Symbolic execution and symbolic model checking for EFSM-style models

### Takeaway
Two families fit our symbolic run. Symbolic execution (KLEE-style) forks on unbound predicates, keeps a path condition per branch, and asks an SMT solver which branches are feasible; this is exactly "unbound predicates fork, choices become assumptions". Symbolic model checking (BDD, SAT/SMT-based BMC, IC3/PDR, k-induction; Apalache for TLA+) encodes all paths of bounded length as one formula. It is good for "is a bad state reachable within k steps" and for invariant proofs, but it yields one counterexample at a time, not a browsable tree. State merging trades fewer paths for harder solver queries and loses the per-path "script" view.

### Cited Findings
- **KLEE (classic, 2008).** It forks execution at branches on symbolic values and uses a constraint solver to check feasibility and generate concrete inputs per path — [Cadar, Dunbar, Engler, OSDI 2008](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/cadar/cadar.pdf).
- **Path explosion and state merging (classic, 2012).** Merging paths at join points turns path conditions into disjunctive formulas. "Query count estimation" decides when merging pays off, and the paper reports large speedups on GNU Coreutils — [Kuznetsov et al., PLDI 2012](https://dl.acm.org/doi/10.1145/2254064.2254088). Veritesting switches between dynamic symbolic execution and static merging of bounded regions — [Avgerinos et al., ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568293).
- **Bounded model checking (classic, 1999)** unrolls the transition relation k times into a SAT formula, and finds shortest counterexamples up to k — [Biere, Cimatti, Clarke, Zhu, TACAS 1999](https://link.springer.com/chapter/10.1007/3-540-49059-0_14).
- **k-induction (classic, 2000)** extends BMC to proofs: if the property holds for k steps from init and k consecutive good states imply a good next state, the property is invariant — [Sheeran, Singh, Stålmarck, FMCAD 2000](https://link.springer.com/chapter/10.1007/3-540-40922-X_8).
- **IC3/PDR (classic, 2011)** builds inductive invariants incrementally from relative-induction SAT queries without unrolling — [Bradley, VMCAI 2011](https://link.springer.com/chapter/10.1007/978-3-642-18275-4_7). nuXmv offers BDD, BMC, k-induction and IC3 engines for finite and infinite-state transition systems — [nuXmv](https://nuxmv.fbk.eu/).
- **Apalache** encodes bounded symbolic executions of TLA+ as SMT (Z3) constraints instead of enumerating states. It checks bounded safety and inductive invariants, needs type annotations, and in Quint uses `--max-steps` (default 10). Against TLC it is sometimes faster and sometimes slower: better for large parameter values, worse for very small ones — [Apalache](https://apalache.informal.systems/); [Konnov, Kukovec, Tran, OOPSLA 2019](https://dl.acm.org/doi/10.1145/3360549); [Quint model checkers](https://quint-lang.org/docs/model-checkers); [TLA+ Trifecta, arXiv 2211.07216](https://arxiv.org/pdf/2211.07216).
- **P's PSym** uses a symbolically instrumented IR for its exhaustive mode, i.e. symbolic techniques inside an explicit-schedule checker — [P docs: PSym](https://p-org.github.io/P/advanced/psym/whatisPSym).

### Inferences
- **The symbolic run is symbolic execution over predicates, not BMC.** Most of our predicates are named, uninterpreted Booleans ("token expired", "user exists"), so the "solver" is often just propagating Boolean assignments and checking consistency, cheap enough to run interactively. Use SMT only where predicates have arithmetic or relational structure (`balance >= amount`) or link to each other (one implies another).
- **Each fork is an assumption record** (predicate, bound value, alternatives). A path is a conjunction of bindings, which matches the checkpoint's assumption format directly. Keep a symbolic execution tree (one node per fork, labelled by the predicate) as the data structure behind "untaken branches appear faintly".
- **Do not merge states in the user-facing tree;** merging destroys per-path scripts. Merging or BMC can serve as a backend query: "is there any binding that reaches `report.status = failed`?" → one witness binding → replay it concretely to get a script.
- **k-induction or IC3 are later, "prove it" features** (invariant holds for all depths). They are not needed for the first interpreter.

### Gaps
- I found no study of symbolic execution over LLM-extracted EFSM models with named uninterpreted predicates; this framing is our own.
- I did not verify any 2025–2026 advance in symbolic execution of message-passing models beyond the tools above.

## 4. Counterexample traces: generation, minimization, explanation and visualization

### Takeaway
Shortest counterexamples come for free from BFS (TLC) or BMC. Random or PCT runs give long traces that need shrinking. Delta debugging over schedule differences, trace simplification that reduces context switches, DEMi's domain-aware minimization, and Hypothesis's internal reduction over the choice sequence are the main techniques. Minimization must preserve "the same bug". Explanation works best when the trace is presented in domain terms: Elle names the anomaly cycle and the transactions; TLA+ and P show per-step state.

### Cited Findings
- **Delta debugging (classic, 2002)** systematically narrows a failure-inducing difference to a 1-minimal subset — [Zeller & Hildebrandt, IEEE TSE 2002](https://doi.org/10.1109/32.988498). Applied to thread schedules, it isolates the failure-inducing difference between a passing and a failing schedule, often down to a single pair of switching points — [Choi & Zeller, ISSTA 2002](https://dl.acm.org/doi/10.1145/566172.566211).
- **Trace simplification for concurrency (2010)** reduces the number of thread context switches in a buggy trace while preserving the bug, making it easier to read — [Jalbert & Sen, FSE 2010](https://dl.acm.org/doi/10.1145/1882291.1882296).
- **DEMi (NSDI 2016)** minimizes faulty executions of distributed systems (Akka; tested on akka-raft and Spark). It cuts execution length by up to 97% within a small time budget and lands within 1x–4.6x (median 1.6x) of the smallest possible bug-triggering execution. It is about 14k lines of Scala — [Scott et al., NSDI 2016](https://usenix.org/conference/nsdi16/technical-sessions/presentation/scott); [PDF](https://people.eecs.berkeley.edu/%7Enecula/Papers/faulty_executions_nsdi16.pdf).
- **Hypothesis internal reduction (ECOOP 2020).** Shrinking acts on the generator's recorded random choices, so every reduced case is one the generator could have produced, which avoids the test-validity problem of external reducers. It has been Hypothesis's only reduction method since 2016, and the evaluation also reveals a scaling problem — [MacIver & Donaldson, ECOOP 2020](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2020.13); [PDF](https://www.doc.ic.ac.uk/~afd/papers/2020/ECOOP_Hypothesis.pdf).
- **QuickCheck plus PULSE (classic, 2009)** finds Erlang race conditions with randomly generated parallel command sequences, a controlled scheduler (PULSE) and linearizability checking, then shrinks failing parallel tests and visualizes the resulting interleavings — [Claessen et al., ICFP 2009](https://dl.acm.org/doi/10.1145/1596550.1596574).
- **Elle explains anomalies as short, readable cycles over a Direct Serialization Graph (Adya's formalism).** It names the transactions and the dependency edges (ww, wr, rw), and detects G0, G1a, G1b, G1c, G-single, G2 plus aborted and intermediate reads — [Kingsbury & Alvaro, PVLDB 14(3)](https://www.vldb.org/pvldb/volumes/14/paper/Elle%3A%20Inferring%20Isolation%20Anomalies%20from%20Experimental%20Observations); [arXiv 2003.10554](https://arxiv.org/pdf/2003.10554).
- **P trace artifacts.** P emits a readable step trace from the initial to the error state plus `.trace.json`, and the `.schedule` file single-steps the program in replay — [P docs](https://p-org.github.io/P/advanced/debuggingerror).
- **TLA+ trace tooling.** The VS Code extension runs TLC and shows error traces as per-state variable values — [tlaplus/vscode-tlaplus](https://github.com/tlaplus/vscode-tlaplus).

### Inferences
- **Minimize at two levels.** (a) Drop whole inputs, requests or faults with delta debugging or DEMi-style removal. (b) Reduce preemptions and context switches with Jalbert–Sen-style simplification, so the race shows as "A reads, B reads, B writes, A writes" with no noise.
- **Representing the schedule as a choice sequence over enabled transitions** gives Hypothesis-style internal shrinking for free: every shrunk schedule is valid by construction. The bug predicate is "same violated invariant at the same step kind".
- **Explanations should name the conflict in model terms,** as Elle does: "two `POST /checkout` lanes read `stock` before either writes (rw anti-dependency cycle)". It should not be a raw step list. This maps directly onto the checkpoint's `non-atomic` segment marker.
- **The minimal schedule becomes the default script for a race;** the unminimized one stays available as provenance.

### Gaps
- I found no primary documentation of a dedicated P trace visualizer (sometimes called "PeasyViz") on the fetched P pages.
- I found no published user study comparing counterexample visualization styles (sequence diagram vs. state table vs. animation) for concurrency bugs.

## 5. Race and anomaly detection at the model level for web systems

### Takeaway
Web-system races fall into two classes, both detectable on a model. The first is database-mediated: check-then-act across requests under weak isolation or with no transaction at all. ACIDRain, IsoDiff, Elle and Feral concurrency control describe it. The second is event-loop ordering races: NodeRacer, Node.fz, NRace, NodeRT, NACD. ACIDRain's key finding is that most real vulnerabilities come from wrong transaction scoping, not weak isolation levels. That is a model-level property: which reads and writes share a transaction and a guard.

### Cited Findings
- **ACIDRain (SIGMOD 2017)** defines attacks that send concurrent requests to exploit non-serializable behavior in database-backed web apps. Using "Abstract Anomaly Detection" (2AD) over traces of database activity, the authors found 22 vulnerabilities in self-hosted eCommerce apps spanning over 2M websites — [Warszawski & Bailis, SIGMOD 2017](https://dl.acm.org/doi/10.1145/3035918.3064037); [summary](https://blog.acolyer.org/2017/08/07/acidrain-concurrency-related-attacks-on-database-backed-web-applications/). Per a presentation summary, 17 of the 22 were due to incorrect transaction usage and 5 to weak isolation (secondary source) — [Rahmani defense slides](https://kiarahmani.github.io/files/defense.pdf).
- **Feral concurrency control (2015)** shows that Rails apps enforce invariants such as uniqueness and associations in application code (validations) rather than in the database, which is unsafe under concurrency — [Bailis et al., SIGMOD 2015](https://dl.acm.org/doi/10.1145/2723372.2737784).
- **IsoDiff (PVLDB 2020)** helps developers find anomalies caused by weak isolation (Snapshot Isolation, Read Committed). It was applied to TPC-C and seven open-source applications (per search summary; URL is the NSF PAR copy identified by search, not opened) — [Gan et al., NSF PAR](https://par.nsf.gov/servlets/purl/10227054).
- **Elle** limits: it ignores predicates, so it cannot distinguish repeatable read from serializability and misses some predicate anomalies — [arXiv 2003.10554](https://arxiv.org/pdf/2003.10554). Cobra checks serializability of black-box key-value histories with SMT plus GPU-accelerated pruning — [Tan et al., OSDI 2020](https://www.usenix.org/conference/osdi20/presentation/tan). Jepsen's published analyses apply these checkers to real databases — [Jepsen analyses](https://jepsen.io/analyses).
- **NodeRacer (ICST 2020).** It records a sample run, builds a happens-before graph of callbacks, then systematically postpones events to explore alternative orderings, implemented in JavaScript with no Node.js changes. It finds event-race errors more often than the Node.fz fuzzer, with fewer false alarms thanks to its happens-before relations — [NodeRacer page](https://cs.au.dk/~amoeller/papers/noderacer); [paper](https://users-cs.au.dk/~amoeller/papers/noderacer/paper.pdf); [GitHub](https://github.com/andreendo/noderacer).
- **NodeRacer's limit and its fix.** It explores only interleavings of events seen in the sample run, so it can miss races; NACD (ECOOP 2025) addresses this by injecting delays into core async operations — [NACD, ECOOP 2025](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2025.9).
- **Other Node.js race detectors.** NRace (ASE 2021) models happens-before across events and multi-priority queues (nextTick, promises, timers, I/O) — [NRace PDF](https://changxiaoning.github.io/files/2021-ase-nrace.pdf). NodeRT (ISSTA 2023) is a more practical dynamic detector — [NodeRT](https://2023.issta.org/details/issta-2023-technical-papers/98/NodeRT-Detecting-Races-in-Node-js-Applications-Practically). Node.fz fuzzes event-loop schedules — [Davis et al., EuroSys 2017](https://dl.acm.org/doi/10.1145/3064176.3064188).

### Inferences
- **Model-level check-then-act detection is a DPOR dependency question.** Two lanes (requests) touch the same row-level state variable, at least one writes, and the read that feeds a guard is not in the same atomic region (transaction with a sufficient isolation level, or a lock) as the write. Annotate DB access transitions with `{store, key-class, txn-id, isolation}`, and the explorer only needs to reorder conflicting accesses across lanes.
- **Isolation level should be a per-store or per-transaction assumption record** (`reason: config-default`). The user flips Read Committed to Serializable and re-runs the same schedule: under Serializable the anomalous schedule becomes infeasible (abort or block), and the diff shows it.
- **The event loop belongs in a platform pack:** V8/Node queue semantics (microtasks before macrotasks, nextTick priority), following NRace's multi-priority queue model. Event-loop races then become ordinary schedule nondeterminism among enabled callbacks.
- **Report anomalies in Elle's vocabulary** (G-single, lost update, write skew), plus an ACIDRain-style label ("non-transactional check-then-act").

### Gaps
- I could not verify PolySI, Plume or IsoVista (2023–2025 isolation checkers) with fetched primary pages; the search returned unrelated results. They are omitted from findings.
- I found no tool that detects check-then-act on an abstract model, as opposed to traces or code; this appears to be open.

## 6. Determinism and replay: schedules as first-class values, re-running after a change, trace diffing

### Takeaway
Every tool surveyed reproduces bugs from (seed or schedule, code version): P `.schedule`, Coyote replay traces, TigerBeetle seed plus commit, FoundationDB seed, Antithesis branch points. Re-running a schedule against changed code is much weaker. A seed reproduces nothing after a change, and an index-based schedule drifts as soon as the enabled set changes. DEMi and Hypothesis show the two workarounds: match events by domain identity, or treat the schedule as a choice sequence that the generator re-interprets. I found no mature trace-diffing tool for concurrent executions.

### Cited Findings
- **P replay** takes the `.schedule` file and test-case name — `p check --replay <buggy>.schedule -tc <testcase> -v` — [P docs](https://p-org.github.io/P/advanced/debuggingerror).
- **Coyote** records every nondeterministic choice; `coyote replay` re-executes it under a debugger — [Coyote sample](https://microsoft.github.io/coyote/samples/actors/failure-detector).
- **TigerBeetle VOPR** determinism is "seed number and the Git commit"; the seed alone does not reproduce on other code — [VOPR docs](https://docs.tigerbeetle.com/about/vopr).
- **Antithesis** treats input points as branch points in a tree of executions and branches from saved states rather than replaying from the start — [Antithesis blog](https://antithesis.com/blog/deterministic_hypervisor/).
- **DEMi** minimizes by re-running modified schedules: once events are removed, later messages may differ or be missing, so it must decide which events still correspond. It is "domain-aware" for exactly this reason — [Scott et al., NSDI 2016 PDF](https://cs.nyu.edu/~apanda/assets/papers/nsdi16.pdf).
- **Hypothesis** stores and replays the underlying choice sequence and re-interprets it through the generator, which keeps reduced inputs valid — [ECOOP 2020](https://www.doc.ic.ac.uk/~afd/papers/2020/ECOOP_Hypothesis.pdf).
- **PCT schedules are fully determined by the algorithm's random choices,** which makes them replayable — [Thomson et al., TOPC 2016](https://www.doc.ic.ac.uk/~afd/papers/2016/TOPC.pdf).

### Inferences
- **Represent a schedule as a list of semantic choices, not indexes or seeds.** Each entry is `{lane-id, transition-id (semantic path), binding of choose()}`, plus a hash of the configuration it was taken from. On a changed model, replay greedily:
  - if the recorded transition is enabled, take it;
  - if it no longer exists, mark the step `diverged` and fall back to a policy (skip, or nearest equivalent by anchor);
  - if new transitions appear, schedule them by a declared default (run to completion within the lane).

  This is DEMi's correspondence problem solved with the checkpoint's semantic IDs.
- **A run is `(model version, binding = assumption set, schedule)`.** Concrete run = full binding plus deterministic default scheduler; symbolic run = partial binding plus default scheduler; exploration = partial or empty binding plus a strategy that emits schedules. Every explored result is stored as a replayable triple, so any exploration hit opens as a normal script.
- **Trace diff.** Align two pomsets by semantic step ID first and by anchor second, then by order within lanes. Report inserted, removed and changed steps (changed `data_out` or state writes), plus the first divergence point. "Fix check" = the same schedule on the new model: either the invariant violation disappears, the schedule becomes infeasible (a guard or lock now blocks it), or it diverges earlier.
- **Store a checkpoint of the configuration every N steps** so step-back is "restore checkpoint plus replay ≤N steps" (rr-style). This follows from determinism; rr does the equivalent for real processes — [rr](https://rr-project.org/).

### Gaps
- I found no published algorithm or tool for diffing concurrent traces (pomsets) across program versions; this looks open. Differential testing tools compare outputs, not executions.
- None of the tools document how replay behaves on a changed program, beyond "seed plus commit".

## 7. Recommendations for our interpreter

### Takeaway
Build a single step function plus a pluggable chooser. Ship concrete runs and symbolic forking first. Add bounded exhaustive exploration with DPOR and fingerprinting only over conflicting shared-state accesses, plus PCT-style sampling for larger models. Represent every result as a replayable (model version, binding, schedule) triple, minimize counterexamples before showing them, and explain races in Elle and ACIDRain vocabulary.

### Cited Findings
- Real tools already put random, prioritization (PCT) and portfolio strategies behind one controlled scheduler — [Coyote](https://microsoft.github.io/coyote/overview/how); [P docs](https://p-org.github.io/P/advanced/debuggingerror); [Shuttle](https://docs.rs/crate/shuttle/latest/source/tests/basic/pct.rs).
- Coverage-guided schedule generation beats random sampling on industrial P models (authors' NSDI '26 result) — [Fest](https://www.usenix.org/conference/nsdi26/presentation/li).
- DPOR needs only a dependency relation between operations, available from shared-object accesses — [Flanagan & Godefroid 2005](https://dl.acm.org/doi/10.1145/1040305.1040315). Optimal DPOR in polynomial memory is available as a design to copy — [TruSt](https://plv.mpi-sws.org/wmc/popl2022-trust-full.pdf); [POP](https://arxiv.org/abs/2405.11128).
- Small preemption bounds expose many concurrency bugs — [Musuvathi & Qadeer 2007](https://dl.acm.org/doi/10.1145/1250734.1250785). PCT's guarantee depends on bug depth d — [Burckhardt et al. 2010](https://dl.acm.org/doi/10.1145/1736020.1736040).
- Explicit-state exhaustive exploration is infeasible for large P models; the docs recommend small configurations — [P PSym docs](https://p-org.github.io/P/advanced/psym/whatisPSym).
- Most ACIDRain vulnerabilities were wrong transaction scoping (secondary source) — [slides](https://kiarahmani.github.io/files/defense.pdf); [ACIDRain](https://dl.acm.org/doi/10.1145/3035918.3064037).

### Inferences (the recommendations)
1. **Core API.** The model gets one entry point:

   ```
   step(config, choice) -> (config', events)
   enabled(config) -> [choice]
   ```

   Each choice is either a scheduling choice (which lane or transition runs) or a value choice (which side of an unbound predicate). All modes are strategies over `enabled`:
   - concrete: a deterministic default policy;
   - symbolic: fork on value choices only, under a deterministic scheduler;
   - exploration: fork on scheduling choices, and optionally value choices.

2. **Build order for strategies.**
   1. Concrete run with the default scheduler: run-to-completion per request lane, FIFO queues.
   2. Symbolic run: DFS over value choices, with Boolean propagation over named predicates (SMT only for arithmetic guards). The output is a symbolic execution tree whose edges are assumption records.
   3. Bounded exhaustive exploration: DPOR (sleep sets plus source sets) over conflicting shared-state accesses only. DB rows, caches and queues are the shared objects; port messages are ordered by wires. It is BFS-capable for shortest traces, deduplicates with fingerprints, and applies symmetry over replica stacks and identical request lanes.
   4. PCT or prioritization random sampling with depth 1–3 for models too big for (3).
   5. Later, coverage-guided sampling in the Fest style, and k-induction or IC3 via an SMT backend for "prove the invariant".

3. **Bounds the user sees** (each one is part of the result's provenance, e.g. "no race found with ≤2 checkout requests, ≤1 preemption, depth ≤40"):
   - the number of concurrent request lanes per entry port (default 2);
   - the preemption bound (0, 1 or 2; default 1);
   - maximum depth in macro steps;
   - loop unroll bound;
   - the number of replicas, which symmetry then collapses.

4. **Granularity.** Explore at the coarsest level whose macro steps are atomic (`atomicity: stutter | isolated | guarded`). Only `non-atomic` segments need small-step interleaving. This is Lipton reduction, and it is our largest state-space saving.

5. **Schedules are first-class.** Store them as semantic choice lists (lane ID, transition semantic path, bound value), with configuration hashes as checks. Replay on a changed model uses greedy matching and marks divergence. Seeds are only a compact encoding for sampling strategies, never the stored artifact.

6. **Minimize counterexamples before showing them.**
   1. Delta-debug requests and faults (DEMi-style).
   2. Reduce preemptions (Jalbert–Sen).
   3. Shrink value choices (Hypothesis-style, over the choice list).

   Keep the original run as provenance.

7. **Turn results into scripts and assumptions.** Each leaf of the symbolic tree or each violating schedule becomes a script (a pomset: `after` edges from lane order plus cross-lane dependencies only, so incomparable steps render as parallel lanes). Value choices become assumption records. Scheduling choices that matter (the dependent reorderings DPOR found) become "schedule assumptions" the user can flip: "what if B's read happens after A's write?". That is exactly a DPOR backtrack point.

8. **Explain races as named anomalies:** lost update, write skew, G-single, non-transactional check-then-act, event-loop order violation. Show the conflict cycle overlaid on the two lanes (Elle-style), and mark the involved segments `non-atomic` so they are never collapsed.

9. **Diffing.** Align runs by semantic step ID, then anchor, then lane order. Report the first divergence, inserted and removed steps, and changed state writes. "Fix verified" = the violation is gone under the same schedule, and a fresh exploration within the same bounds finds no other violating schedule.

### Gaps
- None of the recommendations have been validated on hand-written gold models yet. Checkpoint build-order step 2 (gold models plus a minimal interpreter) is the place to measure state-space sizes and the payoff of DPOR and symmetry for the login and queue-worker models.
- No surveyed source covers the UX of flipping a schedule choice (as opposed to a value choice) in a debugger; this is a novel interaction to prototype.
