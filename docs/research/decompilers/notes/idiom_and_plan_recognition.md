# Idiom, library-function, design-pattern and plan recognition

How decompilers, binary-analysis tools, program-understanding systems, design-pattern detectors and bug-pattern checkers give a name and a meaning to a recognized fragment of a program, how they rank ambiguous matches, how they report partial matches, and what that means for lifting mentalc's effect graph level by level. Researched 2026-11-25.

**Evidence tags.** **[R]** read in the primary source (or, where said, a detailed secondary write-up of it) this session. **[S]** seen only in a search-result summary or a short secondary page. **[T]** from training knowledge, not checked this session; the work is named so it can be checked. Many sources here are old (1990s plan recognition, 2000s pattern detection); the year is given where it matters.

| System | One line |
|---|---|
| Ghidra decompiler | P-code SSA rewritten by hundreds of local rules to a fixpoint; idioms (magic division, switch tables) are rules with dataflow preconditions; a failed idiom leaves a warning at the address [T] |
| Hex-Rays | Microcode optimizer with built-in idiom passes and plugin hooks; folds multiply-shift back to `/`, inlined `memcpy`/`strlen` back to calls [T] |
| Binary Ninja | Layered IL (LLIL, MLIL, HLIL); WARP signatures are exact GUIDs over masked basic blocks, with constraints to separate collisions [R] |
| RetDec | LLVM-based; a per-compiler idiom pass (GCC, MSVC, magic div/mod) rewrites LLVM IR patterns [T] |
| IDA FLIRT | First 32 bytes with variant bytes masked, CRC16 of the tail, referenced names to split collisions; "a false positive is worse than a false negative" [R] |
| IDA Lumina | Shared server of function metadata keyed by a hash of the masked function bytes [T] |
| Ghidra Function ID | Full hash of masked code plus a specific hash that adds call context; parent/child relations break ties [S, T] |
| Ghidra BSim | Fuzzy: TF-IDF-weighted feature vectors from normalized p-code dataflow and control flow, cosine-style similarity in [0, 1] [R, secondary] |
| BinDiff | Two-binary matching by a cascade of matchers ordered from most to least reliable, each match with similarity and confidence [T] |
| Programmer's Apprentice | Plan Calculus: plans as data/control-flow graphs plus overlays between levels; a library of clichés [T] |
| GRASPR (Wills) | Programs as attributed flow graphs, clichés as an attributed flow-graph grammar, recognition as chart parsing; yields a hierarchical design description [S, T] |
| Concept Recognizer (Kozaczynski & Ning) | Concept hierarchy matched over the AST with data and control constraints [T] |
| DECODE (Quilici) | Memory-based: an index element proposes candidate plans, constraints confirm them; interactive [T] |
| DESIRE (Biggerstaff) | Concept assignment using names, comments and clustering, by plausible reasoning rather than parsing [T] |
| Tsantalis DPD | Patterns and systems as relation matrices; Blondel-style similarity scoring per inheritance hierarchy [S] |
| DeMIMA / Ptidej (Guéhéneuc) | Explanation-based constraint programming; relaxes constraints to find "distorted" micro-architectures (near misses) [T] |
| SPQR | Elemental design patterns composed into patterns by rules in a calculus, inferred with a theorem prover [T] |
| Pinot | Classifies GoF patterns by whether structure or behavior decides them; checks behavior with dataflow [T] |
| DECOR | Rule cards (a DSL) for smells and anti-patterns, compiled into detectors [T] |
| FindBugs / SpotBugs | Hand-written bug-pattern detectors with confidence and rank; inconsistent-synchronization is a near-miss detector [T] |
| Semgrep | Patterns in target-language syntax; `pattern-inside` / `pattern-not` express "X without Y" [T] |
| CodeQL | Datalog-like queries plus models-as-data; each query declares a precision level [T] |
| RacerD (Infer) | Compositional race detector; uses the code's own locking as evidence of intent to be thread-safe [T] |
| Feral concurrency (Bailis 2015) | 67 Rails apps; application-level uniqueness and association validations are unsafe under concurrency [S] |
| ACIDRain / 2AD (2017) | SQL traces to an abstract history; cycles are anomaly witnesses; 22 attacks in 12 e-commerce apps [R, secondary] |
| Ad hoc transactions (Tang 2022) | 91 application-coordinated transactions in 8 web apps; 53 have concurrency bugs [S] |
| IsoDiff (2020) | Finds anomalies caused by weak isolation from traces and suggests which transactions to strengthen [T] |
| Enterprise Integration Patterns | 65 messaging patterns, among them Idempotent Receiver, Competing Consumers, Dead Letter Channel [T] |
| microservices.io | Transactional outbox, saga, idempotent consumer, polling publisher [T] |
| Patterns of Distributed Systems (Joshi) | Write-ahead log, lease, generation clock, idempotent receiver, versioned value and others [T] |

