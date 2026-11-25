# Lifting and multi-level IRs in decompilers

Researched 2026-11-25. Evidence tags on every claim: **[read]** read in the primary source this session (source code, official docs, the paper itself), **[search]** seen only in a search-result summary, **[training]** from training knowledge, not checked this session. Older sources are dated where they appear.

| System | One line |
|---|---|
| Ghidra decompiler | SLEIGH lifts to raw p-code; one "universal" pipeline of actions and opcode-indexed rule pools runs to a fixpoint, then variables are merged and control flow structured into C |
| IDA Hex-Rays | Microcode raised through eight maturity levels (GENERATED to LVARS), then turned into a C tree (ctree); plug-ins add instruction, block and generation callbacks |
| Binary Ninja (BNIL) | Lifted IL, LLIL, MLIL, HLIL, each with an SSA form; separate instruction indices per level with cross-level links; analysis staged as a DAG of "activities" |
| angr | VEX IR lifted to AIL; the Clinic runs 16 named stages and staged optimization passes; SAILR structures the result |
| BAP / BIL | CMU's binary analysis platform and its IL; TIE, the first subtype-based machine-code type inference, runs on it |
| RetDec | Lifts machine code to LLVM IR, optimizes with LLVM passes, emits C through its own high-level IR (not checked this session) |
| remill / McSema / Anvill | remill lifts instructions to LLVM bitcode and leaves memory and some control flow as intrinsics that later tools implement |
| rev.ng | QEMU's front half lifts to tiny code, then to LLVM IR; a YAML model is the project file; a C backend moves to an MLIR dialect, Clift |
| Rellic | LLVM bitcode to goto-free C by pattern-independent structuring, using Z3 |
| MLIR raising | Dialect conversion with legality targets; raising SCF to Affine (from Polygeist, being upstreamed in 2025); mlirSynth raises by program synthesis |
| Phoenix (2013) | Semantics-preserving structural analysis with iterative refinement, on BAP and TIE |
| TIE (2011), Retypd (2016), BinSub (2024) | Type inference over subtype constraints; Retypd adds recursive and polymorphic types, BinSub restates it as algebraic subtyping |
| OSPREY (2021) | Variables and structures recovered as posterior probabilities over probabilistic constraints |
| TRex (2025) | Structural, behaviour-capturing types instead of recovering source types; converted to C types late |

## Ghidra: raw P-code to high P-code, and the action/rule pipeline

### Takeaway

Ghidra raises code inside one function-local graph, not through separately stored levels: raw p-code becomes SSA, a fixed tree of actions and opcode-indexed rule pools rewrites it until nothing changes, and only then are SSA values merged into high variables and blocks into structured statements. The same universal pipeline is cut into different "roots" by naming subsets of rule groups.

### Cited Findings

