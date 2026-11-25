# Neural and LLM decompilers, and how decompiled output is checked

Scope: neural and LLM decompilers (end-to-end, refinement, naming and typing), hybrid designs, their failure modes, the ways the field checks a decompilation (re-executability, recompilation, coverage equivalence, symbolic equivalence, metamorphic testing of decompilers, translation validation), readability evaluation, and what transfers to checking a pattern-lifted upper level of a Mental model against the level below. Sweep done on 2026-11-25.

Evidence tags on every claim:

- **[V]** read in the primary source this session (the arXiv abstract or HTML page, the project README, the vendor or author page). Most were read through a page fetch that returns an extract, not the full PDF; where only the abstract was read, the claim says so.
- **[S]** seen only in a search-result summary of the linked page; the page itself was not opened. Spot-check before quoting.
- **[B]** training knowledge, not re-fetched this session.

Translation validation (Pnueli, Necula, Alive2) and CompCert-style simulation proofs are covered in depth in the sibling note [compilers and computed abstractions](../../abstraction-levels/notes/compilers_and_computed_abstractions.md); this note only places them relative to decompilers.

## At a glance

| System | Year, venue | What it does | Headline number |
|---|---|---|---|
| LLM4Decompile-End | 2024, arXiv 2403.05286 (EMNLP 2024 [B]) | Assembly to C, end to end, 1.3B to 33B | 6.7B: 45.4% re-executable on HumanEval-Decompile, GPT-4o 16.0% [V] |
| LLM4Decompile-Ref | 2024, same | Rewrites Ghidra's pseudo-C into compilable C | 22B: 64.2%; v2 9B: 64.9% [V] |
| Nova / Nova+ | ICLR 2025 | Assembly LM with hierarchical attention and contrastive training | +14.8 to 21.6 points Pass@1/@10 over prior work [S]; 37.5% on ExeBench as measured by Idioms [V] |
| DecGPT | 2023, arXiv 2310.06530 | LLM repair loop to make Ghidra output recompile | Recompilation 45% to 75% on 300 CodeContest cases [S] |
| DecLLM | ISSTA 2025 | Repair loop with recompiler and runtime oracles | About 70% of non-recompilable outputs made recompilable [S] |
| DeGPT | NDSS 2024 | Referee, advisor, operator; the operator rejects edits that change semantics | 24.4% less "cognitive burden" on Ghidra output [S] |
| Idioms | 2025, arXiv 2502.04536 | Fine-tuned LLM on Hex-Rays or Ghidra output; predicts code and user-defined type definitions | 54.4% ExeBench tests passed vs LLM4Decompile 46.3%, Nova 37.5% [V] |
| SK2Decompile | 2025, arXiv 2509.22114 | Two phases: "skeleton" IR with placeholder names, then "skin" naming | +21.6% re-executability over GPT-5-mini; +29.4% R2I over Idioms [V, abstract] |
| AutoDecompiler | 2026, arXiv 2606.16162 | RL-trained multi-turn refinement on compiler, execution and test feedback | No numbers in the abstract [V, abstract] |
| ReSym | CCS 2024 | LLM proposals for variable names, types and structs, aggregated and checked by program analysis | 56.4% names, 64.6% types [S] |
| DIRE / DIRTY | ASE 2019 [B] / USENIX Sec 2022 | Neural names (DIRE), names and types (DIRTY) over Hex-Rays output | DIRTY 66.4% names; 36.9% on functions not seen in training [S] |
| VarBERT | IEEE S&P 2024 | BERT transfer learning for variable names | 54.4% (IDA), 54.5% (Ghidra) at O2; 50.7% vs DIRTY 35.9% on a shared subset [S] |
| GenNm | NDSS 2025 | Generative naming with caller/callee context and voting | Unseen names 17.3% to 22.8% [S] |
| SLaDe | CGO 2024 | 200M-parameter seq2seq plus type inference, several ISAs | Up to 6× Ghidra and 4× ChatGPT accuracy [S] |
| BTC | 2022 (NDSS BAR [B]) | Retargetable neural decompiler by machine translation | Baseline in SLaDe [S] |
| Binary Ninja Sidekick | 2024 onward, commercial | Chat agent over a query language (BNQL), function docs, component grouping, structure recovery | No accuracy figures published [S] |
| IDA (Hex-Rays) | 2026, commercial | "Assist" add-on agent, IDA MCP; third-party Gepetto [B], aiDAPal, IDAssist, ida-pro-mcp | No accuracy figures [V] |
| Ghidra plugins | community | GhidraMCP, GhidrAssist and similar: LLM drives the deterministic decompiler and renames/comments [B] | No accuracy figures |
| HumanEval-Decompile | 2024 | 164 HumanEval tasks ported to C with assertions; metric: re-executability | Defined in LLM4Decompile [V] |
| ExeBench / ExeBench-FHD | 2022 [B] / 2026, arXiv 2610.28212 | Executable C functions with I/O tests; FHD fixes nine benchmark defects | 38.9% of functions carry a defect flag [V, abstract] |
| Decompile-Bench | 2025, arXiv 2505.12668 | 2M binary-source pairs; eval on HumanEval, MBPP, post-2025 GitHub | +20% re-executability from training on it [V, abstract] |
| DecompileBench | 2025, arXiv 2505.11340 | 23,400 OSS-Fuzz functions; branch-coverage equivalence; LLM-judge readability | Hex-Rays CER 0.418 beats every LLM (best GPT-4o 0.346) [V] |
| Liu & Wang (EMI) | ISSTA 2020 | Equivalence-modulo-inputs testing of C decompilers | 1,423 failing inputs, 13 bugs in two open-source decompilers [S] |
| D-Helix | USENIX Sec 2024 | Recompile, symbolic differencing, heuristic toggling to root-cause | 17 new bugs in Ghidra and angr [S] |
| R2I | FSE 2024 | Relative readability index from 31 features | Adopted by Decompile-Bench and SK2Decompile [S] |
| Fidelity taxonomy (Dramko et al.) | USENIX Sec 2024 | Open-coded taxonomy of decompiler defects | 15 top-level, 52 total categories [S] |
| Human study of annotations | DSN 2025 | 40 people, decompiled code with and without DIRTY-style names and types | No significant gain in speed or accuracy [V for result, S for 40] |
| Memorization floor | 2026, arXiv 2609.17236 | Pre-registered control for what LLM refinement recovers from input vs prior | Naming gain unchanged when dataflow is destroyed (+0.001) [V, abstract] |