## Decompiler idiom recognition: how Ghidra, Hex-Rays, Binary Ninja and RetDec encode and apply compiler idioms

### Takeaway

In decompilers an idiom is a local, exact rewrite with a dataflow precondition, applied over an SSA IL until nothing changes. A precondition either holds and the rewrite fires, or it fails and the lower form stays. Nothing is scored, and the only "near-miss" output is a warning at an address saying a construct could not be recovered.

### Cited Findings

- Compilers replace division by a constant with a multiply-high by a "magic" reciprocal and shifts; the method and its proofs are in Granlund & Montgomery, "Division by invariant integers using multiplication", PLDI 1994. Decompilers detect the multiply-shift shape, check that the constant and shift amounts match some divisor, and fold it back to `x / d` [T].
- Ghidra's decompiler lifts machine code to p-code, puts it in SSA form, and applies a large set of rewrite rules grouped into actions, repeated to a fixpoint (`ruleaction.cc` in the decompiler source; the division rule recovers `/` from the multiply form) [T].
- Ghidra recovers a switch by slicing back from an indirect jump to the index variable, bounding it from the guarding comparison and reading the table; when it cannot, it prints a warning in the decompiled output such as "Could not recover jumptable" and treats the jump as unknown [T].
- Hex-Rays decompiles through a microcode IL with an optimizer that runs built-in idiom passes; its SDK exposes instruction-level and block-level optimizer callbacks so a plugin can add rewrites [T].
- Binary Ninja exposes three IL tiers (LLIL, MLIL, HLIL); WARP uses "our BNIL representations to identify the instructions to mask off" before hashing — the IL decides which bytes are variant [R] ([Binary Ninja blog, WARP](https://binary.ninja/2025/08/22/warp.html)).
- RetDec, LLVM-based, has a `bin2llvmir` idiom pass with one module per compiler family (GCC, Visual Studio, Borland, Intel, LLVM) and one for magic division and modulo; it rewrites LLVM IR instruction patterns back to their source operations. The approach is described in Křoustek et al.'s papers on instruction-idiom detection in a retargetable decompiler (around 2014) [T].
- Inlined `memcpy`, `memset` and `strlen` (for example `rep movsb`, or unrolled word loops) are recognized by shape and replaced by calls, so the reader sees the library operation [T].

### Inferences

- The decompiler model maps directly onto mentalc's ladder: each idiom is a rule "this subgraph, with these dataflow facts, is this one higher node". Mentalc's effect graph is already the analogue of SSA p-code, and packs are already the analogue of the per-compiler idiom tables.
- Decompilers work because the idioms come from a closed source (what a compiler emits). Application patterns (outbox, claim) come from programmers, so they vary much more; that is where decompiler technique stops and plan recognition starts.
- Decompilers never report "almost an idiom". Mentalc wants exactly that report, so it needs something the decompiler lineage does not have.

### Gaps

- I did not read the Ghidra, Hex-Rays or RetDec idiom source or docs this session; the rule names and module list above are from memory and should be checked in the repositories before being cited.
- No decompiler publishes precision/recall for idiom rewriting; correctness is by construction (the precondition implies equivalence).

## Library function recognition: FLIRT, Lumina, Function ID, BSim, WARP, BinDiff

### Takeaway

Library recognition is exact hashing of normalized code, plus context (referenced names, callees, neighbours) to break collisions, plus human curation, with a stated policy that a wrong name is worse than no name. The fuzzy tools (BSim, BinDiff) give a score and leave the decision to a threshold or a person.

### Cited Findings

- FLIRT: "Patterns are first 32 bytes of a function where all variant bytes are marked"; patterns go in a prefix tree, so lookup "grows logarithmically with the number of functions"; when two functions share the 32 bytes, a CRC16 is taken "starting from position 33 until the first variant byte" [R] ([Hex-Rays, FLIRT in depth](https://docs.hex-rays.com/core/flirt/concepts/ida-f.l.i.r.t.-technology-in-depth.md)).
- FLIRT separates `_strupr` and `_strlwr`, identical except for the function each calls, by the referenced name (`_toupper` vs `_tolower`); such matches may wait for a second pass once the referenced names are known [R] (same source).
- FLIRT's policy: "We consider that a false positive ... is worse than a false negative" and "It is better not to recognize than to recognize wrongly"; collisions that cannot be separated are resolved by "the human creator of a signature file" choosing which functions to keep; identical functions are "a frequent situation in standard libraries, especially in C++ libraries" [R] (same source). The page is from the IDA 3.6 era and gives no collision rates [R].
- FLIRT skips functions shorter than 4 bytes with no external references [R] (same source).
- Lumina is a Hex-Rays server where users push and pull function metadata (names, prototypes, comments) keyed by a hash of the function's masked bytes [T].
- Ghidra Function ID stores a full hash (masked code) and a specific hash (adds call-target information) per function; a clone of it tries the specific hash first and falls back to the full hash [S] ([rsleigh-fid, a Ghidra FID clone](https://docs.rs/rsleigh-fid/0.4.1/src/rsleigh_fid/lib.rs.html)). Ghidra's own analyzer also uses parent/child call relations and an instruction-count threshold so short functions do not match on too little evidence [T].
- BSim builds a sorted vector of 32-bit hashes from normalized "High P-code": varnode hashes refined over 3 Weisfeiler-Lehman-style iterations on inputs, block hashes from control flow, weighted by IDF (a table of 1000 common hashes) and a sqrt(1 + log2(tf)) term; the score is cosine-style in [0, 1] and "Unmatched features inflate the denominators without contributing to the numerator" [R, secondary] ([Quarkslab, BSim explained](https://blog.quarkslab.com/bsim-explained-once-and-for-all.html)). BSim queries also take a significance threshold, which grows with how much a match says (bigger functions) [T].
- WARP gives "a GUID for each unique basic block", with "'variant' instructions" zeroed first, and derives a function GUID from them; matching is exact; similarity scoring is future work; WARP's premise is that "we do not need to hand hold the creation of signatures or prematurely prune problematic signatures, which is a big problem with SigKit and other similar tools" [R] ([Binary Ninja blog](https://binary.ninja/2025/08/22/warp.html)). When a GUID returns several functions, constraints (a GUID plus an optional offset, for example an adjacent function) pick the match; enforcing constraints is the user's choice; partial matching is not planned [S] ([WARP docs](https://docs.binary.ninja/guide/warp.html), [vector35/warp](https://github.com/vector35/warp)). Growing the dataset caused "collisions on functions which previously had none", and those matches were dropped [R] (blog).
- Binary Ninja's older SigKit annotated `operator new()` in place of `operator delete()` with little feedback about why [R] (WARP blog); it is being retired after 5.1 [R] ([Binary Ninja docs](https://docs.binary.ninja/dev/annotation.html)).
- BinDiff matches functions between two binaries by an ordered list of matching steps, the most reliable first (hashes, names, call-graph position), weaker structural ones later, and propagates matches to neighbours; each match carries a similarity and a confidence [T].

### Inferences

- Three rules transfer directly to mentalc: normalize before matching (mask what varies: names, literals, the order of independent reads); separate near-identical candidates by context (what the fragment calls, what store it touches); and when context cannot separate them, report the collision instead of picking one (FLIRT refuses, WARP drops, mentalc would raise a gap).
- FLIRT's "false positive worse than false negative" is mentalc's "fail rather than degrade" applied to naming. A matched pattern becomes a fact the reader relies on; a missed one leaves the lower level visible, which is safe.
- Fuzzy similarity (BSim, BinDiff) needs a threshold and a person; its output is a ranked suggestion, not a fact.

### Gaps

- I found no published precision/recall for FLIRT, Lumina, Function ID or WARP; the vendors give anecdotes only.
- The exact WARP constraint semantics and Ghidra FID scoring options were not read in primary docs.

## Plan and cliché recognition: Programmer's Apprentice, GRASPR, Kozaczynski & Ning, DECODE, Biggerstaff; why they stalled, what scaled

### Takeaway

This is the closest prior art to mentalc's idea: a library of clichés, matched against a flow-graph form of the program, yielding a hierarchical description whose nodes are named plans. It stalled on hand-built libraries, NP-hard matching, implementation variation and delocalized plans, and was demonstrated only on small programs. What survived is the indexed, constraint-checked, domain-specific form, and the observation that a partial match is useful.

### Cited Findings

- GRASPR recognizes "cliches" (stereotypical algorithms and data structures) and builds "hierarchical design descriptions"; it "draws its power from graph-based representations and graph grammar parsing" and "is able to deal with a broad range of program variations" [R] ([GRASPR page, Georgia Tech](https://sites.cc.gatech.edu/reverse/graspr.html)).
- Programs are represented as attributed dataflow graphs and the cliché library as an attributed graph grammar, building on Brotsky's flow-graph parser; a plan may contain cycles while a flow graph may not, and converting between them is one of the obstacles the work addresses [S] ([Wills, AITR-1358, 1992](https://ftp.mirrorservice.org/sites/www.bitsavers.org/pdf/mit/ai/aim/AITR-1358.pdf); "Automated program recognition: a feasibility demonstration", Artificial Intelligence, 1990; "Using attributed flow graph parsing to recognize clichés in programs", LNCS, 1996).
- GRASPR used a chart parser, so all partial parses are kept; Wills argued that partial recognition (recognizing some clichés in a program that is not fully understood) is the realistic goal; the test programs were small (tens to hundreds of lines) [T].
- The Programmer's Apprentice (Rich & Waters, *The Programmer's Apprentice*, ACM Press/Addison-Wesley, 1990) defined the Plan Calculus: a plan is a graph of data and control flow between typed parts, and an overlay maps one plan onto another (an implementation onto a specification), which is the formal form of "this lower graph is that upper step" [T].
- Kozaczynski, Ning & Engberts ("Program concept recognition and transformation", IEEE TSE 18(12), 1992) matched a hierarchy of abstract concepts over the AST with control- and data-flow constraints, bottom-up, so higher concepts are recognized from lower ones [T].
- Quilici's DECODE ("A memory-based approach to recognizing programming plans", CACM 1994) indexes each plan by a distinctive element; finding the element proposes the plan and its constraints are then checked; Woods & Yang (mid-1990s) showed that plan matching formulated as constraint satisfaction is NP-hard, and Quilici & Woods measured CSP algorithms for it [T].
- Biggerstaff ("The concept assignment problem in program understanding", ICSE 1993; with Mitbander & Webster, CACM 1994) argued that domain concepts ("reserve an airline seat") cannot be recognized by parsing alone and need informal evidence (names, comments, clustering) with plausible reasoning; tool DESIRE [T].

### Inferences

- Why it stalled, compressed: (1) the library was written by researchers and stayed small; (2) matching subgraphs of a whole program is exponential in the worst case; (3) one plan has many implementations, and the parts of one plan are spread across a program (delocalized); (4) nobody measured recall on real code at scale.
- Mentalc removes much of (2) and (3) before matching starts. Behaviors are small (one per provided port), packs have already turned library calls into a few effect kinds, so `session.query(...).filter(...).first()` and `cur.execute("SELECT …")` are both a `read` of one store. The variation GRASPR fought is the variation packs normalize.
- (1) remains: the pattern library is the cost. The distributed-systems catalogs below are the source; the number of patterns needed for job-queue and web code is likely tens, not hundreds.
- Biggerstaff's split is mentalc's split: structure by mechanical matching, the domain name ("claim the report") by the LLM from names and comments.

### Gaps

- I could not open Wills' thesis this session, so the parser's complexity bounds and her measured program sizes are from memory.
- No study I found measured cliché recognition precision/recall on real systems.

## Design-pattern detection in source code: DeMIMA, Tsantalis, SPQR, Pinot, ML; precision/recall; partial patterns, anti-patterns and decay

### Takeaway

Detectors encode a pattern as roles plus relations (structural) and sometimes behavior (call and data flow). Exact matchers miss variants; scoring and constraint relaxation find variants and give a natural ranking and a natural near-miss report. Reported accuracy is high on each author's own benchmark, but tools disagree with each other, and the field still lacks a shared benchmark.

### Cited Findings

- Tsantalis, Chatzigeorgiou, Stephanides & Halkidis, "Design Pattern Detection Using Similarity Scoring" (IEEE TSE 32(11), 2006): systems and patterns as graphs, similarity scores between vertices, restricted to inheritance hierarchies to keep graphs small; evaluated on three open-source projects [S] ([abstract](https://www.academia.edu/9427174/Design_Pattern_Detection_Using_Similarity_Scoring)). Secondary pages report precision near 100% with few false negatives on JHotDraw, JRefactory and JUnit; I could not verify the figures against the paper [S]. A 2015 paper says such approaches "miss to detect patterns variants" [S] ([SCITEPRESS 2015](https://www.scitepress.org/Papers/2015/55205/55205.pdf)).
- Guéhéneuc & Albin-Amiot (2001) and Guéhéneuc & Antoniol, "DeMIMA: a multilayered approach for design pattern identification" (IEEE TSE, 2008): binary class relationships (association, aggregation, composition) recovered by static and dynamic analysis, then explanation-based constraint programming; when the full pattern has no solution, the solver relaxes constraints and reports "distorted" micro-architectures, each with the constraints it violates, which was used to propose corrections [T]. The P-MARt repository collects pattern instances as a benchmark [T].
- SPQR (Smith & Stotts, ASE 2003): elemental design patterns (small relations like "delegate", "redirect") are found in code, and GoF patterns are inferred from them by rules in a calculus with a theorem prover, so large patterns are compositions of small ones [T].
- Pinot (Shi & Olsson, ASE 2006) sorted GoF patterns into those decided by structure and those decided by behavior, and checks behavior with flow analysis (for example that a Singleton's instance is created lazily, once) [T].
- DECOR (Moha, Guéhéneuc et al., IEEE TSE 2010) specifies smells and anti-patterns as "rule cards" in a DSL and generates detectors from them [T].
- Izurieta & Bieman (ESEM 2007) named pattern decay: "grime" (unrelated code accumulating in a pattern's classes) and "rot" (the pattern's structure breaking) [T].
- Yarahmadi & Hasheminejad's review (Artificial Intelligence Review, 2020): learning-based detectors show high precision and recall "but may not distinguish some similar patterns", metric-based ones low precision and recall; it recommends combining graph-based and learning-based methods [S] ([Springer](https://link.springer.com/article/10.1007/s10462-020-09834-5)).
- Moreira, Fernandes & Figueiredo (PLoP 2022; UFMG thesis 2023) found 42 tools, of which only ten are available online, and "a low agreement among the detection results obtained by different tools" [S] ([PLoP 2022 preprint](https://homepages.dcc.ufmg.br/~figueiredo/publications/plop2022preprint.pdf)).
- An SEI experiment with an ML classifier had promising F1 but "is not accurate enough to put into production" [S] ([SEI blog](https://www.sei.cmu.edu/blog/using-machine-learning-to-detect-design-patterns)). A 2023 review names a random forest and PatternScout (SPARQL queries) as the best performers [S].

### Inferences

- The low agreement between tools comes largely from different definitions of the same pattern. A pattern library for mentalc must define each pattern precisely, as data, so "claim a job" means one thing; the catalog prose is not enough.
- DeMIMA's relaxation is the mechanism mentalc's near-miss idea needs: match the pattern with every constraint; on failure, find the smallest set of constraints whose removal gives a match; the removed constraints are the finding ("claim with no conditional update").
- SPQR's composition and Pinot's behavior checks fit a ladder: small patterns at the move level compose into larger ones at the activation and goal levels, and behavior (what is read before what is written) matters more than structure in mentalc's domain.

### Gaps

- Precision/recall figures for DeMIMA, SPQR and Pinot were not read this session.
- No study measured detectors of distributed or persistence patterns (as opposed to GoF patterns).

## Bug-pattern detectors as near-miss reporters, and detectors of check-then-act, lost-update and idempotency bugs in database-backed code

### Takeaway

The most useful near-miss reporters report "a pattern the code intends but does not complete", and they gate the report on evidence of intent: locks used elsewhere (RacerD), a field locked most of the time (FindBugs), an application-level validation (Feral), a read-then-write in one API call (ACIDRain). The database-backed-application studies show that these bugs are common, mostly caused by application code rather than isolation level, and are confirmed by building a witness.

### Cited Findings

- FindBugs (Hovemeyer & Pugh, "Finding bugs is easy", 2004) is a set of hand-written detectors; its inconsistent-synchronization detector reports a field that is accessed under a lock most of the time but not always, using the locked share as evidence of intent; each report has a confidence and SpotBugs adds a rank from 1 to 20 [T].
- Semgrep expresses "X without Y" directly: `pattern` with `pattern-not` or `pattern-not-inside`; CodeQL queries declare `@precision` (low to very-high) and Mental's packs already use CodeQL's models-as-data form [T].
- RacerD (Blackshear et al., OOPSLA 2018) is compositional and summary-based; it reports races in classes marked thread-safe or that use locks, treating the code's own locking as the declaration that the class is meant to be safe; it was deployed at Facebook [T].
- Feral concurrency control (Bailis et al., SIGMOD 2015) analyzed 67 Rails applications; uniqueness and association validations done in the application are not safe under concurrency (uniqueness is not I-confluent), while most built-in validations are; a test application produced duplicates despite a uniqueness validation [S] ([paper PDF](https://amplab.cs.berkeley.edu/wp-content/uploads/2015/06/feral-sigmod2015.pdf), [The Morning Paper](https://blog.acolyer.org/2015/09/04/feral-concurrency-control-an-empirical-investigation-of-modern-application-integrity/)). The Rails docs say that uniqueness checks at the application level "are inherently prone to race conditions" and recommend a unique index [S].
- ACIDRain's 2AD records SQL traces of normal use, generalizes them into an abstract history with operation, transaction and API-call nodes, and reports cycles: "cycles of API nodes in the abstract history correspond to potentially anomalous behavior in API calls"; it separates level-based anomalies (isolation level too weak) from scope-based ones (transaction boundaries wrong in the code); on 12 e-commerce applications it gave a median of 37 witnesses per application, analysis under 10 seconds per application after restricting to the relevant tables, about two hours to trigger and verify each attack, 22 vulnerabilities (9 inventory, 8 voucher, 5 cart), 5 level-based and 17 scope-based [R, secondary] ([The Morning Paper on ACIDRain](https://blog.acolyer.org/2017/08/07/acidrain-concurrency-related-attacks-on-database-backed-web-applications/)).
- Tang et al., "Ad Hoc Transactions in Web Applications: The Good, the Bad, and the Ugly" (SIGMOD 2022): 91 ad hoc transactions (database operations coordinated by application code) in 8 open-source web applications, up to 16 per application, 71 on critical APIs such as check-out and payment; 53 have concurrency bugs and 28 have severe consequences such as overcharging; under high contention they can perform well because they use application semantics [S] ([CACM research highlight](https://cacm.acm.org/research-highlights/many-faces-of-ad-hoc-transactions), [researchr](https://researchr.org/publication/TangWZYZG022)). A journal version is "Ad Hoc Transactions through the Looking Glass", ACM TODS 49(1), 2024 [S].
- The ad hoc transactions are built from a few primitives: locks (in the database row, in Redis, in memory) and optimistic validation (a version column or a conditional `UPDATE … WHERE`) [T]; I could not open the paper to check the exact taxonomy and counts.
- IsoDiff (Gan et al., VLDB 2020) finds anomalies caused by weak isolation and suggests the smallest set of transactions to run at a stronger level [T].

### Inferences

- 17 of 22 ACIDRain attacks were scope-based, that is, a code pattern (read, decide, write outside one transaction), not a database setting. That is exactly what mentalc's move verdicts already classify, which supports matching claim, check-then-act and idempotency patterns at the move level.
- ACIDRain's two hours per witness and Tang's manual study show that the expensive part is confirmation. Mentalc's witness search in mentalvm does that part mechanically, so a near-miss can be labelled a prediction until a witness exists, as findings are today.
- The intent gate decides precision. "Every read followed by a write is a broken claim" floods; "a read that tests a status column and then writes that column, in a port that can run twice at once" is what RacerD and FindBugs teach.
- Patterns that matter for job-queue and web code, from these studies: claim (test-and-set on a status), uniqueness check then insert, counter or stock decrement, voucher or credit spend once, idempotency key, enqueue after commit, outbox.

### Gaps

- I found no static detector specific to Celery, RQ or other task-queue code; the database-backed studies are about web request handlers.
- I could not read the Tang paper's full text (its host did not resolve), so primitive counts are not given.

## Distributed-systems pattern catalogs as a pattern library

### Takeaway

The catalogs give names, intent and forces in prose, with diagrams and sample code, not machine-checkable definitions. They are the right vocabulary for mentalc's upper levels, but each pattern must be redefined as a graph pattern over effects with constraints before it can be matched.

### Cited Findings

- Hohpe & Woolf, *Enterprise Integration Patterns* (2003, [enterpriseintegrationpatterns.com](https://www.enterpriseintegrationpatterns.com/)): 65 messaging patterns, among them Idempotent Receiver, Competing Consumers, Guaranteed Delivery, Dead Letter Channel, Transactional Client and Message Expiration [T].
- Richardson's [microservices.io](https://microservices.io/patterns/) catalog: Transactional Outbox (write the message to an outbox table in the same transaction as the state change, then relay it), Polling Publisher, Transaction Log Tailing, Saga (a sequence of local transactions with compensations), Idempotent Consumer (record processed message IDs) [T].
- Joshi, *Patterns of Distributed Systems* (Addison-Wesley 2023; [martinfowler.com](https://martinfowler.com/articles/patterns-of-distributed-systems/)): write-ahead log, lease, generation clock, high-water mark, idempotent receiver, versioned value, quorum, heartbeat and others, each with a problem, solution and code from real systems [T].
- Nygard, *Release It!* (2007, 2nd ed. 2018): timeout, circuit breaker, bulkhead, retry with backoff as stability patterns [T].

### Inferences

- Each catalog pattern decomposes into effect-level constraints mentalc can check. Transactional outbox: a `write` to the state store and a `write` to an outbox table under one `commit`, and a separate behavior that reads the outbox and `send`s. Enqueue after commit: the `send` is ordered after the `commit` on every path. Idempotent consumer: a `read` or conditional `write` keyed by message ID guards the side effect. Claim a job: a conditional `write` of the status (guarded verdict) before the work.
- Each also has a named near-miss with a known consequence: `send` before `commit` (message for a row that may never exist), outbox write outside the transaction (lost or phantom message), consumer without a key check under at-least-once delivery (double effect), claim with an unconditional write (duplicate claim). These are the patterns worth writing first.
- Catalog patterns mix levels (an outbox spans two behaviors), so some patterns sit at the activation or goal level and need lower patterns already matched, like SPQR's composition.

### Gaps

- No catalog provides formal definitions; I found no prior work that formalized EIP or microservice patterns as graph patterns over code for detection.

## Fit for mentalc

Mentalc's constraints: pure and deterministic, every record anchored, fail rather than degrade, no LLM for facts, LLM only for names and summaries. Ranked by fit for lifting the effect graph to named upper steps and reporting near-misses.

1. **Decompiler-style rewrite rules over the effect graph, with pack-supplied idioms.** Best fit. Exact patterns with dataflow preconditions, applied bottom-up to a fixpoint, are deterministic, each match is anchored to the nodes it consumed, and the lifted node is a record the interpreter can run. Packs already play the role of per-compiler idiom tables. Weakness: on its own it reports nothing when a pattern almost matches.
2. **Constraint matching with relaxation (DeMIMA, Quilici/Woods) as the near-miss reporter.** Strong fit. A pattern is roles plus constraints; a full match lifts; a match that succeeds only after removing a small, named set of constraints is a finding that names the missing constraint ("claim: no conditional update"). Ranking is deterministic: fewer relaxed constraints first, then by a fixed constraint weight. Matching stays tractable because the search is inside one behavior and its callees, and because packs normalize implementation variation. Must be gated on intent (below) or it floods.
3. **Intent-gated bug patterns (RacerD, FindBugs inconsistent synchronization, Semgrep "X without Y", ACIDRain scope-based).** Strong fit, and already the shape of mentalc's atomicity check. The lesson to adopt: report a near-miss only when the code shows the pattern's intent (a status test before a write, a key column with a uniqueness check, an outbox table that exists), and label it a prediction until mentalvm produces a witness.
4. **Flow-graph grammar parsing (GRASPR, Plan Calculus overlays).** Good fit in concept, cost in practice. A parse tree is exactly a step map (each nonterminal an upper step, each production a closing rule with a name), and partial parses give near-misses. But a general graph-grammar parser is the most expensive option to build and has not been shown at scale. Better used as the design model for how patterns compose across levels (move patterns into activation patterns into goal patterns, as SPQR composes elemental patterns), implemented with options 1 and 2.
5. **Exact signatures for library code (FLIRT, Function ID, WARP).** Good fit, narrow scope: it is how packs could recognize vendored or copied library code inside a repository. Adopt FLIRT's policy and WARP's constraints: normalize, separate collisions by context, and when context cannot separate them, emit a gap rather than pick.
6. **Similarity scoring (Tsantalis, BSim, BinDiff).** Weak fit. A score in [0, 1] ranks candidates but is not a fact; thresholds are arbitrary and results drift as the library grows (WARP saw matches disappear when the dataset grew). Usable only to suggest a pattern to a person or to order ambiguous candidates, never to lift a node.
7. **Informal concept assignment (Biggerstaff).** Fits only the LLM's existing job: naming a lifted step and writing its "so that" sentence from identifiers and comments. It must not decide that a fragment is a pattern.
8. **ML or LLM pattern classifiers.** Do not fit. They are not deterministic, not anchored to a reason, confuse similar patterns, and tools disagree with each other even on GoF patterns. They break the rule that the LLM never supplies a fact.

**Ambiguity.** When two patterns match the same nodes, the tools agree on an order: the more specific match wins (FID's specific hash, FLIRT's referenced names, WARP's constraints, BinDiff's reliability order), and when nothing separates them the match is refused (FLIRT, WARP). For mentalc: a pattern that strictly contains another's constraints wins; otherwise the build records an ambiguity gap that lists both candidates, rather than guessing.

**Measuring precision and recall.** The fields offer weak models: own-benchmark numbers (pattern detection), anecdotes (library recognition), manual confirmation of each witness (ACIDRain, about two hours each). Mentalc can do better because it can confirm mechanically. Precision: the share of near-miss findings for which mentalvm finds a witness. Recall: seed known bugs (report-jobs' four) and mutate correct code (drop the `WHERE status = …`, move the `send` before the `commit`, remove the key check) and require each mutant to produce the expected near-miss; and require the unmutated pattern to lift with no finding.

Not checked this session: the Ghidra, Hex-Rays and RetDec idiom sources, Wills' thesis, the DeMIMA, SPQR and Pinot papers, and the full Tang and Bailis papers; claims from them are tagged [T] or [S]. The ranking is a judgment from these sources, not a measurement on mentalc.
