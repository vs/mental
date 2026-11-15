# Verified evidence: LLM resolution of types, call targets and build environments, and deterministic checking of LLM proposals (state as of November 2026)

How these notes were made. Every paper below was downloaded as a PDF (arXiv, publisher or author page) on 2026-11-14, converted to text, and the numbers were read from the paper text itself. Page numbers are PDF pages of the version downloaded. No number comes from a search snippet. For most papers the abstract, the result tables and the result sections were read, not every page; where only the abstract was read this is stated. Status words: CONFIRMED (number seen in the paper), REFUTED (paper says something else than earlier research), NOT FOUND. "Derived" marks arithmetic done here, not printed in the paper.

One line per source:

- Venkatesh et al. 2410.00603 (v2, July 2025): LLMs beat HeaderGen and HiTyper on micro-benchmark type inference; lose to PyCG, Jelly and HeaderGen on call graphs.
- TypeEvalPy 2312.16882 (Jan 2024): the benchmark itself; best static tool exact-matched 564 of 845.
- TypeGen 2307.09163 (ASE 2023): GPT-3.5 with static-analysis prompts, 79.2% top-1 on ManyTypes4Py.
- TIGER 2407.02095 (2024): fine-tuned CodeT5 generate-then-rank, 86.2% top-1; unseen user types 67.5%.
- TypeT5 2303.09564 (ICLR 2023): CodeT5 plus static context; rare types 58.6% against 10.3% for HiTyper.
- HiTyper 2105.03595 (ICSE 2022): static inference plus rejection rules for neural predictions.
- TypyBench 2507.22086 (July 2025): whole-repository inference; LLMs similar on type similarity, far apart on type-check errors.
- PyTIR 2512.21591 (Dec 2025): type checker in the loop, 92.7% fewer introduced type errors.
- TypePro 2604.02702 (Apr 2026): 88.9% top-1 on ManyTypes4Py with inter-procedural slices.
- QuAC (OOPSLA 2024): non-LLM attribute-based inference; its "LLM baseline" is TypeT5.
- SemTaint 2601.10865 (Jan 2026): LLM resolves only unresolved CodeQL call edges on taint paths; 106 of 162 missed vulnerabilities found.
- SEA 2408.04344 (ASE 2024): LLM filters indirect-call candidates from type analysis in C; F1 up, about 1% recall lost.
- Facet 2609.33547 (Sep 2026): symbolic rules bound and check LLM decisions for residual indirect calls in C.
- iResolveX 2601.17888 (Jan 2026): binary indirect calls, learned scorer, not an LLM.
- AutoPruner 2209.03230 and the pruning study 2402.07294: learned call-graph pruning gains precision and loses recall.
- IRIS 2405.17238 (ICLR 2025): LLM-inferred taint specifications for CodeQL, 55 against 27 detections, false discovery rate still about 85%.
- LLift 2308.00245 (2023): GPT-4 triages static UBI warnings, 13 true bugs of 26 reported.
- InferROI 2311.04448: LLM infers resource intentions, static analysis checks paths.
- LLMDFA 2402.10754 (2024): compilation-free dataflow, 87.10% precision and 80.77% recall.
- Tencent false-alarm study 2601.18844 (Jan 2026): LLM filters remove most false alarms and lose 6 to 46% of true bugs depending on technique.
- ExecutionAgent 2412.10133: 33 of 50 projects, 74 minutes, USD 0.16.
- Repo2Run 2502.13681: 361 of 420 Python repositories, 29 minutes.
- EnvBench 2503.14443: 6.69% Python, 29.47% JVM.
- SetupBench 2507.09063: 34.4 to 62.4% overall.
- Installamatic 2412.06294: 55% of 40 repositories at least once in ten runs.
- Multi-Docker-Eval 2512.06915: best fail-to-pass 37.7%.
- RepoLaunch 2603.05026: 78% build success, about USD 1.7 per instance.
- SWE-rebench 2505.20411: working install recipe for at least one task in 31% of repositories.
- EnvPilot 2609.07357 and SpecBuild 2610.33129 (2026): newer agents, 75.00% Pass@1 on a 112-instance benchmark; plus 6.4 to 12.0 points on Multi-Docker-Eval.
- Sulír and Porubän 2016, Sulír et al. 2020, Sulír et al. EMSE 2026, Hassan et al. 2017, Tufano et al. 2017, Robles et al. 2022: Java buildability baselines.
- Gistable 1808.04919 and DockerizeMe 1905.11127: Python snippet executability.
- PPA (OOPSLA 2008), JCoffee 2009.05090, LExecutor 2302.02343: partial-program analysis.
- JackEE (PLDI 2020), Jasmine (ASE 2022): framework modelling.
- SpecGen 2401.08807: LLM specifications accepted only if a verifier proves them.

## 1. Venkatesh et al. (arXiv 2410.00603), TypeEvalPy (arXiv 2312.16882) and follow-ups

### Takeaway
All headline numbers are CONFIRMED from the paper's tables. On micro-benchmarks the best LLMs exact-match about 94% of type annotations against 65% for HeaderGen and 43% for HiTyper, while every LLM is below the static tool on call graphs, and far below on flow-sensitive call sites (28.5% against 91.3% exact match). No follow-up with newer models on the same benchmark was found.

