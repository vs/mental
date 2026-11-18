# Levels of abstraction in hardware design, multi-resolution simulation and hierarchical planning

Scope: how three fields define levels, link adjacent levels and check one level against another. Hardware: gate-versus-RTL equivalence checking, sequential and C/C++-to-RTL transaction equivalence, instruction-set (ISA) specifications and checking processors against them, TLM-versus-RTL consistency since 2018, and the hardware stack as a whole. Simulation: multi-resolution modeling, aggregation and disaggregation, HLA practice. Planning: hierarchical task networks (HTN), the downward refinement property, angelic semantics, HTN plan verification, and 2023–2026 work where an LLM proposes abstract actions that a symbolic checker tests against refinements. Last, network protocol layering where one layer is checked against the service it should provide. This note extends, and does not repeat, `../../model-interpreter-semantics/notes/cosimulation_mixed_abstraction.md` (SystemC TLM, gem5, FMI) and `devs_formalisms.md` (DEVS, ml-DEVS, MRT-DEVS). State as of 13 November 2026.

Source note: every finding links to the page it came from. Three markers flag weaker evidence. "(search snippet)" means the claim comes from a search-result summary of the linked page and I did not open the page itself, usually because the server refused the fetch (RAND, DTIC, MDPI and Cadence returned HTTP 403). "(vendor claim)" marks statements by a tool vendor about its own product. "(training knowledge; not re-fetched)" marks facts I did not open a source for in this session. Unmarked findings were read in the primary text.

## At a glance

| System | What it is | How it handles levels | Take for Mental |
| --- | --- | --- | --- |
| Combinational equivalence checking (Conformal, Formality, EQY) | Proof that a gate netlist computes the same as the RTL | Registers and ports are matched by name, then by function; only the logic between matched points is compared | The generator emits its transformations as hints; the checker validates or rejects each hint |
| Sequential and C-to-RTL equivalence (SLEC, VC Formal DPV, Jasper C2RTL) | Proof that untimed C++/SystemC and clocked RTL give the same outputs per transaction | State counts may differ; only outputs at transaction boundaries are compared | Works per block with human case splits; the checkers themselves had soundness bugs, so test the checker |
| Arm ISA-Formal | Bounded model checking of Arm cores against the official ISA specification | A hand-written abstraction function extracts architectural state from the pipeline; one check per instruction class | The abstraction function is the costly, bug-prone part; debug it on simple hand-written cases first |
| Arm ASL, Sail | Machine-readable, executable ISA specification languages | One authored top level; emulator, prover definitions and checks are generated from it | Keep one executable top-level artifact and generate every check from it |
| riscv-formal (RVFI) | Open framework that checks any RISC-V core through a standard trace port | The core exposes one record per retired instruction; generic checks run on the records | Give every level a standard per-step observation record, so checks are written once |
| TLM vs RTL, 2018–2026 | Consistency of virtual prototypes with RTL | Still no equivalence proof; co-simulation, fuzzing and symbolic execution across levels | Per-run comparison is the state of the art when levels disagree on what a step is |
| Hardware stack as a whole | System model, RTL, gates, layout | A different check per adjacent pair; nothing end to end | Check each adjacent pair with the method that fits that pair |
| Multi-resolution modeling (Davis, Hillestad, Bigelow) | Families of simulation models at several resolutions | Consistency is a commuting diagram: aggregate-then-run against run-then-aggregate, compared approximately | The same diagram is Mental's projection check; decide per pair whether "approximately" is allowed |
| Aggregation–disaggregation, multiple-representation entities, HLA | Switching one simulated unit between coarse and fine form during a run | Either one level exists at a time, or both are kept and core attributes are synchronized | Keep both levels and re-run rather than translate state mid-run |
| Downward refinement property (Bacchus and Yang) | Condition under which every abstract plan can be refined without backtracking | Measured as a refinement probability per level pair; low-probability levels are merged | Measure how often an upper path has a lower realization; merge levels that fall below a threshold |
| Angelic semantics (Marthi, Russell, Wolfe) | Meaning for high-level actions as sets of reachable states | Upper and lower bounds on the reachable set classify an abstract plan as works, fails or unknown | Use the upper bound for actors Mental does not control; the lower bound only fits choices the system itself makes |
| HTN plan verification | Deciding whether an action sequence is a valid decomposition of a task hierarchy | NP-complete in general without the decomposition; polynomial when totally ordered | Store the step map as a witness; never reconstruct it from the trace |
| ChatHTN, Ada, LLM-Modulo | LLM proposes decompositions or abstract actions; a symbolic planner checks them | A proposal survives only if its claimed effects hold after lower-level execution | The closest precedent for LLM-proposed levels: accept per run, score per operator, discard below a threshold |
| Netsem TCP specification | Protocol-level and service-level specifications of TCP in HOL4 | A testable abstraction function maps protocol states to service states, checked on captured traces | Direct precedent for checking refinement by projecting traces, on the protocol web services run on |
| Verdi, sublayering | Verified distributed systems and protocol stacks | A proof under an ideal network is carried to a faulty network by a verified transformer | Treat the fault model as a level transition with its own reusable pattern |

## 1. Gate against RTL: combinational equivalence checking

### Takeaway

Gate-versus-RTL checking is routine because both sides keep the same state elements, so the tool pairs them up and compares only the logic between pairs. The pairing is found by name first and by function second. Where synthesis changed the state elements (retiming, state-machine re-encoding, merged registers), the synthesis tool hands the checker a file of hints describing what it did, and the checker proves each hint before using it. That hint-then-validate arrangement is the most transferable idea in this note.

### Cited Findings

