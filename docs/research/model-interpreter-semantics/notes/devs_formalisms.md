# DEVS and the discrete-event formalism family as a precedent for the model interpreter

Scope note: written 2026-11-11 against the project framing in `docs/brainstorm/checkpoint-01.md` ("Framing: a model interpreter") and the record model in `docs/research/static-analysis-llm-system-models/report.md`. Several primary PDFs (Chow & Zeigler 1994, Barros 1995/1997, Zeigler & Chi 1992, WSC 2020 "super dense time" paper) could not be read in full in this session (binary PDF extraction failed); claims taken from them rely on abstracts, search snippets or secondary descriptions and are flagged as such.

## 1. Classic DEVS and Parallel DEVS: atomic and coupled models, ports and couplings, closure under coupling

### Takeaway

An atomic DEVS is ⟨X, Y, S, δint, δext, λ, ta⟩ (Parallel DEVS adds δconf over input bags); a coupled DEVS wires components by input/output couplings; and DEVS is closed under coupling: any coupled model has an equivalent atomic model whose state is the product of the children's total states and whose time advance is the minimum of their remaining times. That construction is a precise, citable definition of "a closed entity behaves as one atomic thing", but it is an exact product construction, not an abstraction, and it relies on a deterministic tie-breaker (`select`) or on simultaneous-bag semantics that hide exactly the interleavings our interpreter wants to explore.

### Cited Findings