- The decompiler states its own design as compiler theory run backwards: an RTL (p-code), SSA form, basic blocks and CFGs, term rewriting rules, dead-code elimination, symbol tables; plus tasks with no compiler analogue such as variable merging. [read] — [docmain.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/docmain.hh)
- Documented work flow: entry point, raw p-code, basic blocks and CFG, inspect sub-functions, adjust p-code, then a main simplification loop of SSA generation, dead-code elimination, local type propagation, term rewriting, CFG adjustment and control-flow structure recovery. [read] — [docmain.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/docmain.hh)
- SSA is built incrementally. Stack references that cannot be resolved until register rewriting has run are promoted to full SSA variables when discovered, which "often requires 1 or more additional passes". Unknown effects (aliasing, effects across calls) are recorded with `INDIRECT` ops: the output "is derived from the input by some indirect (frequently unknown) effect". [read] — [docmain.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/docmain.hh)
- Term rewriting: "Each rule matches some potential configuration in a portion of the syntax tree, and after the rule matches, it specifies a sequence of edit operations". The stated goal is "not to optimize as a compiler would, but to simplify and normalize for easier understanding and recognition". [read] — [docmain.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/docmain.hh)
- Framework. `Action` is one transformation with flags `rule_repeatapply` ("Apply rule repeatedly until no change"), `rule_onceperfunc` and `rule_oneactperfunc`. `ActionGroup` runs its children in sequence. `ActionRestartGroup` reruns the whole group when a child sets "restart pending", up to `maxrestarts`. `ActionPool` holds `Rule`s indexed by opcode (`perop[CPUI_MAX]`); each rule declares the opcodes it can apply to (`getOpList`). [read] — [action.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/action.hh)
- Fixpoint mechanics. `Action::perform` loops while `(lcount<count)&&((flags&rule_repeatapply)!=0)`, so a repeating action reruns until a pass changes nothing; there is no iteration cap on this loop. `ActionPool::processOp` tries the rules for an op's opcode in registration order; if a rule changes the opcode, the rule list for the new opcode is scanned from the start. `ActionRestartGroup` stops after `maxrestarts` with the warning "Exceeded maximum restarts with more pending" and clears all analysis before each restart. [read] — [action.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/action.cc)
- Shape of the universal action (from `ActionDatabase::universalAction`): a restart group `universal` (maxrestarts 1) holds start-up actions, then a repeating `fullloop`, which holds a repeating `mainloop` (unreachable code, varnode properties, `ActionHeritage` for SSA, dead code, `ActionInferTypes`, a repeating `stackstall` group containing rule pool `oppool1`, block structure, rule pool `oppool2`, determined branches), then a `cleanup` pool, then structuring (`ActionStructureTransform`), `ActionAssignHigh`, the variable-merging actions (`ActionMergeRequired`, `ActionMarkExplicit`, `ActionMarkImplied` "must come BEFORE general merging", `ActionMergeCopy`, `ActionMergeAdjacent`, `ActionMergeType`), `ActionNameVars`, `ActionSetCasts`, `ActionFinalStructure`. Ordering constraints are kept as comments ("Must come before restructurevarnode and infertypes"). The file registers 163 rules. [read] — [coreaction.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/coreaction.cc)
- Every action and rule is tagged with a group name (`analysis`, `deadcode`, `typerecovery`, `blockrecovery`, `merge`, ...). A root is a named set of groups: `decompile` (all), `jumptable`, `normalize`, `paramid`, `register`, `firstpass` (`base` only). Each root is a clone of the universal action keeping only members of its groups. [read] — [coreaction.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/coreaction.cc), [action.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/action.hh)
- Debug support: per-action and per-rule breakpoints, change counters (`count_tests`, `count_apply`), and warnings issued when a flagged action makes a change. [read] — [action.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/action.hh)
- "High" p-code: a `HighVariable` is a set of SSA varnodes manipulated as one variable; their covers (live ranges) must not intersect, "as that represents the variable holding two or more different values at the same place", and the variable's cover is their union. [read] — [variable.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/variable.hh)
- Instructions SLEIGH does not model are emitted as user-defined p-code ops (`CALLOTHER`), which the decompiler prints as calls to named pseudo-functions. [training]

### Inferences