## What the systems do and how well

### Takeaway

The field has split into three task shapes: end-to-end assembly-to-source (LLM4Decompile-End, Nova, SLaDe, BTC), refinement of a deterministic decompiler's output (LLM4Decompile-Ref, DecGPT, DecLLM, DeGPT, Idioms, SK2Decompile), and recovery of names and types only, leaving the code as the decompiler wrote it (DIRE, DIRTY, VarBERT, ReSym, GenNm). On toy single-function benchmarks the best refiners pass about two thirds of test cases; on real code measured strictly, no LLM yet beats Hex-Rays on behavioural correctness, and name recovery sits at 20–55% exact match, falling sharply on code not seen in training.

### Cited Findings

- LLM4Decompile-End-6.7B is re-executable on 45.37% of HumanEval-Decompile on average (68.05% at O0, 37.20% at O3); GPT-4o end to end gets 16.01%, Ghidra 20.12% [V] — [LLM4Decompile paper, HTML v3](https://arxiv.org/html/2403.05286v3)
- LLM4Decompile-Ref, which rewrites Ghidra output, reaches 52.74% (6.7B) and 64.18% (22B, a CodeStral base); Ghidra followed by GPT-4o refinement reaches 35.22% [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- The larger 33B End model does worse (31.54%) than 6.7B because it was trained on about 10% of the data [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- The README lists llm4decompile-9b-v2 at 64.94% re-executability "on the Decompile benchmark" [V] — [LLM4Decompile README](https://github.com/albertan017/LLM4Decompile)
- Re-executability is defined as compiling the decompiled function and passing all of the benchmark's assertions [V] — [LLM4Decompile README](https://github.com/albertan017/LLM4Decompile)
- HumanEval-Decompile starts from HumanEval's 164 problems, translated to C and filtered to those that compile with GCC and pass their assertions; the scope is C, GCC, x86-64 Linux, single functions [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- On ExeBench only 2,621 of 5,000 test samples compiled under a standard setup [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- Idioms passes 54.4% of ExeBench tests vs LLM4Decompile 46.3% and Nova 37.5%, and is "at least 95% better" on its Realtype benchmark [V] — [Idioms abstract](https://arxiv.org/abs/2502.04536)
- Idioms recovers structurally correct user-defined types 15.1% of the time on Realtype (nominal 5.7%) with its best model, CodeGemma-7b; baselines score 0% because they emit no type definitions; adding neighbouring call-graph context raises structural accuracy from 9.2% to 15.1% [V] — [Idioms HTML v2](https://arxiv.org/html/2502.04536v2)
- Nova uses three attention levels (intra-instruction, preceding-instruction, inter-instruction) and reports 14.84–21.58 absolute points higher Pass@1/Pass@10 on decompilation; earlier arXiv versions claimed "up to 146.54%", a relative figure [S] — [Nova ICLR 2025](https://proceedings.iclr.cc/paper_files/paper/2025/hash/ef283d62b4bce30854a8d4827f331229-Abstract-Conference.html)
- SK2Decompile splits decompilation into a structure-recovery model that emits an IR with all identifiers replaced by placeholders (RL reward: compiler-valid syntax and semantics) and a naming model (RL reward: semantic similarity to the reference) [V, abstract] — [SK2Decompile](https://arxiv.org/abs/2509.22114)
- AutoDecompiler (June 2026) trains an LLM with RL to revise over several turns on "stage-aware diagnostic feedback" from compiler errors, execution failures and failed tests; rewards cover validity, recompilability, execution consistency and "semantic fidelity" [V, abstract] — [AutoDecompiler](https://arxiv.org/abs/2606.16162)
- DecGPT raised recompilation of Ghidra output from 45% to 75% on 300 CodeContest cases; DecLLM (ISSTA 2025) uses recompilation and runtime behaviour as oracles in a repair loop and makes about 70 of 100 non-recompilable outputs recompilable, against 8% for rule-based recompilation [S] — [DecLLM PDF](https://daoyuan14.github.io/papers/ISSTA25_DecLLM.pdf), [DecGPT](https://arxiv.org/pdf/2310.06530)
- A slide deck (not the paper) reports one-shot LLM repair at 17.3% vs 74% for the iterative test-fed loop [S, unverified against the paper] — [DecLLM slides](https://docs.google.com/presentation/d/1KnDW37jokxkvv_DibcxqLujQuOMYXwY_/htmlpresent)
- DeGPT splits the LLM into referee (plans), advisor (proposes edits) and operator (checks the edit does not change the function's semantics); it reports 24.4% less cognitive burden and 62.9% of comments carrying "practical semantics" [S] — [DeGPT, NDSS 2024](https://www.ndss-symposium.org/ndss-paper/degpt-optimizing-decompiler-output-with-llm)
- ReSym combines LLM queries with program analysis that cross-checks several LLM answers, recovering local variables and user-defined structures; 56.4% name and 64.6% type accuracy; CCS 2024 distinguished paper [S] — [ReSym PDF](https://www.cs.purdue.edu/homes/lintan/publications/resym-ccs24.pdf)
- DIRTY matches developer names 66.4% of the time on its own dataset, falling to 36.9% when functions present in training are removed (DIRE: 31.8%) [S] — [DIRTY, USENIX Sec 2022](https://www.usenix.org/conference/usenixsecurity22/presentation/chen-qibin)
- VarBERT predicts 54.43% (IDA) and 54.49% (Ghidra) of variable names at O2; on a shared subset 50.70% vs DIRE 38.00% and DIRTY 35.94% [S] — [VarBERT, S&P 2024](https://adamdoupe.com/publications/varbert-oakland2024.md)
- GenNm fine-tunes CodeGemma-2B and CodeLlama-7B/34B, passes names across callers and callees iteratively, and votes; in the hardest setting (names never in training) accuracy rises from 17.3% to 22.8%; an earlier arXiv version gave 48.1% to 57.9%, so numbers moved between versions [S] — [GenNm, NDSS 2025](https://ndss-symposium.org/ndss-paper/unleashing-the-power-of-generative-model-in-recovering-variable-names-from-stripped-binary)
- SLaDe is a 200M-parameter Transformer with type inference, evaluated on 4,000 functions across ISAs; up to 6× more accurate than Ghidra and 4× than ChatGPT; BTC was the only public neural baseline at the time [S] — [SLaDe](https://arxiv.org/abs/2305.12520)
- Binary Ninja Sidekick offers a chat assistant that navigates the binary through BNQL, a notebook of findings, automatic function documentation, script generation and hierarchical grouping of functions ("component creation") with structure recovery; Sidekick 26.1 works across binaries in a project [S] — [Sidekick docs](https://docs.sidekick.binary.ninja/guide/guide), [Sidekick 26.1 blog](https://sidekick.binary.ninja/blog/sidekick-26-1-a-proper-home-for-sidekick)
- Hex-Rays lists an "Assist" add-on ("an agent that reverse engineers alongside you") and IDA MCP; its July 2026 guest post argues interfaces should show what an agent believes, what evidence it used, and where a human should step in, but gives no accuracy figures or validation method [V] — [Hex-Rays blog, 2026-07-30](https://hex-rays.com/blog/llms-have-reshaped-how-we-think-about-decompilation-and-collaboration)
- Hex-Rays' 2026 priorities mention an "IDA Semantic Engine" beta; third-party IDA plugins include IDAssist (RAG, knowledge graph), AiDA, Rikugan and ida-pro-mcp [S] — [Hex-Rays 2026 priorities](https://hex-rays.com/blog/2026-product-direction-priorities), [IDAssist](https://github.com/jtang613/idassist)
- In every commercial or plugin integration found, the LLM drives or annotates a deterministic decompiler (rename, retype, comment, explain); none replaces the decompiler's structuring [B, plus the pages above]

### Inferences

- The numbers are not comparable across papers: HumanEval-Decompile, ExeBench, Realtype and Decompile-Bench-Eval differ in size, realism and test strength, and several papers changed their headline figures between versions (Nova, GenNm).
- Single-function, O0, x86, GCC is the regime where the impressive figures live; every paper reports a sharp drop at O1 and above.

### Gaps

- ReSym's and the DSN 2025 study's full PDFs could not be text-extracted in this session (no PDF tools); their numbers are from search summaries.
- No vendor (Sidekick, Hex-Rays) publishes an accuracy or hallucination rate for its LLM features.
- DecGPT as a named system: the arXiv 2310.06530 paper ("Refining Decompiled C Code with Large Language Models") is what the literature calls DecGPT; the name was not confirmed on the page itself.

## Hybrid designs: deterministic structure, LLM names

### Takeaway

The evidence favours keeping a deterministic decompiler's structure and narrowing what the LLM may change. Refining Ghidra output beats raw assembly on toy benchmarks, but when the LLM is allowed to rewrite the code of a strong decompiler (Hex-Rays) on real programs, behavioural correctness falls by up to half. The cleanest designs separate structure from naming outright (SK2Decompile's skeleton and skin; DIRTY, VarBERT, ReSym and GenNm leave the code untouched). No paper was found that measures "rename only" against "full rewrite" on the same inputs with the same correctness metric.

### Cited Findings

- GPT-4o end to end is re-executable on 16.01% of HumanEval-Decompile; given Ghidra's output to refine it reaches 35.22%; the fine-tuned Ref model beats the fine-tuned End model of the same size (52.74% vs 45.37% at 6.7B) [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- Idioms takes decompiler output rather than assembly as input, citing LLM4Decompile's finding that this more often yields correct output, and reports anecdotally that Hex-Rays input is better than Ghidra's [V] — [Idioms HTML v2](https://arxiv.org/html/2502.04536v2)
- On DecompileBench (real OSS-Fuzz functions), Hex-Rays has recompile success 0.583 and coverage-equivalence rate 0.418; GPT-4o 0.559 / 0.346, DeepSeek-V3 0.553 / 0.329, Qwen2.5-Coder-32B 0.542 / 0.296, LLM4Decompile 0.264 / 0.164; Ghidra 0.413 / 0.278 [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- LLM-enhanced Hex-Rays output had recompile success 0.2–45.3% lower and coverage equivalence 17.2–52.2% lower than Hex-Rays alone; yet the LLM-based "MLM" scored an Elo of 1581 for understandability against Hex-Rays' 1162 [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- DecompileBench's conclusion: reliability-critical work should use Hex-Rays or Ghidra; LLM tools suit quick comprehension such as malware triage [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- SK2Decompile replaces all identifiers with placeholders in phase one so that structure is learned and rewarded separately from naming [V, abstract] — [SK2Decompile](https://arxiv.org/abs/2509.22114)
- DeGPT lets the LLM propose structural simplifications but has a non-LLM operator reject any edit that changes semantics [S] — [DeGPT](https://www.ndss-symposium.org/ndss-paper/degpt-optimizing-decompiler-output-with-llm)
- ReSym uses the LLM for proposals and program analysis to aggregate them and reject inconsistent ones [S] — [ReSym](https://www.cs.purdue.edu/homes/lintan/publications/resym-ccs24.pdf)

### Inferences

- Two regimes: when the deterministic baseline is weak (Ghidra on HumanEval), letting the LLM rewrite helps; when it is strong (Hex-Rays on real code), the rewrite mostly destroys correctness while improving readability. The LLM is good at surface and bad at preserving behaviour it was asked to keep.
- The division "deterministic structure, LLM names" is the one the commercial tools ship and the one the strongest correctness results support. It matches Mental's split.

### Gaps

- No controlled comparison of rename-only vs full rewrite on identical inputs and an equivalence metric was found.

## Hallucination and failure modes, and how they are measured

### Takeaway

LLM decompilers fail plausibly: invented types and headers, dropped parameters, reordered operands, calls renamed so the code compiles, and names drawn from the model's prior rather than the input. They are measured by tests (complete but unsound), by recompile rate (gamed by workarounds), by coverage or dependency-graph equivalence, and, in the newest work, by ablations that remove the evidence and see whether the output changes. Readability stays high when the output is wrong, so it gives the reader no warning.

### Cited Findings

- LLM outputs on real code hallucinate types (an undefined `archive_t` in place of `__int64`), inject speculative headers and drop parameters; GPT-4o-mini nearly matches Hex-Rays on recompile rate (0.582 vs 0.583) partly by renaming undefined calls so they compile with a warning, and then fails at runtime [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- For LLM4Decompile-End-6.7B, about 64% of HumanEval-Decompile errors are code that compiles but fails assertions; on ExeBench about 50% of errors are undeclared functions and 28% improper use of structures; GPT-4o from assembly collapses user-defined types to `int` or pointers [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- The base model "could generate code that seemed correct but failed to retain the original program semantics" [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- Control-flow flattening and bogus control flow sharply reduce success for both Ghidra and LLM4Decompile [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- Idioms names its failure modes: repeated degenerate type definitions after sensible output; wrong data structure inferred when a function touches only part of a type (a hash table read as a linked list); neural output reordering or misusing operands, which breaks dependency equivalence [V] — [Idioms HTML v2](https://arxiv.org/html/2502.04536v2)
- Idioms calls ExeBench unit tests "complete but unsound", an optimistic proxy, and on Realtype uses static dependency-graph equivalence instead (strict: types must match too; relaxed: a bijection of field and function names); it notes the metric misses equivalent forms such as `x * 2` and `x << 1` [V] — [Idioms HTML v2](https://arxiv.org/html/2502.04536v2)
- DIRTY's name accuracy falls from 66.4% to 36.9% on functions not in training, a measure of memorization [S] — [DIRTY](https://www.usenix.org/conference/usenixsecurity22/presentation/chen-qibin)
- A pre-registered 2026 study refines a function, then refines it again from an input with identifiers destroyed; destroying dataflow changes the naming gain by +0.001 (95% CI −0.026 to +0.026), and the pattern replicates across two vendors' models; readability stays at ceiling throughout, so "a reader is given no signal"; the author also reports five instrument failures, including an equivalence checker never validated on the paper's inputs [V, abstract] — [Memorization floor](https://arxiv.org/abs/2609.17236)
- ExeBench has nine defects that distort scores (mixed int/float parameter order, unused parameters, constants outside the function, missing returns, placeholder types, functions removed by the optimizer, and others); 38.9% of functions carry at least one flag; fixing the inputs raised one model's re-execution by up to 26.5% on affected subsets [V, abstract] — [ExeBench-FHD](https://arxiv.org/abs/2610.28212)
- ReSym and GenNm reduce hallucinated names by querying several times and keeping answers that agree or that program analysis accepts (cross-checking, majority voting) [S] — [ReSym](https://www.cs.purdue.edu/homes/lintan/publications/resym-ccs24.pdf), [GenNm](https://ndss-symposium.org/ndss-paper/unleashing-the-power-of-generative-model-in-recovering-variable-names-from-stripped-binary)

### Inferences

- "Invented call", "dropped branch" and "dropped parameter" are each caught only by a check that compares behaviour or structure with the input; tests catch them only on the paths they exercise.
- Agreement between several LLM answers measures stability, not truth: the memorization-floor result shows a model can agree with itself on names that do not come from the input.

### Gaps

- No paper found reports a hallucination rate per kind (invented call, dropped branch) on a common benchmark; DecompileBench lists kinds qualitatively.

## How the field checks that a decompilation is right

### Takeaway

Five families, from weakest to strongest: recompilation (does it build), re-executability (does it pass the tests), differential or coverage equivalence (does it behave like the original on a corpus, branch for branch), metamorphic testing of the decompiler itself (EMI), and symbolic equivalence per function (D-Helix, the decompiler analogue of translation validation). Only the last two say anything beyond the tests, and only D-Helix also localizes the cause. Bisimulation is not used by name in the decompiler literature found.

### Cited Findings

- Recompilation success is the first gate in DecGPT, DecLLM and DecompileBench, and is gameable (renaming undefined calls) [V for DecompileBench, S for the others] — [DecompileBench](https://arxiv.org/html/2505.11340v1), [DecLLM](https://daoyuan14.github.io/papers/ISSTA25_DecLLM.pdf)
- Re-executability (compile, run the assertions) is the standard metric of LLM4Decompile, Decompile-Bench and SK2Decompile [V] — [LLM4Decompile README](https://github.com/albertan017/LLM4Decompile), [Decompile-Bench](https://arxiv.org/abs/2505.12668)
- Decompile-Bench-Eval adds compiled GitHub repositories released after 2025 to limit training-data leakage [V, abstract] — [Decompile-Bench](https://arxiv.org/abs/2505.12668)
- DecompileBench substitutes the decompiled function into the full program, runs the fuzzer's seed corpus, and requires matching per-branch execution counts and true/false ratios under SanitizerCoverage (coverage-equivalence rate) [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- Liu and Wang (ISSTA 2020) test C decompilers by equivalence modulo inputs: programs that behave the same on a given input are decompiled and recompiled, and checksums of global variables are compared; 1,423 failing inputs across four decompilers, 13 bugs confirmed by hand in two open-source ones; they conclude the field's view of decompiler correctness was too pessimistic [S] — [Liu & Wang, ISSTA 2020](https://monkbai.github.io/publication/2020-issta-decompiler-testing)
- BIN2WRONG later reported 48 new decompiler bugs, 30 confirmed [S] — [DecompileBench related work](https://arxiv.org/html/2505.11340v1)
- D-Helix recompiles each decompiled function, builds symbolic models of the original binary function and the recompiled one, compares them with an SMT solver at IR level ("SymDiff"), and a "Tuner" toggles decompiler heuristics one at a time to find which one causes a mismatch; 17 new bugs (11 Ghidra, 6 angr), root cause located automatically for 7; reported scale 93K functions (abstract) or 56K (slides), a discrepancy [S] — [D-Helix, USENIX Sec 2024](https://www.usenix.org/conference/usenixsecurity24/presentation/zou), [D-Helix code](https://www.github.com/purseclab/D-helix)
- The fidelity taxonomy relies on an alignment between source and decompiled code that "cannot be established with line numbers or naive heuristics" because decompilers split expressions, inline statements and drop code [S] — [Dramko et al., USENIX Sec 2024](https://www.usenix.org/conference/usenixsecurity24/presentation/dramko)
- Translation validation checks each run of an unverified translator rather than the translator (Pnueli, Siegel and Singerman, TACAS 1998; Necula, PLDI 2000); Alive2 (Lopes et al., PLDI 2021) checks refinement of LLVM function pairs with SMT, with loops unrolled to a bound [B; covered with sources in the sibling note] — [compilers and computed abstractions](../../abstraction-levels/notes/compilers_and_computed_abstractions.md)
- Classic structuring decompilers match control-flow regions against a fixed set of patterns (if-then-else, loops) and emit `goto` where no pattern matches; residual gotos are a standard quality measure (Phoenix, USENIX Sec 2013; DREAM "No More Gotos", NDSS 2015) [B]

### Inferences

- D-Helix is a translation validator for decompilers plus a cause-locator: it checks every output against its input and, on failure, toggles one rule at a time. That second half is what a failing upper-level check most needs.
- EMI tests the decompiler, not one output: it is a test-suite method, run by the decompiler's authors, not a per-build gate.
- Pattern structuring's fallback (a `goto` where no region pattern fits) is the decompiler precedent for "a near-miss of a pattern is a signal": in decompilers it marks unusual or obfuscated control flow.

### Gaps

- I found no decompiler paper that uses bisimulation or simulation relations by name; the closest are D-Helix's symbolic equivalence per function and CompCert's simulation diagrams (sibling note).
- No source quantified how often residual gotos point at real defects in the source rather than decompiler weakness.

## Readability evaluation

### Takeaway

Readability is measured by feature metrics (R2I), LLM judges calibrated on a few human labels, and human studies. The human evidence is sobering: names and types that people prefer did not make them faster or more accurate, and LLM output is most readable exactly where it may be wrong.

### Cited Findings

- R2I (FSE 2024) is a relative readability index for decompiled C built from 31 features drawn from decompiler improvements and readability research; it first repairs malformed decompiler output so it parses; Decompile-Bench and SK2Decompile adopt it [S] — [R2I, FSE 2024](https://2024.esec-fse.org/details/fse-2024-research-papers/123/R2I-A-Relative-Readability-Metric-for-Decompiled-Code)
- DecompileBench uses Qwen-2.5-Coder-32B as a pairwise judge on 12 criteria, turned into Elo scores; agreement with two human annotators on 30 samples was Cohen's κ = 0.778 [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- LLM4Decompile scores readability with GPT-4o (1–5): its End-6.7B output 3.54, Ghidra 2.57, GPT-4o 2.44; the authors call the scores partly subjective [V] — [LLM4Decompile paper](https://arxiv.org/html/2403.05286v3)
- The DSN 2025 human study (Lacomis, Yang, Grandel, Leach, Vasilescu, Le Goues) found DIRTY-style names and types did not significantly improve task completion speed or accuracy; participants still preferred annotated code [V] — [Schwartz blog on the DSN 2025 paper](https://edmcman.github.io/blog/2025-06-16--a-human-study-of-automatically-generated-decompiler-annotations-published-at-dsn-2025); 40 participants, students and professionals [S] — [DSN 2025 PDF](https://edmcman.github.io/papers/dsn25.pdf)
- LmPa (the earlier GenNm line) reports a user study in which 75% of its names were judged good vs 6% for DIRTY [S] — [LmPa](https://ar5iv.labs.arxiv.org/html/2306.02546)
- Decomperson (Burk et al., USENIX Sec 2022) observed humans decompiling by hand to recompilable source and used it to inform decompiler design [B]
- Readability stayed at ceiling in the memorization-floor study regardless of whether the input supported the names [V, abstract] — [Memorization floor](https://arxiv.org/abs/2609.17236)

### Inferences

- Preference and performance diverge; a readability score is not evidence that a name is true. Name-truth checks must be separate from readability scoring.

### Gaps

- Dramko et al. also wrote on readability; only the 2024 taxonomy and the 2025 Idioms paper were found, not a separate 2024 human study under that name.

## Fit for mentalc

### Takeaway

Mental's upper levels are already built the way the strongest decompiler evidence says to build them: deterministic structure, the LLM only naming, and a per-build check of the output against its input. What the decompiler field adds is (1) a cause-locator for a failed check (D-Helix's rule toggling), (2) metamorphic testing of mentalc itself (EMI), (3) an ablation control for name truth (the memorization floor), and (4) a precedent for treating a pattern near-miss as a reportable signal (residual gotos). End-to-end LLM lifting, LLM-judged readability as acceptance, and agreement-between-samples as truth do not fit the rule that the LLM never adds facts.

### Cited Findings

- Mental's specification already has mechanical extractors own structure and the LLM only name and summarize, with checks of coverage, guard algebra, summary agreement, name truth, a barrier on vocabulary, and a pair check that runs the lower level and projects upward [V] — [specification, sections 2, 9, 11](../../../specification.md)
- On real code an LLM allowed to rewrite a strong deterministic decompiler's output lost 17–52% of coverage equivalence [V] — [DecompileBench](https://arxiv.org/html/2505.11340v1)
- LLM naming gains can come from the model's prior, invisible to readability [V, abstract] — [Memorization floor](https://arxiv.org/abs/2609.17236)
- D-Helix pairs a per-function equivalence check with heuristic toggling to name the rule at fault [S] — [D-Helix](https://www.usenix.org/conference/usenixsecurity24/presentation/zou)

### Inferences

Ranked by fit with the rule that mechanical code owns facts and every LLM answer is checked:

| Rank | Method | Fit | Reason |
|---|---|---|---|
| 1 | **Translation validation per build** (Necula, Alive2, D-Helix's SymDiff) | High; Mental's pair check already is one | Checks each lifted model against its own lower level, so the lifting rules need no proof. Mental's levels are finite-state and bounded, so the check can be exhaustive within stated bounds rather than test-sampled. Keep the property that a validator says "no" when unsure. |
| 2 | **Deterministic structure, LLM names only** (SK2Decompile's skeleton/skin, DIRTY/VarBERT/ReSym over unchanged code, every commercial tool) | High; confirms the current design | The only hybrid that preserves correctness on real code. The LLM sees a placeholder-named skeleton and returns names; it never touches edges, guards or boundaries. |
| 3 | **Cause localization by rule toggling** (D-Helix Tuner) | High, new | When a pair check fails, re-run it with one closing rule or pattern disabled at a time; the rule whose removal flips the result is named in the repair task or the finding. Cheap given that rules are a configurable strategy. |
| 4 | **Semantic-preservation gate on every LLM edit** (DeGPT's operator, DecLLM's oracles) | High, already present as summary agreement and guard algebra | An LLM answer that would change any fact is rejected mechanically, then repaired with the failure named. DecLLM's one-shot vs iterative gap (17% vs 74%, unverified) supports the repair loop. |
| 5 | **Ablation control for name truth** (memorization floor) | Medium-high, new, as an experiment | Ask for the same name with the code's own identifiers permuted or removed, leaving only the footprint (what the step reads, writes, decides). A name that survives was earned by the footprint; one that changes came from the identifiers or the prior. Measures whether name truth checks what it claims. |
| 6 | **Metamorphic (EMI) testing of mentalc** | Medium-high, as tests, not as a build check | Mutate the input repository in ways that must not change the upper model (dead code, renamed locals, reordered independent statements, an unrelated port) and assert the step map and verdicts are unchanged; mutate in ways that must change it (remove a guard) and assert they change. Finds lifting bugs the way EMI found decompiler bugs. |
| 7 | **Near-miss as signal** (residual gotos in pattern structuring) | Medium | Precedent exists: a region that no structuring pattern fits is reported, not forced. In Mental a lower-graph fragment that almost matches an upper pattern (a claim without its guard) can be reported as a candidate finding, labelled a prediction until `mentalvm` gives a witness. Decompilers show the fallback is often benign (compiler output, unusual style), so the false-alarm rate needs measuring. |
| 8 | **Differential or coverage equivalence on a corpus** (DecompileBench CER) | Medium-low | Mental already runs the lower level and projects, which subsumes this; corpus sampling adds nothing where bounded-exhaustive search is available. Useful only past the bounds. |
| 9 | **Self-consistency and voting** (ReSym, GenNm) | Low-medium | Improves name stability and can feed the "generated once, stored" rule, but agreement is not truth; never an acceptance check on its own. |
| 10 | **Recompilation / load success** | Low | Mental's structural and anchor checks already play this role; the decompiler evidence shows it is gameable and says nothing about behaviour. |
| 11 | **Readability metrics and LLM judges** (R2I, Elo judges) | Low for acceptance | Readability does not track correctness or task performance (DSN 2025, memorization floor). At most an offline score for comparing naming prompts. |
| 12 | **End-to-end LLM lifting** (LLM4Decompile-End, Nova) | Reject | The LLM would place boundaries and write structure, i.e. add facts; correctness on real code is the lowest measured. |

- The owner's idea of a per-level intermediate language with pattern matching is the classic structuring decompiler, not the neural one: deterministic region patterns, idiom recognition, and a visible fallback. The neural literature's lesson is about the naming layer on top, not the lifting.
- A per-level IL also gives the decompiler field's alignment problem a clean answer: the step map is the alignment, stored, so the source-to-decompiled correspondence that the fidelity taxonomy had to rebuild by hand exists by construction.

### Gaps

- No decompiler work found checks a lifted artefact at more than one level of abstraction at once; Mental's ladder of pair checks has no direct precedent in this field.
- Whether near-miss reports in a system model have a tolerable false-alarm rate is unmeasured; decompilers give no number for residual gotos as defect indicators.
