# Operational semantics of executable modeling languages (foundation for the model interpreter)

Reading notes for the writer:

- Evidence tiers. **[V]** = the claim was read in a search result or fetched page during this session (November 2026). **[B]** = the claim is from the researcher's background knowledge of a well-known primary source; the URL is the canonical reference but was **not opened in this session** (PDF extraction tools were unavailable, and several publisher pages were paywalled or bot-blocked). Treat [B] items as needing a spot check before they are quoted verbatim.
- "Superseded/old" marks results that are historical foundations rather than current practice.
- Little genuinely new (2025–2026) work on the *semantics* of these languages was found; the field's definitions are stable and date from 1981–2020. The recent work found is on reduction/atomicity (CIVL, 2026) and on Lingua Franca coordination (2024). See Gaps.

## 1. SOS basics and executable semantics frameworks

### Takeaway

Every language studied defines its meaning as a labeled transition relation over *configurations* (small-step SOS), and the modern executable-semantics frameworks (K, Redex, DynSem, GEMOC) all make that one definition directly runnable and reuse it for debugging, exploration and symbolic execution. Small-step is the right base for us because interleaving, stuck states and non-termination are only expressible there; big steps should be *derived* (as macro steps), never primitive.

### Cited Findings

- [B] Plotkin's Structural Operational Semantics defines behavior by inference rules over a transition relation between configurations (program fragment plus state), with labels for observable actions; concurrency is modeled as interleaving of component transitions. Foundational, 1981, still the standard. — [Plotkin, "A Structural Approach to Operational Semantics"](https://homepages.inf.ed.ac.uk/gdp/publications/sos_jlap.pdf)
- [V] Terminology clash to avoid: in programming-language theory "big-step semantics" means natural semantics relating a program to its final result; in the modeling-language literature a "big step" is a model's reaction to one environmental input, made of a sequence of small steps. The search results for "big-step semantics" mix both senses. — [Esmaeilsabzali & Day, FASE 2010 (Springer)](https://link.springer.com/doi/10.1007/978-3-642-12029-9_12); [PL-sense example: "A meta-theory for big-step semantics"](https://ar5iv.labs.arxiv.org/html/2104.05558)
- [V] K: an operational semantics written once is used for symbolic execution and verification. The K prover "uses the semantics for symbolic execution", reasons over patterns and calls Z3 for constraint satisfiability. — [Runtime Verification blog, K vs Coq part 3](https://runtimeverification.com/blog/k-vs-coq-as-language-verification-frameworks-part-3-of-3)
- [V] All-path reachability logic is a language-independent proof system that takes "any unconditional operational semantics as axioms", is sound and relatively complete, and targets non-deterministic (concurrent) languages; implemented in K. RTA 2014, journal version LMCS 2019. — [Ștefănescu et al., arXiv 1810.10826](https://arxiv.org/abs/1810.10826v2); [FSL publication page](https://fsl.cs.illinois.edu/publications/stefanescu-ciobaca-mereuta-moore-serbanuta-rosu-2014-rta.html)
- [B] K organizes a configuration as nested labeled *cells* (code, environment, store, threads, message buffers) and rules rewrite parts of cells; this is the most direct precedent for a structured configuration tuple. — [K framework](https://kframework.org/)
- [B] PLT Redex (Racket) gives runnable reduction semantics with evaluation contexts, a stepper/trace visualizer and randomized testing of metatheory; a lightweight way to prototype and test a semantics before implementing it. — [PLT Redex](https://redex.racket-lang.org/)
- [B] Spoofax DynSem is a DSL for dynamic semantics specifications from which an interpreter is derived; Rascal offers similar meta-programming for interpreters. — [Spoofax](https://www.spoofax.dev/); [Rascal](https://www.rascal-mpl.org/)
- [V] GEMOC Studio: a common API lets "any kind of metaprogramming approach used to define discrete-event operational semantics" plug into an execution engine; generic runtime services (omniscient debugging, animation) are shared across engines. — [Eclipse GEMOC](https://eclipse.org/gemoc); [Execution Framework of the GEMOC Studio (tool demo), HAL](https://hal.archives-ouvertes.fr/hal-01355391)
- [V] GEMOC omniscient debugging for xDSMLs targets operational (not translational) semantics; the engine observes *execution steps* and records a trace so the debugger can go backwards. — [Bousse, Corley, Combemale et al., SLE 2015](https://gray.cs.ua.edu/pubs/sle-2015.pdf)
- [V] A 2022 SoSyM paper shows one generic GEMOC engine supporting two ways of specifying concurrency in the operational semantics, and uses it for concurrent omniscient debugging. — [SoSyM 2022, doi 10.1007/s10270-022-01073-2](https://link.springer.com/content/pdf/10.1007/s10270-022-01073-2.pdf)
- [V] GEMOC also maps execution of a compiled (translational) target model back to source-level steps through a feedback manager — a precedent for lifting co-simulation results back into the model. — [Domain-level debugging for compiled DSLs (MODELS 2017 demo)](https://se.jku.at/domain-level-debugging-for-compiled-dsls-with-the-gemoc-studio-tool-demo/)

### Inferences

- The checkpoint's roles (model = program, engine = interpreter, script = trace, UI = debugger) are exactly the GEMOC architecture: an xDSML whose semantics *is* the engine's step function, with step-begin/step-end notifications feeding generic debugger services. The step granularity the engine reports is the debugger's stepping unit, so the semantics must name its steps.
- K's cell structure and "one semantics, many tools" result justify defining a single `enabled`/`step` pair and deriving all three run modes from it (section 6).
- Prototyping the rules in Redex or K before writing the Rust engine is cheap insurance: random testing of "macro step agrees with small steps" is precisely what Redex's property testing is for.

### Gaps

- DynSem and Redex primary papers were not opened; claims are background knowledge.
- No source was found comparing these frameworks on trace-recording cost for reverse stepping at our scale.

## 2. Concrete language semantics compared

### Takeaway

The languages split into three families: (a) statechart family, where a macro step is a chain of micro steps and the variants differ on a small set of orthogonal choices; (b) actor/communicating-machine family (P, Rebeca, Erlang), where the atomic unit is "dequeue one message and run its handler to completion" and nondeterminism lives in the scheduler; (c) coordination-layer family (BIP, Ptolemy II, Lingua Franca, KPN), where composition glue or a director fixes the scheduling discipline, in the limit making the model deterministic. Our record model (per-port guarded automata, wires, stores) sits in family (b) with a BIP-like explicit glue layer for wires and adapters.

### Cited Findings

#### Statechart family

- [V] STATEMATE (Harel & Naamad, TOSEM 1996) was the first executable statechart semantics. A *step* moves the system from one status to the next; reactions to events generated in a step are sensed only after the step completes, and events live for only the following step. STATEMATE offers both a synchronous time model and an asynchronous one (in which steps chain into a superstep before the environment is sampled again). Old but foundational. — [Harel & Naamad paper copy](https://chess.eecs.berkeley.edu:443/design/2009/discussion/Pdf/p293-harel.pdf); [Finn's presentation of the paper](https://chess.eecs.berkeley.edu/design/2010/discussions/slides/john_finn_statecharts.pdf)
- [V] The Harel–Naamad description is informal and imprecise in places; slides citing von der Beeck note about 20 statechart variants with no official semantics. — [search-result summary of secondary sources incl. Roscoe et al.](https://cs.ox.ac.uk/oucl/work/bill.roscoe/publications/115.pdf); [Statemate slides](https://chess.eecs.berkeley.edu/design/2010/discussions/slides/Statemate.pdf)
- [V] Esmaeilsabzali, Day, Atlee, Niu, "Deconstructing the semantics of big-step modelling languages", Requirements Engineering 15(2):235–265, 2010: a big step is deconstructed into orthogonal semantic aspects that capture the main differences among most BSMLs; a BSML is a language where a model reacts to an environmental input via a sequence of small steps, each possibly a set of concurrent transitions. — [dblp record](https://dblp1.uni-trier.de/rec/journals/re/EsmaeilsabzaliDAN10.html); [author PDF](https://cs.uwaterloo.ca/%7Ejmatlee/Papers/REJ10c.pdf); [FASE 2010 companion](https://link.springer.com/doi/10.1007/978-3-642-12029-9_12)
- [B, partially V] The aspects in that taxonomy are: concurrency and preemption (one vs many transitions per small step), big-step maximality (syntactic / take one / take many), internal event lifeline (remainder of big step / next small step / next combo step), environmental input events, interface events, enabledness memory protocol (guards read values from start of big step vs current small step), order of small steps, and priority (hierarchical, explicit, negation of triggers). The search snippet confirmed the option names "Many", "Take Many" and "Present in Next Combo Step"; the full list was not re-read. — [author PDF](https://cs.uwaterloo.ca/%7Ejmatlee/Papers/REJ10c.pdf)
- [V] SCXML (W3C Recommendation, 2015; unchanged since): a *microstep* is "the execution of the transitions in an optimal enabled transition set"; a *macrostep* is "a series of one or more microsteps ending in a configuration where the internal event queue is empty". Two queues: `<raise>` appends to the internal queue; external events wait in the external queue. Order: eventless transitions first, then internal events until the internal queue is empty, then (macrostep over) `<invoke>` for newly entered states, then block on the external queue. — [W3C SCXML §3.13](https://www.w3.org/TR/scxml/)
- [V] SCXML conflict resolution is fully deterministic: two transitions conflict "if their exit sets … have a non-null intersection"; a descendant's transition beats an ancestor's, then document order decides. Executable content is "a single blocking operation and … no events are processed until all the executable content has completed" (run-to-completion). `<send>` is "fire and forget", with a `delay` attribute for deferred delivery. — [W3C SCXML §3.13, §4.10, §6.1](https://www.w3.org/TR/scxml/)
- [V] fUML/PSSM: PSSM extends fUML's operational semantics (itself an executable model, i.e. the semantics is an execution engine). Each active object has an event pool of event occurrences; PSSM adds a second pool for deferred events and a `CompletionEventOccurrence`; a dispatch loop takes one event from the pool per run-to-completion step. — [Márton et al., "To Do or Not to Do: Semantics and Patterns for Do Activities in UML PSSM State Machines", arXiv 2309.14884](https://arxiv.org/pdf/2309.14884); [OMG PSSM](https://www.omg.org/spec/PSSM/); [OMG fUML](https://www.omg.org/spec/FUML/)
- [V] UML's RTC premise: a single run-to-completion step applies to the whole state machine including orthogonal regions; an event is dequeued only after the previous step finishes; an event that enables nothing and is not deferred is discarded; completion events are dispatched ahead of other pooled events. — [UML 2.x §15.3.12 copy](https://site.uottawa.ca/~tcl/gradtheses/mnojoumian/ThesisFiles/FinalSpec/UML/15.3.12.html)
- [V] Even the "precise" semantics has open ambiguity: OMG issues PSSM11-22 (order of transitions in RTC steps) and PSSM11-21 (atomicity of transition, entry behavior and state completion — which parallel transition fires first determines which completion event enters the pool first). — [OMG PSSM issue tracker](https://issues.omg.org/issues/spec/PSSM)
- [V] A 2023 ACM Computing Surveys article covers 1997–2021 work on formalizing UML state machine semantics, mostly by translation to PROMELA, automata or Petri nets — evidence that the informal standard needed many competing formalizations. — [André et al., doi 10.1145/3579821](https://unpaywall.org/10.1145%2F3579821)

#### Actor / communicating-machine family

- [V] P: "Each P state machine has an unbounded FIFO buffer"; sends are "asynchronous and non-blocking" and add the event to the target's buffer; after its entry function a machine "tries to dequeue an event from its input buffer or blocks if the buffer is empty". Dequeue "removes the first event that is not deferred in its current state, keeping rest of the buffer unchanged"; an ignored event is dropped. — [P manual: state machines](https://p-org.github.io/P/manual/statemachines/); [P manual: statements](https://p-org.github.io/P/manual/statements/)
- [V] P: by default a machine must handle every event that can arrive in every state, and the checker reports violations (unhandled event = error); deferral is the explicit escape, with a liveness check that an event is not deferred forever. `receive` blocks for listed events and auto-defers all others. `raise` ends the current function and handles the event immediately; `new` creates a machine and passes a payload to its start state's entry function. `$` gives controlled nondeterministic choice. — [Desai et al., PLDI 2013 (MSR page)](https://www.microsoft.com/en-us/research/publication/p-safe-asynchronous-event-driven-programming-2/); [P manual: statements](https://p-org.github.io/P/manual/statements/)
- [V] In P's systematic testing, context switches happen only at machine creation, sends and receives; the delay-bounded scheduler is a deterministic scheduler plus a "delay" operation and prioritizes schedules following the causal order of events. — [P presentation (UIUC CS524)](https://courses.grainger.illinois.edu/cs524/fa2020/p-presentation.pdf); [MSR tech report](https://microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-8.pdf)
- [V] PObserve checks P specifications (global invariants written as monitors) against service logs from tests or production, with no extra instrumentation; a log parser turns each line into a timestamped PObserve event whose classes are generated from the P spec. — [PObserve docs](https://p-org.github.io/P/advanced/pobserve/pobserve); [log parser](https://p-org.github.io/P/advanced/pobserve/logparser)
- [V] Rebeca is "an operational interpretation of the actor model with formal semantics and model-checking tools"; messages go into the receiver's buffer; in Timed Rebeca "messages are executed atomically and are not preempted"; the semantics is given in SOS. Two state-space semantics coexist for the same language: Timed Transition System (states at time points) and the coarser Floating-Time Transition System (one transition per message-server execution), the latter used for event-based properties and state-space reduction. — [Sirjani et al., "Timed Actors and Their Formal Verification", arXiv 2309.07302](https://arxiv.org/pdf/2309.07302); [Timed Rebeca SOS, arXiv 1108.0228](https://ar5iv.labs.arxiv.org/html/1108.0228)
- [V] Erlang: the only ordering guarantee is pairwise — signals from one sender to one destination keep their order; nothing is promised across senders. Fredlund's semantics delivers atomically (instant global order), which is stricter than the real guarantee; a 2024 formalization separates send from arrival; selective receive consumes messages out of arrival order. — [erlang-questions thread](https://erlang.org/pipermail/erlang-questions/2013-January/072133.html); [Cloud Haskell semantics draft](https://haskell-distributed.github.io/static/semantics.pdf); [Core Erlang formal semantics, Computers 2024](https://www.mdpi.com/2073-431x/13/11/276); [reversible Erlang debugging, arXiv 2112.12869](https://arxiv.org/pdf/2112.12869v2)

#### Coordination-layer family

- [V] BIP has three layers: atomic components (behavior), connectors (interactions) and priorities (choosing among enabled interactions). The semantics is given as LTSs with SOS rules. The reference implementation uses *global-state* semantics: an Engine computes enabled interactions, selects a maximal one under priorities, then lets the involved components move. — [Basu, Bozga, Sifakis, "Modeling Heterogeneous Real-time Components in BIP" (SEFM 2006)](https://typeset.io/pdf/modeling-heterogeneous-real-time-components-in-bip-1ta614nln6.pdf); [Rigorous component-based system design using the BIP framework (IEEE Software 2011)](https://infoscience.epfl.ch/record/170496/files/ieee-software.pdf?version=1)
- [V] BIP *partial-state* semantics drops the assumption that an interaction plus its internal computation is atomic. The translation from global-state to partial-state does **not** in general preserve observational equivalence; it is restored by adding an oracle (a predicate over a dependency relation between interactions) to the rule premises. — [Basu, Bidinger, Bozga, Sifakis, "Distributed Semantics and Implementation for Systems with Interaction and Priority" (FORTE 2008)](https://springerlink.bibliotecabuap.elogim.com/doi/10.1007/978-3-540-68855-6_8)
- [B] BIP composition rule in one line: an interaction *a* (a set of ports) is enabled in global state (q1..qn) iff each participating component has an enabled transition labeled with its port in *a*; firing moves exactly the participants and leaves the rest unchanged; with priorities, *a* fires only if no higher-priority interaction is enabled. Connectors may carry data-transfer functions executed as part of the interaction. — [SEFM 2006 paper](https://typeset.io/pdf/modeling-heterogeneous-real-time-components-in-bip-1ta614nln6.pdf)
- [V] Ptolemy II: the director determines the model of computation; actors expose an abstract semantics where `fire` is an output function of inputs and state, `postfire` is the state-update function, and "only the postfire() method can change the state of the actor". Tripakis et al. give a modular formal semantics on this interface. — [Extending Ptolemy II lecture](https://ptolemy.berkeley.edu/projects/embedded/concurrency/lectures/ExtendingPtolemyII.pdf); [Tripakis et al., modular formal semantics for Ptolemy](https://ptolemy.berkeley.edu/projects/chess/pubs/999/TripakisEtAl_Semantics_03.pdf)
- [V] Lingua Franca: "All events occur at an instant in logical time"; "A reaction is always invoked at a well-defined logical time, and logical time does not advance during its execution"; events with equal tags are logically simultaneous. Tags are superdense (time value, microstep); reactions within one reactor never run in parallel. Determinism comes from tag order plus a static precedence order on reactions. — [LF docs: Time and Timers](https://www.lf-lang.org/docs/next/writing-reactors/time-and-timers); [Menard et al., TACO 2023, arXiv 2301.02444](https://arxiv.org/pdf/2301.02444); [Lohstroh PhD thesis, EECS-2020-235](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2020/EECS-2020-235.html)
- [V] LF's deterministic semantics lets formal axiomatic models be built automatically from LF programs for verification; 2024 work extends coordination to distributed programs with cycles and multiple logical timelines. — [Lin et al., LF verification](https://par.nsf.gov/servlets/purl/10473651); [Distributed discrete-event systems, arXiv 2405.12117](https://arxiv.org/html/2405.12117v1)
- [B] Kahn process networks: deterministic processes connected by unbounded FIFO channels with blocking reads and non-blocking writes compute a unique result (channel histories) independent of scheduling — determinism bought by forbidding "test for empty" and merge. 1974; foundation for dataflow MoCs. — [Kahn process networks (overview)](https://en.wikipedia.org/wiki/Kahn_process_networks)

### Inferences

- Comparative table (derived from the findings above):

| Language | Configuration | Small step | Macro step | Queue discipline | Who resolves nondeterminism |
| --- | --- | --- | --- | --- | --- |
| STATEMATE | active states + variables + events of this step | one set of non-conflicting transitions | superstep (async model) | none; events live one step | priority rules; residual nondeterminism reported |
| SCXML | active state set + datamodel + 2 queues | microstep = optimal transition set | until internal queue empty | internal + external FIFO per session | fully fixed by document order |
| fUML/PSSM | object activations + event pool + deferred pool | activity/action node firing | RTC step per dispatched event | one pool per active object | unspecified in places (open OMG issues) |
| P | machines (state, locals, FIFO buffer) | statement | dequeue + handler to next dequeue | one FIFO per machine, defer set per state | external scheduler; explicit `$` |
| Rebeca | actors (vars, queue) | (coarse) one message server | same | one queue per actor | model checker explores all |
| Erlang | processes + mailboxes + in-transit "ether" | expression reduction / signal send / arrival | none built in | per-process mailbox, pairwise order, selective receive | runtime scheduler (uncontrolled) |
| BIP | tuple of component states | one interaction | none (interaction is atomic) | none (rendezvous/broadcast) | priorities + Engine |
| Ptolemy II | actor states + director state | fire / postfire | one director iteration | per-director (channels, event queue) | the director |
| Lingua Franca | reactor states + event queue ordered by tag | one reaction | all reactions at one tag | tagged event queue | none left: deterministic |
| KPN | process states + channel histories | read / write | none | FIFO per channel | irrelevant (confluent) |

- Our model matches P/Rebeca most closely (a port behavior = a message server/handler), but with *ports and wires as first-class glue*, which is BIP's contribution; and "queue per port" rather than per machine is closer to SCXML-per-session or to a BIP connector with a buffer component.
- The Esmaeilsabzali aspects are a ready-made checklist: for each aspect our language must pin one option and write it down, or it reproduces the "20 variants" problem. Proposed pins are in section 8.
- Rebeca's two semantics for one language (fine TTS, coarse FTTS) is a direct precedent for "granularity is a refinement level": the coarse one is sound for properties that only observe events at handler boundaries.
- The PSSM open issues show that "orthogonal regions inside one RTC step" is where precise semantics goes wrong. Having no intra-entity parallel regions (parallelism only between activations/entities, and explicit `par` nodes) avoids this entire class.

### Gaps

- The Rhapsody semantics (Harel & Kugler 2004) and Crane & Dingel's UML/Rhapsody/Stateflow comparison were not found in this session; known from background only: Rhapsody uses one event queue per active object and RTC, with inner-first priority like UML and unlike STATEMATE's outer-first.
- BIP's formal rules could not be read in the primary PDF (extraction failed); the one-line rule above is background knowledge.
- P's documentation does not state any loss/duplication/reordering model for messages; unreliable networks are modeled explicitly by the user (background knowledge, not confirmed on the pages read).
- No 2025–2026 Lingua Franca semantics paper was found; one candidate (arXiv 2504.04224, Lin, Lohstroh, Lee) could not be characterized.

## 3. Message-passing semantics, request/response, timeouts, adapters

### Takeaway

The safest general semantics is asynchronous send into an explicit in-transit buffer with ordering guaranteed only per (sender, receiver) channel; synchronous call is derived as send + blocked activation + reply, and a timeout is just another enabled transition that races the reply. No studied language has a first-class "message changes form at a boundary" construct; the closest precedents are BIP connectors with data transfer, SCXML `<invoke>`/`<finalize>`, and UML/SysML delegation (pure forwarding), so adapters are our own small extension and should be defined as glue transitions, not as hidden magic.

### Cited Findings

- [V] Queue granularity varies: one FIFO per machine (P), one pool per active object plus a deferred pool (PSSM), internal + external queue per session (SCXML), per-actor buffer (Rebeca), per-process mailbox with selective receive (Erlang). — [P manual](https://p-org.github.io/P/manual/statemachines/); [arXiv 2309.14884](https://arxiv.org/pdf/2309.14884); [W3C SCXML](https://www.w3.org/TR/scxml/); [arXiv 2309.07302](https://arxiv.org/pdf/2309.07302); [arXiv 2112.12869](https://arxiv.org/pdf/2112.12869v2)
- [V] Ordering: Erlang guarantees order only between one sender and one destination; a semantics that delivers atomically at send time (Fredlund) imposes a global order the real system does not have, and a semantics with an unordered "ether" can violate even the pairwise guarantee. — [erlang-questions](https://erlang.org/pipermail/erlang-questions/2013-January/072133.html); [Computers 2024](https://www.mdpi.com/2073-431x/13/11/276)
- [V] Message-passing race detection needs a precise happens-before for sends, deliveries, spawns and selective receives; Concuerror's had not been formally defined until recent work proposed a trace notion for it. — [arXiv 2210.03026](https://www.arxiv.org/pdf/2210.03026)
- [V] Handling of unexpected messages is a semantic choice with three options seen in practice: error (P default), discard (UML: "the event occurrence is discarded"), defer (P `defer`, PSSM deferred pool, P `receive` auto-defers). — [PLDI 2013](https://www.microsoft.com/en-us/research/publication/p-safe-asynchronous-event-driven-programming-2/); [UML §15.3.12](https://site.uottawa.ca/~tcl/gradtheses/mnojoumian/ThesisFiles/FinalSpec/UML/15.3.12.html); [P statements](https://p-org.github.io/P/manual/statements/)
- [V] Request/response in an async language: P's `receive` blocks the handler mid-body until one of the listed reply events arrives, deferring everything else. — [P statements](https://p-org.github.io/P/manual/statements/)
- [V] Timers/timeouts: SCXML models them as `<send>` with `delay`; Coyote treats "the delivery of timeouts" as one more source of nondeterminism the test scheduler controls; Lingua Franca gives them a logical tag so their order relative to other events is deterministic. — [W3C SCXML §6.1](https://www.w3.org/TR/scxml/); [Coyote: how it works](https://microsoft.github.io/coyote/overview/how); [LF docs](https://www.lf-lang.org/docs/next/writing-reactors/time-and-timers)
- [V] Synchronous interaction as primitive: BIP interactions are atomic multi-party synchronizations chosen by the Engine; implementing them over asynchronous message passing requires a protocol and breaks observational equivalence unless constrained. — [FORTE 2008](https://springerlink.bibliotecabuap.elogim.com/doi/10.1007/978-3-540-68855-6_8)
- [V] The prior project report already notes that UML/SysML delegation is pure forwarding and that a typed transformer on the wire is the project's own extension; and that layered messages follow nested session protocols. — [UML connector](https://www.uml-diagrams.org/composite-structure-diagrams/connector.html); [Demangeon & Honda, nested protocols](https://mrg.cs.ox.ac.uk/publications/nested-protocols-in-session-types/subsessioncam.pdf)

### Inferences

- Define channels per *wire* (one FIFO per wire instance, i.e. per sender-port/receiver-port pair), and let a provided port's inbox be the nondeterministic merge of its incoming wires. This is exactly Erlang's pairwise guarantee, it is what TCP connections and broker partitions actually give, and it exposes cross-sender races that a single global FIFO would hide. Channel properties (`fifo | bag`, `lossy`, `duplicating`, `bounded(n)`) should be attributes of the wire/pack so that "at-least-once redelivery" is a transition of the channel, not prose.
- Sync call = `send(req, corr)` + activation moves to `await(corr)` + matching `reply(corr)` resumes it. The "port stays visibly held while the call is pending" in the checkpoint is the `await` position of the lane. Timeout = a timer transition enabled while in `await`; reply-vs-timeout is an ordinary race the exhaustive mode finds.
- Adapter semantics, three cases:
  1. *1:1 re-framing* (DNAT, veth forward, deserialize): a `deliver` step whose only effect is `m' = adapt(m)`; it changes the layer stack (push/pop/rewrite header) and no state. Relative to the upper layer's observation it is a stuttering step — which is why the generator may collapse it.
  2. *Stateful or n:m transformation* (TLS termination, TCP segmentation/reassembly, HTTP parsing over a stream): the adapter is itself an entity with a behavior from a pack; the "adapter on a wire" is syntactic sugar for an interposed boundary entity. Its macro step ("request reaches service") refines to the pack's small steps.
  3. *Routing adapters* (LB, router): a `deliver` step with a choice of target wire; the choice is a predicate (bound, assumed, or forked).
- Messages should carry a *layer stack* (HTTP in TLS in TCP in IP), a causal id and a correlation id. Encapsulation is push, decapsulation is pop, and the projection rule "an event lifts to the deepest visible ancestor" is then a function on labels, not on semantics.
- Unhandled message policy: follow P (stuck/error) because the checkpoint says "a stuck configuration is a model gap"; allow `ignore`/`defer` only when declared by the behavior.

### Gaps

- No primary source was found that formalizes message *transformation at delegation boundaries* for component models; this is an inference from adjacent mechanisms.
- Session-typed request/response pairing (multiparty session types as a typing discipline for wires) was not researched here beyond the prior report's citation.

## 4. Macro steps, refinement and atomicity

### Takeaway

"Macro step" has two distinct meanings that must be kept apart: a *language-defined* run-to-completion unit (always atomic by fiat, as in SCXML/P/Rebeca) and a *view-defined* collapsed segment (atomic only if a reduction argument holds). The standard correctness notions are stuttering refinement for hiding invisible steps and Lipton/mover reduction for collapsing steps under concurrency; BIP's partial-state result is the cautionary tale for getting it wrong.

### Cited Findings

- [V] Run-to-completion as macro step: SCXML macrostep ends when the internal queue is empty and no eventless transition is enabled; UML dequeues the next event only after the previous RTC step completes; Rebeca executes a message server atomically without preemption. — [W3C SCXML §3.13](https://www.w3.org/TR/scxml/); [UML §15.3.12](https://site.uottawa.ca/~tcl/gradtheses/mnojoumian/ThesisFiles/FinalSpec/UML/15.3.12.html); [arXiv 2309.07302](https://arxiv.org/pdf/2309.07302)
- [V] Rebeca's coarse FTTS is used for event-based properties and state-space reduction, while the fine TTS is needed for properties about variable values at given times — i.e. the coarse semantics is only sound for a restricted observation. — [arXiv 2309.07302](https://arxiv.org/pdf/2309.07302)
- [B] Abadi & Lamport: an implementation refines a specification if a refinement mapping sends each low-level step to a high-level step *or a stuttering step* (no change in the high-level state); specifications must therefore be stuttering-invariant. This is the standard account of "many small steps implement one big step". 1991, still the basis of TLA+ refinement. — [Abadi & Lamport, "The Existence of Refinement Mappings"](https://lamport.azurewebsites.net/pubs/abadi-existence.pdf)
- [B] Lipton reduction (1975): an action is a right mover if it commutes later past any other thread's action, a left mover if it commutes earlier; a sequence of right movers, at most one non-mover, then left movers can be treated as one atomic action without changing reachable outcomes. — [Lipton, CACM 1975](https://doi.org/10.1145/361227.361234)
- [V] CIVL operationalizes this: "Sequences of R*L* are atomic"; it checks pairwise commutativity from declared mover types and stacks refinement layers so that safety of one layer implies safety of the one below. — [CIVL, FMCAD 2021 slides](https://fmcad.org/FMCAD21/slides/civl.pdf); [CIVL site](https://civl-verifier.github.io)
- [V] Inductive sequentialization (PLDI 2020) shows an asynchronous message-passing program refines a sequential program that summarizes one fixed interleaving — the strongest form of "the collapsed view is faithful". — [Kragl, Enea, Henzinger, Mutluergil, Qadeer, PLDI 2020](https://www.lix.polytechnique.fr/~cenea/papers/pldi20is.pdf)
- [V] Current as of 2026: a January 2026 arXiv paper generalizes Lipton reduction to atomic sections containing procedure calls, with mover notions more general than CIVL's and asymmetric side conditions for left and right movers; an ESOP 2026 paper "Reduction for structured concurrent programs" is listed by CIVL. — [arXiv 2601.13341](https://arxiv.org/pdf/2601.13341); [CIVL site](https://civl-verifier.github.io)
- [V] Counter-example to naive collapsing: BIP's move from atomic interactions (global state) to non-atomic ones (partial state) does not preserve observational equivalence in general; an oracle on interaction dependencies restores it. — [FORTE 2008](https://springerlink.bibliotecabuap.elogim.com/doi/10.1007/978-3-540-68855-6_8)
- [V] PSSM issue PSSM11-21 is literally an atomicity question (is "transition effect + entry behavior + state completion" one atomic unit?), showing the hazard exists even inside a standardized RTC step. — [OMG PSSM issues](https://issues.omg.org/issues/spec/PSSM)
- [V] Ptolemy's actor interface separates `fire` (compute outputs, no state change) from `postfire` (commit state) so a director can iterate to a fixed point within one iteration before committing — a hierarchical model exposes one composite `fire/postfire` to its parent. — [Extending Ptolemy II](https://ptolemy.berkeley.edu/projects/embedded/concurrency/lectures/ExtendingPtolemyII.pdf); [Tripakis et al.](https://ptolemy.berkeley.edu/projects/chess/pubs/999/TripakisEtAl_Semantics_03.pdf)

### Inferences

- Three macro-step relations are needed, each with its own obligation:
  1. **Activation RTC** (language-level): from dequeue/resume to the next blocking point (`await`, end). This is a *scheduling unit*, not automatically an atomic unit: its internal sends and store accesses are still individually visible to other activations unless reduction applies. (P makes context switches only at send/receive/create — the same idea: internal local steps are both-movers, so only communication points matter.)
  2. **Summary vs full** (level-of-detail): the report's obligation "every path through the full graph maps to exactly one case with the same effects at ports" is a stuttering simulation where the observation is the sequence of port-visible labels (sends, store reads/writes, reply) and all other nodes map to stutter steps.
  3. **Collapsed segment / closed entity** (view-level): hiding internal labels of an entity is the usual LTS hiding operator; the collapsed step is faithful iff the hidden small steps are invisible to every *other* lane's choices.
- The computed `atomicity` values map onto the theory cleanly: `stutter` = every small step maps to a stutter under the current observation (Abadi–Lamport); `isolated` = every small step is a both-mover because its read/write/port footprint is disjoint from every activation that can be concurrently enabled; `guarded` = the segment has the shape right-movers (acquire, receives) · at most one non-mover · left-movers (sends, release) (Lipton); `non-atomic` = none of the above, so collapse would hide interleavings. In an asynchronous setting sends to unbounded channels are left movers and receives are right movers (this is the standard mover classification used in the sequentialization literature; stated here from background, see Gaps).
- The "must agree" requirement in the checkpoint should be stated as: for every schedule of the fine semantics there is a schedule of the coarse semantics with the same observation (soundness of collapse), and the engine checks this by construction (footprint analysis) rather than by search.
- BIP's lesson: if a segment is declared atomic at the coarse level, then at the fine level the scheduler must be *constrained* so no conflicting step interleaves, or the levels diverge. Since we cannot constrain the real system, the direction must be the other way: atomicity is *computed from* the fine level, never asserted.

### Gaps

- The explicit statement "send is a left mover, receive is a right mover for unbounded FIFO/bag channels" was not located in a fetched source this session; it is background knowledge from the reduction literature (Lipton 1975; Kragl et al.) and should be verified against [the PLDI 2020 paper](https://www.lix.polytechnique.fr/~cenea/papers/pldi20is.pdf) before being quoted.
- No source was found on *presentation-oriented* collapse (a debugger that auto-collapses reducible segments); this appears to be novel to the project.
- Branching bisimulation / weak bisimulation literature for hiding (van Glabbeek & Weijland) was not researched; it is the precise equivalence for "closed entity looks the same from outside".

## 5. Symbolic semantics

### Takeaway

Symbolic execution is not a second semantics: it is the same rules run over configurations that carry a path condition, with a rule firing whenever its guard is *satisfiable* rather than *true*. K-line work proves this construction generically (feasible symbolic runs and concrete runs mutually simulate). Because our guards range over a finite named predicate vocabulary, the "solver" is propositional and forking is cheap.

### Cited Findings

- [B] King (1976): symbolic execution runs a program on symbolic inputs, maintaining a symbolic store and a path condition; a branch whose condition is not decided by the path condition forks into two states, each conjoining one outcome. — [King, CACM 1976](https://doi.org/10.1145/360248.360252)
- [V] Arusoaie, Lucanu, Rusu: from a language's operational semantics a *symbolic* semantics is generated automatically (same syntax, rules rewrite over formulas denoting sets of states); "the symbolic execution of concrete programs is, by definition, the execution of the same programs with the symbolic semantics"; feasible symbolic executions and concrete executions mutually simulate each other. Prototype in K; SLE 2013, journal version J. Symbolic Computation 2017. — [HAL hal-01238696](https://hal.umontpellier.fr/CRISTAL-DREAMPAL/hal-01238696v2); [SLE 2013 chapter](https://springerlink.bibliotecabuap.elogim.com/content/pdf/10.1007/978-3-319-02654-1_16.pdf)
- [V] K's prover uses the semantics for symbolic execution with Z3 for constraints; for deterministic languages one-path and all-path reachability coincide, for concurrent ones all-path is needed. — [RV blog](https://runtimeverification.com/blog/k-vs-coq-as-language-verification-frameworks-part-3-of-3); [arXiv 1810.10826](https://arxiv.org/abs/1810.10826v2)
- [V] State-machine-specific precedents: symbolic execution of communicating, hierarchically composed UML-RT state machines (Zurowska & Dingel, NFM 2012); symbolic execution with induction for temporal properties of UML state machines (Balser et al.); input/output symbolic transition systems (IOSTS) for test purposes; symbolic execution trees track path constraints and suffer path explosion on large models. — [CEUR Vol-1531 paper 7 (related work)](https://ceur-ws.org/Vol-1531/paper7.pdf); [Balser et al.](https://opus.bibliothek.uni-augsburg.de/opus4/files/44769/44769.pdf); [concolic testing of state-based models](https://www.researchgate.net/profile/Reza-Ahmadi-14/publication/333433556_Concolic_Testing_for_Models_of_State-based_Systems/links/5d485348299bf1995b6818ac/Concolic-Testing-for-Models-of-State-based-Systems.pdf)
- [V] The project's prior report grounds guards in control-flow automata with assume edges and symbolic automata over a Boolean algebra of predicates. — [lazy abstraction](https://dl.acm.org/doi/10.1145/503272.503279); [D'Antoni & Veanes](https://link.springer.com/chapter/10.1007/978-3-319-63387-9_3)

### Inferences

- Give every configuration a path condition `φ` = a partial valuation of named predicates (plus optional relations between predicates, e.g. mutual exclusion declared by an input characteristic). A guarded transition with guard `g` is enabled iff `φ ∧ g` is satisfiable; firing sets `φ := φ ∧ g`. Then:
  - *Concrete run* = `φ0` is total (all predicates bound) so at most one branch of each decision is satisfiable; no fork ever happens.
  - *Symbolic run* = `φ0` is partial; when both `g` and `¬g` are satisfiable the configuration forks; the default policy "take the base choice and record an assumption" is a *search strategy* over the fork tree, not a different semantics. An assumption record is exactly the conjunct added to `φ`.
- "Flipping an assumption regenerates everything downstream" = re-run from the fork configuration with the other conjunct; with stored configurations this is a resume, not a recompute.
- Predicates that are *per-occurrence* (the report's `upload-ok: [false, true]` per delivery attempt) need indexing by occurrence (activation id / attempt number) or they become unsatisfiable on the second evaluation. This is the symbolic-execution analogue of fresh symbols per loop iteration, and must be decided in the semantics: each predicate declares its *stability scope* (run-constant, per-activation, per-evaluation, or derived from a state variable).
- Predicates derived from state variables (e.g. `status-is-pending`) must not be free: their truth is computed from the store valuation, otherwise the symbolic run explores paths the stores forbid (over-approximation the model already has the data to rule out).
- Symbolic stores: when a state variable's value is unknown, treat `var = v` as a predicate and case-split lazily on first read (lazy initialization), keeping one mechanism.

### Gaps

- No source specific to symbolic execution of *actor/queue* models with symbolic queue contents was found; our queues hold concrete message kinds with symbolic payload predicates, which avoids the problem but was not validated against literature.
- State merging/subsumption techniques for fork explosion were not researched.

## 6. Nondeterminism and schedulers

### Takeaway

Best practice (P, Coyote, GEMOC, BIP's Engine, Ptolemy's directors) is to keep the semantics as a nondeterministic relation and make the *resolver* of nondeterminism an explicit, replaceable component whose decisions are recorded; a run is then a deterministic function of (model, binding, schedule). Lingua Franca shows the alternative — remove nondeterminism by logical time — which is right for specifying a design but wrong for a tool whose job is to expose the races in someone else's system.

### Cited Findings

- [V] Coyote "takes over the non-determinism in your program", including "the delivery of timeouts"; a bug comes with a trace of "the global order of all scheduling decisions and nondeterministic choices", and "the trace can be replayed reliably over and over again"; exploration uses a portfolio of strategies and is heuristic, not a proof. — [Coyote: how it works](https://microsoft.github.io/coyote/overview/how); [Coyote: concurrency unit testing](https://microsoft.github.io/coyote/concepts/concurrency-unit-testing)
- [V] P: scheduling points only at create/send/receive; a delaying scheduler = deterministic default scheduler + bounded number of "delay" deviations, prioritizing causal order. — [P presentation](https://courses.grainger.illinois.edu/cs524/fa2020/p-presentation.pdf); [MSR TR](https://microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-8.pdf)
- [V] BIP: the Engine computes the enabled interaction set, filters by priorities and picks one; priorities are a declarative way to restrict nondeterminism. — [IEEE Software 2011](https://infoscience.epfl.ch/record/170496/files/ieee-software.pdf?version=1)
- [V] Ptolemy II: the director is a pluggable scheduler defining the model of computation over a fixed actor interface. — [Extending Ptolemy II](https://ptolemy.berkeley.edu/projects/embedded/concurrency/lectures/ExtendingPtolemyII.pdf)
- [V] SCXML removes nondeterminism by document order; LF by tags and reaction precedence ("logical time does not advance during [a reaction's] execution"). — [W3C SCXML](https://www.w3.org/TR/scxml/); [LF docs](https://www.lf-lang.org/docs/next/writing-reactors/time-and-timers)
- [V] Exhaustive exploration needs partial-order reduction: Optimal DPOR explores exactly one interleaving per Mazurkiewicz trace (POPL 2014, implemented for Erlang in Concuerror); the observers extension (TACAS 2018) handles message passing and can beat plain optimal DPOR. TransDPOR (2012) targets actor programs specifically. — [Abdulla, Aronis, Jonsson, Sagonas, POPL 2014](https://user.it.uu.se/~parosha/publications/papers/popl2014.pdf); [Optimal DPOR with observers](https://link.springer.com/chapter/10.1007/978-3-319-89963-3_14); [TransDPOR reference via DSCMC paper](https://arxiv.org/pdf/1603.03536)
- [B] DPOR's base definition: two transitions are dependent if they do not commute or one can enable/disable the other; only reorderings of dependent transitions need exploring. — [Flanagan & Godefroid, POPL 2005](https://doi.org/10.1145/1040305.1040315)
- [V] GEMOC's generic engine supports concurrent semantics by exposing the set of possible next steps to a pluggable decider and supports omniscient debugging over it. — [SoSyM 2022](https://link.springer.com/content/pdf/10.1007/s10270-022-01073-2.pdf)

### Inferences

- Split the engine API into `enabled(C) → set of choices` and `fire(C, choice) → C'` with `fire` a pure function. A *schedule* is the sequence of choices. Sources of choice are exactly four and should be typed: (1) which enabled activation/channel moves (scheduling), (2) which branch of an undetermined predicate (data), (3) environment actions — external inputs, timer expiry, channel faults such as loss/duplication, crash (environment), (4) explicit `choose` in a behavior (e.g. LB replica pick). Recording all four gives Coyote-style replay; "fix check: same binding and schedule, diff the traces" follows.
- Because choices must stay meaningful across model edits, a schedule entry should name the choice by *semantic id* (activation's causal id + transition id), not by index into the enabled set; otherwise "re-run the same schedule after the fix" breaks when the enabled set changes.
- Default scheduler for concrete runs: a deterministic, human-plausible policy — run the activation caused by the most recent event to its next blocking point (depth-first causal order; P's default), ties broken by semantic id. It should be documented as a *strategy*, and the script should mark steps where `|enabled| > 1` so the UI can show "another order was possible here".
- Exhaustive mode = search over `enabled`; independence for DPOR comes for free from the same footprints used for atomicity (store read/write sets, channel endpoints). One Mazurkiewicz trace = one pomset = one script, so "enumerate scripts up to equivalence" is precisely optimal DPOR's output. A race is a pair of dependent, co-enabled transitions from different lanes.
- LF-style logical time is still useful in a limited role: give timers and delays a superdense tag `(t, microstep)` so that timeouts are ordered deterministically *relative to each other*, while their order relative to message arrival remains a scheduler choice.
- Priorities (BIP) are the right mechanism for modeling real deterministic schedulers inside packs (e.g. the JS event loop: microtasks before macrotasks) without hard-coding them in the engine.

### Gaps

- PCT and other probabilistic strategies in Coyote could not be confirmed from Coyote's own docs (only from a third-party README), so no claim is made.
- Fairness (needed for liveness properties such as "the retry eventually succeeds") was not researched; P checks liveness with fair schedules but details were not fetched.
- State-space bounds appropriate for interactive use (how many schedules can be explored in < 1 s for models of our size) have no source.

## 7. Dynamic structure

### Takeaway

All actor-family languages treat creation as an ordinary transition that extends the configuration with a fresh identity and an initial local state, and treat termination as either a special event or the removal of the component; statically-structured coordination languages (BIP, LF, classic Ptolemy) mostly avoid dynamic structure or bound it. For us, the entity tree is mostly static while *activations* and *replica instances* are dynamic, which keeps the hard part small.

### Cited Findings

- [V] P: `new` creates a machine instance and passes the constructor payload to the entry function of its start state; creation is one of the three scheduling points. — [P statements](https://p-org.github.io/P/manual/statements/); [P presentation](https://courses.grainger.illinois.edu/cs524/fa2020/p-presentation.pdf)
- [V] SCXML: `<invoke>` "causes an instance of the external service to be created" after the state's `<onentry>`, and invocations are cancelled when the state is exited — lifetime tied to a state. — [W3C SCXML §6.1, §3.13](https://www.w3.org/TR/scxml/)
- [V] Erlang/Concuerror-style semantics must include dynamic process spawning in the happens-before relation (a process's first step depends on its spawn). — [arXiv 2210.03026](https://www.arxiv.org/pdf/2210.03026)
- [V] fUML/PSSM: object activations (with their event pools) are created when an active object's classifier behavior starts. — [arXiv 2309.14884](https://arxiv.org/pdf/2309.14884)
- [V] Dynamic reconfiguration in BIP required a separate extension (Dy-BIP) rather than the base semantics. — [Dy-BIP (EPFL)](https://infoscience.epfl.ch/record/181948/files/dy-bip-1.pdf)
- [B] Rebeca fixes the set of rebecs statically (known rebecs declared up front) to keep model checking finite; dynamic creation exists only in extensions. — [Rebeca](https://rebeca-lang.org/)

### Inferences

- Put topology in the configuration: `E` = set of live entity instances with lifecycle status (`absent | starting | running | stopping | dead`). `spawn` adds an instance of a declared *type* under a declared parent with an id that is a semantic path plus an instance index minted deterministically from the spawning step's id (so replay and diff are stable). `kill` / crash moves it to `dead`; a message delivered to a dead instance's port is a defined outcome (drop, error reply, or connection-reset per channel attribute), never undefined.
- Replicas (`×3`) are instances of one type created at initialization; the LB's choice among them is a typed `choose`. Symmetry between identical replicas is a natural state-space reduction for exhaustive mode.
- Wires to dynamic instances are created by *templates* on the type (a port of every instance of type T is wired to X), evaluated at spawn.
- Bounding: exhaustive mode needs a bound on instances per type and on activations per port, declared in the exploration config and reported when hit ("bound reached" is a result, not a silent truncation).

### Gaps

- Dy-BIP's actual rules were not read.
- No source was found on stable identity for dynamically created entities across model regeneration; the semantic-path scheme is the project's own.

## 8. Recommendation: a small semantic definition for the model language

### Takeaway

Adopt a P/Rebeca-style asynchronous core (per-port behaviors as message servers, run to the next blocking point) with BIP-style explicit glue (wires, adapters, channel attributes), per-wire FIFO channels (Erlang's pairwise guarantee), an external recorded scheduler (Coyote), a path condition in every configuration (K-style single semantics for concrete and symbolic), and computed, never asserted, atomicity (Lipton/CIVL) for macro steps. Pin every Esmaeilsabzali aspect explicitly. Everything in this section is a design inference built on the cited findings above; the citations here point to the precedent each choice rests on.

### Cited Findings

- [V] Precedent for each borrowed element: P handler/queue/defer semantics — [P manual](https://p-org.github.io/P/manual/statemachines/); atomic message servers and two-granularity state spaces — [Rebeca, arXiv 2309.07302](https://arxiv.org/pdf/2309.07302); glue as a separate layer with an Engine and priorities — [BIP, IEEE Software 2011](https://infoscience.epfl.ch/record/170496/files/ieee-software.pdf?version=1); pairwise ordering — [Erlang](https://erlang.org/pipermail/erlang-questions/2013-January/072133.html); recorded schedule + replay — [Coyote](https://microsoft.github.io/coyote/overview/how); symbolic semantics derived from the concrete one with mutual simulation — [Arusoaie et al.](https://hal.umontpellier.fr/CRISTAL-DREAMPAL/hal-01238696v2); R*L* atomicity — [CIVL slides](https://fmcad.org/FMCAD21/slides/civl.pdf); macrostep/microstep with internal queue — [SCXML](https://www.w3.org/TR/scxml/); superdense tags — [LF, arXiv 2301.02444](https://arxiv.org/pdf/2301.02444); semantics as an execution engine with generic debugger services — [GEMOC](https://hal.archives-ouvertes.fr/hal-01355391); log conformance against the model — [PObserve](https://p-org.github.io/P/advanced/pobserve/pobserve); one interleaving per Mazurkiewicz trace — [Optimal DPOR](https://user.it.uu.se/~parosha/publications/papers/popl2014.pdf).
- [V] Hazards each pin avoids: 20 statechart variants — [Statemate slides](https://chess.eecs.berkeley.edu/design/2010/discussions/slides/Statemate.pdf); unresolved order/atomicity inside UML RTC steps — [OMG PSSM issues](https://issues.omg.org/issues/spec/PSSM); atomic-vs-non-atomic levels diverging — [BIP FORTE 2008](https://springerlink.bibliotecabuap.elogim.com/doi/10.1007/978-3-540-68855-6_8); atomic-delivery semantics hiding real orderings — [Computers 2024](https://www.mdpi.com/2073-431x/13/11/276).

### Inferences

#### 8.1 Configuration

```text
C = ⟨ E, S, Ch, A, T, φ, now ⟩

E   : instance-id ⇀ (type, parent per facet, status)        -- live topology (dynamic part only)
S   : state-var-instance ⇀ value | ⊥(unknown)               -- stores; machines constrain legal writes
Ch  : wire-instance ⇀ sequence of messages                  -- one channel per wire; attrs: fifo|bag, lossy, dup, bound
A   : activation-id ⇀ ⟨entity, behavior, pc, locals, awaiting, cause⟩   -- "the position of each lane"
T   : set of pending timers ⟨tag, target activation/port, kind⟩
φ   : path condition = partial valuation of named predicates (binding ∪ assumptions made so far)
now : superdense tag (t, microstep), used only to order timers

message m = ⟨id, kind, layers: stack of frames, payload-shape, corr?, cause⟩
```

The static model (types, ports, wires, behaviors, machines, adapters) is the program and is not part of `C`. The trace (pomset) is the *output*, not part of `C`: two configurations are equal regardless of how they were reached, which is what makes state caching, reverse stepping and state-space search well-defined. `pc` is a node of the full graph, or for a summary-level behavior a pair (case, effect index).

**8.2 Small-step rules** (label in brackets; each rule is one event kind from the checkpoint's vocabulary)

| Rule | Premise (enabled when) | Effect |
| --- | --- | --- |
| ENV-IN `[input]` | an `external` port has a scenario input pending | append `m` to the channel of that port's entry wire |
| DELIVER `[route/transform]` | `m` at head of `Ch(w)`, `w` a delegation or assembly wire ending at a port that is not a behavior port | move `m` to the next wire; if `w` has a 1:1 adapter, `m.layers := adapt(m.layers)`; if routing, pick target by predicate or `choose` |
| RECEIVE `[receive]` | `m` at head of some `Ch(w)` into provided port `p` of a `running` instance; `p`'s behavior accepts `m.kind` (not deferred); activation bound for `p` not exceeded | pop `m`; create activation `a` at behavior entry with `cause = m.id` |
| LOCAL `[action]` | `a.pc` at an action node | advance `pc` |
| BRANCH `[assume]` | `a.pc` at a branch with edge guard `g`, and `φ ∧ g ∧ facts(S)` satisfiable | `φ := φ ∧ g`; advance along that edge |
| CASE (summary) | `a` at entry of a summary behavior; case guard `g` satisfiable as in BRANCH | `φ := φ ∧ g`; `pc := (case, 0)`; the case's effects are then executed one per step by the rules below |
| SEND `[send]` | `a.pc` at a call node on required port `r` wired by `w` | append fresh `m` (with `corr` if a reply is expected) to `Ch(w)`; if synchronous, `a.awaiting := corr` and optionally add a timeout timer; else advance |
| REPLY `[send]` | `a` at a reply/outcome node | append reply with `a`'s request `corr` to the reverse channel; remove `a` if finished |
| RESUME `[receive]` | reply `m` with `m.corr = a.awaiting` at head of its channel | pop; clear `awaiting`; cancel timer; advance |
| TIMEOUT `[error]` | timer for `a.awaiting` is the earliest in `T` | clear `awaiting`; continue on the timeout edge (a late reply later hits RECEIVE-orphan: dropped with a marker) |
| READ / WRITE `[write state]` | `a.pc` at a store access; for a write of var `x` to `v`, the machine of `x` has a transition `S(x) → v` | read: if `S(x) = ⊥`, case-split lazily and record in `φ`; write: `S(x) := v`. A write with no machine transition → **error configuration** (model/code disagreement), not stuck |
| ACQUIRE / RELEASE | lock or transaction boundary declared by the behavior or pack | standard; gives `guarded` atomicity its shape |
| PAR / JOIN | `par` node | create child activations; parent awaits all |
| SPAWN / KILL / CRASH `[spawn/die]` | spawn node, or environment fault enabled by the exploration config | add/remove in `E`; on death, in-flight messages to its ports resolve per channel attribute |
| CH-FAULT | wire is `lossy`/`dup` and exploration enables faults | drop or duplicate head message |
| SLOT / UNKNOWN | `pc` at an unfilled slot or `unknown` node | no rule: **stuck**, reported as a model gap with the node id |

A configuration is *terminal* if `A`, `Ch`, `T` are empty; *stuck* if non-terminal and nothing is enabled except possibly ENV-IN with no input left (deadlock or model gap — distinguished by whether some activation sits at an unmodeled node or an unaccepted message heads a channel).

#### 8.3 Pinned choices (Esmaeilsabzali checklist)

| Aspect | Pin | Why |
| --- | --- | --- |
| Concurrency in a small step | exactly one transition (interleaving) | simplest; true concurrency is recovered in the pomset via independence |
| Preemption / orthogonal regions | none inside a behavior; parallelism only via `par` and separate activations | avoids the PSSM open issues |
| Big-step maximality | an activation runs to its next blocking point; no implicit re-triggering | P/Rebeca |
| Event lifeline | messages persist in channels until received (no "present for one step" events) | STATEMATE-style transient events do not match queues/sockets |
| Enabledness memory | guards read the *current* store and `φ` at the moment of the step | required for races to be visible |
| External inputs | only at ENV-IN; scenario supplies them as a partial order | open-system boundary is explicit |
| Order of small steps | chosen by the scheduler, recorded | Coyote |
| Priority | none in the core; packs may declare priorities between their own transitions (event-loop phases) | BIP |
| Unhandled message | stuck/error unless behavior declares `ignore` or `defer` | P; "stuck = model gap" |

#### 8.4 Scheduler and the three modes over one relation

```text
enabled : C → Set<Choice>            Choice = (rule, activation-or-wire id, branch?)   -- semantic ids
fire    : C × Choice → C × Label     -- pure
```

- **Concrete run:** `φ0` total; scheduler = default deterministic strategy (causal depth-first, tie by id) or a recorded schedule. Result: one trace. Deterministic given (model, binding, schedule) → reverse stepping by snapshot + replay.
- **Symbolic run:** `φ0` partial; same scheduler; BRANCH/CASE may offer several satisfiable guards → the run forks. Default search policy takes the base choice and emits an assumption record (the added conjunct); other branches are kept as fork stubs holding the pre-branch configuration id.
- **Exhaustive exploration:** DFS/BFS over all `enabled` choices (optionally also over data forks and enabled environment faults), with DPOR using independence = disjoint footprints (stores read/written, channels pushed/popped, instances spawned/killed), visited-set on canonical `C`, symmetry over replicas, and explicit bounds. Output: equivalence classes of schedules (one pomset each), races (dependent co-enabled pairs from different lanes), stuck and error configurations.
- **Trace conformance (PObserve-style):** the observed event stream acts as the scheduler — at each observed event pick the enabled choice whose label matches; no match = divergence ("surprise"), unmatched enabled obligations = "contradicted/missing". Internal steps without log evidence are taken eagerly (τ-closure).

#### 8.5 Macro steps

- Script step = label of one `fire`. The pomset's `after` relation = causal dependence between steps (same activation order, send→receive, write→later dependent access), computed, not authored.
- **RTC macro step** of activation `a`: maximal sequence of `a`'s steps between blocking points. Used as the default stepping unit ("step over").
- **Summary/full obligation:** for every full-graph path there is exactly one case with the same sequence of port-visible labels (sends, store accesses, reply) — a stuttering simulation with observation = port-visible labels; checked statically per behavior.
- **Collapsed segment:** a set of steps closed under "between" in one lane (or a closed entity's internal steps). Its `atomicity` is computed from footprints against every activation that can be co-enabled anywhere in the segment: `stutter` (no store effect, no branch, 1:1 adapters only → maps to stutter steps at the viewed layer), `isolated` (all steps both-movers), `guarded` (right-movers · ≤1 non-mover · left-movers, typically lock/transaction-delimited), else `non-atomic`. Only the first three may be auto-collapsed; the macro step's effect is the composition of the small steps' effects, and by reduction every fine schedule has a coarse schedule with the same observation.
- **Spatial projection:** hiding = relabel steps internal to a closed entity as τ and lift port events to the deepest visible ancestor; the closed view is a correct abstraction iff the hidden steps are `stutter`/`isolated`/`guarded` w.r.t. everything outside.

#### 8.6 Pitfalls to avoid

1. *Treating a summary case as atomic.* A case's effects are still separate small steps; the worked example's claim-without-row-lock race (read `PENDING` … write `RUNNING`) only appears if another activation can interleave between them.
2. *Global FIFO or atomic delivery.* Hides cross-sender races and redelivery; use per-wire channels with explicit attributes.
3. *Asserting atomicity instead of computing it* (BIP partial-state lesson), and trusting LLM-declared atomicity (the prior report cites this as the characteristic LLM modeling error).
4. *Free predicates that are actually functions of state*, and *run-constant predicates that are really per-occurrence*; both corrupt symbolic runs. Each predicate needs a declared stability scope.
5. *Positional schedule entries.* Schedules must name choices by semantic id or "same schedule after the fix" is meaningless.
6. *Baking a scheduling policy into the rules* (SCXML document order, LF determinism). Keep policy in the scheduler/strategy and in pack-declared priorities.
7. *Trace as part of state.* Makes state comparison, caching and exploration impossible.
8. *Stuck vs error vs deadlock conflation.* Model gap (unknown/slot/unaccepted message), model–code disagreement (illegal write), and genuine deadlock need distinct outcomes because they send the user to different fixes.
9. *Unbounded exploration.* Loops with trip class `many`/`input-dependent`, unbounded channels and spawning need declared bounds, with "bound reached" surfaced.
10. *Zero-time cycles.* A cycle of DELIVER/τ steps with no blocking point never ends an RTC macro step; detect statically (SCXML and synchronous languages have the same hazard).
11. *Intra-entity orthogonal regions and transient events.* The main source of statechart-variant disagreement; do not add them.
12. *Late replies and dead receivers.* Give orphan replies, messages to dead instances and timer cancellation defined outcomes; these are where real systems' bugs live.

### Gaps

- This sketch has not been validated by mechanizing it (Redex/K/TLA+) or by running the three gold models; the mover classification per rule (which rules are left/right/both movers under which channel attributes) needs a careful table and proof sketch, which no single source provides for this combination.
- Timed semantics is deliberately minimal (timers only). If durations become first-class, Timed Rebeca's FTTS/TTS pair and DEVS (assigned to another researcher) are the references to consult.
- Interaction between `guarded` atomicity and database isolation levels (a "transaction" is not serializable under READ COMMITTED) is not covered by the reduction literature surveyed; pack-level modeling of isolation is an open design item.
- No late-2025/2026 publications were found that change any of the recommendations; the newest relevant items located were [arXiv 2601.13341](https://arxiv.org/pdf/2601.13341) (reduction, January 2026) and the ESOP 2026 entry on the [CIVL site](https://civl-verifier.github.io). A targeted check of MODELS 2025/2026 and SoSyM for new executable-semantics work is still open.