### Cited Findings
- CONFIRMED. Scope: 24 LLMs; TypeEvalPy extended "from 860 to 77,268 type annotations"; version read is v2 dated 16 Jul 2025 (abstract, p.1). Open models were run with 4-bit quantization and greedy decoding on one H100 (Section 6.4, p.23). Models date from 2024: gpt-4o, gpt-4o-mini, mistral-large-2407, llama3.1, qwen2, gemma2, phi3.5, codellama. — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Type inference, TypeEvalPy micro-benchmark, exact matches out of 860 (Table 7, p.30): gpt-4o 806; mistral-large-it-2407 804; gpt-4o-mini 783; llama3.1-it-70b 770; codestral-22b 760; codellama-it-13b 728; HeaderGen 563; HiTyper (with Type4Py, "hityperdl") 367; tinyllama 102. By kind for gpt-4o: return 217 of 239, parameter 81 of 88, local variable 508 of 533. — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. As percentages (Table 9, p.32): gpt-4o 93.72% micro and 96.10% autogen; mistral-large 93.49% and 96.97%; gpt-4o-mini 91.05% and 87.28%; HeaderGen 65.47% and 65.73%; HiTyper 42.67% and 27.43%. — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Autogen benchmark, exact matches out of 77,268 (Table 8, p.31): mistral-large 74,979; gpt-4o 74,257; HeaderGen 50,802; HiTyper 21,191. Two models collapsed on it: mixtral-8x7b from 79.77% to 4.72%, phi3.5-moe from 71.51% to 4.38% (Table 9), which the authors read as "a possible overfitting to the string datatype". — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Python flow-insensitive call graph, PyCG micro-benchmark, 126 cases and 599 edges (Table 3, p.24). "Complete" counts test cases with no false positive, "Sound" counts test cases with no missed edge. PyCG: complete 107, sound 110, exact matches 569, 0.41 s. mistral-large: 76, 79, 518, 534 s. gpt-4o: 63, 75, 486. gpt-4o-mini: 47, 47, 397. In percent (Table 5, p.27): mistral-large 60.32% complete, 62.70% sound, 86.64% exact match; gpt-4o 50.00%, 59.52%, 81.14%. PyCG is 84.9% complete and 87.3% sound (text, p.24). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- REFUTED in part. The earlier report wrote that an LLM was "complete on 76 of 126 cases against PyCG's 107". The figures are right but 76 is mistral-large-it-2407, not gpt-4o; gpt-4o is complete on 63 (Table 3). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. JavaScript, SWARM-JS, 126 cases and 596 edges (Table 4, p.26): Jelly complete 49, sound 85, exact matches 490 (text: 38.8%, 67.4%, 82.2%); mistral-large 51, 54, 458 (40.4%, 42.8%, 76.8%); gpt-4o 44, 64, 451. TAJS failed with analysis errors on 102 of 126 cases because it supports only ECMAScript 3 (p.25). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Flow-sensitive call sites, HeaderGen micro-benchmark, 122 cases and 357 call sites (Table 6, p.28): HeaderGen complete 111, sound 112, exact 326 (90.9%, 91.8%, 91.3%), 10.94 s; mistral-large 38, 32, 102 (31.1%, 26.2%, 28.5%); gpt-4o 31, 26, 74; gpt-4o-mini 17, 12, 36. The authors: "The increased complexity of flow-sensitive analysis, which requires specificity about the location of function calls, poses additional challenges for LLMs." — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Stated LLM failure patterns (Section 8.1 and 8.2, Tables 10 to 13, pp.33 to 37): Python: indirect imports through intermediate modules, multi-level returns such as `func4()()`; JavaScript: inheritance, chained attribute references, destructured assignments, indirect argument flows, cross-file imports, type coercion of object keys. Classes category in JavaScript: Jelly sound on 20 of 21 cases, mistral-large on 3 of 21 (Table 12). In the dicts category mistral-large beat PyCG (12 of 12 sound against 11 of 12). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. Where HeaderGen loses on types (Table 14, p.38): decorators, autogen benchmark, local variables 75.7% for mistral-large against 6.1% for HeaderGen; generators local variables 91.9% against 34.4%. HeaderGen "falls back to the generic type Any" for augmented assignment, star unpacking and nested tuple unpacking (p.36). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. The authors' own limits (Section 9, pp.42 to 43): one prompt for all models; open models "frequently deviated from the expected output formats"; micro-benchmarks "may miss the complexity and variability of real-world workloads". They also state that hybrid use "requires addressing challenges related to confidence calibration and transparency" (p.42). — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- CONFIRMED. TypeEvalPy original paper (v2, 2 Jan 2024): 154 code snippets, 845 annotations, 18 categories; 239 function-return, 88 parameter, 518 local-variable annotations (p.2). Top-1 exact matches (Table 2, p.3): HeaderGen 564, HiTyper-DL 369 (top-5: 441), Type4Py 157 (top-5: 314). "even the top-performing tools achieving a soundness rate of only 44%" (p.1). HiTyper "did not produce predictions for 34 out of the 154 cases". — [arXiv 2312.16882](https://arxiv.org/abs/2312.16882)
- NOT FOUND. A 2025 to 2026 re-run of TypeEvalPy or SWARM-CG with newer models. The benchmark repository README, fetched 2026-11-14, lists the same models and counts as Table 8 of the paper. — [TypeEvalPy README](https://github.com/secure-software-engineering/TypeEvalPy)

### Inferences
- The gap between "exact match of edges" (81 to 87% for the best LLMs on Python) and "sound test cases" (60 to 63%) means that an LLM gets most single edges right and still misses at least one edge in four out of ten small programs. For a per-reference proposer that is the favourable reading: the unit Mental would ask about is one edge, not a whole graph.
- The flow-sensitive result is the closest to Mental's task (a target for one specific call site) and it is the weakest: 28.5% for the best 2024 model. This is whole-file generation, not a single-site question, so it is an upper bound on difficulty, not a measurement of single-site accuracy.
- All numbers are for 2024 models. No measurement for 2025 to 2026 frontier models on these benchmarks exists in what was found.

### Gaps
- No per-category failure statistics for gpt-4o are printed, only for mistral-large.
- The micro-benchmark programs are a few lines to a few dozen lines each; nothing here measures LLM call resolution on real repositories.

## 2. LLM-based and LLM-assisted type inference for Python with numbers

### Takeaway
On ManyTypes4Py the reported top-1 exact match rose from 79.2% (TypeGen, GPT-3.5, 2023) to 86.2% (TIGER, 2024) and 88.9% (TypePro, 2026), but accuracy on user-defined types never seen in training is 64 to 68% at top-1. On whole repositories the picture changes: LLMs reach about 0.79 to 0.80 type similarity and still introduce hundreds of type-check errors per repository.

### Cited Findings
- CONFIRMED. TypeGen (ASE 2023), ManyTypes4Py, full test set of 85,205 instances, top-1 exact match (Table IV, p.8): TypeGen 79.2% overall, arguments 73.1%, returns 68.7%, variables 82.2%; Type4Py 76.6% overall. Top-5: TypeGen 87.0%. Models: OpenAI "text-davinci-003" and "gpt-3.5-turbo-0301", 50 samples at temperature 1.0 ranked to top-5 (p.7). — [arXiv 2307.09163](https://arxiv.org/abs/2307.09163)
- CONFIRMED. TypeGen zero-shot against prompted, sampled test set of 10,000 (Table VI, p.9): GPT-3.5 zero-shot 62.0% top-1, standard in-context 69.7%, TypeGen prompts 78.9%; ChatGPT 61.3%, 68.0%, 78.8%. Ablation (answer to RQ3, p.9): "code slicing improves the overall performance of type inference by 24%, type hints improve the performance of user-defined type inference by 11%". — [arXiv 2307.09163](https://arxiv.org/abs/2307.09163)
- CONFIRMED. TypeGen predictions passed through HiTyper's static rejection (Table V, p.8): HiTyper alone 52.4% exact match (arguments 8.0%); with Type4Py top-5 as recommender 85.7%; with TypeGen top-5 88.3% (arguments 84.9%, returns 77.9%). "HiTyper can reject wrong types" is the stated reason for feeding top-5. — [arXiv 2307.09163](https://arxiv.org/abs/2307.09163)
- CONFIRMED. TIGER (fine-tuned CodeT5 generator plus similarity ranker), ManyTypes4Py, 10,000-instance test sample, top-1 exact match (Table II, p.9; columns elementary, parameterized, user-defined with unseen in parentheses, all): TIGER 94.5, 75.1, 80.2 (67.5), 86.2; TypeGen 89.8, 60.0, 79.0 (63.6), 79.2; Type4Py 95.1, 67.9, 43.0 (6.9), 76.9; HiTyper 94.0, 60.0, 61.3 (51.3), 76.3. Top-5 for TIGER: user-defined 94.1 (unseen 85.0), all 94.6. "user-defined types in this paper also include third-party types" (p.1). — [arXiv 2407.02095](https://arxiv.org/abs/2407.02095)
- CONFIRMED. TypeT5 (ICLR 2023), rare types, full accuracy (Table 3, p.7): TypeT5 58.56% on BetterTypes4Py and 53.44% on InferTypes4Py; CodeT5 49.47% and 51.64%; HiTyper 10.30% and 9.36%. Common types (Table 2): TypeT5 79.24% and 81.75%. — [arXiv 2303.09564](https://arxiv.org/abs/2303.09564)
- CONFIRMED. HiTyper: rare types are "types with proportions less than 0.1% among the annotations", and "99.7% and 79.0% of rare types are user-defined types in ManyTypes4Py and Typilus's dataset" (Section 4.3, p.9). HiTyper gives "> 30% boost" on rare types over deep-learning models (answer to RQ2). It "is still unable to infer some variable types (around 10%)" (Section 5, p.10). — [arXiv 2105.03595](https://arxiv.org/abs/2105.03595)
- CONFIRMED. TypyBench (July 2025), 50 Python repositories, whole-repository inference (Table 2, p.7; TypeCheck is the mean number of mypy errors, lower is better; the human-annotated ground truth scores 141.8): gpt-4o TypeSim 0.804, TypeCheck 366.0, missing rate 0.099; gpt-4o-mini 0.789, 310.6; claude-3.5-sonnet 0.788, 127.1; deepseek-v3 0.795, 214.6; grok-2 0.787, 190.1; llama-3.1-8b 0.603, 465.7. Exact match scores are "nearly vanishing for types of depth 3 and above" (Section 6.3). Claude 3.5 Sonnet "still performs worse than the ground truth on 29 repositories". — [arXiv 2507.22086](https://arxiv.org/abs/2507.22086)
- CONFIRMED (abstract and introduction only). PyTIR (Dec 2025, labelled FSE'26): 12 Python repositories; "a TypeSim score of 0.89 and a TypeExact score of 0.84, representing a 27% and 40% relative improvement over the strongest baseline"; "PyTIR reduces 92.7% of introduced type errors compared to the SOTA methods, resulting in an average of only 46.17 errors per repository" (p.3). The motivating section states that QuAC, TIGER, DeepSeek-V3, DeepSeek-R1 and GPT-4o "all ... fail to produce correct type annotations" on examples from Click and Flask (p.4). — [arXiv 2512.21591](https://arxiv.org/abs/2512.21591)
- CONFIRMED (abstract only). TypePro (Apr 2026, FSE'26): "Top-1 exact match (EM) rates of 88.9% and 86.6%" on ManyTypes4Py and ManyTypes4TypeScript, "7.1 and 10.3 percentage points over the second-best approach". The LLM used was not located in the pages read. — [arXiv 2604.02702](https://arxiv.org/abs/2604.02702)
- CONFIRMED (abstract and Section 6.2 only). QuAC (OOPSLA 2024) is not LLM-based. Its "LLM-based method" baseline is TypeT5; QuAC "is nearly two orders of magnitude faster than an LLM-based method while covering nearly half of its errorless non-trivial type predictions". — [QuAC paper](https://jifengwu2k.github.io/static/pdfs/quac_oopsla_2024.pdf)
- NOT FOUND. A paper named "RepoTyper". The arXiv title search for repository-level Python type inference returned TypyBench, PyTIR and TypePro instead.

### Inferences
- Not comparable: TypeEvalPy exact match (about 94%, tiny programs, mostly builtin types), ManyTypes4Py exact match (79 to 89%, single slots from real projects, labels are human annotations), and TypyBench TypeSim (whole repositories, similarity not equality). The same model family scores very differently on each.
- The numbers that matter for Mental's last-resort step are the user-defined and unseen rows: 63.6% (TypeGen, GPT-3.5) and 67.5% (TIGER) at top-1, rising to 70.8% and 85.0% at top-5. A proposer that returns several candidates and a checker that picks among them fits these data better than a single answer.
- TypyBench shows that slot-level similarity hides cross-file inconsistency: gpt-4o has the best similarity and 2.6 times the type-check errors of the human annotations.

### Gaps
- No paper found reports exact-match accuracy of 2025 to 2026 frontier models specifically on user-defined or library types in real repositories.
- TypePro and PyTIR were read only at abstract depth; their per-category tables were not extracted.

## 3. LLM call resolution limited to unresolved or indirect calls

### Takeaway
Three designs hand only the leftover call sites to an LLM. SemTaint (JavaScript, GPT-5 family, 2026) reports end-to-end vulnerability recall, not the precision of the resolved edges. SEA and Facet (C) bound the LLM to a candidate set produced by static analysis; Facet additionally checks each LLM decision with symbolic rules and abstains at small sites. None publishes a false-acceptance rate per resolved call site.

### Cited Findings
- CONFIRMED. SemTaint design: "uses static program analysis to calculate a call graph and defers to an LLM to resolve call edges that cannot be resolved statically" (abstract). An unresolved call is "an invocation with no resolved callee at precision 0" of CodeQL's `getACallee`, test directories excluded (Section 4.3.1, p.8). Models: "GPT-5 for the Source/Sink and Flow Summary Agents and GPT-5-mini for the CallGraph Agent"; each agent runs 3 times, "union-aggregation for the Source/Sink and CallGraph Agents and majority-vote for the Flow Summary Agent" (Section 5.1.3, p.11). CodeQL v2.21.2. — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- CONFIRMED. SemTaint result: 172 npm vulnerabilities that CodeQL misses, 10 held out for validation, "On the remaining 162 instances, SemTaint identified the vulnerability in 106 instances, which represents a recall of 65.43%" (p.2; Table 2, p.12). Of the 106, 81 (76.42%) needed only source and sink specifications and 25 (23.58%) needed call-graph repair (Table 4, p.13). Across validation and test, 31 of 115 detections (27.0%) required call-graph repair. — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- CONFIRMED. SemTaint volume and focus (Section 5.3, p.12): "94,909 unresolved calls" across 162 packages; taint reachability cuts this to "10,184 calls (an 89.2% reduction)"; 4,989 are auto-assigned as third-party by CodeQL queries, "leaving 5,195 calls for agent resolution"; "average agent calls drop from 585.9 to 32.1" per package. Third-party flow summaries analysed drop "from 6,639 candidates to just 449". — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- CONFIRMED. SemTaint precision evidence is small: validation set, 30 alerts with 6 false positives, "80% precision" (Table 1, p.12). On 10 open-source packages, 34 alerts, 13 true positives corresponding to 4 new vulnerabilities (Section 5.5, p.13). "We did not investigate false positives in the testing dataset". One validation miss was caused by the LLM: "the Flow Summary Agent incorrectly classified a third-party call as sanitizing, causing the missed detection" (p.12). — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- CONFIRMED. SemTaint cost: Table 5 (p.20) gives, for the 10-vulnerability validation set, "Recall 60% ... Cost $77" for the Gemini 2.5 configuration and "90% ... $92.75" for the GPT-5 configuration. Derived: about USD 9 per package for the GPT-5 configuration. No cost per call site is printed. — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- CONFIRMED. SEA (ASE 2024), C indirect calls, 31 projects, models "qwen" and "llama3" at five temperatures (Section 4.2, p.8). SEA classifies each edge produced by FLTA, MLTA or Kelp as true or false. "SEA achieves a significant improvement in F1 score at the cost of approximately 1% recall. Specifically, SEA enhances F1 by 14.5% for Kelp, 17.1% for MLTA, and 28% for FLTA." "SEA's precision, at approximately 41%, is partly influenced by the dataset." With FLTA and qwen, "the F1 score increases from 34.8% to between 58.5% and 59.4%, with a maximum recall loss of only 1.5%". — [arXiv 2408.04344](https://arxiv.org/abs/2408.04344)
- CONFIRMED. Facet (arXiv Sep 2026), C over LLVM opaque pointers, 14 programs of 12 to 620 KLoC, LLM "Gemini 3.7 Flash at temperature 0", abstention threshold 2 (p.10). Design: "an LLM decides only the cases that the evidence leaves open. The symbolic rules bound the model's candidates and check its decisions" (p.2); "After the query, the symbolic rules check each decision and determine whether it is adopted" (Section 3.4, p.8). Facet "abstains wherever the symbolic result is non-empty and holds at most τ targets" (p.9). Results: mean target set per site 25.9 to 5.2; "macro F1 rises from 0.399 to 0.708 and recall from 0.772 to 0.984" (p.13); best prior tool F1 0.520. — [arXiv 2609.33547](https://arxiv.org/abs/2609.33547)
- CONFIRMED. Facet ablation on a 120-site nginx ground truth (table on p.13; columns precision, recall, F1, targets per site, spurious edges, missed edges): Facet without LLM 0.296, 0.956, 0.355, 60.6, 6,872, 29; Facet 0.377, 0.956, 0.457, 13.0, 1,156, 25; without pruning rules 0.391, 0.952, 0.472, 11.6, 989, 25. — [arXiv 2609.33547](https://arxiv.org/abs/2609.33547)
- REFUTED as an LLM example. iResolveX (arXiv 2601.17888) resolves indirect calls in stripped binaries with value-set analysis plus "a learning-based soft signature scorer"; the only mentions of language models are in its reference list. Its numbers: 19.2% fewer predicted targets at 98.2% recall; 44.3% fewer at 97.8% recall, "only a 0.4% loss" (abstract). — [arXiv 2601.17888](https://arxiv.org/abs/2601.17888)
- CONFIRMED (abstract only). AutoPruner (2022) is a fine-tuned code model, not a prompted LLM: "improving on F-measure by up to 13% in identifying false-positive edges", "halving the false alarm rate on null pointer analysis". — [arXiv 2209.03230](https://arxiv.org/abs/2209.03230)
- CONFIRMED (abstract only). Call-graph pruning on real Java programs: "substantial improvements in the CG precision (+25%) meet reduced recall (-9%)"; pruned graphs are "much smaller (69%)" with a "3.5x speed-up". — [arXiv 2402.07294](https://arxiv.org/abs/2402.07294)

### Inferences
- Candidates are generated three ways in this literature: free search by an agent with repository tools and union over three runs (SemTaint), the candidate set of a sound type-based analysis that the LLM only filters (SEA), and a symbolically bounded set with a post-check and abstention (Facet). Only the last two bound what the LLM can assert.
- In Facet the LLM's net effect on the nginx ground truth is an 83% cut in spurious edges (6,872 to 1,156, derived) with recall unchanged. This is the strongest measured case that a checked, bounded LLM step helps without losing recall, on one program with 120 labelled sites.
- SemTaint's union aggregation raises recall and cannot lower false edges; the paper's 80% alert precision is measured on 30 alerts.

### Gaps
- NOT FOUND: any paper that reports precision and recall of LLM-proposed call targets per call site for Python, JavaScript or Java against a ground truth. SemTaint reports only downstream vulnerability detection.
- NOT FOUND: cost per call site. Only SemTaint's validation-set total (USD 92.75 for 10 packages) is printed.
- Facet is an arXiv preprint; no venue was confirmed.

## 4. Hybrid static plus LLM designs, and the cost of letting the LLM filter findings

### Takeaway
When the LLM writes specifications that a static engine consumes, detections roughly double (IRIS: 27 to 55 of 120) while the false discovery rate stays near 85 to 95%. When the LLM filters static findings it removes most false alarms and also removes true ones: in an industrial study of 2025-generation models the retained share of true bugs was 54 to 94% depending on model and technique.

### Cited Findings
- CONFIRMED. IRIS (ICLR 2025), CWE-Bench-Java, 120 vulnerabilities (Table 1, p.8): CodeQL detects 27 (22.50%), average false discovery rate 90.03%, average F1 0.076. IRIS with GPT-4: 55 (45.83%), 84.82%, 0.177. With GPT-3.5: 47, 90.42%. Llama-3 8B: 41, 95.55%. Llama-3 70B: 54, 90.96%. DeepSeekCoder 7B: 52, 95.40%. — [arXiv 2405.17238](https://arxiv.org/abs/2405.17238)
- CONFIRMED. IRIS on false positives: the average false discovery rate "is an upper bound"; "we manually analyzed 50 random alarms reported by IRIS (using GPT-4) and found that 27/50 alarms exhibit potential attack surfaces, yielding a more refined estimated false discovery rate of 46%" (p.7). LLM-inferred sinks reach 87.11% recall against CodeQL's own sink specifications with GPT-4 (p.8). The LLM-based contextual filter helps only strong models: "only GPT-4, GPT-3.5, and Llama-3 70B see a positive impact after contextual analysis, while the smaller models see negative" (p.9). — [arXiv 2405.17238](https://arxiv.org/abs/2405.17238)
- CONFIRMED. LLift (2023), Linux kernel use-before-initialization warnings left undecided by UBITect; model "gpt4-0613" (p.7). Random-1000 sample: 182 cases filtered automatically, 818 analysed; "LLift reports 26 positives among the Random-1000 dataset, where half of them are true bugs ... a precision of 50%" (Section 6.2). On the 52 confirmed bugs ("Bug-50") it "demonstrated full effectiveness in identifying all real bugs" (p.8). The paper states that hallucination "can lead to both false positives and false negatives" (p.6). — [arXiv 2308.00245](https://arxiv.org/abs/2308.00245)
- CONFIRMED (abstract and p.2 only). InferROI: bug detection rate 59.3% and 62.5%, false alarm rate 18.6% and 19.5% on DroidLeaks and JLeaks; the LLM's intention inference itself has "a precision of 74.6% and a recall of 81.8%" by manual annotation. Prompted model named as GPT-4; Llama3-8B and Gemma2-9B also evaluated. — [arXiv 2311.04448](https://arxiv.org/abs/2311.04448)
- CONFIRMED (abstract and Section 4 setup only). LLMDFA (2024): "On average, LLMDFA achieves 87.10% precision and 80.77% recall"; the LLM synthesises code that calls a parser and "an automated theorem prover to validate path feasibility"; models gpt-3.5-turbo-0125, gpt-4-turbo-preview, gemini-1.0-pro, claude-3-opus; total experiment cost USD 1,622.02 (p.6). — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754)
- CONFIRMED. LLM as filter of static findings, Tencent (Jan 2026): 433 alarms, 328 false positives and 105 true positives (abstract). Table 2 (p.8), recall on true bugs and share of false positives removed (FPR_R): basic prompt GPT-4o 0.66 and 0.45, Claude-Opus-4 0.83 and 0.26; few-shot Claude-Opus-4 0.74 and 0.72; bug-type-enhanced GPT-4o 0.54 and 0.52; LLM4SA GPT-4o 0.84 and 0.84, Qwen-3-Coder 0.65 and 0.96, DeepSeek-R1 0.86 and 0.92; LLM4PFA GPT-4o 0.75 and 0.98, Claude-Opus-4 0.88 (FPR_R cut off in the extract). Cost per alarm 2.1 to 109.5 seconds and USD 0.0011 to 0.12 (Table 4). — [arXiv 2601.18844](https://arxiv.org/abs/2601.18844)
- REFUTED as an LLM example. Artemis (arXiv 2502.21026) in its v3 text is a rule-based PHP taint analysis: "207 true vulnerable paths (106 true SSRFs) with 15 false positives" on 250 applications. The arXiv listing title still says "LLM-Assisted", but the abstract of the downloaded version does not mention an LLM. — [arXiv 2502.21026](https://arxiv.org/abs/2502.21026)

### Inferences
- Derived from the Tencent table: even the best filter configurations discard 12 to 25% of confirmed bugs (recall 0.75 to 0.88), and plain prompting discards up to 46%. This supports a rule that an LLM may add a candidate fact but may not suppress an unresolved marker.
- IRIS shows the specification route keeps the static engine as the judge of the final claim, at the price of a still-high alert volume; its 5-point FDR improvement comes from the contextual filter, which hurts with weaker models.

### Gaps
- IRIS's per-model precision of inferred specifications is only in a figure (Fig. 7); no number was extractable from text.
- InferROI and LLMDFA result tables were not read beyond the abstract-level figures.

## 5. Environment and build setup by agents

### Takeaway
Reported success ranges from 6.69% to 86.0% and the spread is explained by benchmark selection and by what counts as success. Cost where printed: USD 0.16 and 74 minutes per project (ExecutionAgent, GPT-4o-mini), 29 minutes (Repo2Run, GPT-4o), about USD 1.7 per instance (RepoLaunch, GPT-5).

### Cited Findings
- CONFIRMED. ExecutionAgent (ISSTA 2025): 50 open-source projects, 14 languages; "successfully executes the test suites of 33/50 projects, while matching the test results of ground truth test suite executions with a deviation of only 7.5%"; "an execution time of 74 minutes and LLM costs of USD 0.16, on average per project"; "6.6x" over the best prior technique (abstract). Model: "OpenAI's GPT-4o-mini" (Section 3.1, p.10). Success is verified by comparing passed, failed and skipped test counts to a manually built ground truth. — [arXiv 2412.10133](https://arxiv.org/abs/2412.10133)
- CONFIRMED. Repo2Run: 420 Python repositories with unit tests; model "gpt-4o-2024-05-13", temperature 0.2 (p.6). Table 1 (p.7): Repo2Run 100% Dockerfiles generated, 86.0% environments built (361); SWE-agent 26.9% and 9.0% (38); pipreqs 29.8% and 6.0%; plain LLM generator 47.6% and 22.1%. Success definition: "the generated Dockerfile must not only build successfully but also allow tests to run by 'pytest' ... regardless of whether they pass or fail". "The average time for successful building using Repo2Run is 29.03 minutes"; sampled manual building took 21.33 minutes (Section 5.1, p.8). — [arXiv 2502.13681](https://arxiv.org/abs/2502.13681)
- CONFIRMED. EnvBench: 329 Python and 665 JVM repositories, chosen to exclude "projects that can be fully configured by simple deterministic scripts"; metrics are "a static analysis check for missing imports in Python and a compilation check for JVM languages"; models GPT-4o and GPT-4o-mini; "The best approach manages to successfully configure 6.69% repositories for Python and 29.47% repositories for JVM" (abstract), achieved by the Bash Agent with GPT-4o (p.2). Success is binary: "zero exit code and zero reported errors" (p.4). Best Python baseline still leaves "52.00 missing imports per repository" on average (p.7). Derived: about 22 of 329 and 196 of 665. — [arXiv 2503.14443](https://arxiv.org/abs/2503.14443)
- REFUTED in part. The earlier report gave SetupBench as "38.9 to 57.4%". That range is the repository-setup category only. SetupBench has 93 instances over seven language ecosystems and five database engines; success is "a deterministic one-line validation command". OpenHands overall (Table 2, p.4): GPT-4o 34.4%, GPT-4.1 50.5%, Claude 3.5 53.8%, Claude 3.7 57.0%, Claude 4 62.4%, with 303K to 1,129K average tokens and 23.2 to 47.1 average steps. Repo setup: 38.9% to 57.4%. Local database setup: 20.0% to 53.3%. Dependency resolution: 25.0% (GPT-4o) to 87.5% (Claude 3.7 and Claude 4). "38-69% of actions being unnecessary compared to optimal human behavior" (abstract). — [arXiv 2507.09063](https://arxiv.org/abs/2507.09063)
- CONFIRMED. Installamatic: 40 open-source Python projects with test suites; "55% of the studied repositories can be automatically installed by our agent at least one out of ten times" (abstract); model "gpt-4o-mini-2024-07-18" (p.6); success oracle is execution of the repository's test suite (p.2). — [arXiv 2412.06294](https://arxiv.org/abs/2412.06294)
- CONFIRMED. Multi-Docker-Eval (Dec 2025): 334 issues from 40 repositories in 9 languages, repositories with 1,000 to 1,500 stars (p.3). Success is fail-to-pass of the issue's tests in the built environment. Table 1 (p.5), SWE-Builder framework, F2P and commit rate: DeepSeek-v3.1 37.72% and 52.89%; Kimi-K2-0905 37.62% and 55.49%; Claude-Sonnet-4 35.53% and 47.41%; GPT-5-Mini 34.13% and 49.60%; Gemini-2.5-Flash 29.44% and 40.62%; GPT-OSS-20B 17.17% and 29.44%. Input tokens 101K to 340K per task. By language, Go is best with F2P 54.50% (p.7). — [arXiv 2512.06915](https://arxiv.org/abs/2512.06915)
- CONFIRMED. RepoLaunch (Microsoft; the downloaded version is dated October 2026 and states acceptance at NeurIPS 2026): "achieves a 78% build success rate, outperforming the Python/Linux-only prior system by 18%" (abstract); default model "GPT-5-Thinking-Medium" (p.7). Table 1 (p.8), instances retained at the build stage from the previous stage: Python 76.2%, JS/TS 89.1%, C/C++ 79.1%, C# 92.0%, Java 78.0%, Go 72.3%, Rust 72.1%, Windows 70.5%. "Launching one instance costs about $1.7 on average"; single-command repositories cost "around $0.15", yet "about one-third of them still encounter build or test errors". — [arXiv 2603.05026](https://arxiv.org/abs/2603.05026)
- CONFIRMED. SWE-rebench: installation recipes generated by "Qwen2.5-72B-Instruct", up to three candidates per task, refined from error logs; "Our approach successfully produces a working installation recipe for at least one task in 31% of all repositories" (p.4). Dataset: over 21,000 Python tasks. — [arXiv 2505.20411](https://arxiv.org/abs/2505.20411)
- CONFIRMED (abstract only). EnvPilot (Sep 2026): AES-Bench, 112 GitHub instances across 9 languages; "75.00% Pass@1 success rate". The backbone model for the headline number was not located; DeepSeek-R1 is named for experience distillation (p.6). — [arXiv 2609.07357](https://arxiv.org/abs/2609.07357)
- CONFIRMED (abstract only). SpecBuild (Oct 2026): on Multi-Docker-Eval, "improves the success rate by 6.4 to 12.0 percentage points over the strongest baseline, while reducing average setup time by 22.4% to 41.6%" across three backbone models. — [arXiv 2610.33129](https://arxiv.org/abs/2610.33129)

### Inferences
- The success definitions form a ladder and are not comparable: script exits and static import check passes (EnvBench), tests can be started (Repo2Run), tests run and counts match a ground truth (ExecutionAgent), a task-specific check command passes (SetupBench), a specific failing test turns passing (Multi-Docker-Eval). Benchmarks also select differently: EnvBench removes easy repositories, Repo2Run requires existing unit tests, RepoLaunch reports retention between pipeline stages.
- Every one of these agents executes repository code. None is a model for a step that must not run the build; they bound what the user's coding agent could achieve if it is allowed to run it.
- Cost per repository in 2025 to 2026 is on the order of 100K to 1,000K tokens and tens of minutes.

### Gaps
- Repo2Run, EnvBench, Installamatic and SetupBench print no dollar cost in the sections read.
- Multi-Docker-Eval's wall time column is in "Ks" without a stated denominator; not used here.
- No 2026 result of a frontier model on EnvBench was found.

## 6. Buildability baselines

### Takeaway
The Java figures are CONFIRMED: 38.05% of 7,264 projects failed in 2016, 59% of 7,233 in 2020, and the 2026 journal study finds success rates of only 30 to 44% per JDK from Java 8 onward, with about 32% of projects failing under every JDK. Comparable repository-level studies for Python, JavaScript, Go or Rust were not found; only Python snippet studies were.

### Cited Findings
- CONFIRMED. Sulír and Porubän 2016: 10,000 random GitHub Java projects, 7,264 with a recognised build file, 4,494 built, 2,764 failed, 6 timed out (Figure 1, p.2); "38.05% of builds failed" (p.3); "the largest portion of builds are dependency-related (39.2%)" among failures (Section 3.4, p.5); 292 error types grouped into 12 categories. Maven error types (Table 2): DependencyResolution 39.11%, maven-compiler-plugin 18.16%, UnresolvableModel 10.59%, javadoc plugin 8.17%. Gradle (Table 3): CompilationFailed 22.36%, MissingProperty 11.59%. Ant (Table 4): Target:compile 21.83%, CannotImport 17.03%. — [arXiv 1712.01024](https://arxiv.org/abs/1712.01024)
- CONFIRMED. 2020 replication (data descriptor, MDPI Data 2020): "7264 build logs from a study executed in 2016 and 7233 logs from the 2020 execution ... The proportion of failed builds in our dataset is 38% in the 2016 execution and 59% in the 2020 execution." — [Sulír et al. 2020](https://sulir.github.io/papers/Sulir20large.pdf)
- CONFIRMED. Registered report (ESEM 2024): plan to build 2,500 random plain Java projects under Java 6 to 23; it restates that the 2020 replication used JDK 11 and "The proportion of failures increased to 59%". — [arXiv 2408.11544](https://arxiv.org/abs/2408.11544)
- CONFIRMED. Published results, Empirical Software Engineering (2026) 31:78: "Our hypothesis was not confirmed; there is no monotonic trend in the projects' build success rates. From Java 8 onward, they range from 30 to 44%, with a peak at JDK 17. Downgrading the latest JDK can fix 52% of failed builds. ... about 32% of projects fail for all JDKs. Gradle consistently has the lowest success rate of all tools, and builds most frequently fail during initialization and compilation." Largest drop: 6.4 percentage points between JDK 8 and 9 (p.14). 186 error types, the 93 most frequent categorised by hand (p.10). — [EMSE article](https://link.springer.com/article/10.1007/s10664-026-10806-6); abstract also at [EASE 2026 journal-first page](https://conf.researchr.org/details/ease-2026/ease-2026-journal-first/6/Local-Software-Buildability-across-Java-Versions)
- CONFIRMED. Hassan et al. ESEM 2017: top 200 Java projects on GitHub; "99 of 200 top Java projects cannot be built successfully with default build commands, among which 2 do not have source code, 11 are not using Maven, Ant or Gradle as their building tools, and the remaining 86 projects have various build failures"; 91 build failures in those 86 projects; leading causes "backward-incompatibility of JDK and building tools, non-default parameters in build commands, and project defects in code / configuration files, which accounts of 19, 33, and 14 build failures"; 12 are explained in the readme and 27 in the failure log; "at least 52 can be automatically resolved" (p.2; abstract says "at least 57%"). Gradle and Maven are used in 174 of 200. — [Hassan et al. 2017](https://foyzulhassan.github.io/files/ESEM_2017_1.pdf)
- CONFIRMED. Tufano et al. 2017: 219,395 snapshots of 100 Apache Java projects using Maven; "broken snapshots occur in most (96%) of the projects"; "On average, only 38% of the change history of project systems is currently successfully compilable" (summary, p.1); 58% of errors arise "during the Resolution of an artifact", with DependencyResolutionException 32% and ArtifactResolutionException 26% (Section on causes, p.10). — [Tufano et al. 2017](https://www.cs.wm.edu/~denys/pubs/JSEP17.ThereAndBack.pdf)
- CONFIRMED (abstract only). Robles et al., EMSE 2022, replication of Tufano in 2020: 79 repositories and 139,389 commits, plus 80 other Java projects and 300,873 commits; "We observed degradation of compilability over time, due to vanishing of dependencies and other external artifacts." — [Robles et al. 2022](https://link.springer.com/content/pdf/10.1007/s10664-022-10117-6.pdf)
- CONFIRMED (abstracts only). Python, snippets not repositories: Gistable (2018), "75.6% of gists require non-trivial configuration"; DockerizeMe (ICSE 2019), "over 50% of public Python gists fail due to an import error for a missing library", and its inference fixes import errors in 892 of nearly 3,000 hard gists. — [arXiv 1808.04919](https://arxiv.org/abs/1808.04919), [arXiv 1905.11127](https://arxiv.org/abs/1905.11127)

### Inferences
- Agent success rates in section 5 should be read against these baselines: about 40 to 60% of random Java repositories do not build with the default command, and about a third fail under any JDK tried.
- Indirect evidence for other ecosystems comes only from agent benchmarks: Multi-Docker-Eval and RepoLaunch rank Go, Python and JavaScript as easier than C/C++ and Java, with different metrics.

### Gaps
- NOT FOUND: a large random-sample buildability study for Python, JavaScript, Go or Rust repositories comparable to Sulír's. One arXiv title search and the related-work sections read turned up none.
- The EMSE 2026 per-category failure percentages were not extracted.

## 7. Partial-program analysis accuracy

### Takeaway
PPA's figures are CONFIRMED: 91.2% correct and 2.7% erroneous type facts on average with one class visible. JCoffee and LExecutor give the nearest later equivalents, neither LLM-based in the prompted sense.

### Cited Findings
- CONFIRMED. PPA (OOPSLA 2008), four systems analysed one class at a time without dependencies: "on average, Partial Program Analysis could uncover 91.2% of correct type facts when analyzing one class at a time and only produced 2.7% of erroneous type facts" (conclusion, p.14). Table 4 (p.11), full configuration, correct and erroneous: Lucene 93.80% and 2.71%; JFreeChart 94.40% and 0.85%; Jython 88.22% and 3.13%; Spring 90.97% and 3.72%; unknown 2.46% to 6.21%. Baseline without inference: correct 81.20% to 89.23%. — [PPA paper](https://www.sable.mcgill.ca/ppa/oopsla2008.pdf)
- CONFIRMED (introduction only). JCoffee (ICSME 2020): uses compiler feedback to make partial Java snippets compilable; "An evaluation of our tool on 9133 partial code snippets. JCoffee successfully disambiguated over 90% of the inputs, with an overall accuracy of 93.3%" (p.1). — [arXiv 2009.05090](https://arxiv.org/abs/2009.05090)
- CONFIRMED (abstract and Section 3 only). LExecutor (FSE 2023), Python: fine-tuned CodeT5 and CodeBERT predict missing values; "The neural model predicts realistic values with an accuracy between 79.5% and 98.2%"; line coverage rises from 4.1% executing as-is to 51.6%. — [arXiv 2302.02343](https://arxiv.org/abs/2302.02343)
- NOT FOUND. A tool named "Incompleter". No paper of that name was located or opened.

### Inferences
- PPA's erroneous rate (0.85% to 3.72% per system) is a non-LLM reference point for "inferred without the whole program": a few wrong facts in a hundred, undetectable without the missing code.
- JCoffee is an existing instance of the accept-on-compiler-agreement pattern: an inferred declaration is kept only if the compiler then accepts the snippet.

### Gaps
- No static partial-program type recovery study for Python with correct and erroneous rates was found.

## 8. Framework modelling gain

### Takeaway
JackEE's claim is CONFIRMED from the paper: in-application reachability averages 58.04% against 14.48% for plain Doop. Jasmine's gain over JackEE is stated in tables; the share of spurious edges among added edges is up to about 20% per benchmark.

### Cited Findings
- CONFIRMED. JackEE (PLDI 2020): "our techniques achieve high degrees of completeness (on average more than 4x higher than conventional techniques) and speedups of about 6x compared to the most precise conventional analysis" (abstract). "JackEE's analysis averages in-app reachability of 58.04%, while dropping to no less than 43.48% (for alfresco). In comparison, Doop averages 14.48% in-app coverage while dropping to approximately 1.8% and 0.0% coverage for two benchmarks (alfresco and pybbs respectively)" (evaluation, p.11). — [JackEE paper](https://yanniss.github.io/enterprise-pldi20.pdf)
- CONFIRMED. Jasmine (ASE 2022): evaluated on Spring micro-benchmarks and 18 real-world Spring programs; Jasmine facts fed to JackEE "achieve a remarkable increase in the number of reachable methods and calling edges compared to native JackEE" (p.9, Table III). Table IV gives, per benchmark, the share of added application edges by cause and the share that is spurious: pybbs 19.98% spurious, shopizer 13.07%, SpringBlog 7.89%, WebGoat 0.00%, mall-admin 4.79%, FEBS-Cloud-auth 20.04%, FEBS-Cloud-system 20.77%. — [Jasmine paper](https://weihang-wang.github.io/papers/ASE2022-Jasmine.pdf)

### Inferences
- Jasmine's Table IV is a rare published false-edge rate for declarative framework modelling: 0 to about 21% of the added edges per application.

### Gaps
- A single aggregate percentage for Jasmine's gain in reachable methods is not stated in the text extracted; Table III numbers were garbled by PDF extraction and are not quoted.

## 9. Verification of LLM output by a deterministic checker

### Takeaway
Documented designs exist in four forms: static rejection rules over neural type predictions (HiTyper, TypeGen plus HiTyper), a type checker in the loop (PyTIR, TypyBench's metric, TypeT5's error count), a verifier that must prove an LLM-written specification (SpecGen), and symbolic rules that bound and check an LLM's call-target decision (Facet). Acceptance rates are published; false-acceptance rates almost never are.

### Cited Findings
- CONFIRMED. HiTyper: "type rejection rules to inspect the correctness of neural predictions"; static inference and neural prediction alternate "until the TDG reaches a fixed point" (abstract; Section 3.2, p.4). Effect measured through TypeGen (Table V, p.8 of the TypeGen paper): top-5 TypeGen predictions filtered by HiTyper give 88.3% exact match against 79.2% top-1 without the filter and 87.0% top-5 without a selector; arguments 84.9% against 73.1% top-1. — [arXiv 2105.03595](https://arxiv.org/abs/2105.03595), [arXiv 2307.09163](https://arxiv.org/abs/2307.09163)
- CONFIRMED. TypeT5 measures coherence by type-checker error count, not by acceptance: full method 73.02% accuracy with 5,087 type errors; independent decoding 71.68% with 6,876; without preamble context 64.20% with 6,067 (Tables 4 and 5, p.8). Only errors "directly related to type coherence" are counted, since the checker "also reports errors unrelated to type coherence, such as unresolved import statements". — [arXiv 2303.09564](https://arxiv.org/abs/2303.09564)
- CONFIRMED. TypyBench shows what a type checker catches after LLM inference: mean mypy errors per repository 127.1 (claude-3.5-sonnet) to 465.7 (llama-3.1-8b) against 141.8 for human annotations (Table 2, p.7). — [arXiv 2507.22086](https://arxiv.org/abs/2507.22086)
- CONFIRMED (abstract and introduction only). PyTIR: "a type-checker-in-the-loop strategy that validates and corrects inferences on-the-fly"; introduced type errors reduced by 92.7% to "an average of only 46.17 errors per repository". — [arXiv 2512.21591](https://arxiv.org/abs/2512.21591)
- CONFIRMED (abstract and p.6 only). SpecGen (ICSE 2025): LLM-written JML specifications are kept only if the verifier accepts them; "SpecGen succeeds in generating verifiable specifications for 279 out of 385 programs"; model "gpt-3.5-turbo-1106", temperature 0.4, up to 10 conversation rounds; failed specifications are mutated and re-verified. Derived acceptance rate: 72.5% of programs. — [arXiv 2401.08807](https://arxiv.org/abs/2401.08807)
- CONFIRMED. Facet: candidates are bounded symbolically "to the address-taken functions of the site's signature category", the LLM answers keep-or-prune per candidate, "the symbolic rules check each decision and determine whether it is adopted", and the tool abstains at sites with at most 2 symbolic targets (Sections 3.4, pp.8 to 9). Every adopted edge decision carries a "warrant" naming the rule, the evidence checked and the unverified premise (Section 3.3, p.7). — [arXiv 2609.33547](https://arxiv.org/abs/2609.33547)
- CONFIRMED. LLMDFA: the LLM does not judge path feasibility itself; it synthesises a script that invokes "an automated theorem prover to validate path feasibility" (abstract). — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754)
- CONFIRMED. JCoffee: compiler acceptance as the check for inferred declarations, 93.3% overall accuracy on 9,133 snippets. — [arXiv 2009.05090](https://arxiv.org/abs/2009.05090)

### Inferences
- A checker that accepts is not a checker that confirms. Type-checker acceptance means "consistent with the visible code", and TypyBench shows human annotations themselves score 141.8 errors per repository, so zero errors is neither necessary nor sufficient for correctness. SpecGen's verifier accepts specifications that hold but may be weak. The literature publishes acceptance (72.5% of programs for SpecGen, 88.3% exact match after HiTyper filtering) and leaves false acceptance unmeasured.
- The designs with the best-supported outcomes share three properties: the candidate set comes from the deterministic side, the LLM chooses and does not invent, and the tool records the basis of each accepted fact (Facet's warrant) or abstains.
- For `mentalc` this argues for a check that re-derives the proposed target from source (the symbol exists at the stated location, its signature fits the call site, the proposed receiver type has that member) and for measuring false acceptance directly, since no published rate can be borrowed.

### Gaps
- NOT FOUND: a measured false-acceptance rate for any "LLM proposes, deterministic checker accepts" design in type or call resolution. HiTyper's paper does not report how often its rejection rules pass a wrong type, in the sections read.
- NOT FOUND: a design that verifies LLM claims by quoting source anchors and re-matching them, with numbers, in the software-analysis literature read here.