- Ghidra's "levels" (raw, SSA, simplified, high) are phases of one mutable graph, not stored ILs with maps between them. The ordering that makes it work lives in code comments, not in declared dependencies.
- The group/root idea is directly reusable: one rule set, with each level (or each consumer, as Ghidra's `jumptable` root is a cheap pre-pass) a named subset.

### Gaps

- How `SeqNum` (address plus order) and the C token-to-op links carry each output token back to p-code was not checked this session.

## IDA Hex-Rays: microcode maturity levels and plug-in optimizers

### Takeaway

Hex-Rays raises one microcode container through ordered maturity levels, each a checkpoint where specific facts become available (call ABIs, merged blocks, SSA local variables), then builds a separate C tree. Plug-ins add instruction and block rules that the kernel calls at every maturity, plus generation filters that replace how one machine instruction is lifted.

### Cited Findings

- Levels, lowest to highest: `MMAT_GENERATED`, `MMAT_PREOPTIMIZED`, `MMAT_LOCOPT`, `MMAT_CALLS`, `MMAT_GLBOPT1`, `MMAT_GLBOPT2`, `MMAT_GLBOPT3`, `MMAT_LVARS`. GENERATED is a direct translation with `und` for unimplemented instructions and blocks split at calls; PREOPTIMIZED uses use-def lists for dead-code deletion and propagation; LOCOPT names stack offsets as locals; CALLS applies call ABIs so arguments are known; GLBOPT1 merges call-split blocks and removes prologue and epilogue; LVARS runs SSA and turns registers into local variables. [read, secondary: notes on Ilfak Guilfanov's Recon 2018 talk] — [icecr4ck gist](https://gist.github.com/icecr4ck/6c744d489efbb07a32bb22e8a3c748e3)
- `MMAT_ZERO` means no microcode yet; at `MMAT_GLBOPT3` the microcode is considered fixed. [search] — [Elastic Security Labs](https://www.elastic.co/security-labs/introduction-to-hexrays-decompilation-internals)
- The ctree view is available only at `MMAT_LVARS` or higher, because local variables must be allocated first. [search] — [Hex-Rays docs](https://docs.hex-rays.com/core/decompiler/how-tos/microcode-view)
- A microinstruction (`minsn_t`) has left, right and destination operands; an operand (`mop_t`) can itself hold a nested microinstruction, so expressions are trees. Each instruction carries use and def location lists (`mlist_t`). `m_ext` is an "external insn, not microcode". [read, secondary] — [icecr4ck gist](https://gist.github.com/icecr4ck/6c744d489efbb07a32bb22e8a3c748e3)
- `optinsn_t::func()` "must inspect the provided microinstruction and try to optimize it, returning a non-zero value if it can"; it is installed with `install_optinsn_handler()` and "called all throughout the maturity lifecycle", so authors "write our patterns targeting the lowest maturity level at which we can reasonably match them". A deobfuscator (control-flow unflattening) was built as an `optblock_t`. Safe edits modify existing objects; `verify()` "applies a comprehensive set of test suites to the microcode objects". [read; Rolf Rolles, Sep 2018] — [Hex-Rays blog](https://hex-rays.com/blog/hex-rays-microcode-api-vs-obfuscating-compiler)
- `microcode_filter_t` has `match()` and `apply()`; before generating microcode for an instruction the decompiler visits every registered filter; `apply()` returns `MERR_OK` when it produced microcode, or `MERR_INSN` to fall back to the standard path. [search] — [IDA C++ SDK](https://cpp.docs.hex-rays.com/structmicrocode__filter__t.html)
- `mba_t` records the current `maturity` and a required `reqmat`, so a client can ask for microcode only up to a chosen level. [search] — [IDA C++ SDK, mba_t](https://cpp.docs.hex-rays.com/classmba__t.html)
- D810, a rule-based deobfuscation plug-in, is built on these optimizer hooks. [search] — [eShard blog](https://www.eshard.com/blog/d810-deobfuscation-ida-pro)
- Instructions with no C equivalent are emitted as calls to helper pseudo-functions (intrinsics such as `__readfsqword`). [training]

### Inferences

- A maturity level is a promise about which facts hold ("arguments are known", "variables are allocated"), and a client can stop at one. That is a cleaner contract than Ghidra's phase ordering in comments.
- Rules that fire at every maturity make patterns fragile: Rolles had to pick the earliest level at which a pattern still looked the same. A pattern that declares its level avoids that.

### Gaps

- The `hexrays.hpp` header was not read directly; filter registration and the exact re-optimization loop after a nonzero return are unconfirmed.

## Binary Ninja: LLIL, MLIL, HLIL and their SSA forms

### Takeaway

Binary Ninja keeps every level as its own stored IL with its own instruction indices and SSA form, and links instructions across levels many-to-many. Each level removes one kind of machine detail: flags at LLIL, registers and the stack at MLIL, unstructured control flow at HLIL.

### Cited Findings

- Lifted IL is "a straight-forward translation from native instruction semantics". LLIL removes NOPs and folds flags into conditionals. MLIL translates registers and memory accesses into variables, associates types with variables, generates typed call sites, computes data flow and propagates constants. HLIL adds "higher level control flow instructions", dead-code and variable passes, and folding, and is an AST. [read] — [BNIL overview](https://docs.binary.ninja/dev/bnil-overview.html)
- "Each IL instruction needs to have its own index separate from its address", because translation "is a many-to-many relationship"; "you can not rely on an instruction from one layer existing in another". LLIL instructions are expression trees. [read] — [LLIL docs](https://docs.binary.ninja/dev/bnil-llil.html)
- What cannot be lifted is explicit: `LLIL_UNIMPL` ("Unimplemented instruction"), `LLIL_UNIMPL_MEM` (unimplemented but accesses memory), `LLIL_UNDEF`, and `LLIL_INTRINSIC`, "operations with `output` and `params` and an `intrinsic` where the exact behavior is not modelled" (example: AES instructions). [read] — [LLIL docs](https://docs.binary.ninja/dev/bnil-llil.html)
- MLIL: each variable represents "one and only one storage location"; "The stack as a concept is not present"; types carry a `confidence`; call sites have inferred parameters; `MLIL_VAR_PHI` and `MLIL_MEM_PHI` merge variable and memory versions. When stack resolution fails, a call stays `MLIL_CALL_UNTYPED` with no parameter list. [read] — [MLIL docs](https://docs.binary.ninja/dev/bnil-mlil.html)
- Cross-level navigation goes through instruction properties (LLIL to MLIL to HLIL and back) and function-level `ssa_form`; the Rust API builds an instruction from an index through the core's index-mapping routines. [search] — [BN API docs](https://dev-api.binary.ninja/binaryninja.mediumlevelil-module.html), [Rust source](https://rust.binary.ninja/src/binaryninja/medium_level_il/function.rs.html)
- The exact properties are `.llil`, `.mlil`, `.hlil`, `.ssa_form`, `.non_ssa_form`, plus `get_ssa_var_definition` and `get_ssa_var_uses` for def-use chains. [training]
- Analysis is organized as Workflows: a DAG of named activities that a plug-in can clone and extend by inserting an activity before or after a named one; activities may read and replace each IL level. The feature is marked experimental, and the docs warn that, depending on where an activity is inserted, stack offsets or data flow may not be computed yet. [search] — [Workflows docs](https://docs.binary.ninja/dev/workflows.html), [Modifying IL](https://docs.binary.ninja/dev/bnil-modifying.html)

### Inferences

- BNIL is the closest decompiler analogue to Mental's ladder: stored levels, one vocabulary per level, explicit "not lifted" nodes, and maps between levels. Its warning that an instruction may not survive into the next layer is the case Mental's "support" marking covers.

### Gaps

- The maintained index maps (how many-to-many links are stored and kept after edits) were not documented in the pages read.

## angr, BAP, RetDec, remill/McSema, rev.ng, Rellic, Phoenix and MLIR raising

### Takeaway

The open systems repeat two patterns: a small set of stored ILs (VEX, AIL; LLVM IR, then C) with passes that declare the stage they run at, and explicit placeholders (intrinsics, casts) for what a level cannot express. MLIR is the only framework that makes "what may remain at this level" a declared, checkable target.

### Cited Findings

- angr's Clinic "lifts a function to AIL and runs the decompiler's simplification pipeline on it" through 16 ordered stages: initialization, AIL graph conversion, return sites, argument list, stack pointer tracking, constant propagation, call sites, post-call-sites, SSA level 0 fixups and transformation, pre-SSA level 1 simplifications, SSA level 1 transformation, post-SSA simplifications, variable recovery, semantic variable naming, externs. Callers can choose `start_stage`, `end_stage` and `skip_stages`. Type inference (Typehoon) has `max_type_constraints` and a `type_constraint_set_degradation_threshold`. [read] — [angr Clinic docs](https://docs.angr.io/en/latest/api/angr.analyses.decompiler.clinic.html)
- angr optimization passes declare when and where they run as class attributes: `STAGE` (an `OptimizationPassStage`), `ARCHES`, `PLATFORMS`, `STRUCTURING` (which structurers they serve, e.g. `['sailr', 'dream']`), plus `NAME` and `DESCRIPTION`. Examples: inlined `memcpy`/`strcpy` simplifiers, `CrossJumpReverter`, `DeadblockRemover` ("Remove blocks with unsatisfiable conditions"). [read] — [angr optimization passes](https://docs.angr.io/en/latest/api/angr.analyses.decompiler.optimization_passes.html)
- SAILR (USENIX Security 2024) structures code by "precisely inverting" GCC's goto-inducing transformations; the algorithm is split between the structurer and several optimization passes. [search] — [SAILR paper](https://adamdoupe.com/publications/sailr-usenix2024.md), [angr SAILR docs](https://docs.angr.io/en/latest/api/angr.analyses.decompiler.structuring.sailr.html)
- angr lifts machine code with VEX, the IR of Valgrind, through pyvex, before converting to AIL. [training]
- TIE is a static type reconstruction tool on CMU's BAP platform. [read, as described in the Retypd paper] — [Retypd, arXiv](https://ar5iv.arxiv.org/html/1603.05495)
- remill models instruction semantics in LLVM bitcode but "defers the 'implementation'" of memory access and some control flow "to the consumers of the produced bitcode": memory is reached only through intrinsics such as `__remill_read_memory_8`, so later tools can tell modelled memory accesses from ordinary loads and stores; `__remill_sync_hyper_call` stands for instructions like `cpuid`. [read] — [remill INTRINSICS.md](https://github.com/lifting-bits/remill/blob/master/docs/INTRINSICS.md)
- rev.ng uses the first half of QEMU to lift to tiny code, converts it to LLVM IR, and keeps user-facing facts (functions, prototypes, names) in a YAML model that is the project file; its roadmap replaces the text C backend with an MLIR dialect, Clift. [search] — [rev.ng blog](https://rev.ng/blog/open-sourcing-renvg-decompiler-ui-closed-beta), [rev.ng roadmap](https://rev.ng/roadmap)
- Rellic produces goto-free C from LLVM bitcode with a pattern-independent structuring algorithm and depends on Z3. [search] — [Rellic README](https://github.com/trailofbits/rellic)
- Phoenix (USENIX Security 2013) runs structural analysis that only applies a region schema when it preserves semantics, and when no schema applies, iteratively refines the graph by removing an edge (emitting a goto) and retrying. [training]
- RetDec lifts to LLVM IR, optimizes with LLVM and its own passes, and emits C through a separate high-level IR. [training]
- MLIR dialect conversion: a `ConversionTarget` marks ops legal, dynamically legal (legal only under a condition) or illegal. Partial conversion lets ops not marked illegal "remain unconverted"; full conversion fails unless "only known operations will exist after the conversion process"; analysis conversion computes what would convert and applies nothing. Where a value's type changes, the driver inserts `builtin.unrealized_conversion_cast` to bridge old and new IR. In rollback mode the driver can undo patterns when it gets stuck. [read] — [MLIR Dialect Conversion](https://mlir.llvm.org/docs/DialectConversion/)
- MLIR's greedy pattern driver works from a worklist, applies the highest-benefit pattern, and reapplies "until a fixed point is reached" or a configurable maximum iteration count; patterns name a root op; patterns have debug names and labels that can enable or disable them; a pattern that recurses without declaring bounded recursion makes the driver fail. [read] — [MLIR Pattern Rewriter](https://mlir.llvm.org/docs/PatternRewriter/)
- A `raise-scf-to-affine` pass, ported from Polygeist, was proposed upstream in September 2025 (PR #158267); its acceptance is not confirmed. [search] — [mlir-commits](https://lists.llvm.org/pipermail/mlir-commits/2025-September/130540.html)
- mlirSynth raises lower dialects to higher ones without hand-written rules: it "uses available dialect definitions to construct a program space and searches it effectively using type constraints and equivalences". [read, abstract only] — [mlirSynth, arXiv 2310.04196](https://arxiv.org/abs/2310.04196)

### Inferences

- angr's `STAGE` attribute is the Hex-Rays lesson made explicit: a rule states the level it reads, instead of being called at all of them.
- remill's intrinsics and MLIR's unrealized casts are the same move: the lower level keeps a typed, named placeholder where the upper meaning is not yet known, and a later step must remove it.

### Gaps

- The Phoenix PDF could not be turned into text in this session, so its refinement details are from training knowledge. RetDec and BAP's current Core Theory were not checked.

## Traceability from high-level constructs back to instructions

### Takeaway

Every system keeps an address on each IL instruction, and some keep per-level indices with cross links, but none stores a validated map that says which lower steps make up each upper construct. Maps are exact only before structure-changing passes.

### Cited Findings

- Binary Ninja gives every IL instruction its own index and an address, and links levels many-to-many; an instruction may not exist in another layer. [read] — [LLIL docs](https://docs.binary.ninja/dev/bnil-llil.html)
- angr keeps `unoptimized_graph`, "a copy taken before the first structure-altering optimization pass; use it for an exact instruction-to-AIL mapping", only when asked (`save_unoptimized_graph=True`); data references carry `ins_addr`, `stmt_idx` and `block_addr`. [read] — [angr Clinic docs](https://docs.angr.io/en/latest/api/angr.analyses.decompiler.clinic.html)
- Hex-Rays microinstructions and ctree items carry the address they came from; Ghidra p-code ops carry a sequence number (address plus order) and output tokens link to ops. [training]
- MLIR attaches a location to every op and fuses the locations of ops a rewrite combines. [training]
- LLVM optimization remarks report passes that applied, passes that were missed and why, and analysis notes, each tied to a source location. [training]

### Inferences

- Mental's anchors (file, lines, quote) plus its step map are stronger than what decompilers keep: decompilers trace to an address but not to the set of lower steps a construct stands for. The angr note gives the practical rule: build the map at the moment a step is cut, before any upper-level simplification.
- LLVM's missed-optimization remarks are the nearest precedent for reporting a near-miss match as a located diagnostic.

### Gaps

- No source was found that describes how Hex-Rays or Ghidra keep address maps correct through merges; this is training knowledge only.

## Data flow, SSA, variable recovery and type recovery

### Takeaway

Decompilers give meaning to low-level values in two steps: data flow (SSA, def-use, value sets) groups definitions into variables, and constraint inference over those variables assigns types. The field moved from unification (over-merges) to subtyping (TIE, Retypd), to probabilities (OSPREY), and to behaviour-defined structural types (TRex).

### Cited Findings

- Ghidra variables are SSA definitions merged when their live ranges do not intersect; type propagation pushes a type along data-flow edges only when it is "more specific than the current data-type", visiting each varnode once. After seven passes it stops with "Type propagation algorithm not settling" ("This constant arrived at empirically"). [read] — [coreaction.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/coreaction.cc), [variable.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/variable.hh)
- Hex-Rays allocates local variables and runs SSA at `MMAT_LVARS`. [read, secondary] — [icecr4ck gist](https://gist.github.com/icecr4ck/6c744d489efbb07a32bb22e8a3c748e3)
- Binary Ninja MLIL variables are one storage location each, with types carrying a confidence, and data flow exposed as possible value sets. [read] — [MLIL docs](https://docs.binary.ninja/dev/bnil-mlil.html)
- TIE (NDSS 2011) "was the first machine-code type-inference system to track subtype constraints and explicitly maintain upper and lower bounds on each type variable". [read, as described by Retypd] — [Retypd, arXiv](https://ar5iv.arxiv.org/html/1603.05495)
- Retypd (PLDI 2016): each value gets a type variable; operations give subtype constraints (`X ⊑ Y` for a copy) and derived variables with labels (`.load`, `.store`, `.in`, `.out`, `.σN@k` for an N-bit field at offset k). Entailed constraints form a pushdown system whose solutions are "sketches", trees of capabilities whose nodes carry elements of a lattice of C type names, API typedefs and user semantic tags. Each procedure gets a most-general scheme, solved bottom-up over call-graph SCCs and specialized at call sites. Satisfiability is checked late, when converting to C types. [read] — [Retypd, arXiv](https://ar5iv.arxiv.org/html/1603.05495)
- Retypd's case against unification: "Type systems that model value assignments as type unifications are vulnerable to over-unification", and "Since unification is non-local, bad constraints in one part of the binary can degrade all type results". Reported mean distance to ground truth: 0.54 for Retypd against 1.58 for TIE. [read] — [Retypd, arXiv](https://ar5iv.arxiv.org/html/1603.05495)
- BinSub (2024) restates Retypd in algebraic subtyping and reports a 63x average speedup on 1,568 functions. [search] — [BinSub, arXiv 2409.01841](https://arxiv.org/abs/2409.01841)
- OSPREY (IEEE S&P 2021) treats variables as random variables linked by probabilistic constraints from program analysis and reports posterior probabilities as the result, arguing deterministic methods have no systematic way to handle the uncertainty compilation leaves. [search] — [OSPREY record](https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_ieee_primary_9519451)
- TRex (USENIX Security 2025) argues source types are often unrecoverable and reconstructs structural types defined by observable behaviour, converted to C late; it reports better types than Ghidra on 123 of 125 binaries in the paper (124 of 125 in the artifact appendix: the sources disagree). [search] — [TRex, USENIX](https://www.usenix.org/conference/usenixsecurity25/presentation/bosamiya)

### Inferences

- Analogues for Mental. A state variable is a `HighVariable`: many write and read sites of one stored location, merged into one named thing. A state variable's domain (enum values or literals written) is Retypd's lattice of atomic types with semantic tags. Retypd's capabilities (`.load`, `.store`, `.in`, `.out`) correspond to which effects touch a store and through which port. A predicate is a capability that a branch node decides.
- Subject recovery (which rows, keys and messages are the same subject, for goals and outcomes) is a variable-merging problem. Retypd's lesson applies: merge by local, directed constraints, not global unification, or one wrong edge merges two subjects everywhere.
- TRex's stance fits Mental's: name things by what the code does with them, and convert to a reader's vocabulary late.

### Gaps

- OSPREY's and TRex's full texts were not read; their solving details are not in this note.

## What cannot be raised

### Takeaway

No decompiler refuses to produce output. Each keeps an explicit marker for what it could not raise (unimplemented op, intrinsic, untyped call, cast, goto) and some just warn. MLIR alone lets the caller choose between "leave it" and "fail".

### Cited Findings

- Binary Ninja: `LLIL_UNIMPL`, `LLIL_UNIMPL_MEM`, `LLIL_INTRINSIC`, `MLIL_CALL_UNTYPED` when "stack resolution could not be determined". [read] — [LLIL docs](https://docs.binary.ninja/dev/bnil-llil.html), [MLIL docs](https://docs.binary.ninja/dev/bnil-mlil.html)
- Hex-Rays: `und` for unimplemented instructions, `m_ext` for an external instruction; a generation filter may return `MERR_INSN` to fall back to the standard lifter. [read, secondary] — [icecr4ck gist](https://gist.github.com/icecr4ck/6c744d489efbb07a32bb22e8a3c748e3); [search] — [IDA C++ SDK](https://cpp.docs.hex-rays.com/structmicrocode__filter__t.html)
- Ghidra: `INDIRECT` for unknown effects; restart cap and type propagation cap end with warnings, not errors. [read] — [docmain.hh](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/docmain.hh), [action.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/action.cc), [coreaction.cc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Features/Decompiler/src/decompile/cpp/coreaction.cc)
- remill: intrinsics for memory and hyper calls, implemented by consumers. [read] — [remill INTRINSICS.md](https://github.com/lifting-bits/remill/blob/master/docs/INTRINSICS.md)
- angr: a degradation threshold on the type-constraint set. [read, name only] — [angr Clinic docs](https://docs.angr.io/en/latest/api/angr.analyses.decompiler.clinic.html)
- MLIR: partial conversion leaves unconverted ops, full conversion fails, `unrealized_conversion_cast` bridges type changes until removed. [read] — [MLIR Dialect Conversion](https://mlir.llvm.org/docs/DialectConversion/)
- Phoenix and most structurers emit `goto` where no schema applies. [training]

### Inferences

- For Mental the marker is what matters, not the fallback: an unraised lower step must appear at the upper level as a named, anchored residue with a diagnostic whose code fixes its severity. A warning that hides a capped loop (Ghidra's seven passes) is the opposite of "fail rather than degrade".

### Gaps

- None beyond those noted per system.

## Fit for mentalc

Ranked by fit with mentalc's constraints: pure and deterministic, no LLM for facts, every record anchored, fail rather than degrade. The running example is the claim in `generate_report`: read the row, test `PENDING`, write `RUNNING`, with no conditional update.

1. **A declared legality target per level (MLIR dialect conversion).** Each upper level states which lower constructs it accepts, and "dynamically legal" states the condition. Full conversion is the coverage check (every lower step class mapped or marked support); analysis mode answers "what would this level cover" without writing. A *claim* pattern at the move level can be legal only when its write is guarded: the claim above matches every part but the guard, and that partial match is the duplicate-claim finding, anchored to the write. Fits all four constraints; the one change is that a partial conversion's residue must be a diagnostic, never silent.
2. **Rule pools indexed by root kind, run to a fixpoint, grouped and cut into roots (Ghidra `ActionPool` and `ActionDatabase`, MLIR greedy driver).** Index patterns by the effect kind they start from (a write, a commit), apply in a fixed registration order, repeat until nothing changes; tag rules with groups so each level, and cheap pre-passes, are named subsets of one rule set. Deterministic if the order is fixed and the graph is visited in ID order. Termination must be proved by construction (a rule only adds upper records, or strictly shrinks a measure), and a cap that is hit must fail the build with the rule named, not warn as Ghidra does.
3. **Explicit unraised nodes (Binary Ninja `UNIMPL`/`INTRINSIC`/`CALL_UNTYPED`, remill intrinsics, MLIR casts).** Each level's IL has a residue node that carries the lower steps it could not raise, with their anchors. It keeps the model complete without guessing, and the near-miss case above becomes "a residue next to a pattern it almost matched".
4. **Stored levels with their own IDs and cross-level maps (Binary Ninja), built at the cut (angr's `unoptimized_graph`).** Mental already plans this as the step map. The decompilers add two lessons: expect many-to-many links and the "support" case, and record the map when a step is cut, before any later simplification, because later passes break exactness.
5. **Levels as maturity contracts and passes that declare their stage (Hex-Rays `MMAT_*`, angr `STAGE`).** Each level states which facts hold at it, and each pattern states the level it reads. This avoids Hex-Rays' problem of rules firing at every maturity. Fits directly; it also gives a "build up to level N" option like `reqmat`.
6. **Constraint inference over directed, local constraints (TIE, Retypd, Ghidra's ordered type push).** Use it for state-variable domains and for subject identity: which keys, rows and messages belong to one subject. Deterministic solvers fit; prefer subtyping-style constraints to unification, as Retypd shows unification spreads one bad merge everywhere. Polymorphic per-function schemes match Mental's per-behavior summaries. Cost: a solver to build and test.
7. **Schema matching with refinement, inverting known transformations (Phoenix, SAILR).** A pattern library of upper-level shapes, applied only when it preserves what is seen at the upper level (the same test Mental uses for a level holding), with a residue when none applies. SAILR's "invert what GCC does" maps to packs that know what an ORM or a task framework does. Fits; the risk is a growing library tuned to the measured subjects.
8. **Restart groups (Ghidra `ActionRestartGroup`).** Rerun earlier phases when a later one discovers something (Ghidra promotes stack references to variables). Deterministic, but it couples levels; Mental's stratified order (effects, moves, activations, goals, outcomes, each finished before the next) makes it unnecessary unless subject recovery feeds back into moves.
9. **In-place mutation with a verifier (Hex-Rays `verify()`).** Keep the verifier, drop the mutation: run the structural and anchor checks after each level is built. Mental's records are written, not edited.
10. **Probabilistic recovery (OSPREY) and confidence-weighted types (Binary Ninja).** Poor fit: a fact with a probability is not a mechanical fact. At most, ranked candidates could feed the LLM's naming, which never decides structure.
11. **Raising by program synthesis (mlirSynth).** Poor fit for now: it searches a program space and needs an equivalence oracle. `mentalvm`'s pair check could serve as that oracle in a later experiment, but synthesis would not be anchored to code idioms.
12. **Capped loops that warn and continue (Ghidra type propagation, angr's degradation threshold).** Reject: it is degrading in place of failing.
