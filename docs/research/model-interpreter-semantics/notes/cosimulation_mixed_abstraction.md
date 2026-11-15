# Mixed-abstraction simulation and co-simulation as precedents for the model interpreter

Scope: SystemC/TLM-2.0, virtual platforms and ISS fidelity switching, FMI/SSP co-simulation, SIL/HIL and network emulation, service virtualization, debugger/record-replay coupling and refinement mappings, and what they imply for our interpreter (entities, ports, wires, state, predicates, guarded transitions; macro steps vs small steps; hand-off below the model's depth). State as of November 2026.

Source note: items marked "(training knowledge; not re-fetched this session)" are well-established facts I could not open the primary text for during this session; the URL given is the canonical primary location and should be spot-checked before quoting.

## 1. SystemC and TLM-2.0 (IEEE 1666): structure, coding styles, mixed TLM/RTL, consistency checking, status through 2026

### Takeaway
TLM-2.0 is the clearest precedent for "the user chooses granularity": the same module boundary (a socket) carries either a single blocking call with an annotated delay (loosely-timed) or a four-phase handshake (approximately-timed), transactors adapt transactions to pin-level RTL, and speed is bought with temporal decoupling under an explicit quantum. Its weak spot is exactly ours: consistency between levels is checked by simulation, protocol checkers and assertions, not proven; formal TLM-to-RTL equivalence is still described as an open problem.

### Cited Findings
**Status and evolution**
- The current LRM is IEEE Std 1666-2023, free under the IEEE GET program with Accellera sponsorship; announced 8 Nov 2023 — [Accellera press release](https://accellera.org/news/press-releases/386-accellera-announces-ieee-1666-2023-standard-available-through-ieee-get-program); [systemc.org](https://www.systemc.org).
- The reference implementation SystemC 3.0.0 implements IEEE 1666-2023 and makes C++17 the baseline; the version macro `IEEE_1666_SYSTEMC` is `202301L` — [SystemC release notes (mirror)](https://github.imc.re/accellera-official/systemc/blob/main/RELEASENOTES.md).
- The latest listed release is SystemC 3.0.2 (bug fixes: coarse time resolution, `sc_prim_channel` mutexes, more efficient `sc_time` under C++17, "initial thread-safety improvements", CMake improvements); 3.0.1 aligned `sc_bind` and `trigger_reset_event` with 1666-2023. No TLM-2.0 API changes are listed, only a member-initialization fix in `simple_target_socket`; no new deprecations — [SystemC RELEASENOTES.md](https://github.com/accellera-official/systemc/blob/main/RELEASENOTES.md). Release date of 3.0.2 not stated on that page.
- 1666-2023 adds stage callbacks (`sc_register_stage_callback`/`sc_unregister_stage_callback`), implemented since 3.0.0 despite signature typos in the online standard — [SystemC RELEASENOTES.md](https://github.com/accellera-official/systemc/blob/main/RELEASENOTES.md).
- OSCI TLM-2.0 was first released June 2008; TLM-2.0.1 with its LRM in July 2009; an open-source TLM-2.0 Base Protocol Checker exists to check model compliance — [Doulos TLM-2.0 knowhow](https://www.doulos.com/knowhow/systemc/tlm-20/).
- SystemC CCI (Configuration, Control & Inspection) 1.0 is the current CCI standard; reference implementation 1.0.1 was released in 2023 and development moved to Accellera's public GitHub — [Accellera newsletter Aug 2023](https://accellera.org/news/newsletters/2023-august); [Accellera TC update (DVCon)](https://dvcon-proceedings.org/wp-content/uploads/General2-Accellera-Overview-TC-Update.pdf).
- Accellera's Federated Simulation Standard working group (proposed 2023, approved mid-2024, first meetings Sept 2024) aims at an open API and infrastructure for interoperability between simulation frameworks; SystemC Summer of Code 2025 lists TLM, FMI, QEMU and Zenoh as technologies it combines — [Federated Simulation PWG intro](https://www.accellera.org/images/activities/Federated-Simulation-Standard-PWG-intro.pdf); [SystemC Summer of Code 2025](https://systemc.org/events/ssoc2025). I found no released federated-simulation standard as of Nov 2026 (see Gaps).

**Coding styles, temporal decoupling, quantum keeper**
- Loosely-timed (LT) models usually use the blocking interface; approximately-timed (AT) use the non-blocking interface; temporal decoupling generally suits LT — [Embecosm app note EAN1 §2.3.4](https://www.embecosm.com/appnotes/ean1/html/ch02s03s04.html); [EAN1 §9.1](https://www.embecosm.com/appnotes/ean1/html/ch09s01.html).
- In LT each transaction has exactly two timing points (call and return); delays passed in `b_transport`/`nb_transport` are local time offsets relative to the start of the quantum; LT is enough to boot an OS (timers, interrupts) — [Cambridge ESL notes p.28](https://www.cl.cam.ac.uk/teaching/0910/P35/obj2.2/page28.html).
- The quantum keeper is a utility that lets each decoupled thread track the global quantum, its local quantum and its local time offset; one per thread; temporal decoupling is a property of threads, not modules, and the standard does not enforce compliance — [EAN1 §9.1.3](https://www.embecosm.com/appnotes/ean1/html/ch09s01s03.html); [EAN1 §9.1.1](https://www.embecosm.com/appnotes/ean1/html/ch09s01s01.html).
- Large quantum = fast simulation; small quantum = more accurate interleaving of transactions — [Cambridge ESL notes p.29](https://www.cl.cam.ac.uk/teaching/0910/P35/obj2.2/page29.html).
- Temporal decoupling can cause functional errors from missed synchronization; prior fixes checkpoint and roll back but are intrusive (model and kernel changes); Jung et al. (DATE 2019) roll back non-intrusively using `fork()` as the checkpoint — [Speculative Temporal Decoupling Using fork(), DATE 2019](https://past.date-conference.com/proceedings-archive/2019/html/0086.html).
- Base protocol and payload (training knowledge; not re-fetched this session): TLM-2.0 defines initiator/target sockets bundling forward (`b_transport`, `nb_transport_fw`, `get_direct_mem_ptr`, `transport_dbg`) and backward (`nb_transport_bw`, `invalidate_direct_mem_ptr`) interfaces; the base protocol has four phases `BEGIN_REQ`, `END_REQ`, `BEGIN_RESP`, `END_RESP`; the generic payload carries command, address, data pointer/length, byte enables, streaming width, DMI hint and response status, plus an extension array for protocol-specific data (ignorable extensions keep base-protocol interoperability); TLM-2.0 was merged into IEEE 1666-2011 — [IEEE 1666-2023 via systemc.org](https://www.systemc.org).

**Mixing TLM and RTL; consistency**
- Transactors let TLM testbenches be reused in TLM–RTL co-simulation: refined TLM tests are driven into the RTL through a user-defined transactor and monitored outputs/assertions are compared — [Bombieri et al. as summarized in UF assertion paper](https://www.cise.ufl.edu/research/cad/Publications/vlsi13assertion.pdf).
- "Equivalence checking between system-level (i.e., TLM) and RTL implementations is still an open problem" because "there are neither temporal nor structural similarities between TLM and RTL descriptions" — [Bombieri/Fummi/Pravadelli, MEMOCODE 2007](https://eprints.soton.ac.uk/263822/1/jpms-memocode07.pdf). This is an older (2007) result; I found no newer source claiming the general problem solved.
- Their 2008 work defines TLM/RTL equivalence via simulation and quality measures on automatically generated stimuli rather than formal equivalence — [EWDTS 2008 record](https://unpaywall.org/10.1109%2FEWDTS.2008.5580149).
- Another line checks TLM-to-RTL consistency by refining TLM assertions to RTL and comparing assertion coverage and assertion ordering — [UF, VLSI 2013](https://www.cise.ufl.edu/research/cad/Publications/vlsi13assertion.pdf).
- Mixed TLM-2.0 AT/RTL work explicitly checks throughput/latency consistency and states "Functional equivalence checking between SystemC and RTL is a separate issue that is not addressed" — [DVCon: Bridging the gap between TLM-2.0 AT models and RTL](https://dvcon-proceedings.org/wp-content/uploads/bridging-the-gap-between-tlm-2-0-at-models-and-rtl-experiments-and-opportunities.pdf).
- Simulation-based TLM equivalence/coverage tooling exists (e.g., SimEC-TLM, GLSVLSI 2011) — [Uni Bremen](https://agra.informatik.uni-bremen.de/doc/konf/11glsvlsi-simecTLM.pdf).

### Inferences
- Our "collapsed segment = macro step" maps to LT `b_transport` (one call, one annotated delay); "expanded" maps to AT phases. TLM shows the same port (socket) can stay fixed while the protocol granularity on it changes; we should keep port identity invariant across granularity and vary only the *protocol phase set* visible at that port.
- The generic payload + extensions pattern is a direct template for our nested messages: a base message shape every entity understands, with typed, ignorable extensions per layer (TLS record fields, HTTP headers). Ignorable extensions are how a coarse entity can pass through a message it does not fully understand.
- Temporal decoupling is the hardware name for our "presentation time vs logical order" problem and for interleaving accuracy under collapse: a collapsed lane runs ahead within a quantum; correctness requires a rule for when it must sync (our `atomicity: non-atomic` never-collapse rule plays the role of a forced sync point).
- The TLM field's 15-year experience that cross-level equivalence is checked by simulation, protocol checkers and assertions (not proofs) suggests our "macro step must agree with its small steps" should be enforced as a runtime check (run both, compare projections) and a protocol checker per port, not as a static proof.

### Gaps
- Could not open IEEE 1666-2023 text this session; TLM phase/payload details above are training knowledge.
- No 2025–2026 Accellera announcement found for a new CCI version or a released Federated Simulation standard; Accellera news pages surfaced only through 2024.
- No 2020s survey found of formal TLM-vs-RTL equivalence (HLS C-to-RTL sequential equivalence checking is a related commercial area not covered here).

## 2. Virtual platforms, ISSs, and switching fidelity during a run

### Takeaway
Virtual platforms run software fast at a functional level and turn on detail only where needed: Simics changes quantum and enables timing/tracing mid-run; gem5 drains the system and swaps CPU models (KVM/atomic to detailed) at a checkpoint; sampling (SimPoint, LoopPoint) simulates only representative regions in detail and fast-forwards the rest, with explicit "warming" error. This is the closest precedent for "collapsed by default, expand on demand" with a well-defined hand-over protocol.

### Cited Findings
- Simics' goal is running target code at a "functional level of abstraction with simplified timing"; detailed timing and tracing "should only be turned on dynamically when needed"; the temporal-decoupling "time quantum length ... can be changed during a simulation"; the core gives device models "simple serial semantics"; target semantics are "the same regardless of the host"; existing simulators can be embedded in modules, making Simics an integration platform — [Intel Simics Simulator technology](https://www.intel.com/content/www/us/en/developer/articles/technical/simics-simulator-technology.html).
- Simics' SystemC Library: an adapter alone calls `sc_start`; SystemC time advances per Simics cycle via an adapter frequency (default 1 ps per cycle); each processor runs its time quantum before handing over — [Simics SystemC Library: execution](https://intel.github.io/simics/docs/systemc-library/execution-of-systemc-models-in-the-simics-simulator.html).
- Optimal Simics quanta are typically ~500k–1M instructions — [Intel: How Simics executes instructions](https://community.intel.com/t5/Blogs/Products-and-Solutions/Software/How-the-Intel-Simics-Simulator-Executes-Instructions/post/1543049); speed vs virtual-time trade-off discussed in [Intel: Chasing Simics performance](https://www.intel.com/content/www/us/en/developer/articles/technical/quit-thinking-and-look-chasing-simics-performance.html).
- QBox (Qualcomm/GreenSocs) embeds QEMU in SystemC as a TLM-2.0 component (libqemu → libqemu-cxx → libqbox) — [qualcomm/qbox](https://github.com/qualcomm/qbox); [ERTS 2016 paper](https://hal-hprints.archives-ouvertes.fr/ERTS2016/hal-01292317v1). A third-party evaluation reports the main obstacle is organizational: libqemu is not part of official QEMU — [MINRES](https://www.minres.com/?p=1430).
- Xilinx libsystemctlm-soc co-simulates Xilinx QEMU, SystemC/TLM-2.0 and RTL; MachineWare VCML + QEMU builds virtual ECUs — [QBox search context / DVCon Virtual ECUs](https://dvcon-proceedings.org/wp-content/uploads/Virtual-ECUs-with-QEMU-and-SystemC-TLM-2.0.pdf).
- gem5 CPU switching steps: drain the system, `switchOut()` old CPUs, change the memory system mode to what the new CPUs need, flush caches when moving to/from hardware virtualization; CPUs not initially running are created with `switched_out=True`; `--fast-forward N` runs N instructions before switching; checkpoints restore on Atomic CPU by default (`--restore-with-cpu` overrides) — [gem5 configs/common/Simulation.py (historical)](https://git.sanchayanmaity.net/sanchayanmaity/gem5/src/commit/82f600e02ded4d7162dd9286f790462b9ab5d554/configs/common/Simulation.py); [gem5 checkpoints doc](https://gem5.googlesource.com/public/gem5-website/+/1d2da29f1c8e66714ac24f59829eced33f63bf97/_pages/documentation/general_docs/checkpoints.md). These are older sources; mechanism persists in the stdlib switchable processor (next item).
- The gem5 stdlib uses a switchable processor that boots on KVM then switches to a detailed CPU, and can fork the simulation after boot — [boot_kvm_fork_run.py](https://gem5.googlesource.com/public/gem5/+/1b0b59f49f4779498cec7239a9d07427eb67ee2b/tests/gem5/configs/boot_kvm_fork_run.py); KVM only works when host and guest ISA match — [cnlelema notes](https://cnlelema.github.io/memo/en/arch/basics/gem5/gem5-kvm); KvmCPU cannot fast-forward past unsupported instructions (2025) — [gem5 ISCA 2025 PinCPU slides](https://www.gem5.org/_pages/static/events/isca_2025/01-pincpu-gem5-2025-06-21-export.pdf).
- gem5 20.0 paper: KVM used to write checkpoints later simulated in parallel; forked simulations estimate the cache-warming error KVM fast-forwarding introduces between samples — [The gem5 Simulator: Version 20.0+](https://arxiv.org/pdf/2007.03152); Uppsala FSA/pFSA sampling with KVM — [UART](https://www.it.uu.se/research/group/uart/modeling/fast_simulation/parallel_full_speed_ahead.html).
- SimPoint flow in gem5: profile basic-block vectors on Atomic CPU (`--simpoint-profile --simpoint-interval`), run SimPoint 3.2 analysis separately, take checkpoints at simpoints, restore with `--restore-simpoint-checkpoint -r N` — [gem5 SimPoints commit](https://git.sanchayanmaity.net/sanchayanmaity/gem5/commit/fcfc8b8c4ff98d69aa15fba60e723a9a1c4cda42); stdlib added a SimPoints API in v23.0 — [gem5 RELEASE-NOTES](https://gem5.googlesource.com/public/gem5/+show/refs/heads/develop/RELEASE-NOTES.md).
- LoopPoint extends sampling to multi-threaded workloads (loop-bounded regions); gem5 stdlib supports LoopPoint checkpoints saved at region starts on a no-cache board — [LoopPoint](https://looppoint.github.io); [create-looppoint-checkpoints.py](https://gem5.googlesource.com/public/gem5/+/master/configs/example/gem5_library/looppoints/create-looppoint-checkpoints.py); [gem5 LoopPoint API](https://gem5.org/documentation/general_docs/stdlib_api/gem5.resources.looppoint.html); journal version [LoopPoint TACO 2026](https://alenks.github.io/pdf/looppoint_taco2026.pdf).

### Inferences
- Fidelity switching in every mature simulator is a *quiescence protocol*: drain in-flight work, transfer architectural state, adopt the new level's mode, then resume. Our expand/collapse during a run needs the same: expanding mid-macro-step requires a configuration where no message is "in flight inside" the segment, or a defined mapping of in-flight state to the fine level.
- gem5 separates *architectural state* (transferable between CPU models) from *microarchitectural state* (caches, pipelines; must be warmed or is lost). Our analogue: the model's state variables are architectural; anything only the fine level has (e.g., TLS handshake state inside a collapsed "HTTPS request") must be reconstructed on expansion, either by re-running the fine level from the segment start (gem5 "restore then detailed") or by assuming it (warming error = an assumption record).
- Checkpoint-then-switch makes "expand from here" cheap if configurations are snapshot at macro-step boundaries — the same as our reverse-stepping storage.
- Sampling is a precedent for "detail follows speed": detailed simulation of chosen regions with explicit error accounting.

### Gaps
- Did not find current (2025–2026) gem5 docs on the `SimpleSwitchableProcessor` API beyond test configs.
- Did not research Simics reverse execution/checkpoint internals (Simics is known for reverse debugging; no source opened this session).

## 3. FMI 2.0/3.0 co-simulation, master algorithms, rollback, determinism, SSP

### Takeaway
FMI standardizes only the slave/unit API (do-step, get/set state, capability flags) and deliberately leaves the master ("importer") algorithm unstandardized; correctness problems in practice—step rejection, algebraic loops, simultaneous events, nondeterminism from evaluation order—live in that gap. The literature's answer is capability flags plus rollback (get/set state) and determinate master algorithms; FMI 3.0 added events, clocks, early return and intermediate updates; SSP 2.0 standardizes the system wiring separately from the units.

### Cited Findings
- FMI 3.0 Co-Simulation: the importer calls `fmi3DoStep` with a communication point and a step size "required to be larger than 0.0"; events may shrink the step "potentially even down to 0.0"; the FMU can set `earlyReturn` and report `lastSuccessfulTime`; the importer can request early return via `earlyReturnRequested` in `fmi3IntermediateUpdateCallback`; in CS the FMU "typically contains its own solver or scheduler" — [FMI 3.0 spec §1.2, §2.2.6](https://fmi-standard.org/docs/3.0/).
- FMI 3.0 Scheduled Execution: time is "under the sole control of the importer", passed via `fmi3ActivateModelPartition`; repeating failed calls "is not allowed" — [FMI 3.0 §2.2.4, §2.2.6](https://fmi-standard.org/docs/3.0/).
- Rollback: `fmi3GetFMUState`/`fmi3SetFMUState`/`fmi3FreeFMUState` copy and restore internal state, callable only if `canGetAndSetFMUState`; serialization needs `canSerializeFMUState`; the spec's example saves state at each accepted step and restarts if the next step is rejected — [FMI 3.0 §2.2.7.4](https://fmi-standard.org/docs/3.0/).
- FMI 3.0 adds Event Mode and Intermediate Update Mode to CS; intermediate update replaces `fmi2SetRealInputDerivatives`; clocks are `fmi3Clock` variables with interval variabilities (constant, fixed, tunable, changing, countdown, triggered); super-dense time orders events at the same instant with an integer index — [FMI 3.0 §1.1, §2.1, §2.2.8](https://fmi-standard.org/docs/3.0/); [FMI FAQ](https://fmi-standard.org/faq/); [Modelica 2021 FMI 3.0 paper](https://2021.international.conference.modelica.org/proceedings/papers/Modelica2021session1A_paper1.pdf).
- Early return in CS has limited tool support; intermediate update is more widely available — [LearnModelicaFMI: Events in FMUs](https://dr-clementcoic.github.io/LearnModelicaFMI/026-EventsInFMUs.html) (secondary source).
- The spec names the environment the "importer" and refers to "the co-simulation algorithm of the importer"; it does not standardize a master algorithm (in the parts read) — [FMI 3.0 §1.2, §2.2.6](https://fmi-standard.org/docs/3.0/).
- Gomes et al., "Co-simulation: a Survey", ACM CSUR 51(3), art. 49, 2018, DOI 10.1145/3179993 — [KTH PDF](https://people.kth.se/~dbro/papers/gomes-et-al-2018-co-simulation-survey.pdf); extended preprint "Co-simulation: State of the art" (157 pp.) — [arXiv 1702.00686](https://arxiv.org/abs/1702.00686).
- From the preprint: a *simulation unit* is "the composition of a simulator with a dynamical system"; the *orchestrator* "controls how the simulated time progresses in each simulation unit"; in DE co-simulation, simultaneous events are all processed before time progresses and a *select* function deterministically picks among ready units; conservative orchestration "avoids causality violations but is too pessimistic"; optimistic (Time Warp) orchestration requires "rollback capabilities of simulation units"; single rollback needs only the last committed state; determinism via select or via *confluence* (all interleavings "lead to the same behavior trace"), which is hard to prove for black boxes; CT units exchange values at communication step H with extrapolation between points (Jacobi style shown); hierarchical scenarios can themselves act as a single DE unit; the stated research needs are "generic approaches for modular, stable and accurate coupling of simulation units" and "expressing the adaptations required to ensure that the coupling is correct" — [arXiv 1702.00686 (ar5iv) §2.3–§4.2](https://ar5iv.labs.arxiv.org/html/1702.00686).
- Broman et al. (EMSOFT 2013) give conditions on FMUs and master algorithms for *determinate* co-simulation and a convergent, determinate, maximal-progress master; at most one "legacy" FMU without rollback or step-size prediction can be tolerated; Ptolemy's implementation is independent of FMU names/evaluation order — [Berkeley TR EECS-2013-153](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2013/EECS-2013-153.html); [Ptolemy pub page](https://ptolemy.berkeley.edu/projects/chess/pubs/1002.html). (Analyzed FMI 2.0 beta 4; older.)
- SSP 2.0 released 20 Dec 2024 (SSP 2.0.1 listed 2026-05-05): exchange of architecture specifications "and their incremental refinement", Modelica components, causal and acausal connections, FMI 3.0 vECUs, and layered-standard metadata as a basis for an upcoming simulation-traceability standard — [SSP news](https://ssp-standard.org/news/2024-12-20-ssp-2-0-release/); [SSP docs](https://ssp-standard.org/docs/main/); [Modelica news](https://modelica.org/news/2025-01-08-ssp-20-released/).
- FMI layered standards: FMI-LS-BUS (network communication over FMI 3.0) appeared as release candidate v1.0.0-rc.1 — [Modelica newsletter 2025-01](https://newsletter.modelica.org/2025-01/index); final status unverified.
- SystemC models are being wrapped automatically as FMUs for software-defined-vehicle co-simulation (2025) — [arXiv 2508.19665](https://arxiv.org/pdf/2508.19665).

### Inferences
- Adopt FMI's separation: a *unit contract* (what any "below-the-model" runtime adapter must expose: step to next event, get/set or replay state, capability flags) and a *system structure* (our model's wires, SSP-like) separately from the *orchestrator* (our interpreter's scheduler). Capability flags (`canGetAndSetFMUState`, `mightReturnEarly`) map to adapter capabilities like "can snapshot (CRIU/rr)", "can only re-run from start", "nondeterministic".
- Our interpreter is the master; determinism of traces under a binding needs Broman-style rules: deterministic select among enabled transitions (by stable semantic ID, not insertion order), and rollback or replay for any unit whose step may be rejected.
- Early return is the FMI name for "the real runtime hit something the model must see" (an outbound call crossing back into modeled territory). Our co-simulation boundary needs exactly this: run real code until it reaches a model port, return early with the time/position and the event.
- "Semantic adaptation" (Gomes) is our transactor/lifting layer and is named in the survey as an open research need, so we should expect to own it.

### Gaps
- Could not read the survey's sections on Gauss-Seidel, step-size control, algebraic loops and semantic adaptation (PDF not text-extractable here; ar5iv excerpt cut off at §4.2).
- No source found on 2025–2026 FMI 3.x point releases beyond layered standards.

## 4. SIL/HIL, digital twins for software, ns-3 with real applications

### Takeaway
ns-3 offers two ways to put real code in a modeled system: Direct Code Execution (load real binaries into simulated nodes, simulated time, deterministic) and emulation (real hosts/namespaces bridged via TapBridge/FdNetDevice in wall-clock time, non-deterministic and throughput-limited). The trade-off—simulated time and determinism versus unmodified real execution in real time—is exactly our hand-off choice.

### Cited Findings
- TapBridge lets a "real" host participate in an ns-3 simulation as if it were a simulated node; FdNetDevice/EmuFdNetDevice attach simulated nodes to real devices — [ns-3 Emulation Overview](https://www2.nsnam.org/docs/models/html/emulation-overview.html); [ns-3 users list](https://groups.google.com/g/ns-3-users/c/d4zQE0zd0F8).
- ns-3 real-time emulation must run in real-time mode, which limits throughput and topology complexity; measured throughput 14–120 Mbps on simple setups (IEEE Access, Jan 2025) — [DOAJ record](https://doaj.org/article/fb722410c1d045a781add4682986f0ea).
- Hypatia satellite emulation with TapBridge found substantial run-to-run differences under load, indicating an overloaded simulation process — [arXiv 2510.27027](https://arxiv.org/pdf/2510.27027).
- DCE runs real applications (iperf, ping, Quagga routing daemons, CCNx, bash, UMIP) inside ns-3; basic mode uses ns-3 TCP stacks, advanced mode uses the Linux kernel network stack; the official status page dates from 2013; a 2018 training deck lists maintainers; development is on GitHub — [DCE status](https://www.nsnam.org/wiki/Direct_Code_Execution_status); [DCE 1.3 manual](https://www.nsnam.org/docs/dce/release/1.3/manual/singlehtml); [ns-3 DCE training 2018](https://www.nsnam.org/tutorials/consortium18/slides/ns-3-training-dce-2018.pdf). DCE appears lightly maintained; treat as older.
- Virtual ECUs (QEMU + SystemC TLM-2.0) are the automotive SIL form of running real binaries against modeled hardware — [DVCon: Virtual ECUs](https://dvcon-proceedings.org/wp-content/uploads/Virtual-ECUs-with-QEMU-and-SystemC-TLM-2.0.pdf); SSP 2.0 adds FMI 3.0 vECU connections — [SSP news](https://ssp-standard.org/news/2024-12-20-ssp-2-0-release/).

### Inferences
- Two hand-off regimes for us: (a) "DCE-like": run the real function in-process with virtualized time/IO under our control (deterministic, replayable, requires interposition); (b) "emulation-like": let a real process run against real or stubbed dependencies in wall-clock time and only observe (simple, nondeterministic). Mark lifted results with the regime, since (b) evidence is not reproducible.
- The kernel/syscall surface is DCE's interposition boundary; our "kernel is the floor" design already makes syscalls the natural boundary for intercepting a real run.

### Gaps
- Did not find substantive primary sources on "digital twins for software systems" (as opposed to physical systems); the term is used loosely and I found nothing citable that defines a software-system twin with model/real coupling.
- Did not research HIL-specific timing guarantees.

## 5. Mocking, stubbing and service virtualization (model stands in for the real component)

### Takeaway
Service virtualization tools are the inverse co-simulation direction and already embody the boundary modes we need: capture (observe the real), simulate (model only), spy (model where it matches, real otherwise), modify/synthesize (model computes the response), diff (compare model and real). Contract testing adds the missing consistency check: the stand-in's expectations are verified against the real provider.

### Cited Findings
- Hoverfly has six mutually exclusive modes: capture (proxy and record request/response pairs, exportable as a JSON "simulation"), simulate (match inbound requests; "all the JSON fields must match"), spy ("like simulate, but any requests that can't be matched will be relayed to the remote target"), synthesize and modify (middleware creates or alters traffic), and diff — [Hoverfly modes](https://docs.hoverfly.io/en/latest/pages/keyconcepts/modes/modes.html).
- WireMock record/playback proxies traffic and writes stub mappings, served immediately after recording; snapshotting converts already-received requests to mappings; per-request conditional proxying is supported — [WireMock record/playback](https://wiremock.org/docs/record-playback/); [WireMock service virtualization](https://wiremock.org/docs/solutions/service-virtualization/).
- Recorded mappings are static and can drift from the real service (vendor-affiliated opinion) — [WireMock blog 2026](https://www.wiremock.io/post/service-virtualization-limitations-alternatives).
- (Training knowledge; not re-fetched this session) WireMock supports stateful stubs via named scenarios whose state transitions on matched requests — [WireMock stateful behaviour](https://wiremock.org/docs/stateful-behaviour/); Pact consumer-driven contracts record consumer expectations in a pact file and replay them against the real provider for verification — [Pact docs](https://docs.pact.io/).

### Inferences
- Our behavior packs are service virtualization at model level. Hoverfly's mode set gives a ready taxonomy for the co-simulation boundary per port: `model` (simulate), `real` (pass-through), `spy` (model if the binding is covered, else hand off), `capture` (run real, record as observed evidence), `diff` (run both, compare = our trace conformance).
- Drift is the known failure of recorded stand-ins; our provenance (`observed` with a commit/version anchor) plus a Pact-like re-verification step against the real runtime addresses it.
- WireMock scenarios show that useful stand-ins need state, i.e., the guarded-transition form our model already has.

### Gaps
- Did not survey commercial service-virtualization products (Broadcom/CA, Parasoft, Traffic Parrot) in depth.

## 6. Coupling a model to a real program run: DAP, GDB/MI, rr, deterministic replay, lifting via abstraction/refinement mappings

### Takeaway
The control channel exists off the shelf (DAP adapters over GDB/LLDB/language debuggers, with step granularity, data breakpoints, set-variable, restart-frame and reverse execution), and rr makes a real run deterministic and reverse-steppable. The theory for lifting is the refinement mapping (Abadi–Lamport): a function from concrete state (plus history/prophecy variables) to abstract state; PObserve shows the industrial version—parse real logs into abstract events and check them against a P model.

### Cited Findings
- DAP: `next`/`stepIn`/`stepOut`/`stepBack` take an optional `granularity` (default `statement`; the type also allows `line` and `instruction` — training knowledge); `stepBack`/`reverseContinue` need `supportsStepBack`; `restartFrame` needs `supportsRestartFrame`; `setVariable`/`setExpression` need matching capabilities; `evaluate` has contexts `watch`, `repl`, `hover`, `clipboard`, `variables`; data breakpoints (`dataBreakpointInfo`, `setDataBreakpoints`) with `breakpointModes`; `goto`/`gotoTargets`; instruction breakpoints; stopped-event reasons include `step`, `breakpoint`, `exception`, `pause`, `entry`, `goto`, `function breakpoint`, `data breakpoint`, `instruction breakpoint`; `supportsSingleThreadExecutionRequests` lets stepping affect one thread — [DAP specification](https://microsoft.github.io/debug-adapter-protocol/specification).
- rr records a Linux program's execution and replays it in gdb with efficient reverse execution, breakpoints and data watchpoints — [rr-project.org](https://rr-project.org/); chaos mode randomizes scheduling to expose intermittent bugs — [rr man page](https://linuxcommandlibrary.com/man/rr); use `watch -l` before reverse-continue — [rr wiki Usage](https://github.com/rr-debugger/rr/wiki/Usage).
- rr targets "real-world low-parallelism workloads with low overhead", is entirely user-space on stock hardware, compilers, runtimes and OS, needs no VM recording, kernel changes, pervasive instrumentation or compiler changes, but requires CPU/OS "non-obvious constraints" (modern Intel CPUs and Linux) — [O'Callahan et al., arXiv 1705.05937](https://arxiv.org/abs/1705.05937). (Training knowledge, from the full report: rr serializes threads onto one core and uses hardware performance counters to replay asynchronous events — verify in the paper.)
- GDB's native `record` supports reverse execution limited by the recorded instruction log and is slow — [GDB: Process Record and Replay](https://doc.ecoscentric.com/gnutools/doc/gdb/Process-Record-and-Replay.html); [Johnny's SW Lab](https://johnnysswlab.com/?p=732) (secondary).
- (Training knowledge; not re-fetched) GDB/MI is GDB's line-based machine interface for front ends — [GDB/MI manual](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html).
- Abadi & Lamport (TCS 1991): if S1 implements S2 then, under reasonable assumptions, adding history and prophecy variables to S1 guarantees a refinement mapping (implementation state → specification state); prophecy variables "predict the future" — [MSR page](https://www.microsoft.com/en-us/research/?p=145187); [SRC RR-29](https://ftp.mirrorservice.org/sites/www.bitsavers.org/pdf/dec/tech_reports/SRC-RR-29.pdf). Prophecy variables proved hard to use in practice; Lamport & Merz give a simpler form — [Prophecy Made Simple, TOPLAS 2022](https://members.loria.fr/SMerz/papers/2022-toplas.html).
- PObserve checks P specifications against implementations using existing service logs: a log parser turns logs into time-stamped events checked against P global invariants, in design, testing and production — [PObserve docs](https://p-org.github.io/P/advanced/pobserve/pobserve); [P repo](https://github.com/p-org/P).

### Inferences
- The hand-off recipe falls out: (1) choose an entry point = the model port where the expanded small step bottoms out; (2) bind concrete values satisfying the step's predicates/input class (the binding is the "stimulus", as in TLM testbench reuse through transactors); (3) launch under DAP (or rr for replayability), set function/data breakpoints at every *outgoing* model port (calls that cross back into modeled entities) = FMI early return; (4) at each stop, apply the abstraction function (concrete stack/heap/return value → model state variables and predicate truth values); (5) either continue the real run or substitute a model response (Hoverfly spy).
- The abstraction function must be total on the observed state and recorded per port; where the model needs information the concrete state does not yet determine (which branch will be taken), that is a prophecy variable — in our terms an assumption to be confirmed later by the run.
- rr's constraints (Linux, x86/selected CPUs, low parallelism) mean deterministic hand-off is a capability flag, not universal; JS/Python runtimes need their own replay or snapshot story.

### Gaps
- Did not verify current rr platform support (e.g., AArch64 status) or Pernosco's current offering.
- Did not research language-level record/replay (e.g., for Node/V8 or Python) or CRIU snapshotting as alternatives.

## 7. Recommendations for our interpreter: abstraction levels, transactors/adapters, co-simulation boundary, consistency checks

### Takeaway
Borrow TLM's fixed-port/variable-protocol structure for levels, FMI's unit contract plus capability flags for the real-runtime boundary, gem5's drain-and-transfer protocol for switching fidelity mid-run, Hoverfly's mode set for per-port boundary policy, and refinement mappings plus PObserve-style conformance for lifting and consistency. The open problem everyone shares—proving coarse and fine levels agree—is handled in practice by running both and comparing projections.

### Cited Findings
- Cross-level agreement in hardware is checked by transactor-driven co-simulation and assertion coverage/ordering, with formal TLM–RTL equivalence still open — [MEMOCODE 2007](https://eprints.soton.ac.uk/263822/1/jpms-memocode07.pdf); [UF VLSI 2013](https://www.cise.ufl.edu/research/cad/Publications/vlsi13assertion.pdf).
- A protocol checker per interface is standard practice (TLM-2.0 Base Protocol Checker) — [Doulos](https://www.doulos.com/knowhow/systemc/tlm-20/).
- Determinate co-simulation requires constraints on both units and master — [Broman et al. 2013](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2013/EECS-2013-153.html).
- Rollback via saved state or `fork()` makes optimistic/decoupled execution safe — [FMI 3.0 §2.2.7.4](https://fmi-standard.org/docs/3.0/); [DATE 2019](https://past.date-conference.com/proceedings-archive/2019/html/0086.html).
- Fidelity switching = drain, switch out, change mode, resume — [gem5 Simulation.py](https://git.sanchayanmaity.net/sanchayanmaity/gem5/src/commit/82f600e02ded4d7162dd9286f790462b9ab5d554/configs/common/Simulation.py).
- Trace conformance against logs is in production use (PObserve) — [PObserve](https://p-org.github.io/P/advanced/pobserve/pobserve).

### Inferences
**Abstraction levels**
- Define a level per *port protocol*, not per entity: each port declares an ordered set of protocol views (e.g., `call/return` → `request phases` → `TLS records` → `TCP segments`), like LT → AT. Port identity and wiring stay fixed across views (TLM sockets do not change when coding style does). A collapsed segment is the coarsest view; expanding selects a finer view on the ports involved.
- Each view must define its *timing points* (LT has 2, AT base protocol has 4). A macro step's observable events are exactly the coarse view's timing points; small steps must produce those same points (projection) plus interior ones.

**Transactors/adapters between levels**
- Make transactors first-class records: a transactor maps one coarse message to a sequence of fine messages and back (split/merge = our "messages change form at boundaries"). Use generic-payload-plus-extensions for message shapes so coarse entities can carry fine-layer data as ignorable extensions.
- The same record type should serve two uses, as in hardware: (a) refining model levels, (b) adapting the model to the real runtime (concrete value ↔ input class/predicate). (b) is Gomes's "semantic adaptation".

**Co-simulation boundary**
- Define a *runtime unit* contract, FMI-style: `instantiate(entry, binding)`, `step_until(port_event | limit)` with early return at any modeled outgoing port, `get_state/set_state` or `replay_to(point)`, `lift() → abstract state`, plus capability flags: `deterministic`, `can_snapshot`, `can_reverse` (rr), `realtime_only` (emulation-like). The interpreter is the master; units never advance on their own.
- Per-port boundary policy from Hoverfly: `model | real | spy | capture | diff`. Default `spy`: model where the binding is covered by a pack/graph path, hand off otherwise.
- Hand-off happens only at quiescent configurations (gem5 drain): no message in flight inside the segment being handed off. If needed, re-run from the segment's start rather than transferring partial interior state.
- Lifted results carry provenance `observed` with regime (deterministic replay vs wall-clock), runtime version and commit anchor; non-replayable evidence is flagged.

**Consistency checks**
- Macro/micro agreement: when a segment is first expanded (or in CI over gold models), run fine and coarse, project the fine trace onto the coarse view's timing points and state variables, and diff. A mismatch is a model bug at the segment, not a rendering issue.
- Per-port protocol checker (TLM base-protocol checker analogue): every view's event sequence at a port must satisfy that port's protocol automaton.
- Model vs real: `diff` mode at a port and PObserve-style lockstep conformance over lifted events; a real event with no enabled model transition = stuck configuration = model gap.
- Determinism: deterministic select over enabled transitions by stable semantic ID; any nondeterminism (scheduling, real runtime) must be an explicit choice point recorded in the trace so replay reproduces it (rr chaos mode is the precedent for exploring those choices on the real side).
- Decoupling accuracy: allow lanes to run ahead under collapse only between sync points; shared-state touches (`non-atomic`) are forced sync points, like a quantum boundary; optionally verify by speculative re-run with smaller quantum (DATE 2019 fork approach).

### Gaps
- No precedent found that lifts concrete program state into a *symbolic* model (predicates, input classes) automatically at a debugger boundary; the closest are refinement mappings (manual) and PObserve log parsers (hand-written). This is likely novel work for us.
- No source quantifies how often coarse and fine levels disagree in practice (TLM vs RTL or otherwise).