- Logic equivalence checking (LEC) uses static analysis to compare an HDL description against a gate netlist "for all inputs and all time". It requires the two designs to have the same number of states at the boundary; interior encodings may differ. Siemens calls it the most popular original use of equivalence checking — [Siemens Verification Horizons blog, 2019](https://blogs.sw.siemens.com/verificationhorizons/2019/07/11/the-many-flavors-of-equivalence-checking-part-1-synthesis-validation-with-lec-and-slec-a-k-a-the-most-popular-formal-apps-ever/) (vendor material).
- Cadence Conformal treats primary inputs and outputs, flip-flops, latches and black boxes as "key points", pairs them between the two designs, then compares. Mapping is name-based first and function-based second; the name phase is quick and the functional phase is slow — [Cadence community forum](https://community.cadence.com/cadence_technology_forums/f/logic-design/22046/best-flow-to-map-most-key-points-before-compare) (search snippet; forum answers, not official documentation).
- Synopsys Formality reads an "automated setup file" (`.svf`) that Design Compiler or Fusion Compiler writes during synthesis. It holds "hints regarding both guidance and setup" and lets Formality "automatically detect, and independently verify" the optimizations made — [Synopsys white paper, Techniques for Achieving Higher Completion in Formality](https://www.synopsys.com/content/dam/synopsys/implementation&signoff/white-papers/higher-completion-formality-wp.pdf) (vendor material).
- The file records object name changes, constant registers, duplicated and merged registers, state-machine re-encoding and datapath optimizations. Each item is "implicitly or explicitly proven before or during verification, or Formality will not use it". The file is "an absolute necessity" for register phase inversion, FSM re-encoding and retiming — [same white paper](https://www.synopsys.com/content/dam/synopsys/implementation&signoff/white-papers/higher-completion-formality-wp.pdf).
- The flow per hint is: read the transformation, check that it is valid, apply it to the tool's view of the RTL, or reject it. Rejected hints are reported per compare point (`report_svf_operation -status rejected`), and the paper's worked example has 22 rejected datapath hints, of which only those touching unverified compare points need attention — [same white paper](https://www.synopsys.com/content/dam/synopsys/implementation&signoff/white-papers/higher-completion-formality-wp.pdf).
- A "hard verification" is one where Formality cannot finish all compare points; it shows no progress for hours or aborts. Optimized datapath is "the most difficult equivalency checking completion issue" because the blocks are both large and structurally different from the RTL — [same white paper](https://www.synopsys.com/content/dam/synopsys/implementation&signoff/white-papers/higher-completion-formality-wp.pdf).
- The open-source EQY tool (YosysHQ) takes a "gold" and a "gate" design, groups matched signals by name pattern into partitions, and proves each partition separately with a chosen strategy; its example reports "1/44 partitions" failing and names the failing one — [EQY quickstart](https://yosyshq.readthedocs.io/projects/eqy/en/latest/quickstart.html).
- A partition is checked in isolation with its boundary signals unconstrained, so a partition can fail although the designs are equivalent in context — [EQY issue 47](https://github.com/YosysHQ/eqy/issues/47) (search snippet).

### Inferences

- The draft report says this check is routine "because registers on both sides can be matched". That is confirmed, with one addition the draft lacks: when registers cannot be matched directly, the fix in practice is not a smarter checker but a cooperating generator. The synthesis tool knows what it changed and says so; the checker trusts nothing and proves each statement.
- Mental's LLM is in the position of the synthesis tool. It should emit the level map (which lower entities form an upper entity, which lower steps form an upper step, which names changed) as a list of small typed hints. The checker validates each hint on its own and reports accepted and rejected hints per upper element. A rejected hint is a finding with a location, not a failed build.
- Partitioning by matched points is why the check scales and why failures are local: "1 of 44 partitions failed, here is its name". Mental's refinement check should report per upper step, not per level.
- The partition false alarm (equivalent in context, not in isolation) will appear in Mental too: an upper step checked alone may need an invariant from its surroundings. That is the place for per-level invariants as assumptions.

### Gaps

- I could not open official Conformal or Formality manuals (login required); the Conformal mapping description rests on forum answers.
- No public data on how often LEC needs manual intervention on current designs.

## 2. Sequential equivalence and C/C++-to-RTL transaction equivalence

### Takeaway

When the two sides have different numbers of states (untimed C++ against clocked, pipelined RTL), the comparison moves to outputs at transaction boundaries. Three commercial tools do this and are used for sign-off, but only at block level, and convergence on hard blocks still needs human-supplied case splits and intermediate lemmas. A 2024 study found soundness bugs in all three vendors' tools, which matters for any design that makes a checker the source of truth.

### Cited Findings

- Sequential logic equivalence checking (SLEC) "allows for the number of states to differ between the two" designs as long as the outputs match. The main driver is high-level synthesis, which must insert registers because C++ has no clocks — [Siemens Verification Horizons blog](https://blogs.sw.siemens.com/verificationhorizons/2019/07/11/the-many-flavors-of-equivalence-checking-part-1-synthesis-validation-with-lec-and-slec-a-k-a-the-most-popular-formal-apps-ever/) (vendor material).
- Siemens SLEC covers hand-written RTL, Catapult-generated RTL and power-optimized RTL, with "bug hunting, bounded-check, and full-proof strategies", and is positioned as a complement to simulation — [Siemens SLEC product page](https://www.siemens.com/en-us/products/ic/catapult-high-level-synthesis/hls-verification/slec/) (vendor claim).
- Synopsys VC Formal DPV proves "transactional equivalence between a golden specification in C/C++ and the RTL designs", and also compares C to C and RTL to RTL — [Synopsys blog](https://www.synopsys.com/blogs/chip-design/datapath-validation-artificial-intelligence-mathematical-cores.html) (vendor claim).
- NVIDIA's reported flow: check assertions embedded in the C++ specification, then use C-to-RTL equivalence to sign off HLS-generated RTL, then RTL-to-RTL equivalence for incremental changes. Intel's reported flow on compression blocks used case splitting and assume-guarantee, proving a lemma per block and assuming it in the next; the hardest blocks took 20–30 hours to converge — [same Synopsys blog](https://www.synopsys.com/blogs/chip-design/datapath-validation-artificial-intelligence-mathematical-cores.html) (vendor material reporting customer talks).
- Cadence Jasper C2RTL checks pipelined RTL datapaths against C/C++ models, handles stalls and wait conditions, and starts from the real reset state — [Cadence blog](https://community.cadence.com/cadence_blogs_8/b/fv/posts/jasper-c2rtl-app-for-datapath-verification) (search snippet; vendor claim; page returned 403).
- C-to-RTL equivalence checkers give "an unbounded proof: valid for any inputs, and for any number of cycles" and are "trusted to authorise design sign-off" — [Pardalos et al., "Who checks the checkers?", DVCon 2024](https://www.doc.ic.ac.uk/~afd/papers/2024/DVCon.pdf).
- The same paper fuzzed the three major commercial checkers with random SystemC programs known to equal trivial RTL. It found 16 distinct bugs, 12 confirmed by vendors, among them 9 unsoundness bugs where the checker claimed equivalence wrongly. No bug involved a single language feature; all came from feature interactions — [Pardalos et al., DVCon 2024](https://www.doc.ic.ac.uk/~afd/papers/2024/DVCon.pdf). (The abstract gives 7 unsoundness and 5 incompleteness bugs as confirmed; the introduction gives 9 and 7 as found.)
- The paper notes that equivalence checking is undecidable in general, so timeouts are an expected result — [Pardalos et al., DVCon 2024](https://www.doc.ic.ac.uk/~afd/papers/2024/DVCon.pdf).

### Inferences

- The draft report says DPV "needs no correspondence between internal state elements". I could not confirm that sentence in the Synopsys pages I opened. What the sources do show is that the problem statement needs only input and output correspondence at the transaction boundary, while proofs on hard blocks converge only after engineers add internal structure by hand (case splits, per-block lemmas). The draft's sentence should be softened to "is specified only at transaction boundaries".
- The tractability recipe is three things together: a small block, an agreed transaction boundary, and an RTL that was generated from the C++ by a tool or written against it. None holds for a whole design.
- For Mental, a "transaction" is one upper step: a request handled, a job run. Comparing only at step boundaries is the right default, and it is the cheap check. Internal correspondences are optional hints that make a failing check explainable.
- The Equifuzz result is a warning for a product whose pitch is "the checker decides what is fact". Mental's checkers can be tested the same way: generate model pairs that are equal by construction (a level and a trivially rewritten copy) and pairs that differ by one seeded fault, and confirm the checker says so.

### Gaps

- No vendor-neutral data on what share of blocks converge without manual help.
- I did not find public documentation of how DPV, SLEC or C2RTL users declare the transaction boundary and latency.

## 3. Instruction-set specifications and checking processors against them

### Takeaway

The instruction-set architecture is the one hardware level with a single, authored, executable specification, and both Arm and RISC-V now generate their checks from it. The link down to a processor is an abstraction function that extracts architectural state (registers, flags, program counter) from pipeline state. It is hand-written per processor, and Arm reports it as the main cost. The checks are bounded and leave out large parts of the processor.

### Cited Findings

- ISA-Formal checks datapath, pipeline control and forwarding and stall logic against Arm's Architecture Reference Manuals, which are mechanically translated to Verilog so that commercial model checkers can be used. It was applied to 8 Arm processors and found bugs in all of them — [Reid et al., CAV 2016, author page](https://alastairreid.github.io/papers/CAV_16).
- The method: let the processor run for some cycles, then execute one instruction and apply "an abstraction function abs which extracts the architectural state of the processor immediately before" and immediately after it. The pipeline is not flushed. Instead of letting the tool choose any instruction, the method enumerates instruction classes and performs a separate check per class — [Reid et al., CAV 2016, PDF](https://static.aminer.org/pdf/20170130/pdfs/cav/wtylpbkhv9vdxulg74wf2uzzmi3iekyf.pdf).
- "The only part that needs to be customized for each processor is the Verilog abstraction function that extracts the effective architectural state from the micro-architectural state." Most processors needed around 2,500 lines of such support code — [same PDF](https://static.aminer.org/pdf/20170130/pdfs/cav/wtylpbkhv9vdxulg74wf2uzzmi3iekyf.pdf).
- "Creating the abstraction code remains a difficult task and involves a lot of work with the CPU designers." The team debugs the abstraction function first against hand-written properties for a few simple instructions, because that "is significantly easier" than debugging against the machine-generated specification, and only then switches to the generated one — [same PDF, section 5.6](https://static.aminer.org/pdf/20170130/pdfs/cav/wtylpbkhv9vdxulg74wf2uzzmi3iekyf.pdf).
- The check is bounded model checking: it shows that "a sequence of n instructions does not go wrong". Unbounded proof would need processor invariants. Completing the job would also need proofs that instructions are not lost, duplicated or reordered, and verification of exceptions, instruction fetch and the memory management unit — [same PDF](https://static.aminer.org/pdf/20170130/pdfs/cav/wtylpbkhv9vdxulg74wf2uzzmi3iekyf.pdf).
- Stated exclusions: floating-point arithmetic itself, instruction fetch, most exception behavior, in-order retirement ("if an instruction does retire, then it has the correct effect") and the memory system, which is "very large and stateful" — [Reid, Limitations of ISA-Formal](https://alastairreid.github.io/isa-formal-limitations).
- Arm's Architecture Specification Language (ASL) is an executable, strongly typed imperative language that Arm obtained by reverse-engineering the pseudocode of its manuals and then repairing it; Arm now writes and tests the specification before implementation — [Reid, notes on ASL](https://alastairreid.github.io/RelatedWork/notes/asl).
- The executable specification was validated against Arm's internal test suites, simulators and physical processors, and "bugs were found in all of these artifacts"; testing a specification is "a two-way street" — [Reid et al., FMCAD 2016, author page](https://alastairreid.github.io/papers/FMCAD_16/).
- Arm is defining a new language version, ASL1; the language reference manual is still marked beta, and a first Readers' Guide is dated 29 October 2025 — [Arm AArch32 release notes, March 2025](https://documentation-service.arm.com/static/67e409754191d739c8e9b9ff); [Arm ASL Readers' Guide](https://documentation-service.arm.com/static/6901ca9402789941c1920b3a) (search snippets). I found no statement that a final ASL 1.0 has shipped.
- Sail is "a language for defining the instruction-set architecture (ISA) semantics of processors". The Armv9.4-A, Armv9.3-A and Armv8.5-A Sail models are "automatically translated from the Arm-internal ASL reference". From one model Sail generates C or OCaml emulators, theorem-prover definitions, documentation snippets, specification coverage and test sequences; a SystemVerilog reference model is marked "in progress" — [Sail repository](https://github.com/rems-project/sail).
- The RISC-V Sail model "has been adopted by RISC-V International". Its generated emulator boots Linux, FreeBSD and seL4. Generating a SystemVerilog reference model "for formal hardware verification" is listed as in progress — [sail-riscv repository](https://github.com/riscv/sail-riscv).
- riscv-formal checks any RISC-V core through the RISC-V Formal Interface (RVFI), an output-only port that reports each retired instruction. The core author supplies a wrapper connecting the core to RVFI and a configuration file with the ISA and check depths — [riscv-formal procedure](https://yosyshq.readthedocs.io/projects/riscv-formal/en/latest/procedure.html); [RVFI](https://yosyshq.readthedocs.io/projects/riscv-formal/en/latest/rvfi.html).
- The generated checks are of two kinds. Instruction checks, one per instruction and channel, confirm that the reported state change matches the instruction. Consistency checks cover what single-instruction checks miss: program-counter chaining forward and backward, register values equal to the last write, causality under out-of-order retirement, uniqueness of retirement order, and liveness. The generated checks "are bounded model checks"; liveness may need added fairness constraints — [riscv-formal procedure](https://yosyshq.readthedocs.io/projects/riscv-formal/en/latest/procedure.html).

### Inferences

- The draft report's sentence on ISA-Formal is right but leaves out three facts that change the lesson: the check is bounded, it covers only part of the core, and the abstraction function is written by hand at about 2,500 lines per processor. The lesson is not "per-instruction checking is cheap" but "the property side can be generated; the mapping side is where the work and the bugs are".
- That maps onto Mental exactly. Upper-level steps (the "instructions") can be proposed by the LLM and checked generically. The abstraction function from lower state to upper state is the artifact that needs the most care, and it should be debugged the way Arm does: first against two or three hand-checked, simple upper steps, before any generated check is trusted.
- riscv-formal splits checking into "each step alone is right" and "the sequence of steps is coherent" (nothing lost, duplicated, reordered; each read sees the last write). ISA-Formal names the same residue as unfinished. Mental's per-run projection check needs both halves: each lower segment realizes its upper step, and the projected sequence has no lost, duplicated or reordered upper steps.
- RVFI is a small standard record that any implementation, in-order or out-of-order, must emit, and all checks are written against it. The Mental equivalent is a fixed observation record per upper step (step identity, order number, upper state read, upper state written), emitted by any lower-level run. Checks are then written once per level, not once per project.
- ASL and Sail show the value of one executable top-level artifact from which emulator, documentation and checks are generated. They also show that the top level is found by testing it against the implementations, with errors on both sides.

### Gaps

- No public figures on ISA-Formal's current coverage at Arm after 2016.
- I did not find a released Sail-generated SystemVerilog model in use for sign-off as of November 2026.

## 4. TLM against RTL since 2018, and the hardware stack as a whole

### Takeaway

The prior note recorded that general equivalence between transaction-level models and RTL was called open in 2007. I found nothing from 2018 to 2026 that closes it. Progress has been in per-run comparison: co-simulating a virtual prototype with the RTL, fuzzing both, and running symbolic execution across the two levels. Across the whole stack, each adjacent pair has its own kind of check, and the strength of the check falls as the two levels' notions of a step diverge.

### Cited Findings

- Herdt, Le, Große and Drechsler (DATE 2018) refine TLM properties to RTL properties automatically by reusing the transactor, relating TLM events to RTL signal combinations at clock cycles; they note manual translation is error-prone — [paper via CORE](https://core.ac.uk/download/161379479.pdf) (search snippet).
- Cross-level processor verification (FDL 2020) couples an instruction-set simulator as reference model to an RTL core in tight co-simulation and evolves the instruction stream during the run — [Herdt et al., FDL 2020](https://vxproxy02.ics.jku.at/files/2020FDL_cross-level-processor-verification-riscv.pdf) (search snippet).
- A 2024 method verifies a RISC-V interrupt controller by fuzzing TLM transactions on the virtual prototype and driving the same stimuli into the RTL through a TLM-to-RTL transactor — [Ahmadi-Pour, Hassan, Drechsler, RISC-V Summit Europe 2024](https://agra.informatik.uni-bremen.de/doc/work/RISCVSummit2024_SAP.pdf) (search snippet).
- Rudkowski, Ahmadi-Pour and Drechsler (2025) compare two ways to run symbolic execution on SystemC peripherals across levels: CrosSym changes the SystemC kernel, SEFOS changes the symbolic execution engine. Both were tested on peripherals at different abstraction levels and on "identifying 300+ mutants"; the prior state of the art was "limited to Transaction Level Modelling" — [arXiv:2509.05504](https://arxiv.org/abs/2509.05504).
- At the bottom of the stack, layout versus schematic (LVS) extracts devices and connections from the layout into a netlist and compares it with the source netlist, reporting shorts, opens and mismatches — [Design & Reuse, LVS debug](https://www.design-reuse.com/article/61170-layout-versus-schematic-lvs-debug/) (search snippet).

The stack, with the check used between each adjacent pair (compiled from sections 1–4 and the prior note):

| Upper level | Lower level | What links them | Check | Strength |
| --- | --- | --- | --- | --- |
| ISA specification | RTL processor | hand-written abstraction function, per-instruction properties | bounded model checking | bounded, partial coverage |
| Virtual prototype (TLM) | RTL | transactor | co-simulation, fuzzing, cross-level symbolic execution | per run |
| Untimed C++/SystemC block | RTL block | transaction boundary | transaction equivalence | proof, block level, human-guided |
| RTL | RTL after retiming or gating | outputs over time | sequential equivalence | proof, may not converge |
| RTL | gate netlist | matched registers plus generator hints | combinational equivalence | proof, routine |
| Gate netlist | layout | extracted netlist | LVS graph comparison | exact, routine |

### Inferences

- The draft report's phrase "open for TLM vs RTL" needs a qualifier. Equivalence of *synthesizable, untimed C++/SystemC blocks* with RTL is commercial practice (section 2). What is open is equivalence of *loosely timed virtual platforms* with RTL, where the two sides share neither state nor timing.
- The table supports the draft's main claim and sharpens it. Checks are proofs where one level was mechanically derived from the other and the derivation left a trace (hints, matched names, a transaction boundary). They are per-run comparisons where the two levels were written independently. Mental's LLM-proposed upper levels are written independently of the code, so per-run comparison is the realistic baseline, and proof-like strength is available only where the upper level is derived deterministically from the lower one.
- No hardware flow has one end-to-end check. The draft already says this; the table is the evidence.

### Gaps

- The 2018–2025 items except arXiv:2509.05504 rest on search snippets.
- I did not find a 2020s survey of formal TLM-versus-RTL equivalence; absence of evidence, not proof that none exists.

## 5. Multi-resolution modeling, aggregation–disaggregation and HLA

### Takeaway

Military and policy simulation defines consistency between a detailed and an aggregate model as a commuting diagram, in a weak form (compare aggregated outputs) and a strong form (compare states after aggregating, running the coarse model and disaggregating). Agreement is expected only approximately and is measured, not proved. The field's hard-won practice is to avoid converting state between levels mid-run, and its senior author warns that the detailed model is not automatically the true one.

### Cited Findings

- Multiresolution modeling is "building a single model, a family of models, or both to describe the same phenomena at different levels of resolution" — [Davis and Bigelow, Experiments in Multiresolution Modeling, RAND MR-1004, 1998](https://apps.dtic.mil/sti/pdfs/ADA355041.pdf) (search snippet; DTIC and RAND refused the fetch).
- Davis and Hillestad (1993) define consistency as "the comparison between the projected state of an aggregate of high-resolution entities which evolved in M, and the projected state of the same aggregate initially controlled by M'". Weak consistency: aggregate the high-resolution outputs and compare with the low-resolution model's outputs; the models are consistent if these are "approximately equal". Strong consistency: aggregate the detailed state, evolve it in the coarse model, disaggregate, and compare with the detailed evolution — as quoted and drawn in [Soyez et al., 2013, arXiv:1311.5108](https://arxiv.org/abs/1311.5108) (secondary source quoting the 1993 paper).
- In practice the same authors measure consistency by running the simulation several times at full detail and several times with dynamic level switching, recording mean values of chosen elements, and computing a dissimilarity — [Soyez et al., arXiv:1311.5108](https://arxiv.org/abs/1311.5108).
- Davis and Bigelow argue that the impression that truth resides in detailed models "is simply wrong", and that families of models should be mutually calibrated top-down, bottom-up and sideways; useful low-resolution models may not be straightforward aggregations of detailed ones — [RAND MR-1004](https://apps.dtic.mil/sti/pdfs/ADA355041.pdf); [Bigelow and Davis, Implications for Model Validation of MRMPM, RAND MR-1750](https://www.rand.org/content/dam/rand/pubs/monograph_reports/2005/MR1750.pdf) (search snippets).
- Reynolds, Natrajan and Srinivasan (1997) criticize classic aggregation–disaggregation because only one resolution of an entity exists at a time. They propose multiple-resolution entities that keep all levels and maintain a set of core attributes consistent — [Consistency maintenance in multiresolution simulation, ACM TOMACS 1997](https://www.bohrium.com/paper-details/consistency-maintenance-in-multiresolution-simulation/811142146655518721-2793) (search snippet).
- Known failure modes of aggregation–disaggregation are chain disaggregation (one unit disaggregating forces its neighbors to), transition latency and difficulty mapping between levels; a follow-up treats consistency maintenance as numerical optimization between two models that make overlapping predictions — [Drewry, Reynolds, Emanuel, 2002](https://libraopen.lib.virginia.edu/public_view/g158bh29z) (search snippet).
- A 2020 systematic review found only 22 papers on multi-resolution modeling for live, virtual and constructive simulation — [Lee, Lee, Rabelo, Information 11(10):480](https://www.mdpi.com/2078-2489/11/10/480) (search snippet).
- Davis's 2023 paper argues for starting analysis with a family of models at varied resolution, giving a broad view, selective depth and the ability to switch perspective — [Broad and Selectively Deep, Information 14(2):134, RAND record](https://www.rand.org/pubs/external_publications/EP69168.html) (search snippet).
- The High Level Architecture was reissued as IEEE 1516-2025 ("HLA 4") in 2025 — [IEEE 1516-2025 record](https://standards.ieee.org/standard/1516-2025.html) (search snippet). HLA provides generic attribute ownership transfer between federates (divestiture and acquisition) — [PyHLA manual, ownership management](https://nongnu.org/certi/PyHLA/manual/node16.html) (search snippet). I found no source showing that HLA itself defines aggregation–disaggregation consistency; it appears to be left to each federation's object model and agreements.

### Inferences

- The draft report says multi-resolution modeling "treats consistency between an aggregate model and a detailed one as an empirical comparison of outputs". That is half right. The *definition* (Davis and Hillestad) is a commuting diagram over an aggregation function, the same shape as a refinement mapping. Only the *evaluation* is empirical and approximate. The draft should say so, because it means this field agrees with the formal-methods fields on what the check is and differs only on tolerance.
- Weak and strong consistency give Mental two useful grades. Weak: project the lower run's outputs upward and compare with the upper run's outputs. Strong: also require that upper state, pushed down, is a state the lower level could be in. Mental needs only the weak form for display and the projection check; the strong form is what mid-run zoom would need, and the prior note already recommends re-running instead.
- Multiple-resolution entities are what Mental already does by keeping every level in the model at once. The "core attributes" idea is worth naming: for each level pair, list the few upper variables that must agree with the abstraction of lower state, and check exactly those.
- "Truth is not in the detailed model" does not hold for Mental's code-backed levels, where code is the truth. It does hold at the top: for a courier or an approving operator there is no lower model, and the business-level description is the only source. It also holds wherever the lower model is itself an LLM summary of a library.
- Approximate equality is a real choice Mental has to make for durations, counts and probabilities. For discrete state the comparison should be exact.

### Gaps

- All RAND primary texts were blocked; the Davis findings rest on search snippets and one secondary paper. They should be re-checked against the PDFs before the report quotes them.
- No source found on how current HLA federations test aggregate-versus-entity consistency in practice.

## 6. Hierarchical planning: downward refinement, angelic semantics, plan verification

### Takeaway

Planning research offers three results. Bacchus and Yang treat refinability as a measurable probability per level pair and merge levels where it is low. Marthi, Russell and Wolfe make refinability hold by construction, by allowing only true descriptions of high-level actions, given as an upper and a lower bound on the reachable set. Behnke and colleagues show that checking whether a concrete action sequence is a valid decomposition of a hierarchy is NP-complete unless the decomposition is supplied.

### Cited Findings

- The downward refinement property (DRP) holds when "every abstract solution can be refined to a concrete-level solution without backtracking across abstraction levels". It "is a strong requirement that is not often met in practice" — [Bacchus and Yang, Downward Refinement and the Efficiency of Hierarchical Problem Solving, Waterloo TR 1992](https://cs.uwaterloo.ca/research/tr/1992/45/drp.pdf) (technical-report version of the Artificial Intelligence 1994 article).
- The paper gives a model of search cost "parameterized by the probability of an abstract solution being refinable", a syntactic test for the DRP in ABSTRIPS-style hierarchies, and the HighPoint algorithm. HighPoint takes the hierarchy proposed by Knoblock's Alpine, estimates a refinement probability for each pair of levels in a testing phase, and collapses levels whose probability is low; the reported runs use a threshold of 0.5 — [same report](https://cs.uwaterloo.ca/research/tr/1992/45/drp.pdf).
- Angelic semantics defines a high-level action by "the set of states reachable by some refinement", with the choice of refinement "resolved in the planning agent's own best interest". The name contrasts with adversarial nondeterminism, where a plan must work "no matter how the choices are made" — [Marthi, Russell, Wolfe, ICAPS 2007](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf).
- Exact reachable sets are rarely concise, so the paper uses "complete descriptions, which provide an upper bound (superset) on the reachable set, and sound descriptions, which provide a lower bound (subset)". Complete descriptions prove a plan cannot reach the goal under any refinement; sound descriptions prove it surely reaches the goal under some refinement; if neither applies, further refinement decides — [ICAPS 2007 paper](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf).
- With true descriptions "the downward refinement property and its converse are automatically satisfied"; "downward refinement can fail only if HLAs are allowed to have false descriptions" — [ICAPS 2007 paper](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf).
- Deciding whether a goal is in the exact reachable set of a high-level action in polynomial time would imply P = NP (Theorem 1), which is why bounds are used — [ICAPS 2007 paper](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf).
- Non-empty sound descriptions "are sometimes hard to come by"; the paper adds a third kind, sound-intersecting descriptions. Deriving descriptions automatically from the hierarchy is "conceptually simple" but deferred, because of the trade-off between accuracy and size — [ICAPS 2007 paper](https://aima.eecs.berkeley.edu/~russell/papers/icaps07-hla.pdf).
- The words "optimistic" and "pessimistic" come from the 2008 follow-up, which adds costs: an optimistic description bounds the true cost-and-reachability valuation from below, a pessimistic one from above, and they "generalize our previous complete and sound descriptions". In a pessimistic description "the agent can force either outcome". The descriptions "are given along with the hierarchy", that is, written by hand — [Marthi, Russell, Wolfe, Angelic Hierarchical Planning, ICAPS 2008](https://aima.eecs.berkeley.edu/~russell/papers/icaps08-hla.pdf).
- In classical planning, checking a plan is polynomial. For HTN planning, verifying that a plan is a solution "is NP-complete, even for very simple HTN planning problems", and stays NP-complete when an executable action sequence is given, because the checker must find a decomposition that produces it — [Behnke, Höller, Biundo, ICAPS 2015](https://gki.informatik.uni-freiburg.de/papers/behnke-etal-icaps15.pdf).
- Later work reports that verification is polynomial when the whole model is totally ordered, and that the lifted (non-grounded) case is harder than the grounded one — [Lin, Olz, Helmert, Bercher, AAAI 2024](https://ai.dmi.unibas.ch/papers/lin-et-al-aaai2024.pdf) (search snippet).
- HDDL is the common input language for hierarchical planning, an extension of PDDL adopted for the first hierarchical track of the International Planning Competition in 2020 — [Höller et al., arXiv:1911.05499](https://arxiv.org/pdf/1911.05499) (search snippet).

### Inferences

- Two corrections to the draft report. First, it attributes "optimistic" and "pessimistic" descriptions to the ICAPS 2007 paper. That paper calls them *complete* and *sound* descriptions; the optimistic and pessimistic names, with costs, are from ICAPS 2008. Second, and more important, the draft reads the pessimistic description as "what it is guaranteed to do". That is not what it means. The lower bound is the set of states the *agent can force by choosing a refinement*. It is a guarantee only when the chooser is on your side.
- This matters for the courier. A courier is not controlled by the system under study. For the question "can this scenario happen?" the courier's choices are angelic and the reachable-set reading fits. For the question "does the system cope with whatever the courier does?" the courier is an adversary, and the 2007 paper itself says that case needs the plan to work under all choices. So for uncontrolled human and physical actors, the bound Mental needs is the *upper* bound (everything the actor may do), and invariants must hold over all of it. The lower bound is useful only for steps where the modeled system chooses, for example a dispatcher service picking any free courier. The 2007 paper notes that complete (upper-bound) descriptions "may work unchanged" in nondeterministic environments, while sound ones need replacing.
- "Downward refinement fails only if descriptions are false" is a clean statement of Mental's risk. An LLM-written summary behavior is a description of an upper step that may be false. Mental has two honest options, matching the two papers: check each summary against the lower level so it is true by construction (angelic), or measure how often upper paths have lower realizations and treat low scores as a defect of the level (Bacchus and Yang).
- HighPoint is a concrete procedure for the second option and for the draft's "inserted level" question. Run sampled or bounded-exhaustive upper scenarios, try to realize each at the lower level, record the fraction that succeed per level pair, and propose merging a level into its neighbor when the fraction is below a threshold.
- The NP-completeness result says: do not try to recover the step map from a trace. The level record's `step_map` is the decomposition witness. With it, checking a run is linear in the trace; without it, the checker is solving a parsing problem that is hard in general. The polynomial totally-ordered case suggests that sequential upper steps are cheap to check and concurrent, interleaved ones are where the cost is.
- Bounds were hand-written in both angelic papers, and useful lower bounds were hard to write even by the authors. Expect an LLM to produce upper bounds more reliably than lower ones.

### Gaps

- I read the 1992 technical report, not the 1994 journal text; statements should match but page references differ.
- I did not find a tool that derives angelic descriptions automatically from a hierarchy, which both papers leave as future work.

## 7. LLM hierarchical planning with checked refinements, 2023–2026

### Takeaway

The work closest to Mental's design lets an LLM propose a decomposition or an abstract action and lets a symbolic component decide whether to keep it. Acceptance is always by execution: the proposal's claimed effects must hold after the lower-level steps run. None of these systems proves a proposal correct in general, and the ones that learn a library score each proposal over many uses and discard low scorers.

### Cited Findings

- Kambhampati et al. argue that autoregressive LLMs "cannot, by themselves, do planning or self-verification" and should be paired with "external model-based verifiers in a tighter bi-directional interaction regime"; LLMs can also help acquire the models that drive the verifiers — [LLMs Can't Plan, But Can Help Planning in LLM-Modulo Frameworks, ICML 2024, arXiv:2402.01817](https://arxiv.org/abs/2402.01817).
- ChatHTN is an HTN planner that calls ChatGPT only when no method decomposes a compound task; the LLM "returns a sequence of primitive tasks" and planning continues — [Muñoz-Avila, Aha, Rizzo, arXiv:2505.11814](https://arxiv.org/abs/2505.11814).
- After every decomposition, from a method or from the LLM, ChatHTN appends a verifier: a primitive task with no effects whose preconditions are the declared effects of the compound task. If they do not hold, the branch is rejected. The authors claim the planner is sound on this basis; it is not complete, and a wrong LLM decomposition can make a solvable problem fail on that branch. ChatGPT "rarely generates the same output when prompted with the same input", so each test got five attempts — [arXiv:2505.11814](https://arxiv.org/abs/2505.11814).
- A November 2025 extension turns verified LLM decompositions into reusable methods, adding a method only if executing its body is sufficient for the effects of the original task — reported in a [secondary summary](https://www.emergentmind.com/topics/chathtn) (search snippet of an aggregator page; not verified against the paper).
- Ada has an LLM propose high-level action abstractions as named operators with preconditions and effects. A symbolic planner builds a high-level plan from candidate operators; each operator defines a subgoal that a budgeted low-level search must reach (1,000 nodes in Mini Minecraft, 50 in ALFRED) — [Wong et al., Learning adaptive planning representations with natural language guidance, ICLR 2024, arXiv:2312.08566](https://arxiv.org/abs/2312.08566).
- Ada scores each operator by s/b, where b counts its appearances in high-level plans and s the times its low-level execution succeeded, and keeps operators above thresholds on both b and s/b. GPT-3.5 "frequently produces syntactically invalid operator proposals"; invented predicates are stripped and operators left with no valid effect are dropped before verification. Ada relies on a predefined predicate set — [arXiv:2312.08566](https://arxiv.org/abs/2312.08566).
- Silver et al. argue that the downward refinability assumption is untenable in many applications and that learned abstractions should tolerate violations of it — [Inventing Relational State and Action Abstractions, GenPlan 2022](https://aair-lab.github.io/genplan22/papers/silver_GenPlan22.pdf) (search snippet).
- TheoryCoder uses high-level PDDL operators to constrain low-level search and has an LLM synthesize Python transition models that ground them — [arXiv:2503.20124](https://arxiv.org/pdf/2503.20124) (search snippet).

### Inferences

- These systems confirm Mental's division of labor (LLM proposes, checker decides) and add a missing mechanic: a *verifier step*. For every upper step, state its declared effect as a predicate over upper state, and after the mapped lower steps have run, evaluate that predicate on the projected state. ChatHTN's verifier task is exactly this, and it is the per-step half of the projection check.
- Ada's s/b score is the modern form of Bacchus and Yang's refinement probability, applied per operator instead of per level. Mental can keep the same two counters per upper step across scenarios: how often it was used, how often its lower realization satisfied its declared effect. A low ratio marks a bad summary; a low count marks an untested one. Both are honest things to show a user.
- All of these accept by execution on the cases tried. Soundness holds per accepted plan, not for the operator in general. Mental's bounded-exhaustive mode can do better than any of them within its bounds, because it can try all schedules rather than the ones a planner happened to pick.
- Ada needed a fixed predicate vocabulary and still had to strip invented predicates. Mental's rule that predicates must be anchored in code is the same guard, and the Ada result suggests applying it before, not after, any semantic check.
- Run-to-run variation in LLM output (ChatHTN's five attempts) is a reason to store accepted level maps as artifacts and to check agreement across samples, as the "who decides what" table in `how-mental-works.md` already requires.

### Gaps

- I found no 2023–2026 system that checks an LLM-proposed abstract action against *all* refinements, or that derives angelic bounds with an LLM.
- Survival rates of LLM proposals are not reported by Ada; ChatHTN reports only three domains.
- The search surfaced few 2026 papers; coverage of the newest work is thin.

## 8. Network protocol layering

### Takeaway

There is real work checking one protocol layer against the service it should provide, and it uses the same two methods as everywhere else. Netsem specifies TCP twice, as a protocol and as a service, and tests an abstraction function between them on captured traces. Verdi proves a system once over an ideal network and carries the proof to a faulty network through a verified transformer. A full proof that TCP implements its service remained out of reach for Netsem, at 11 person-years.

### Cited Findings

- Netsem wrote specifications in HOL4 "both of the service that they provide to applications (in terms of TCP bidirectional stream connections) and of the internal operation of the protocol (in terms of TCP segments and UDP datagrams), together with a testable abstraction function relating the two", validated against "several thousand traces captured from three implementations (FreeBSD, Linux, and WinXP)" — [Bishop et al., Engineering with Logic, Journal of the ACM 66(1), 2018](https://www.trustworthy.systems/publications/full_text/Bishop_FMNRSSW_19.pdf).
- The abstraction function "takes a pair of states of the protocol-level model (one for each end of a TCP connection), together with an abstract model of the state of the intervening IP network (the TCP segments in flight), and constructs a state of the end-to-end service-level model". It "is what one would use as the invariant of a protocol correctness proof". A checker takes a protocol-level trace and "checks whether there is a corresponding trace of the service-level specification, with the abstraction function mapping protocol-level states onto service-level states at each step" — [same paper, section 1.5.2](https://www.trustworthy.systems/publications/full_text/Bishop_FMNRSSW_19.pdf).
- The authors chose testing because the scale of the specifications made a proof "a challenging prospect". The service-level specification was written by hand, not derived; its control block has two fields against the protocol level's many. Service-level validation reached only proof of concept. The whole project took about 11 person-years — [same paper](https://www.trustworthy.systems/publications/full_text/Bishop_FMNRSSW_19.pdf).
- Verdi lets developers "verify their system under an idealized fault model" and then "transfer the resulting correctness guarantees to a more realistic fault model" "without any additional proof burden"; it "formalizes various network semantics with different faults". Case studies include the first mechanically checked proof of linearizability of Raft — [Wilcox et al., PLDI 2015, abstract](https://homes.cs.washington.edu/~mernst/pubs/verify-distsystem-pldi2015-abstract.html).
- Singha et al. propose sublayering: splitting a protocol layer into sublayers that each talk only to their peer, so each can be verified against its own contract and bugs localized "by examining which sublayer fails its contract". TCP resists this because "common state variables like sequence numbers and window sizes entangle" its functions — [If Layering is useful, why not Sublayering?, HotNets 2024](https://conferences.sigcomm.org/hotnets/2024/papers/hotnets24-309.pdf).
- Their experiments: a sublayered bit-stuffing protocol verified in Coq with 57 lemmas and 1,800 lines, using "separate independent correctness lemmas for each sublayer"; a monolithic lightweight TCP (lwIP) in Dafny with 30 lemmas and about 3,500 lines, proving only in-order reliable delivery from an initially empty network — [same paper](https://conferences.sigcomm.org/hotnets/2024/papers/hotnets24-309.pdf).
- Other layered efforts: a TCP layer verified against validated models of the layers beneath it — [AdaCore, Layered formal verification of a TCP stack](https://adacore.com/papers/layered-formal-verification-of-a-tcp-stack); a framework where security-protocol layers are verified independently and composed — [ePrint 2023/1329](https://eprint.iacr.org/2023/1329.pdf); a Bluetooth L2CAP state machine proved against its specification in Dafny and integrated into Android — [MobiCom 2025](https://www.cs.sfu.ca/~yuepeng/pubs/mobicom25.pdf) (all three: search snippets).

### Inferences

- Netsem is the nearest existing instance of the draft report's recommendation 2 (check refinement per run by projection), and it is on TCP, which every web service sits on. The draft cites TLA+ trace validation and hardware for this; Netsem should be added. It also shows the shape of the abstraction function for a connection: it takes *both endpoints plus the messages in flight*. For Mental, the abstraction from service-level state to a business-level "order delivered" will likewise need the states of several entities and the contents of the wires and queues between them, not one entity's state.
- Netsem's service level was authored by hand after the protocol level was understood, and was far smaller. That matches Mental's plan (LLM proposes upper levels after extraction of lower ones) and warns that validating the upper level took effort the project did not have.
- Verdi's transformer is a level pattern the draft's list (hide, group, atomize, rename, summarize, assume) lacks: *same behavior, weaker environment*. A model checked with reliable delivery is re-run with duplication, loss or reordering switched on at the wires, and a known transformer (retry with deduplication, sequence numbers) is what justifies carrying properties across. In an interpreter this is a mode switch on wires plus a check, not a proof.
- The sublayering paper states the abstraction barrier in protocol terms (each sublayer reads only its own header and interface data) and reports the case where it fails: shared state variables. That is the "leak is a finding" rule from `how-mental-works.md`, with TCP as a real example of a level that cannot be cut where intuition says.

### Gaps

- I did not locate the Ensemble/Nuprl layered-protocol work or the Ivy QUIC specification in this session; both are relevant and unverified here.
- No source found that checks an HTTP-level service description against TCP-level behavior the way Netsem relates TCP to streams.

## Recommendations for Mental

1. **Have the LLM emit the level map as typed hints, and validate each hint separately.** This is Formality's guidance file. Hint kinds: rename, group lower entities into an upper entity, merge or split state, map a lower step sequence to an upper step, declare a lower step invisible. Report accepted and rejected hints per upper element. A rejected hint never silently weakens the check; the affected upper element is marked unverified.
2. **Give every level a standard observation record, as RVFI does.** One record per completed upper step: step name, order number, upper state read, upper state written. Any lower-level run must be able to emit it through the abstraction function. Write the generic checks once against the record.
3. **Check two things per run, not one.** Per step: the lower segment mapped to an upper step makes the step's declared effect true on projected state (ChatHTN's verifier, ISA-Formal's instruction check). Per sequence: no upper step is lost, duplicated or reordered, and each upper read sees the last upper write (riscv-formal's consistency checks). The draft report's recommendation 2 covers the first half only.
4. **Treat the abstraction function as the high-risk artifact.** Arm's experience is that it is the hand-written, slow, error-prone part. Debug each new abstraction function against two or three hand-confirmed simple scenarios before trusting generated checks, and show the user which scenarios those were. Expect it to span several entities and the wires between them (Netsem).
5. **Store the step map as a witness.** HTN plan verification is NP-complete without the decomposition. The interpreter should record, during the run, which lower steps it attributed to which upper step, and the checker should verify that attribution instead of searching for one.
6. **Keep refinability counters per upper step and per level pair.** Count uses and successful lower realizations (Ada's s/b, Bacchus and Yang's refinement probability). Flag upper steps with a low ratio as bad summaries and with a low count as untested. Propose merging an inserted level when its pair-wise ratio stays under a threshold; 0.5 is the value HighPoint used.
7. **Split unrefined steps by who chooses.** For actors the system does not control (courier, customer, operator), store an upper bound on what they may do and require invariants to hold across all of it. For choices the modeled system makes, a lower bound (what it can force) is meaningful. Rename the draft's `bounds` field accordingly, for example `may` (upper bound, always present) and `can_force` (lower bound, optional). Do not describe the lower bound as "guaranteed".
8. **Add a fault-model transition to the level patterns.** Same behaviors, weaker wires (loss, duplication, reordering), justified by a named mechanism such as retry with deduplication. Check it by re-running scenarios with faults enabled and comparing projected outcomes (Verdi's idea, done per run).
9. **Use exact comparison for discrete upper state and declared tolerances for quantities.** Multi-resolution modeling accepts "approximately equal"; Mental should allow that only for durations, counts and rates, with the tolerance stored in the level record.
10. **Fuzz the checkers.** Generate level pairs that are equal by construction and pairs with one seeded mismatch, and confirm the refinement checker accepts the first and rejects the second. Commercial equivalence checkers failed this test on feature interactions, not single features.

## Open questions

1. What is the smallest hint vocabulary that covers the level transitions seen in the worked examples, and how many hints does the LLM get wrong per level on a real repository?
2. Can abstraction functions be proposed by the LLM as expressions over extracted state and still be debuggable by a user in the Arm manner, or do they need a restricted form (projection, count, exists, last-written)?
3. For concurrent upper steps whose lower steps interleave, is the recorded step map enough to keep checking linear, or does Mental need the totally-ordered restriction per entity?
4. Is weak consistency (compare projected outputs) enough for every view the UI offers, or does time-zoom in the middle of a run require strong consistency for some level pairs?
5. How should refinability counters be shown without implying a proof: as coverage ("7 of 9 scenarios realized"), or as a status per step?
6. For an uncontrolled actor, who writes the upper bound, and what stops it from being so wide that every invariant fails? Angelic planning had the same difficulty with useful lower bounds.
7. Do the RAND primary texts (MR-1004, MR-1750, Davis and Hillestad 1993) say anything on tolerance selection that the snippets missed? They were unreachable in this session.
8. Is there post-2016 public evidence on how far Arm or RISC-V vendors have pushed ISA-level checks toward unbounded proof and full-core coverage?