- **Atomic classic DEVS** (as presented by Van Tendeloo & Vangheluwe): ⟨X, Y, S, q_init, δint, δext, λ, ta⟩; S is the state set (may hold arbitrary attributes); X and Y are input/output event sets forming the interface; δint: S → S fires when the time advance expires; δext: Q × X → S uses the total state and the input; λ: S → Y ∪ {ϕ} is called just before δint and is read-only on state; ta: S → ℝ≥0 ∪ {+∞}; e is elapsed time since last transition, Q = {(s, e) | 0 ≤ e ≤ ta(s)}. q_init is noted as not part of the original specification — [Van Tendeloo & Vangheluwe, "An Introduction to Classic DEVS", arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- **Coupled classic DEVS** (non-port form): ⟨X_self, Y_self, D, {M_i}, {I_i}, {Z_i,j}, select⟩ with D component names, I_i influencee sets (for i ∈ D ∪ {self}), Z_i,j *translation functions on each connection*, and select: 2^D → D as tie-breaker; constraints: no self-influence (i ∉ I_i) — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- **Coupled DEVS with ports** (Wikipedia's formulation): N = ⟨X, Y, D, {M_i}, C_xx, C_yx, C_yy, Select⟩ where C_xx are external input couplings (EIC), C_yx internal couplings (IC), C_yy external output couplings (EOC, a function that may map to ϕ, i.e. suppress output), Select the tie-breaker; C_xx and C_yx are sets of port pairs (pure port-to-port wiring) — [Wikipedia, DEVS](https://en.wikipedia.org/wiki/DEVS) (page carries quality and conflict-of-interest tags; treat as secondary)
- **Parallel DEVS (PDEVS)** removes select so all imminent models transition simultaneously, uses bags of inputs/outputs, and adds a confluent transition for simultaneous internal and external events; the chapter suggests "internal first, then external" as a sane default but PDEVS leaves the semantics to the modeler — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- PDEVS atomic: (X, Y, S, δext, δint, δcon, λ, ta) with δext: Q × X^b → S over bags, λ: S → Y^b, and the constraint δcon(s, ∅) = δint(s); the confluent function may call δint/δext in either order or compute the new state directly — [OpenModelica DEVSLib, Parallel DEVS guide](https://build.openmodelica.org/Documentation/DESLib.DEVSLib.UsersGuide.ParallelDEVS.html)
- Classic DEVS "does not provide any mechanism to properly handle state transition collisions"; the confluent function is the main difference between classic DEVS and PDEVS; PDEVS is attributed to Chow & Zeigler (1994) — [search-result summary of later papers citing it, e.g. DEVStone metric arXiv:2309.16544](https://arxiv.org/abs/2309.16544); the 1994 original was not retrieved (only an unverified Scribd copy surfaced)
- **Closure under coupling — the construction** (classic DEVS): resultant atomic has X = X_self, Y = Y_self; state S = ×_i Q_i (each child's state *and* elapsed time); ta(s) = min_i σ_i with σ_i = ta_i(s_i) − e_i; imminent set IMM(s) = {i | σ_i = ta(s)}; i* = select(IMM(s)); λ(s) = Z_{i*,self}(λ_{i*}(s_{i*})) if self ∈ I_{i*}, else ϕ; δint: i* does δint, its influencees do δext on the translated output, all others only advance elapsed time; δext: models in I_self do δext on translated input, others advance elapsed time — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Wikipedia states plainly "DEVS is closed under coupling", so a coupled model's behavior is defined via the equivalent atomic model — [Wikipedia, DEVS](https://en.wikipedia.org/wiki/DEVS)
- The canonical theory reference is Zeigler, Muzy & Kofman, *Theory of Modeling and Simulation*, 3rd ed., Academic Press, August 2018, described as 40% revised, with new chapter-length coverage of iterative system specification and DEVS — [Elsevier](https://www.elsevier.com/books/theory-of-modeling-and-simulation/muzy/978-0-12-813370-5)

### Inferences

- **Couplings are not always pure forwarding.** The brief's premise ("DEVS couplings are pure forwarding — no message transformation at boundaries") is true of the port-based EIC/IC/EOC form but *false* of the original classic coupled model, which carries a translation function Z_i,j on every connection, including Z_self,j (outer input to child: our delegation inward) and Z_i,self (child to outer output: delegation outward). Our `adapter` on a delegation wire (DNAT, TLS terminate, HTTP parse) is a Z_i,j. Precedent exists; we should cite it rather than claim novelty.
- Z_i,j is a pure function of one event. Our "messages change form at boundaries" includes stateful transformations (TLS termination keeps session state, HTTP parsing buffers a stream). In DEVS those must be atomic models in the path, not translation functions. Rule for us: a stateless adapter may live on a wire; anything with state is an entity (a boundary entity), which matches the report's boundary-entity record.
- Closure under coupling gives us the *formal* meaning of "zoomed-out entity = one step-able thing": the closed entity's behavior is the resultant atomic model of its interior. But the resultant's state is the full product of children's states, so it is exact, not a summary. Our `summary` level (cases with guards and effects) is an *abstraction* of that resultant; the obligation "every path through the full graph maps to exactly one case, with the same effects at ports" is a morphism claim (resultant → summary), not closure itself. ZPK's system-morphism hierarchy is the natural formal home for that, but I did not retrieve its text (see Gaps).
- `select` and simultaneous-bag semantics are deterministic resolutions of concurrency. For a race-finding interpreter, every place where DEVS calls select, or where δconf picks an order, is a *scheduling choice point* we must enumerate in exhaustive mode rather than fix.

### Gaps

- Could not read Chow & Zeigler (1994) or Chow (1996) directly to quote the PDEVS coupled-model definition and its closure proof; the PDEVS coupled tuple with EIC/EOC/IC is stated here from Wikipedia, not the primary.
- ZPK 3rd ed. definitions of system morphisms (homomorphism between specifications) and of "legitimacy" were not retrieved; both are relevant to defining macro steps and must be checked in the book.

## 2. Abstract simulator protocol, time advance, hierarchy execution, flattening vs hierarchical

### Takeaway

DEVS gives coupled models an operational semantics via a tree of simulators (per atomic) and coordinators (per coupled) driven by a root coordinator using messages (i, *, x, y, done) and per-node t_L / t_N; this mirrors our "engine walks the entity tree" design and is directly reusable. Performance work (DEVStone) consistently shows that flattening the hierarchy (or reducing coupled to atomic) is faster, so hierarchy should be a modeling and display concept, not the execution structure.

### Cited Findings

- Abstract simulator messages: `i` (initialize), `*` (transition), `x` (input), `y` (output), `done`; each carries type, source and simulation time — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Atomic simulator: on `i`, t_L = t − e, t_N = t_L + ta(s); on `*`, if t = t_N send λ(s) to parent, apply δint, update t_L, t_N, send done; on x, check t_L ≤ t ≤ t_N, e = t − t_L, apply δext, update, send done — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Coordinator: forwards `i` to all children; sends `*` to the selected child among those with t_N = current time; routes y via influencee sets applying translation; routes parent x to influencees; when all children are done, t_L = max, t_N = min over children, send done upward. Root coordinator loops: send `*` at current time, wait done, set t = returned t_N — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Flattening (constructing the resultant atomic) is "elegant and formal" but "highly inefficient" to perform at run time; the abstract simulator gives coupled models "operational semantics" instead of "translational semantics"; "a direct implementation of the abstract simulator is inefficient", so real tools differ — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- adevs is reported as the usual best performer in DEVStone comparisons and as reducing coupled models to an equivalent atomic model; Cadmium and non-optimized Python simulators performed lower (TOMACS 2022) — [DEVStone metric, Cárdenas et al., ACM TOMACS 32(3) 2022, arXiv:2309.16544](https://arxiv.org/abs/2309.16544) (findings via search-result summaries of the paper body; abstract fetched directly)
- A WSC 2019 paper states the impact of flattening "has also been measured using DEVStone, showing clear performance improvements" (no per-engine numbers seen) — [WSC 2019 paper 006](https://www.informs-sim.org/wsc19papers/006.pdf) (search snippet)
- DEVStone metric = "the average number of times that a simulator can execute a selection of 12 DEVStone models in one minute" — [arXiv:2309.16544](https://arxiv.org/abs/2309.16544)
- The revisited DEVStone paper adds event-count equations and HOmem (a simpler alternative to HOmod) and compares five simulators on two platforms — [Risco-Martín et al., SIMULATION 93(6) 2017, arXiv:2402.05483](https://arxiv.org/abs/2402.05483)
- Parallel execution "might not necessarily be faster" — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)

### Inferences

- The coordinator algorithm is the hierarchical interpreter we need for *display*: each open entity is a coordinator, each closed entity is a simulator of its summary. "Step out" = run a coordinator to its next `done`; "step into" = descend to the child simulator. The t_L/t_N bookkeeping becomes, in an untimed model, "is this entity imminent" (has an enabled internal step).
- Execution should be flat (a global set of enabled transitions over all leaf behaviors) for speed and for exhaustive exploration, with hierarchy reconstructed for projection (event lifts to deepest visible ancestor). DEVStone results support flat execution; the checkpoint's projection rules already assume hierarchy is a view.
- The coordinator's "send `*` to *the selected* child" is exactly where a single schedule is fixed. Our exhaustive mode replaces select with "branch over every imminent child", which is what model checkers do and what DEVS simulators deliberately avoid.

### Gaps

- No head-to-head DEVStone numbers for flattened vs hierarchical coordinators per engine were retrieved; the TOMACS 2022 and SIMULATION 2017 bodies were not read.

## 3. Variants relevant to us: dynamic structure, Cell-DEVS, Symbolic, Fuzzy/non-deterministic, Routed, multi-level/multi-resolution, verification subclasses

### Takeaway

The family has a precedent for nearly every extension we need, but each covers one axis: DSDEVS for spawn/kill and replicas, ml-DEVS/EB-DEVS for inter-level causation, MRT-DEVS and related work for switching resolution mid-run, RDEVS for routers, FD-DEVS/SP-DEVS and RT-DEVS-to-UPPAAL for exhaustive verification. None combines symbolic *data* inputs, nondeterministic forking and untimed partial-order traces; "Symbolic DEVS" is symbolic in *time*, not inputs.

### Cited Findings

#### Dynamic structure (DSDEVS, DSDE, DynDEVS)

- Barros proposed DSDEVS, composed of atomic and network-executive components, to represent systems that change structure; DSDEVS could not handle simultaneous events so the parallel DSDE was proposed (WSC 1998) — [Barros, WSC 1998](https://www.informs-sim.org/wsc98papers/056.PDF); summary via [Attia et al., arXiv:2404.03661](https://arxiv.org/abs/2404.03661)
- Because a coupled model has no state of its own, a network executive manages the structural state of its scope; its model is a modified basic model with a structure function mapping its states to network structures; some implementations drop the explicit executive and intercept structure-changing messages instead — [search summary of arXiv:2404.03661 and Barros WSC 1998](https://arxiv.org/abs/2404.03661)
- DSDEVS treats model configuration as part of the state managed by a network executive; it is a mathematical model with implementations free to differ; it suits infrequent structural changes, not highly dynamic models — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- PythonPDEVS implementation: opt-in, "an option for local simulation only" (`setDSDEVS(True)`); `modelTransition(state)` is called after each transition and returns True to propagate a structure-change request to the parent; atomic models may only add/remove ports; coupled models may create/delete children, connections and ports; "After a model is removed, there is no way to get it back"; the check is "relatively costly" so disabled by default; `modelTransition` must not change model state (only the passed dict) — [PythonPDEVS docs, Dynamic Structure](https://msdl.uantwerpen.be/documentation/PythonPDEVS/dynamicstructure.html)
- A 2024 benchmark of dynamic-structure formalisms (focus on DEVS extensions) finds most approaches work for some system types and poorly for others, and identifies gaps — [Attia, Foucher & Lavado Villa, arXiv:2404.03661](https://arxiv.org/abs/2404.03661) (abstract only; compared formalisms not confirmed)
- Dynamic structure SOA-DEVS: Muqsith & Sarjoughian combined SOA-compliant DEVS with DSDEVS for structural changes in service composition — [SIMULATION journal record](https://api.crossref.org/works/10.1177%2F0037549710382431) (search snippet)

#### Multi-level and multi-resolution

- ml-DEVS separates MICRO-DEVS and MACRO-DEVS models, models upward and downward causation explicitly (split into information and activation), and restricts inter-level causation to neighboring levels — [Steiniger, Krüger & Uhrmacher, WSC 2012](https://www.informs-sim.org/wsc12papers/includes/files/con445.pdf); [WSC 2014 linked-lives comparison](https://informs-sim.org/wsc14papers/includes/files/267.pdf) (search snippets)
- EB-DEVS (Foguelman, Henning, Uhrmacher, Castro) adds upward/downward channels to DEVS to model emergent behavior — [arXiv:2010.05042](https://ar5iv.labs.arxiv.org/html/2010.05042)
- MRT-DEVS (2022) embeds state and event translation functions in model specifications so resolution conversion happens inside models; model exchange is traditionally complex because states must be converted when resolution changes — [MDPI Systems 10(5):174](https://www.mdpi.com/2079-8954/10/5/174/html) (search snippet)
- Liu & Verbraeck's quotient-space multi-resolution framework on DEVS defines a "false-preserving" principle (a relation absent at low resolution must be absent at high resolution) and a "true-preserving" principle — [TU Delft repository](https://repository.tudelft.nl/record/uuid:a131b1f7-fef2-416a-bf26-80953bd57862) (search snippet)
- Runtime switching of abstraction levels has traditionally required identical interfaces between detailed and abstract components, called "a critical limitation" — [Towards Multi-Level-Simulation using Dynamic..., SciTePress 2016](https://scitepress.org/Papers/2016/59975/pdf/index.html) (search snippet)
- Switching from an abstract to a refined model needs extra information the modeler must supply as domain knowledge — [IoT large-scale simulation, arXiv:1802.09207](https://arxiv.org/pdf/1802.09207) (search snippet)

#### Symbolic DEVS

- Zeigler & Chi, "Symbolic discrete event system specification", IEEE Trans. SMC 22(6), 1992: event times are expressed symbolically by changing the time base from the reals to linear polynomials over the reals, with a consistency-checking algorithm for linear polynomial constraints adapted from LP feasibility — [IEEE record via search](https://manuscript.isc.ac/Inventory/49/864823.htm) (abstract only)

#### Fuzzy / possibilistic / non-deterministic DEVS

- Joslyn (1996) extended DEVS with possibilistic and fuzzy processes, placing deterministic, non-deterministic, stochastic, fuzzy and possibilistic processes side by side; "classical DEVS are entirely deterministic" — [Joslyn, JoC96a](https://CliffJoslyn.github.io/Docs/JoC96a.pdf) (search snippet)
- Fuzzy-DEVS and imprecise DEVS (Bisgambiglia et al., FUZZ-IEEE 2008–09) generalize DEVS to imprecision in events, transitions and state — [HAL hal-01083102](https://hal.archives-ouvertes.fr/hal-01083102); [HAL hal-00563248 (DEVS-Flou)](https://hal.archives-ouvertes.fr/hal-00563248)
- I found no source for a "Kwon" non-deterministic DEVS.

#### Routed DEVS (RDEVS)

- RDEVS (Blas, Gonnet, Leone; WSC 2017) is a subclass of classic DEVS adding a routing layer where each node pairs a behavioral description with a routing policy; foundational journal paper in *Software and Systems Modeling* 21 (2021/22) 1179–1208 — [WSC 2017](https://www.informs-sim.org/wsc17papers/includes/files/057.pdf); [SoSyM](https://link.springer.com/article/10.1007/s10270-021-00928-4); later work on event tracking and a design-pattern redesign — [WSC 2023](https://www.informs-sim.org/wsc23papers/satwcea111.pdf)

#### Cell-DEVS

- Cell-DEVS combines cellular automata with the DEVS simulation basis — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697); Cadmium v2 supports classic and asymmetric Cell-DEVS — [Cadmium v2 repo](https://github.com/SimulationEverywhere/cadmium_v2)

#### Verification subclasses and translations

- FD-DEVS (Hwang & Zeigler) restricts state and event sets to be finite and makes external transitions independent of elapsed time, which yields a finite reachability graph equivalent to the network's behavior (IEEE T-ASE 2009) — [IEEE record](https://manuscript.isc.ac/Inventory/49/1067656.htm); [WSC 2013 paper](https://informs-sim.org/wsc13papers/includes/files/391.pdf)
- SP-DEVS (schedule-preserving) and FD-DEVS were designed for verification; FD-DEVS was originally called "schedule-controllable DEVS"; tools DEVS# (C#) and XSY (Python) support reachability-graph checks for safeness and liveness, and min/max processing-time bounds for SP-DEVS — [Wikipedia, DEVS](https://en.wikipedia.org/wiki/DEVS); deadlock/livelock decidability per [Wikiwand FD-DEVS](https://www.wikiwand.com/en/articles/FD-DEVS) (Wikipedia-derived; verify against originals)
- FD-DEVS is fully deterministic and does not capture stochastic behavior well — [Wikipedia FD-DEVS](https://www.wikipedia.com/wiki/Finite_%26_Deterministic_Discrete_Event_System_Specification)
- ES-DEVS is more expressive than FD-DEVS while remaining verifiable via timed-automata reachability — [WSC 2013](https://informs-sim.org/wsc13papers/includes/files/391.pdf) (search snippet)
- RT-DEVS → UPPAAL translation for exhaustive verification (Furfaro & Nigro 2008); classic DEVS ↔ timed automata via bisimulation (Inostrosa-Psijas et al.); UPPAAL supports only a TCTL subset without timed modalities, so a 2024 paper adds automata observers and property mutation for quantitative properties — [González et al., arXiv:2409.18732](https://arxiv.org/html/2409.18732v3); [FedCSIS 2008](https://annals-csis.org/proceedings/2008/pliks/62.pdf)

### Inferences

- **DSDEVS is the precedent for replicas, spawned processes and restarts.** The network-executive idea (structure is state held by a designated component of the parent) maps well onto our model if the executive is the entity that owns the lifecycle (an orchestrator, a supervisor, an autoscaler). PythonPDEVS's "removed models cannot be restored" and "local only" restrictions are warnings for reverse stepping: structure changes must be journaled like any other state if we want reverse execution.
- **Symbolic DEVS is not what we need.** It symbolizes timestamps, not data. Our symbolic run (unbound predicates fork) has no DEVS-family counterpart I found; the closest analogs are possibilistic/non-deterministic DEVS (Joslyn), which keep sets of possible states but do not track path conditions over named predicates.
- **FD-DEVS is the closest verification precedent to our untimed model.** If presentation time is ignored, our behaviors are finite (closed predicate vocabulary, closed state domains) and external transitions do not depend on elapsed time, which is FD-DEVS's restriction. The reachability-graph construction therefore suggests exhaustive exploration is decidable for the finite fragment, but FD-DEVS is deterministic, so our nondeterministic scheduling and unbound predicates exceed it.
- **Multi-resolution switching mid-run is hard because of state translation.** MRT-DEVS and the "identical interfaces" limitation say that replacing a coarse model by a fine one during a run requires mapping the coarse state into a fine state, which is underdetermined. Our debugger can avoid this: since runs are deterministic given binding and schedule, "expand a collapsed segment" can re-run from the start (or a checkpoint) at the finer level and align, rather than translating state at the switch point.
- Liu & Verbraeck's false-preserving principle is a usable consistency check for our two levels: a behavior absent from the summary must not appear in the full graph (no new effects at ports), which is the report's refinement obligation in other words.
- ml-DEVS's "neighboring levels only" restriction matches our projection rule (an event lifts to the deepest *visible* ancestor) and suggests one-level-at-a-time zoom transitions as the safe default.
- RDEVS's "behavior + routing policy" node is a good template for boundary entities (LB, router, gateway): separate the routing decision (a decision point, never trivial) from the forwarded behavior.

### Gaps

- Barros's DSDEVS formal definitions (1995/1997 TOMACS) and DynDEVS (Uhrmacher) were not read; claims rest on WSC 1998 and secondary summaries.
- Which formalisms Attia et al. benchmark, and their criteria, were not retrieved.
- No source found for non-deterministic DEVS with symbolic data inputs or path conditions.

## 4. Tooling and standards status by late 2026; DEVS used to model software systems, networks, microservices and cloud

### Takeaway

DEVS tooling is alive but academic: PythonPDEVS 2.4.1, Cadmium v2 (C++17, still pre-release), adevs (C++, fastest), DEVS-Suite 7.0.0 (Jan 2023), xDEVS. There is no ratified DEVS standard; standardization produced book chapters (c. 2010) and author-driven interoperability proposals (2024 arXiv). DEVS has been used to model SOA/service systems and fog/edge infrastructure, and in 2025–2026 LLM-to-DEVS generation became an active topic, the closest precedent to our "LLM compiles code into a model" stage.

### Cited Findings

#### Tools

- PythonPDEVS documentation is for version 2.4.1; supports Classic and Parallel DEVS, Dynamic Structure DEVS, tracers, termination conditions, distributed simulation via MPI/PyRO with Time Warp, realtime simulation, activity tracking with automatic relocation — [PythonPDEVS docs](https://msdl.uantwerpen.be/documentation/PythonPDEVS/)
- Cadmium v2: "Object-Oriented implementation of a PDEVS simulator for C++ 17", header-only, supports classic and asymmetric Cell-DEVS, MIT licence; README says the new version is still in development with a first official release planned; no releases listed — [Cadmium v2 GitHub](https://github.com/SimulationEverywhere/cadmium_v2)
- adevs: C++ library for discrete event and hybrid dynamic system simulation implementing DEVS; tests require SUNDIALS and fmi-library; no tagged releases — [adevs GitHub](https://github.com/smiz/adevs)
- DEVS-Suite 7.0.0 (January 2023, ASU ACIMS, Sarjoughian) adds Action-Level Real-Time DEVS, a black-box testing framework and FMU co-simulation; some pages still list 6.1 (April 2021) — [ACIMS DEVS-Suite](https://acims.asu.edu/devs-suite); [SourceForge wiki](https://sourceforge.net/p/devs-suitesim/wiki/Home/?version=7)
- DEVStone metric (12 models, runs per minute) standardizes simulator benchmarking — [ACM TOMACS 2022, arXiv:2309.16544](https://arxiv.org/abs/2309.16544)

#### Standards

- "Standardizing DEVS model representation" and "Standardizing DEVS Simulation Middleware" are chapters (17, 18) in *Discrete-Event Modeling and Simulation: Theory and Applications* (Taylor & Francis, dated 2010 by Carleton; a UCM record says 2016 — conflict); goals were a platform-independent model representation and interoperability middleware; the motivation was the proliferation of language-specific DEVS engines — [Carleton, Standardizing DEVS model representation](https://cell-devs-02.sce.carleton.ca/publications/2010/WADHMMSTTZ10); [Carleton, Middleware chapter](https://cell-devs-02.sce.carleton.ca/publications/2010/WADHMMSTTZ10a); [UCM record](https://produccioncientifica.ucm.es/documentos/63cc8fffab05b07b6665e3f5?lang=en)
- Mittal, Zeigler & Risco-Martín (arXiv 2024) present a formal interoperability standard for M&S / system-of-systems integration implemented in DEVS/SOA, extending their 2008 work — [arXiv:2407.20696](https://export.arxiv.org/abs/2407.20696)
- A standard XML format for FD-DEVS (Mittal and co-workers) is used for UML execution — [Wikipedia, DEVS](https://en.wikipedia.org/wiki/DEVS)
- I found no evidence of an IEEE/SISO-ratified DEVS standard as of 2026.

#### DEVS applied to software, services, networks, cloud

- SOA-compliant DEVS (SOAD) for modeling service-oriented computing systems with throughput/timeliness QoS experiments (Sarjoughian et al., WSC 2008); SOC-DEVS co-design simulator for service-based software (SIMUTOOLS 2010) — [Academia.edu copy](https://www.academia.edu/18755659/A_simulation_framework_for_service_oriented_computing_systems); [EUDL SIMUTOOLS 2010](https://eudl.eu/pdf/10.4108/ICST.SIMUTOOLS2010.8735) (search snippets)
- DEVS/SOA: Java client/server framework with run-time composability of coupled models — [Mittal, Risco-Martín & Zeigler, arXiv:2407.03686](https://arxiv.org/abs/2407.03686)
- DEVSML 3.0 stack uses Docker microservices to deploy a DEVS simulation farm (DEVS *on* microservices, not a model *of* microservices) — [Mittal & Risco-Martín 2017](https://www.ucm.es/jlrisco/file/c-mittal2017)
- Cloud-enabled DEVS parallel/distributed simulation architecture without model modification — [Risco-Martín et al., SIMPAT 118 (2022), arXiv:2302.11242](https://arxiv.org/abs/2302.11242)
- Mercury: DEVS-based M&S&O framework for fog/edge computing on xDEVS (sequential, parallel or distributed without model changes), with FaaS offloading and a detailed 5G radio model — [Cárdenas et al., arXiv:2312.02172](https://arxiv.org/abs/2312.02172)
- A 2025 preprint argues DEVS-based architecture as a formal basis for integrating software systems (DoD MOSA) — [Preprints.org 202510.1207](https://www.preprints.org/manuscript/202510.1207) (search snippet)

#### LLM + DEVS (2025–2026)

- PDEVS-LLM (WSC 2025): agentic framework whose PDEVS-statechart grammars let LLMs generate atomic model behavior from textual descriptions — [WSC 2025 con178](https://www.informs-sim.org/wsc25papers/con178.pdf) (search snippet)
- Vasa, Sarjoughian & Yellig (arXiv, 15 Aug 2026): LLM extracts "plausible facts" and "behavioral conditions" from natural language, checks facts against conditions by propositional entailment in a bounded correction loop, then generates PDEVS statecharts; each statechart's timed-automata counterpart is built *manually* and checked for deadlock and reachability — [arXiv:2608.14956](https://arxiv.org/abs/2608.14956)
- DEVS-Gen (Chen et al., arXiv 2603.03784v2, May 2026): two stages — top-down structural planning into a PlanTree (port schemas, state variables, event/timing logic), then parallel per-component behavioral synthesis as Python modules on xDEVS, with a bottom-up step extracting "as-implemented" interface summaries so parent couplings match actual child interfaces; traces are JSONL tuples (t, entity, event type, payload); scoring = operational (runs, schema-valid trace) plus behavioral (micro consistency rules and macro causality rules); GPT-5.2 single-pass Score_beh 0.87, Score_ope 0.90; enforcing strict xDEVS constraints imposed a "cognitive penalty" (GLM-4.7 0.45/0.30 vs 0.95/0.74 for DEVS-Gen); LLMs introduce naming/signature variations; iterative agents can enter non-terminating debugging loops — [arXiv:2603.03784](https://arxiv.org/html/2603.03784)
- RTSync (vendor) markets LLM extraction of DEVS models from documents into DEVS Java with verification in MS4 Me (marketing, not peer reviewed) — [RTSync](https://rtsync.com/llm-assisted-automated-generation-of-devs-models/)

### Inferences

- No DEVS tool is a usable engine for us off the shelf: all assume concrete values and a single schedule; none offers symbolic inputs, forking or exhaustive interleaving exploration. PythonPDEVS is the best *reference implementation* to read (classic + parallel + dynamic structure in one codebase).
- DEVS-Gen's "bottom-up as-implemented interface summaries" is the same move as the report's per-port behavior generated in one bounded LLM call and reconciled against wires; its finding that strict formal constraints hurt LLM output supports the report's "free-text analysis before decisions, then a checker" rather than asking the LLM to emit the formalism directly.
- DEVS literature on software is mostly *performance* modeling of service systems (throughput, QoS, sizing), not *functional* behavior of code paths. Our use (explaining what code does) is outside DEVS's mainstream.

### Gaps

- xDEVS repository status and version as of 2026 not checked.
- Authors and contents of the WSC 2025 PDEVS-LLM paper not read beyond the snippet.
- No paper found that uses DEVS to model a specific microservice application's request flow (as opposed to infrastructure or simulation farms).

## 5. How DEVS handles time vs our needs (logical order, presentation time, untimed causality)

### Takeaway

DEVS is built on a continuous real time base: ta maps states to ℝ≥0 ∪ {∞}, ∞ means passive (wait for input), 0 means a transitory state. Our model is mostly untimed: it needs happens-before, not timestamps. We can embed our steps in DEVS with ta ∈ {0, ∞}, but then all causality collapses onto one timestamp and ordering is decided by select/bags or super-dense time, which is precisely where DEVS's documented pitfalls (zero-time loops, ordering ambiguity) live.

### Cited Findings

- ta "can be any positive real number, including zero and infinity"; ta = 0 is used for artificial transitory states — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- A loop of zero-time-advance models can cause "a livelock of the simulation" — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Because λ is only invoked before δint, an output in reaction to an external event needs an artificial transitory state (the traffic-light example uses `going_manual`/`going_auto`); ta and λ cannot modify state, so interrupts need a `remaining_time` counter — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Self-influence is disallowed because the order of simultaneous internal and external transitions would be undefined — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697)
- Events scheduled with zero time advance are handled sequentially, which can cause ordering problems; super dense time (Nutaro 2020) augments reals with ordering information for apparently simultaneous events, but sometimes modelers need same-time events treated as simultaneous, which super dense time does not provide — [WSC 2020 paper 230](https://informs-sim.org/wsc20papers/230.pdf) (from search-result quotations; full PDF not extracted)
- Symbolic DEVS replaces the real time base with linear polynomials, so time can be symbolic — [Zeigler & Chi 1992 record](https://manuscript.isc.ac/Inventory/49/864823.htm)
- FD-DEVS makes external transitions independent of elapsed time to obtain finite reachability graphs — [IEEE record, Hwang & Zeigler 2009](https://manuscript.isc.ac/Inventory/49/1067656.htm)

### Inferences

- Our three clocks map cleanly: logical order is DEVS's *causal* structure stripped of timestamps; presentation time is a view (never feed it to ta); observed time is DEVS-style real time and could drive a timed DEVS run later (timeouts, retries with backoff, TTLs, `stale-15m` predicates).
- Timeouts and TTLs are the one place we need real time semantics. DEVS's ta and elapsed-time e are the right tool there; the report's predicate `over: time` should be interpreted as DEVS-style timers (a state with finite ta) rather than as free predicates, otherwise a symbolic run can fire a timeout before the request it guards.
- A DEVS trace is a timed sequence; ours is a pomset. Two events at the same DEVS timestamp are "simultaneous", which is not the same as "concurrent" (incomparable in happens-before). We should not adopt DEVS's notion of simultaneity as our notion of concurrency.
- The "output only before δint" rule mirrors our need for a request/reply port that is "held while pending": in DEVS a synchronous call is two events plus an intermediate state, never one step. This supports modeling sync calls in our engine as send + wait-state + reply, with the port-held state explicit.
- Zero-time-loop livelock is the DEVS analog of our "incomplete model is a run that gets stuck" — but inverted: DEVS gets stuck by *infinite* activity at one instant, ours by *no* enabled transition. We need both checks (Zeno-like forwarding cycles through wires, and stuck configurations).

### Gaps

- Nutaro's super-dense-time paper itself (2020) and the WSC 2020 paper's algorithm were not read.
- No DEVS variant found with a purely logical (untimed, partial-order) time base.

## 6. Mapping table: DEVS concept → our concept, with gaps

### Takeaway

Structure maps almost one-to-one (atomic → leaf entity, coupled → open entity, port → port, EIC/EOC → delegation, IC → assembly wire, Z_i,j → wire adapter, network executive → lifecycle owner). Behavior maps partially (δext ≈ per-port behavior, δint ≈ internal/timer step, λ ≈ effects). Traces, symbolic inputs, nondeterminism, shared state and macro steps do not map: they are where our design must go beyond DEVS.

### Cited Findings

- All structural rows below rest on the classic and port-based coupled definitions — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697); [Wikipedia, DEVS](https://en.wikipedia.org/wiki/DEVS); network executive — [Barros WSC 1998](https://www.informs-sim.org/wsc98papers/056.PDF); our concepts from `report.md` and `checkpoint-01.md` (project files).

| DEVS concept | Our concept | Fit | Notes / gap |
| --- | --- | --- | --- |
| Atomic model ⟨X,Y,S,δint,δext,λ,ta⟩ | Leaf entity (closed) with its port behaviors and state vars | Good | DEVS has one δext for all inputs; we split behavior per provided port. Equivalent if δext dispatches on port. |
| Coupled model | Open entity (interior of parts) | Good | DEVS coupled has no state of its own; our entities may own state (stores). Put owned state in a child store entity to stay compatible. |
| Closure under coupling (resultant atomic) | Zoom out: a closed entity is step-able as one unit | Good as definition | Resultant is an exact product; our summary is an abstraction. Need a morphism/refinement check on top. |
| Input/output ports | Ports (provides/requires, protocol) | Partial | DEVS ports carry values with no direction-pair typing; provides/requires conjugation (SysML) is ours. |
| EIC / EOC | Delegation wires | Good | |
| IC | Assembly wires | Good | |
| Z_i,j translation (classic) | Wire `adapter` (DNAT, deserialize) | Good for stateless | Stateful transformations (TLS) must be entities in DEVS too. Port-based PDEVS dropped Z. |
| select (classic) / bags + δconf (PDEVS) | Scheduler choice in exhaustive mode | Inverted | DEVS fixes one order; we must enumerate. |
| δext(s, e, x) | Per-port behavior case triggered by an arriving message | Good | DEVS state is concrete; our guards are predicates, possibly unbound (fork). |
| δint, ta | Internal steps: timers, retries, sweepers, background loops | Partial | Untimed parts map to ta = 0 (enabled now) or ∞ (passive). |
| λ | Effects: sends on required ports | Partial | DEVS emits only before δint; our cases interleave sends and writes. Encode as transitory states. |
| ta = ∞ passive state | Idle entity waiting for a message | Good | A passive DEVS model is legal; our "stuck" = passive *with messages pending that no behavior accepts*. |
| Network executive (DSDEVS) | Lifecycle owner: orchestrator, supervisor, autoscaler; spawn/kill | Good | Structure becomes state; must be journaled for reverse stepping. |
| Abstract simulator / coordinator tree | Engine + projection to entity tree | Good | Execute flat; reconstruct hierarchy for display. |
| Simulation trace (timed event sequence) | Script (pomset under a binding) | Poor | DEVS traces are total orders by time; ours are partial orders with conflict (forks). |
| — | Predicates, input classes, symbolic run, assumptions | None | Symbolic DEVS symbolizes time only. |
| — | Shared state (DB rows) accessed by several entities | None (by design) | DEVS forbids shared state across models; all interaction is via ports. Our stores-as-entities with read/write ports restore this discipline. |
| Multi-resolution switch (MRT-DEVS) | Expanding a collapsed segment / opening an entity mid-run | Partial | DEVS needs state translation at switch; we can re-run instead. |
| ml-DEVS up/down causation | Projection of events to deepest visible ancestor; platform floors | Partial | ml-DEVS restricts to neighboring levels. |
| FD-DEVS reachability graph | Exhaustive exploration | Partial | FD-DEVS deterministic; ours nondeterministic in schedule and inputs. |
| — | Macro step (collapsed segment with `atomicity`) | None | DEVS has no notion of a named, collapsible sub-trace; closure collapses *structure*, not *time segments*. |
| — | Provenance, anchors to code | None | Outside the formalism. |

### Inferences

- Closure under coupling collapses space (an entity), not time (a segment). Our "zoom in time" needs a separate notion — macro step as a stuttering-equivalent or Lipton-reducible segment, as the report already proposes — so DEVS precedent covers only half of the zoom story.
- DEVS's strict "no shared state, everything through ports" discipline is a strength we partially adopt: stores as entities with read/write ports make shared-state races visible as port interleavings, which exhaustive exploration can enumerate.

### Gaps

- Whether ZPK's morphism theory can express "same visible result at ports when run coarse vs fine then collapsed" was not verified from the text.

## 7. Recommendations: adopt, adapt, avoid

### Takeaway

Adopt DEVS's structural core and closure-under-coupling as the formal meaning of space zoom; adapt its transition functions into per-port guarded behaviors over predicates and its abstract simulator into a flat engine with a hierarchical projection; avoid its deterministic tie-breaking, its real-valued time as the primary clock, and its "translate state at switch time" approach to multi-resolution.

### Cited Findings

- Recommendations below are design inferences built on findings cited in sections 1–6; key anchors: closure construction and pitfalls — [arXiv:1701.07697](https://arxiv.org/abs/1701.07697); flattening performance — [arXiv:2309.16544](https://arxiv.org/abs/2309.16544), [WSC 2019](https://www.informs-sim.org/wsc19papers/006.pdf); dynamic structure — [PythonPDEVS DS docs](https://msdl.uantwerpen.be/documentation/PythonPDEVS/dynamicstructure.html); verification — [Hwang & Zeigler 2009](https://manuscript.isc.ac/Inventory/49/1067656.htm), [arXiv:2409.18732](https://arxiv.org/html/2409.18732v3); LLM generation — [arXiv:2603.03784](https://arxiv.org/html/2603.03784), [arXiv:2608.14956](https://arxiv.org/abs/2608.14956); multi-resolution — [MRT-DEVS](https://www.mdpi.com/2079-8954/10/5/174/html)

### Inferences

#### Adopt verbatim

1. Closure under coupling as the definition of "a closed entity is a step-able unit": the behavior of a closed entity *is* the resultant of its interior (product state, children's enabled steps, wires as influencee/translation maps). The summary level is then a checked abstraction of that resultant.
2. The structural vocabulary: atomic/coupled, ports, EIC/EOC/IC, plus classic DEVS's per-connection translation function as the formal precedent for wire adapters (cite Z_i,j; limit to stateless transforms).
3. The "no self-influence, no shared state outside ports" discipline, enforced by the validator.
4. The network-executive pattern for structure change: a named entity owns spawn/kill/scale, and structure is part of the configuration.

#### Adapt

5. Transition functions → per-port guarded behaviors over predicates; δint/ta → timers and background loops only; untimed steps use ta ∈ {0, ∞} semantics (enabled now / passive).

6. Abstract simulator → flat execution over all leaf behaviors (faster, per DEVStone flattening results; also required for exhaustive exploration), with the coordinator tree used only for projection, stepping (step-out = run until the coordinator's `done`) and breakpoints.

7. select / δconf → explicit scheduler choice points: concrete run picks one and records it in the trace (so replay is deterministic); exhaustive run branches over all; partial-order reduction prunes independent pairs.

8. FD-DEVS's finiteness restrictions (finite states and events, no dependence on elapsed time) as the definition of the "exhaustively explorable fragment"; flag model parts outside it (unbounded loops, real timers) instead of silently bounding them.

9. Multi-resolution: instead of translating state at a switch point (MRT-DEVS's hard problem), re-execute the run (same binding and schedule) at the finer level and align; use the false-preserving principle (no effect at a port in the fine run that is absent from the summary) as the refinement check.

10. LLM generation: follow DEVS-Gen's staged plan (structure top-down, behaviors per component in parallel, bottom-up interface reconciliation) and emit trace records shaped like (time/order, entity, event kind, payload) so conformance checks are mechanical.

#### Avoid

11. DEVS simultaneity as concurrency. Use happens-before (pomsets); timestamps are presentation or observation only.

12. Real-valued ta as the primary clock; reserve timed semantics for timeouts, TTLs, backoff.

13. Unchecked zero-time cycles (DEVS livelock): validate that wire/adapter cycles without a consuming behavior cannot loop.

14. Irreversible structure change (PythonPDEVS "no way to get it back"): journal structural edits for reverse stepping.

15. Asking the LLM to emit a strict formalism directly (DEVS-Gen's "cognitive penalty"); keep the LLM on structured records with a checker in the loop, and do not rely on manual timed-automata translations for verification as the 2026 statechart paper does.

### Gaps

- No empirical evidence yet that DEVS-style closure is sufficient for the UI's "coarse run then fine run then collapse gives the same visible result"; that needs our own refinement check.
- Whether existing PDEVS engines could serve as the concrete-run backend (with our own symbolic/exhaustive layer above) was not evaluated.
