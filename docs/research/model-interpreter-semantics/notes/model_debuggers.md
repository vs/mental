# Omniscient, time-travel and model-level debuggers, and trace conformance, as precedents for the model-interpreter debugger UI (as of November 2026)

Scope note: the target is the debugger UI over the "model interpreter" in `docs/brainstorm/checkpoint-01.md` (playback like a video; step into/over/out = zoom in time and space; breakpoints on state predicates; watchpoints on state variables; reverse stepping; set-variable / flip-assumption mid-run; re-run same binding+schedule and diff; align observed traces with the model). Dates are given for every result; results older than about 2015 are marked "(older)". Where a figure came only from a search snippet or secondary summary rather than the primary text, it is flagged.

## 1. Omniscient / back-in-time debugging: query models, storage and indexing, performance

### Takeaway
Two storage strategies dominate: (a) log every state change and index it (Lewis ODB, TOD, Whyline, Pernosco's database), which makes "when did X change / why" queries instant but costs record-time slowdown and storage; (b) record only inputs and nondeterminism, then recompute any past state by deterministic re-execution from checkpoints (rr, Undo, Replay.io), which is cheap to record but makes each query a re-execution. Pernosco combines both (rr recording, then an offline pass that builds an omniscient database). The most effective user-facing ideas are query-oriented, not stepping-oriented: Whyline's "why did / why didn't" questions, Pernosco's "line executions" with retroactive print/condition expressions, and TTD's LINQ queries over a trace.

### Cited Findings
**Event-logging omniscient debuggers (older, foundational)**
- Lewis's ODB (2003, older) records "every state change (every variable assignment of any type) and every method call," instrumenting Java bytecode, and orders events from threads with a single high-level lock; the paper's central claim is "debugging is easier if you can go backwards." — [Lewis, Debugging Backwards in Time, AADEBUG 2003](https://arxiv.org/abs/cs/0310016)
- Lewis's slides report about 1 µs per event and 100 million events recorded, and estimate a worst case of roughly 10x slowdown for an optimized version (flagged: from search summary of slides) — [Lewis slides, BCS 2011](https://apsg.bcs.org/materials/presentations/20110414lewis.pdf); Balzer built an omniscient debugger for Fortran as early as 1969 — [Lewis, Dr. Dobb's 2005](https://jacobfilipp.com/DrDobbs/articles/DDJ/2005/0506/0506a/0506a.html)
- TOD (Pothier, Tanter, Piquer, OOPSLA 2007, older): distributed event database sustaining about 470k events/s on a 10-node cluster while answering hundreds of queries per second — [Scalable Omniscient Debugging, OOPSLA 2007](https://pleiad.cl/papers/2007/pothierAl-oopsla2007.pdf); about 50–55k events/s on a single machine and about 140 bytes per event — [TOD internals](https://www.pleiad.cl/tod/documentation/internals.html), [TOD FAQ](https://pleiad.cl/tod/documentation/faq.html)
- TOD architecture = weaver (event generation) + query processor + distributed event database + structure database; capture can slow the debuggee by up to 80x; 500k events/s on 10 machines — [Pothier & Tanter, IEEE Software 2009](https://pleiad.cl/papers/2009/pothierTanter-software2009.pdf)

**Whyline (Ko & Myers; older but the main evidence for question-based debugging)**
- The Java Whyline derives "why did" and "why didn't" questions from code and the recorded output; answers use static and dynamic slicing, precise call graphs, and new algorithms for "why a line of code was not reached." The developer demonstrates the behaviour, scrubs the I/O history, clicks an output and picks a question. — [Ko & Myers, Debugging Reinvented, ICSE 2008](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf)
- "Why didn't" explanation is recursive over control dependencies: the method did not run (explain why none of its callers ran), the instruction had not been reached yet, an exception jumped over it, none of its control dependencies executed, or one executed but went the other way — [Ko & Myers 2008](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf)
- Study: Whyline participants finished in a median of 4 minutes (range 1–12) versus 10 minutes (3–38) for the control group (p < .05, Wilcoxon); novices with the Whyline were more than twice as fast as experts without it; novices asked the question they actually had ("Why is the line blue?") while experts first speculated about causes and searched for a question that tested their guess. A preliminary second study found expert Whyline users "twice as successful in half the time." — [Ko & Myers 2008](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf)
- Earlier Whylines: the Alice Whyline cut debugging time by a factor of 8; Crystal (questions about the undo stack) let participants complete 30% more tasks, 20% faster — [Ko & Myers 2008, related work](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf)
- Cost of full tracing in the Java Whyline: slowdown 1.6x to 15.3x over normal execution on five programs; traces from 140k to 35M events, 4.7 MB to 283.6 MB uncompressed (11.6%–34% after zip), loading at 56k to 1.28M events/s — [Ko & Myers 2008, Table 1](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf); expanded journal version: [Ko & Myers, TOSEM 2010](https://faculty.washington.edu/ajko/papers/Ko2010WhylineImplementation.pdf)

**Record-and-replay (record inputs, recompute state)**
- rr (O'Callahan et al., USENIX ATC 2017): user-space record and replay on stock hardware and kernels; recording and replay slowdown under 2x on important low-parallelism workloads, mainly thanks to in-process system-call interception — [Engineering Record and Replay for Deployability](https://arxiv.org/abs/1705.05937), [USENIX ATC 17](https://www.usenix.org/conference/atc17/technical-sessions/presentation/ocallahan)
- Undo (UDB/LiveRecorder): serializes multithreaded execution (with pre-emptive scheduling so races still reproduce), so compute-heavy multithreaded programs see the largest slowdown — [Undo performance benchmarks](https://undo.io/resources/undo-performance-benchmarks); UDB 6.8 "Parallel Search" uses several cores for up to 4x faster time-travel searches — [Undo](https://undo.io/?p=4435)
- Replay.io records runtime inputs and needed nondeterminism, not video and not every state change; a past point is reached by re-running the program to it ("paused on line 10, rewind to line 7: re-run to line 7"); everything in its DevTools is computed at replay time, not record time; recordings replay in the cloud so several people can view one session — [Replay docs: how time travel works](https://docs.replay.io/basics/time-travel/how-does-time-travel-work), [How Replay works](https://www.replay.io/blog/how-replay-works), [Session replay vs time travel](https://blog.replay.io/session-replay-vs-time-travel-debugging)
- Replay saves partial replays so recordings of any size can, in theory, load in fixed time, though exact replay still scales roughly with recording length (flagged: from search excerpts of the Replay blog) — [How to time travel every time](https://blog.replay.io/how-to-time-travel-every-time)
- Reverse-execution design trade-off: record inputs and replay versus save overwritten state; the main cost of state saving is log memory — [Necula et al., reverse execution TR 2011](https://people.eecs.berkeley.edu/%7Enecula/Papers/rev-exec-tr2011.pdf)

**Hybrid: database built from a recording**
- Pernosco (on rr) serves gdb from a gdbserver that reads the omniscient database; stepping in either direction just changes an internal timestamp and fetches results; users "routinely" move through minutes of execution with a 1–2 s delay — [O'Callahan, Supercharging gdb with Pernosco, 2019](https://robert.ocallahan.org/2019/11/supercharging-gdb-with-pernosco.html)
- Pernosco's "line executions" view lists every execution of a function or source line; condition and print expressions are evaluated against the database at each point in time, "potentially in parallel," without rerunning: retroactive printf. The author frames it as a foundation for "query-based debugging" and admits users need time to unlearn old habits — [Omniscient printf debugging in Pernosco, 2019](https://robert.ocallahan.org/2019/11/omniscient-printf-debugging-in-pernosco.html)
- Pernosco is limited by rr to x86-64 Linux and compiled languages (C, C++, Rust) — [Pernosco demo video post](https://robert.ocallahan.org/2019/10/pernosco-demo-video.html)

**WinDbg Time Travel Debugging (TTD)**
- No hard cap on trace size; WinDbg replays traces "well into the hundreds of gigabytes"; the .IDX index is built on first open and is typically 1–2x the .RUN trace — [TTD trace file information](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/time-travel-debugging-trace-file-information), [TTD file size blog](https://learn.microsoft.com/en-us/archive/blogs/windbg/time-travel-debugging-ttd-file-size)
- Microsoft itself lists "large overhead at record time" and large data files as TTD drawbacks — [TTD overview](https://learn.microsoft.com/windows-hardware/drivers/debugger/time-travel-debugging-overview)
- TTD traces are queried through the debugger data model with LINQ (function calls, time positions, memory addresses, registers), results shown as grids via `dx -g` — [TTD queries](https://learn.microsoft.com/en-us/archive/blogs/windbg/time-travel-debugging-queries)

**Scriptable, lazy traces**
- Expositor (Khoo, Foster, Hicks, ICSE 2013, older) treats traces as first-class, scriptable values on top of GDB/UndoDB; traces are lazy, sparse interval trees materialized on demand because each time-travel call is expensive; it adds an "edit hash array mapped trie" for lazy sets and maps; case studies include a Firefox data race — [Expositor ICSE 2013](https://www.cs.umd.edu/~mwh/papers/khoo13expositor.html), [journal version](https://www.cs.umd.edu/~mwh/papers/khoo13expositor-journal.html)

### Inferences
- Our interpreter is deterministic given (binding, schedule), so the cheapest storage is the rr/Replay.io one: store the binding, the scheduler choices and periodic configuration snapshots, and recompute in between. Unlike rr, our "replay" is the same interpreter, so divergence risk (Replay's hardest problem) is absent unless the model changes.
- The Pernosco lesson is that reverse stepping is the least important feature; queries over the whole run ("all times `report.status` changed", "every delivery to port X") with values shown inline are what change debugging. Since our configurations are small and typed, a TOD/Whyline-style full change log per state variable is affordable (TOD's 140 B/event and Whyline's 35M events in 284 MB set the scale), and it makes watchpoints and "why" queries index lookups.
- Whyline's "why didn't" is much easier for us than for Java: every transition has an explicit guard over named predicates, so "why didn't `retry` fire at step 41?" is answered by evaluating the guard and naming the false conjunct, or pointing to the assumption that bound it.
- Record-time slowdowns of 1.6–80x (Whyline, TOD) are irrelevant for us because the interpreter itself is the recorder; the binding constraint is UI latency, where Pernosco's 1–2 s for minutes of execution is the bar to beat by an order of magnitude.

### Gaps
- Chronon (Java "DVR" debugger): found no reliable current primary source on its architecture, numbers or status.
- WinDbg TTD record-time slowdown: Microsoft gives only qualitative statements in the pages read; no measured figure found.
- Pernosco internals (schema, index structure, size per instruction) are not published beyond blog posts.
- Replay.io's checkpoint/fork details sit in a blog series that could not be fully retrieved; the company's late-2026 product status was not checked.

## 2. Model-level and DSL debuggers: GEMOC, Moldable Debugger, statecharts/DEVS, TLA+, P

### Takeaway
Model-level debuggers already provide most of what we want at the model level. GEMOC generates an omniscient debugger and a domain-specific, multidimensional trace (one timeline per dynamic field) for any executable DSL. Van Mierlo et al. give a generalized stepping semantics for model debugging and "god events" to edit state mid-simulation. The TLA+ debugger (DAP-based, 2022; interactive state-space exploration added January 2026) shows how to map step-into/over onto choosing a successor state and how to drive per-state SVG animations. P's Peasy viz (on ShiViz) shows message-sequence-chart viewing, motif search and pairwise trace comparison for model-checker counterexamples. The Moldable Debugger shows that domain views and operations can be cheap plug-ins switched on by activation predicates.

### Cited Findings
**GEMOC Studio / executable DSLs (Bousse et al.)**
- SLE 2015: a partly generic omniscient debugger for executable DSMLs, backed by generated, domain-specific trace management (a domain-specific trace metamodel per language) — [Bousse et al., SLE 2015](https://gray.cs.ua.edu/pubs/sle-2015.pdf)
- Versus copying the model at every step, the approach uses about 6x less memory on average and is at least 2.2x faster when exploring past states, at about 1.6x slower forward execution (flagged: from abstract snippet) — [Omniscient debugging for executable DSLs, JKU summary](https://se.jku.at/omniscient-debugging-for-executable-dsls/)
- SoSyM 2019: generic trace metamodels are slow (sequential structure), memory-hungry (capture superfluous data) and hard to use (conceptual gap with the language); a generated multidimensional, domain-specific trace metamodel gives significant performance gains and much simpler semantic-differencing rules on fUML — [Bousse, Mayerhofer, Combemale, Baudry, SoSyM 2019](https://repositum.tuwien.at/handle/20.500.12708/950); a later compact trace metamodel (CTM) compresses traces losslessly — [same record](https://repositum.tuwien.at/handle/20.500.12708/950)
- GEMOC 3.0 user guide: generating a trace add-on from the xDSML "turns the debugger into an omniscient debugger with forward/backward features and advanced visualization of data changes in the model" — [GEMOC user guide 3.0](https://download.eclipse.org/gemoc/docs/releases/3.0.0/userguide-lw-provide-language-with-tools.html)
- For compiled DSLs, a feedback manager maps execution states of the compiled target back to the source model, enabling omniscient debugging at the source level (MDEbug 2017) — [Domain-level debugging for compiled DSLs](https://se.jku.at/domain-level-debugging-for-compiled-dsls-with-the-gemoc-studio-tool-demo/)
- Concurrency: earlier frameworks were limited to sequential traces; a 2022 SoSyM paper defines a generic protocol for model concurrency and demonstrates a concurrent omniscient debugger that explores different interleavings — [Zschaler et al., SoSyM 2022](https://link.springer.com/article/10.1007/s10270-022-01073-2)

**Moldable Debugger / Glamorous Toolkit (Chiș et al.)**
- A domain-specific debugger = a domain-specific extension (custom views and stepping operations) + an activation predicate that says when it applies at run time — [Chiș et al., Practical domain-specific debuggers using the Moldable Debugger, COMLAN 2015](https://scg.unibe.ch/assets/archive/papers/Chis15c-PracticalDomainSpecificDebuggers.pdf), [Moldable Debugger project](https://scg.unibe.ch/research/moldabledebugger)
- Cost evidence: six custom debuggers built in 60–600 lines each on a 1,500-line template (flagged: from search snippet of the PLATEAU 2015 paper) — [Towards moldable development tools](https://scg.unibe.ch/archive/papers/Chis15d-TowardsMoldableDevelopmentTools.pdf); example: a PetitParser debugger with parser-level stepping — [PetitParser debugger](https://scg.unibe.ch/assets/download/andrei/PetitParserDebugger-ParsingSLE-Abstract.pdf)

**Statecharts and DEVS (Van Mierlo, Van Tendeloo, Vangheluwe)**
- Debugging Parallel DEVS: transposes code-debugging concepts to DEVS, including "god events" that alter model state mid-simulation and reversible debugging; the simulator's modal behaviour and debug operations are themselves specified as a Statechart — [Debugging Parallel DEVS](https://repository.uantwerpen.be/docman/irua/014f33/144592.pdf), [record](https://eld.bl.uk/catalog/100069160729_0x000001)
- A generalized stepping semantics for model debugging (MDEbug 2018) unifies the different step vocabularies of formalisms (big step, small step, etc.) — [A Generalized Stepping Semantics for Model Debugging](https://www.academia.edu/72258379/A_Generalized_Stepping_Semantics_for_Model_Debugging)
- Extended to dynamic-structure DEVS (entities created and destroyed during simulation), with a reusable architecture and PythonPDEVS and NetLogo debugger prototypes (ACM TOMACS 2020) — [Extending explicitly modelled simulation debugging environments with dynamic structure](https://www.bohrium.com/paper-details/extending-explicitly-modelled-simulation-debugging-environments-with-dynamic-structure/812681664214859776-2793)

**TLA+ debugger and explorers**
- The TLA+ Debugger was first released in 2022, runs on TLC, and is built on DAP; it ships for VS Code and Cursor and "will work in any IDE that supports DAP" — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- Breakpoints: "halt after Init and Next"; conditional breakpoints on Init and Next, e.g. `~ENABLED Next` stops at deadlocked or terminal states — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- Stepping over a nondeterministic state space: step-in picks the successor with minimal Hamming distance from the current state, step-over the one with maximal distance, step-back goes to a predecessor (including back to the initial-state set); finite stuttering is kept so traces stay faithful behaviours — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- Interactive state-space exploration (January 2026): the user picks an initial state among all satisfying Init and a successor among all Next allows, moving freely forward and back; credited to Will Schultz's Spectacle — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger), [announcement](https://discuss.tlapl.us/msg06620.html)
- Animation: a watch expression on an animation module's `AnimView` renders an SVG per state as you navigate; the docs note an AI-generated animation module built from the animation guide shipped with the TLA+ MCP server — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- Watch and breakpoint expressions can load modules that export the current trace (`_JsonTrace`, `_TLCTrace` binary for long traces, `_DotTrace`, etc.), aimed at re-checking saved traces as the spec evolves — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- The debugger hooks TLC's interpreter to intercept evaluation of the Next relation — [Kuppe on the TLA+ group](https://groups.google.com/g/tlaplus/c/wuJbBRqZdTQ); in August 2026 the working entry point is "TLA+: Check and debug model with TLC" — [TLA+ group thread](https://groups.google.com/g/tlaplus/c/tO1EeTV-yF0)
- Spectacle: a browser-only TLA+ interpreter (tree-sitter parser compiled to WASM, JavaScript evaluator) for picking enabled actions, backtracking, trace expressions evaluated at every state, and sharing traces as static links; it is explicitly not a fast model checker — [Schultz, interactive formal specs, 2024](https://will62794.github.io/formal-methods/specification/2024/12/12/interactive-formal-specs.html), [TLA+ Conf 2024 slides](https://conf.tlapl.us/2024/WillSchultz-TowardsBetterInteractiveFormalSpecifications.pdf)

**P language**
- Peasy's trace visualizer (VS Code) shows P checker counterexamples as message sequence charts, supports search and motif queries (send/receive patterns with constraints), clustering of many traces (by machine count or against a base trace) and a PAIRWISE side-by-side view; it is built on ShiViz — [Peasy trace visualizer](https://p-org.github.io/peasy-ide-vscode/trace-visualizer/getting_started), [motifs](https://p-org.github.io/peasy-ide-vscode/trace-visualizer/feature_motifs), [compare and contrast](https://p-org.github.io/peasy-ide-vscode/trace-visualizer/feature_compare_contrast)

### Inferences
- GEMOC's central result maps directly onto our design: a trace whose shape is generated from the model's state variables (one value timeline per variable, plus a step timeline) beats a generic event list in memory, speed and diff simplicity. Our record families (state variables, ports, assumptions) are the "dynamic fields" that define the dimensions.
- The TLA+ debugger is the closest precedent for our symbolic and exhaustive modes. Its step-in/over choice among successors shows that in a nondeterministic interpreter, "step" needs a policy for which successor to take; we should expose that choice (a fork picker) rather than hide it behind a heuristic. Our step into/over/out should follow refinement levels (Van Mierlo's generalized stepping semantics), not call depth.
- "God events" (DEVS) and TLA+ interactive successor choice are the model-level form of "set variable": edits are interpreter events that are recorded in the trace, so a what-if branch remains a valid, replayable run.
- The Moldable Debugger's activation predicates fit the pack architecture: a behavior pack (TLS, Celery) could ship a domain view and stepping vocabulary (e.g. "step to next TLS record") that activates when the focus is inside its entities.

### Gaps
- SCXML/statechart debuggers (Qt SCXML, itemis CREATE/YAKINDU, SCION): no primary sources retrieved; their breakpoint and stepping models are unverified here.
- Ptolemy II and Simulink debugging (e.g. Simulink's stepping-back, Ptolemy's actor-level breakpoints): not researched within the call budget; no citations.
- GEMOC user-study evidence (whether users find omniscient model debugging effective) was not found; the published evaluations are performance-oriented.
- No user study of the TLA+ debugger or Peasy viz was found.

## 3. Generic debugger infrastructure: can a model interpreter expose DAP?

### Takeaway
Yes, and it is well precedented: the TLA+ debugger is a non-traditional DAP adapter over a model checker. DAP already has the capabilities the checkpoint lists (reverse stepping, reverse continue, set variable/expression, data breakpoints as watchpoints, conditional breakpoints, goto, stepIn targets, stepping granularity, an invalidated event). Its model is still threads, stack frames, scopes and variables, so DAP fits as a secondary, text-level surface (IDE and agent integration), not as the primary visual debugger.

### Cited Findings
- DAP defines the protocol between an editor/IDE and a debugger or runtime; it is a wire protocol so adapters can be written in any language — [DAP overview](https://microsoft.github.io/debug-adapter-protocol/overview.html)
- Reverse debugging: `stepBack` and `reverseContinue` requests, gated by the `supportsStepBack` capability — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)
- State editing: `setVariable` (`supportsSetVariable`) and `setExpression` (`supportsSetExpression`), the latter used when a variable has an `evaluateName` — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)
- Watchpoints: `dataBreakpointInfo` and `setDataBreakpoints` (`supportsDataBreakpoints`), with access types; the stopped reason is `data breakpoint` — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)
- Navigation: `goto`/`gotoTargets` (`supportsGotoTargetsRequest`); `stepIn` takes an optional `targetId` from `stepInTargets`; `restartFrame` (`supportsRestartFrame`); `next`, `stepIn`, `stepOut` and `stepBack` take an optional `granularity` defaulting to `statement` — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)
- The `invalidated` event (`supportsInvalidatedEvent`) tells the client to refetch areas (default `all`), which an adapter uses after state changes not caused by stepping — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)
- TLA+ is a working non-code DAP adapter: states and actions are presented through DAP in VS Code and Cursor, with conditional breakpoints, step-back and SVG animation through watch expressions — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- GEMOC exposes websocket endpoints from Eclipse (`org.eclipse.gemoc.ws.server`), but I found no source saying it implements DAP — [GEMOC 3.5 components documentation](https://download.eclipse.org/gemoc/docs/releases/3.5.0/_components_documentation.html)

### Inferences
- Mapping for our interpreter: DAP "thread" = lane; "stack frame" = the refinement path of the current step (scenario > segment > sub-step), so step-out = collapse one level; "scopes/variables" = the focused entity's state variables, messages in flight and assumptions; `stepInTargets` = the enabled transitions or forks at this step (TLA+'s successor choice); `goto` = jump to a step ID; `setVariable` on an assumption = flip it; `stepBack`/`reverseContinue` = backward search over the stored trace; data breakpoints = watchpoints on state variables; conditional breakpoints = predicates over the configuration.
- DAP has no notion of camera, zoom in space, playback speed, trace diff or conformance verdicts; those stay in our own protocol. The value of DAP is that coding agents and IDEs that already speak it can drive the interpreter without a custom integration.
- Because the interpreter can recompute any step, adapter requests can be answered from the trace store (as Pernosco answers gdb from its database) rather than by running a live process.

### Gaps
- Prolog, Dafny or other DSL DAP adapters as further precedents: not verified in this pass.
- The current DAP protocol version and changelog (2025–2026 additions) were not read; the specification fetch was truncated before the Types section.
- No source found on how well VS Code's UI handles reverse stepping from non-native adapters in practice.

## 4. Trace visualization at scale, trace diff, and evidence from comprehension studies

### Takeaway
The architecture to copy is Perfetto's: a columnar trace store with an SQL engine, run in the browser as WebAssembly in web workers, with GPU-rendered tracks and virtualized track scrolling. Comparison tools converge on structural collapse plus aligned diff: Jaeger collapses traces to unique service/operation paths; Trace Compass and Contrast (2026) use differential flame graphs and critical paths; ShiViz and Peasy compare executions pairwise. Controlled studies show trace visualization helps: Extravis cut time by 22% and raised correctness by 43%, and ShiViz significantly improved comprehension with a very large effect size.

### Cited Findings
**Perfetto (reference architecture)**
- Trace processor pipeline: ForwardingTraceParser → format-specific ChunkedTraceReader → TraceSorter → TraceStorage → SQL engine; chunked processing for large files; columnar storage with specialised tables (slice, process, thread, counter) — [Perfetto trace processor architecture](https://perfetto.dev/docs/design-docs/trace-processor-architecture)
- The UI runs the trace processor as WebAssembly, uses a WebWorker-based multi-threading design to stay responsive, and processes traces locally with no server — (flagged: from search summary of Perfetto docs) [Perfetto docs index](https://flutter.googlesource.com/third_party/perfetto/+/HEAD/docs/toc.md)
- v55.2 changelog (2026-05-14) mentions WebGL rendering of slice and counter tracks with significant gains on large traces, and virtual track scrolling (only visible tracks rendered) (flagged: read from a Flutter-hosted mirror, not the official changelog) — [Perfetto CHANGELOG mirror](https://flutter.googlesource.com/third_party/perfetto/+/refs/heads/dev/sg/theme/CHANGELOG)
- UI "tracks" are a UI-level organization that does not map 1:1 to trace-processor tracks; stable identifiers (utid, upid) separate identity from OS ids — [Perfetto MCP concepts doc](https://glama.ai/mcp/servers/@antarikshc/perfetto-mcp/blob/fcc8d6e90a302b3191b9f3bd4a7ff205a70dd3ca/docs/Perfetto-MCP-Concepts.md)

**Trace comparison and diff**
- Jaeger 1.7 (2018, older) trace comparison condenses each trace into a tree of unique service/operation paths (a HotROD example: 8 comparison nodes for 50 spans) and highlights paths present in only one trace or with different counts — [Trace comparisons arrive in Jaeger 1.7](https://medium.com/jaegertracing/trace-comparisons-arrive-in-jaeger-1-7-a97ad5e2d05d); presented as "trace diffs" with unified, shared and absent-structure views — [KubeCon 2018 Jaeger deep dive](https://raw.githack.com/mm-k8s-ug/kubecon-slides/master/slides/2018-kubecon-na/Deep%20Dive%20Jaeger%20-%20Pavol%20Loffay,%20Red%20Hat%20&%20Yuri%20Shkuro,%20Uber%20-%20KCCNCNA18%20Jaeger%20Deep%20Dive.pdf); it compares structure, not timing — [SigNoz community thread](https://community-chat.signoz.io/t/443762/hello-team-help-me-please-can-the-signoz-compare-two-traces-?t=v2)
- Trace Compass Execution Comparison compares two groups of traces (good vs bad run, before vs after a patch) with a differential flame graph (red longer, blue shorter) — [Trace Compass Execution Comparison user guide](https://archive.eclipse.org/tracecompass.incubator/doc/org.eclipse.tracecompass.incubator.executioncomparison.doc.user/User-Guide.html), [DORSAL slides, Dec 2023](https://www.dorsal.polymtl.ca/files/december2023/fdaneshgar_december2023_execution_comparison_trace_compass.pdf)
- TraceCompare (2016) compares multi-level executions using a critical-path algorithm to extract the segments that contribute to a task's latency — [Doray & Dagenais 2016](https://publications.polymtl.ca/2961/1/2016_Doray_Diagnosing_performance_variations_comparing_multi-level.pdf)
- Contrast (Anand, Fonseca, Mace, Kaufmann; arXiv July 2026): a mergeable "Trace Projection Object" captures structural, temporal, critical-path and semantic properties of trace populations, so comparison sets can be built at query time; separates trace representation from comparison semantics; interfaces are SpectroViz (critical-path-based visual localization of differences) and Parallax (LLM natural-language explanation of differences); evaluated on DeathStarBench and Uber production traces (abstract gives no numbers) — [Contrast, arXiv 2607.19102](https://arxiv.org/abs/2607.19102)

**Comprehension evidence**
- Cornelissen et al.'s systematic survey of program comprehension through dynamic analysis selected 176 of 4,795 articles (1999–2008) and classified them by activity, target, method and evaluation (older) — [TSE 2009](https://repository.tudelft.nl/record/uuid:2ed676bb-99b5-4ce4-afea-c9ed977e89d6)
- First controlled experiment on trace visualization: Eclipse + Extravis versus Eclipse alone, 8 typical tasks; 22% less time and 43% more correct answers, both statistically significant (older) — [Cornelissen, Zaidman, van Deursen, TSE 2011](https://azaidman.github.io/publications/cornelissenTSE2010.pdf), [ICPC 2009](https://azaidman.github.io/publications/cornelissenICPC2009.pdf)
- ShiViz renders logs with vector clocks as interactive time-space diagrams for three tasks: relative ordering, searching interaction patterns, and structural similarities and differences between two executions; two user studies with 109 students plus a 2-developer case study found significantly more comprehension questions answered correctly, with a very large effect size — [Beschastnikh et al., Visualizing distributed system executions, TOSEM 2020](https://www.cs.ubc.ca/~bestchai/papers/tosem20-shiviz.pdf); runs entirely in the browser, so logs never leave the machine — [same](https://www.cs.ubc.ca/~bestchai/papers/tosem20-shiviz.pdf), [ACM Queue 2016](https://homes.cs.washington.edu/~mernst/pubs/debug-distributed-queue2016.pdf)

### Inferences
- The checkpoint's Rust core in WASM plus WebGPU renderer matches the Perfetto direction (WASM trace processor in workers, a move to GPU track rendering, virtualized tracks). A SQL-like query layer over a columnar step/variable store would give breakpoints, watchpoints, "line executions" lists and diffs one mechanism.
- For fix-check diffs, our semantic step IDs (`parent/kind:slug`) make alignment of two runs of the same binding and schedule nearly free: join on step ID, then report added, removed and changed steps and per-variable value differences. Jaeger's lesson is to diff at a collapsed level first (unique entity/port paths with counts) and expand into the differing subtree. Its weakness, structure without timing, is avoided by also diffing presentation and observed time.
- ShiViz and Peasy evidence supports time-space (lane) diagrams with happens-before edges as the default "flat" view of a pomset script, beside the spatial canvas.

### Gaps
- No controlled study found on video-like playback or animation of traces versus static diagrams for comprehension; this is the main open UX risk for the "play like a video" idea.
- Fittkau et al.'s Extravis vs ExplorViz comparison results were not retrieved — [ICPC 2015 slides](https://oceanrep.geomar.de/id/eprint/28324/7/ICPC2015_Comparing_presentation.pdf).
- Perfetto UI frame-time or trace-size limits (e.g., GB-scale traces in browser memory) not found as published numbers.
- Sequence-diagram generation from traces (beyond ShiViz/Peasy) not covered.

## 5. Conformance checking and runtime verification for observed traces

### Takeaway
Three families apply. (1) Lockstep monitors: PObserve parses service logs into events, sequences them into one sorted stream and feeds them to P spec monitors; Copilot and JavaMOP give the cost model for online monitors. (2) Constrained model checking: TLA+ trace validation asks TLC whether some behaviour of the spec matches a partial trace that records only some variable updates; it found spec/implementation discrepancies in every system it was applied to. (3) Process-mining alignments: optimal alignments explain deviations as "log moves" and "model moves" but cost exponential time in the worst case; prefix-alignments (online) and sliding windows make them incremental and bounded. Token replay is mostly abandoned.

### Cited Findings
**PObserve and P**
- PObserve is a distributed runtime-monitoring framework that checks P specifications against implementations using existing service logs, with no extra instrumentation; usable in design, testing and production — [PObserve docs](https://p-org.github.io/P/advanced/pobserve/pobserve)
- Pipeline: user log parser → time-stamped events → a sequencer merges them into a globally sorted stream → a demultiplexer routes each event to the P spec monitors that observe it; implemented with Java Streams — [PObserve docs](https://p-org.github.io/P/advanced/pobserve/pobserve)
- The log parser maps each log line to a PObserve event class generated by compiling P specs in PObserve mode; parsers run multithreaded — [PObserve log parser](https://p-org.github.io/P/advanced/pobserve/logparser)
- P case studies at AWS — [P case studies](https://p-org.github.io/P/casestudies)

**TLA+ trace validation**
- Cirstea, Kuppe, Loillier, Merz (SEFM 2024): trace validation as constrained model checking with TLC; traces contain only updates to chosen specification variables and may be partial; a Java instrumentation API, TLA+ operators to relate traces to specs, and scripts; discrepancies found in all programs studied; also used to "reverse engineer" a specification from an existing program — [arXiv 2404.16075](https://arxiv.org/abs/2404.16075v2)
- Microsoft's Confidential Consortium Framework applied TLA+ trace validation of implementation traces ("smart casual verification") — [Murat Demirbas summary, 2025](https://muratbuffalo.blogspot.com/2025/02/smart-casual-verification-of.html)

**Process-mining conformance**
- Optimal alignment is exponential in the worst case (A* search; branching factor to the power of depth); recent accelerations do not change this; splitting long traces into windows of length L gives near-optimal results at O(b^{2L}) per window — [A scalable and near-optimal conformance checking approach for long traces, 2024](https://arxiv.org/pdf/2406.05439)
- ConLES (Bogdanov, Cohen, Gal, 2025): sliding-window alignments for long event sequences, significantly lower cost with near-optimal accuracy — [arXiv 2505.21506](https://arxiv.org/html/2505.21506v1); REACH speeds A*-based alignment — [REACH](https://citius.gal/research/publications/reach-researching-efficient-alignment-based-conformance-checking/)
- Online conformance: van Zelst et al. (2019) compute prefix-alignments on event streams; reusing previous prefix-alignments improves memory efficiency while keeping optimality; approximate variants trade memory for error — [Online conformance checking using prefix-alignments, IJDSA](https://pa.win.tue.nl/online-conformance-checking-relating-event-streams-to-process-models-using-prefix-alignments/); incremental computation continues the shortest-path search on an extended synchronous product when an event arrives, and has been distributed on Kafka — [Scalable online conformance checking, arXiv 2101.00958](https://arxiv.org/pdf/2101.00958)
- Token-based replay was once the default but is "almost abandoned" because invisible and duplicate transitions need heuristics; alignments are the main technique, though a faster PM4Py token replay exists (Berti & van der Aalst 2020) — [arXiv 2007.14237](https://arxiv.org/abs/2007.14237v1); alignment-based precision can be non-deterministic when several optimal alignments exist — [arXiv 2609.22614](https://arxiv.org/pdf/2609.22614)

**Runtime verification frameworks**
- Copilot (NASA): a Haskell stream EDSL generating hard real-time C99 monitors that run in constant memory and time; Ogma integrates monitors into cFS and ROS 2 — [Copilot on Hackage](https://hackage.haskell.org/package/copilot-4.6/src/README.md), [copilot-c99](https://hackage.haskell.org/package/copilot-c99)
- Parametric monitoring overhead on DaCapo: RV-Monitor averages about 15%, about half of JavaMOP (e.g., avrora 62% vs 139%, pmd 94% vs 231%) — [RV-Monitor, RV 2014](https://fsl.cs.illinois.edu/publications/luo-zhang-lee-jin-meredith-serbanuta-rosu-2014-rv.pdf); with 137 simultaneous specs, scalable JavaMOP averages 147% versus 262% before (older, 2012) — [Scalable parametric runtime monitoring](https://fsl.cs.illinois.edu/publications/jin-meredith-rosu-2012-tr.html), [JavaMOP ICSE 2012](https://fsl.cs.illinois.edu/publications/jin-meredith-lee-rosu-2012-icse.html)

### Inferences
- Our conformance mode should be layered by cost. Level 1 (cheap, default): PObserve-style lockstep. Map observed log lines to model events with a parser from the pack, sequence them by observed time within happens-before constraints, and step the concrete interpreter; the first event with no enabled matching transition is the divergence, shown on the timeline. Level 2: TLA+-style constrained search. When observations are partial (only some steps are logged, which is the normal case), search the interpreter's state space for a run consistent with the observations, binding unknown predicates as it goes; a found run becomes a script with provenance `observed` on matched steps. Level 3: optimal alignment with log moves and model moves, bounded by windowing or prefix-alignment, used only to explain a divergence ("the model expected `ack` before `retry`; the log skipped it").
- Matched assumptions get promoted: a predicate the observed trace forces (e.g. `upload-ok = false` on the first attempt) can turn an `assumed` record into `observed`, which is exactly the checkpoint's provenance axis.
- The pomset script makes ordering tolerant: observed events that are incomparable in happens-before may appear in any order in logs, which avoids most false divergences that a total-order sequencer would report.

### Gaps
- PObserve's handling of out-of-order or late events, partitioning, scale numbers and violation output are not documented on the pages read.
- No published cost figures were found for TLA+ trace validation runs (state counts, wall time per trace length).
- The Carmona et al. "Conformance Checking" book (2018) was not directly accessed; findings come from papers.
- No source compares alignments with model-checking-based trace validation on the same problems.

## 6. What-if / counterfactual debugging: editing state mid-replay, fix-and-continue

### Takeaway
Editing state in a recorded run is rare in code debuggers because it breaks replay determinism. The precedents isolate the edit as a fork: Microsoft's "tentative execution" patents run a hypothetical branch in isolated state and discard it; database "reenactment" turns edits into what-if queries without touching the real data; DEVS debuggers use "god events"; comparative causality and causal testing apply controlled state swaps or input changes to explain differences between runs. For a model interpreter this is natural: an edit is a new binding or an injected event, and the result is a sibling run that can be diffed against the original.

### Cited Findings
- Microsoft patents describe tentative execution during live or replay debugging: the hypothetical run's state is isolated from the original, can be discarded, and the session resumes at the suspension point; the goal is exploring how execution would differ if parts of state changed — [US10565087B2](https://patents.google.com/patent/US10565087), [US11055197](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11055197)
- Transaction debugger by reenactment: replays past transactions as queries so the live database is not changed; editing a statement or table content yields a what-if scenario — [Arab et al., arXiv 1707.09930](https://arxiv.org/pdf/1707.09930)
- Comparative causality (ICSE 2013, older): replaces part of program state at an earlier point to test whether a difference between two executions can be induced, analysing both symmetrically — [Comparative Causality, ICSE 2013](https://2013.icse-conferences.org/content/comparative-causality-explaining-differences-between-executions.html)
- Causal Testing (Johnson, Brun, Meliou, ICSE 2020): counterfactual causality by modifying and executing tests to observe effects and isolate root causes — [arXiv 1809.06991](https://arxiv.org/pdf/1809.06991.pdf)
- DEVS "god events" alter model state mid-simulation inside a debugger that also supports reverse stepping — [Debugging Parallel DEVS](https://repository.uantwerpen.be/docman/irua/014f33/144592.pdf)
- TLA+ interactive exploration lets the user choose any successor allowed by Next at each step, i.e. steer the nondeterminism by hand — [TLA+ debugger docs](https://docs.tlapl.us/_export/raw/using:debugger)
- DAP already carries `setVariable`, `setExpression`, `restartFrame` and `goto` — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification.html)

### Inferences
- Make every edit a recorded interpreter event (assumption flip, state assignment, injected message, scheduler choice) that starts a branch of the run tree. The original run stays immutable, the branch is replayable from (binding + schedule + edits), and the fix-check diff and the what-if diff are the same operation.
- After an edit, re-solve forward lazily, as Expositor does: compute the branch only as far as the playhead or the viewport needs, and reuse the prefix up to the edit point.
- Model edits ("fix check") differ from state edits: after a model change, step IDs may no longer match, so the diff must fall back to alignment (section 5) for steps whose behavior node changed. GEMOC's SoSyM result suggests diff rules stay simple when the trace is domain-structured.

### Gaps
- No user study found on the effectiveness of mid-run state editing or what-if branches in debuggers.
- "Fix and continue" / hot code replacement in live programming environments (Smalltalk, Elm, Redux DevTools, Hazel) was not researched with primary sources in this pass.
- The CausalFlow paper on counterfactual repair of agent trajectories (2026) was seen only via a secondary summary — [agentpatterns.ai summary](https://agentpatterns.ai/verification/causalflow-counterfactual-agent-repair/); not verified.

## 7. Recommendations: architecture, interaction primitives, conformance approach

### Takeaway
Build a Pernosco-like debugger over a Replay-like store: the run is defined by (model version, binding, schedule, edits). Keep periodic configuration snapshots plus a GEMOC-style per-variable change index, queryable through a Perfetto-like columnar SQL engine in WASM workers. Lead the interaction with queries and "why/why not" questions, with playback and stepping as navigation. Expose a DAP adapter for IDEs and agents. Do conformance in three tiers: lockstep monitor, constrained search for partial traces, bounded alignment for explanations.

### Cited Findings
- Query-first debugging (retroactive print/conditions over all hits, evaluated against a database) is Pernosco's core, and it steps through minutes of execution in 1–2 s — [Pernosco printf](https://robert.ocallahan.org/2019/11/omniscient-printf-debugging-in-pernosco.html), [Pernosco gdb](https://robert.ocallahan.org/2019/11/supercharging-gdb-with-pernosco.html)
- Question-based debugging made novices about 2x faster than experts and cut median time from 10 to 4 minutes — [Whyline ICSE 2008](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf)
- Domain-specific multidimensional traces: about 6x less memory and at least 2.2x faster past-state access than snapshot-per-step, plus simpler semantic diffs — [JKU summary](https://se.jku.at/omniscient-debugging-for-executable-dsls/), [SoSyM 2019](https://repositum.tuwien.at/handle/20.500.12708/950)
- Lazy materialization of traces keeps scripted queries cheap — [Expositor](https://www.cs.umd.edu/~mwh/papers/khoo13expositor.html)
- Record inputs and recompute state is the low-overhead path (rr under 2x; Replay.io computes everything at replay time) — [rr](https://arxiv.org/abs/1705.05937), [Replay docs](https://docs.replay.io/basics/time-travel/how-does-time-travel-work)
- Columnar store + SQL in WASM workers + GPU tracks + virtualized tracks — [Perfetto architecture](https://perfetto.dev/docs/design-docs/trace-processor-architecture), [CHANGELOG mirror](https://flutter.googlesource.com/third_party/perfetto/+/refs/heads/dev/sg/theme/CHANGELOG)
- DAP covers reverse stepping, set variable, data breakpoints, step-in targets and granularity; TLA+ proves the fit for a model checker — [DAP spec](https://microsoft.github.io/debug-adapter-protocol/specification.html), [TLA+ debugger](https://docs.tlapl.us/_export/raw/using:debugger)
- Lockstep spec monitors on logs (PObserve), constrained model checking of partial traces (TLA+), incremental prefix-alignments (process mining) — [PObserve](https://p-org.github.io/P/advanced/pobserve/pobserve), [Cirstea et al.](https://arxiv.org/abs/2404.16075v2), [van Zelst et al.](https://pa.win.tue.nl/online-conformance-checking-relating-event-streams-to-process-models-using-prefix-alignments/)
- Structural collapse before diff (Jaeger), differential coloring (Trace Compass), population comparison with critical paths (Contrast 2026) — [Jaeger 1.7](https://medium.com/jaegertracing/trace-comparisons-arrive-in-jaeger-1-7-a97ad5e2d05d), [Trace Compass](https://archive.eclipse.org/tracecompass.incubator/doc/org.eclipse.tracecompass.incubator.executioncomparison.doc.user/User-Guide.html), [Contrast](https://arxiv.org/abs/2607.19102)

### Inferences
**Architecture (trace store and indexing)**
- Run identity = hash(model version, binding, schedule choices, edit list). Content hashes are cache keys, not identity, consistent with the checkpoint.
- Store per run: (a) the step log (step ID, lane, refinement level, parent macro step, transition, `after` edges, the three clocks); (b) per-state-variable change lists (step index → value), the GEMOC "dimensions", which make watchpoints and "when did X last change" O(log n); (c) configuration snapshots every k steps and at every macro-step boundary, so any step is reached by snapshot + at most k small steps (the rr/Replay model, with no divergence risk); (d) per-port message-in-flight intervals for the canvas.
- Fine levels are computed lazily: a macro step stores only its summary and boundary snapshots, and expanding it re-runs the interpreter at the finer level from the boundary snapshot (Expositor-style laziness; matches "compute detail on demand").
- Query layer: SQL (or a small typed DSL compiled to it) over columnar tables `step`, `var_change`, `message`, `assumption`, `segment`, running in WASM workers next to the renderer, as Perfetto does. Breakpoints, watchpoints, conditional breakpoints and "show every delivery to this port" are all queries; a breakpoint is a saved query whose first hit after the playhead stops playback.

**Interaction primitives (in priority order)**
1. Query list ("executions" panel, from Pernosco): click a port, transition or variable and get every occurrence in the run with chosen expressions evaluated at each; clicking a row moves the playhead.
2. Why / why-not (Whyline): on any step, "why did this happen" walks back through the enabling guard, the predicate bindings and the causal `after` edges; on any untaken fork, "why didn't this happen" names the false guard conjunct or the assumption that bound it. Assumptions are the natural leaves of these explanations.
3. Playback with the three clocks; step into/over/out move along refinement levels (generalized stepping semantics), and reverse versions of each are queries over the store, not re-execution.
4. Fork picker at nondeterministic points (TLA+ interactive exploration): the scheduler's choice is visible and changeable, which creates a sibling run.
5. Edits (flip assumption, set state, inject message) as recorded events that branch the run tree; the diff between parent and branch is always one click away.
6. Diff view: join runs on semantic step IDs, collapse to entity/port paths with counts (Jaeger), color by added, removed or retimed (Trace Compass), expand into the differing subtree; fall back to alignment for steps whose behavior changed.
7. Lane (time-space) view with happens-before edges (ShiViz/Peasy evidence) beside the spatial canvas; motif search over send/receive patterns.

**DAP exposure**
- Ship a DAP adapter as a secondary surface: lanes as threads, refinement path as the stack, the focused entity's state and assumptions as variables, enabled transitions as `stepInTargets`, `goto` to step IDs, `stepBack`/`reverseContinue`, data breakpoints for watchpoints, `setVariable` for edits, `invalidated` after edits. This gives agents and editors a standard control channel; the visual canvas stays on our own protocol, which DAP cannot express (camera, zoom, playback speed, diffs, conformance verdicts).

**Conformance**
- Tier 1, lockstep: pack-supplied log parsers map lines to model events (PObserve's parser/sequencer/demux), checked against the pomset rather than a total order.
- Tier 2, constrained search for partial observations (TLA+ trace validation): BFS or DFS over interpreter configurations restricted to runs consistent with the observed events, binding unknown predicates; the result becomes a script whose matched steps carry `observed` provenance.
- Tier 3, explanation: bounded optimal alignment (windowed, or incremental prefix-alignments for live streams) to report log moves and model moves at the first divergence; never run unbounded A* on long traces.

### Gaps
- No precedent found for a debugger that combines spatial zoom (entity nesting) with time-travel; the zoom-coupled stepping design is new and needs its own user testing.
- No published latency budgets for model-level omniscient debuggers (GEMOC, TLA+) to compare with the checkpoint's targets (60 fps, under 100 ms to expand a level).
- Snapshot interval k and index sizes for our configuration sizes need measurement on the gold models; no external number transfers directly.
