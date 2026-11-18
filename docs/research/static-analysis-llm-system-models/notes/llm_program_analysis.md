# LLM-based and Neuro-symbolic Program Analysis and Repository-level Code Understanding (state as of Nov 2026)

Note on model generations: most peer-reviewed quantitative results below were measured on GPT-3.5 / GPT-4 / GPT-4o / Claude 3.5-era models (2023-2025). As of Nov 2026 the frontier is the Claude 5.x / GPT-5.x generation. Absolute numbers from older papers are lower bounds on today's raw model capability, but the *architectural* lessons (where LLMs fail, what verification buys) have held up across generations. Each finding is tagged with its model generation where known. Most numbers are self-reported by the authors; independent replications are rare.

## 1. LLM-powered data-flow / taint analysis: accuracy and failure modes

### Takeaway

Pure "LLM as analyzer" is unreliable. Systems that decompose the problem (LLM handles small local semantic judgments such as source/sink identification, per-function flow summaries, or intent) and hand the global reasoning (path feasibility, transitive propagation, call graph) to a symbolic engine reach roughly 65-87% precision and 60-80% recall on benchmarks, and find real bugs that classic tools miss. The LLM's single best-proven job is **inferring specifications** (taint sources/sinks, library flow summaries, resource-acquire/release intent) that a classic analyzer then consumes.

### Cited Findings

#### LLMDFA (NeurIPS 2024; gpt-3.5-turbo-0125 / GPT-4 era)

- Decomposes dataflow analysis into three LLM-driven sub-tasks: source/sink extraction, per-function dataflow summarization, and path-feasibility validation; path conditions are discharged by an external SMT solver, and the LLM writes code (scripts) that call parsing libraries rather than answering free-form — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754)
- Average precision 87.10% and recall 80.77% across evaluations; on Juliet (synthetic) with gpt-3.5: DBZ 73.75% P / 92.16% R, XSS 100% P / 92.31% R, OS command injection 100% P / 78.38% R — [arXiv 2402.10754v2](https://arxiv.org/html/2402.10754v2)
- On real-world Android malware (TaintBench): 74.63% precision, 60.24% recall; F1 improvements of 0.12 and 0.35 over the CodeFuseQuery classic analyzer and an end-to-end few-shot CoT LLM baseline respectively — i.e., end-to-end prompting is clearly worse than decomposition — [arXiv 2402.10754v2](https://arxiv.org/html/2402.10754v2)

#### LLMSA / NESA (Purdue; arXiv Dec 2024, journal version renamed NESA, DOI 10.1145/3808161)

- Users write a restricted Datalog-style policy whose relations are either *symbolic* (computed by parsing: syntactic facts) or *neural* (computed by LLM prompts: semantic facts); evaluated with lazy, incremental and parallel prompting to reduce hallucination; compilation-free — [arXiv 2412.14399](https://arxiv.org/html/2412.14399v1); [NESA listing](https://www.citedrive.com/en/discovery/nesa-relational-neuro-symbolic-static-program-analysis/)
- Taint vulnerability detection: 66.27% precision, 78.57% recall, F1 +0.20 over an industrial approach; clients include alias analysis, program slicing, bug detection (Java prototype) — [arXiv 2412.14399](https://arxiv.org/pdf/2412.14399)

#### IRIS (ICLR 2025; GPT-4 + 6 other LLMs)

- LLM infers CWE-specific taint specifications (sources/sinks for library APIs) that CodeQL consumes; a second LLM stage classifies CodeQL paths as true/false positives using contextual prompts — [ICLR 2025 proceedings](https://proceedings.iclr.cc/paper_files/paper/2025/hash/582d4e27fa24168f3af1f4582655034b-Abstract-Conference.html)
- On CWE-Bench-Java (120 manually validated real-world Java vulns): CodeQL alone detects 27, IRIS+GPT-4 detects 55 (+28); found 4 previously unknown vulns in 30 projects (v2 of the preprint said 6 and 8 LLMs; v3/published say 4 and 7) — [arXiv 2405.17238v3](https://arxiv.org/html/2405.17238v3)
- Motivating failure of classic analysis: CodeQL misses cron-utils CVE-2021-41269 purely because of a missing library specification — [arXiv 2405.17238v3](https://arxiv.org/html/2405.17238v3)

#### SemTaint (Jan 2026; multi-agent, JavaScript/npm, CodeQL)

- Explicitly "static analysis as the source of truth"; LLM agents only where static reasoning falls short. Three agents: Source/Sink agent, CallGraph agent (resolves unresolved calls; emits candidate flow summaries for third-party calls), Flow Summary agent (checks only summaries that appear on reported paths; decides propagate vs sanitize) — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- Validation: every proposed fact must **bind to a CodeQL data-flow node** or it is rejected; each agent runs 3 times (union for sources/sinks and call edges, majority vote for flow summaries); calibrated confidence rubrics; outputs are CSV relations read by CodeQL as external predicates (no query recompilation) — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- Detects 106/162 (65.43%) vulns that CodeQL misses by construction; validation-set precision 0.80 (30 alerts, 6 FP); ~73% of detections needed only custom sources/sinks, ~27% also needed call-graph repair; 4 novel vulns from 34 alerts on 10 OSS packages (13 TP; 21 = sanitizer-model issues or intended behavior) — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- Demand-driven LLM use (Taint-Informed Callee Resolution) cut unresolved-call candidates from 94,909 to 10,184 and agent calls by 94.5% (585.9 -> 32.1 per package) — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- Reported failure modes: LLM nondeterminism; a flow-summary agent wrongly marking a call as sanitizing (causing a miss); summaries don't model taint through callbacks; packages >1M tokens excluded — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)

**QLPro (mid-2025)**: extracts taint specs with CodeQL, then uses LLM voting to classify them before generating scanning rules; argues rule quality is bounded by spec quality — [arXiv 2506.23644](https://www.arxiv.org/pdf/2506.23644)

#### LLift (OOPSLA 2024; GPT-4 era)

- Uses an LLM to resolve the ~40% of UBITect (use-before-initialization, Linux kernel) warnings that the static analyzer left undecided; ~50% precision with no missed known bugs; 13 previously undiscovered bugs on ~1000 cases, 4 confirmed by Linux maintainers — [OOPSLA'24 paper PDF](https://www.cs.ucr.edu/%7Ezhiyunq/pub/oopsla24_llift.pdf); [arXiv 2308.00245](https://arxiv.org/abs/2308.00245v3)
- Case-count discrepancy: a secondary summary reports 818 cases / 26 potential / 13 confirmed — [alphaxiv overview](https://www.alphaxiv.org/overview/2308.00245)

#### InferROI (ICSE 2025; GPT-4 era)

- LLM infers resource-oriented *intentions* (acquire, release, reachability validation) without predefined API pairs; a lightweight two-stage static analysis then checks CFG paths for leaks (Java) — [arXiv 2311.04448v4](https://arxiv.org/html/2311.04448v4)
- Intention inference precision 74.6%, recall 81.8% (19 Android resource types in DroidLeaks, 221 in JLeaks); leak detection rates 59.3% (DroidLeaks) / 62.5% (JLeaks) with false-alarm rates 18.6% / 19.5%; ablation shows LLM inference + static analysis together are needed — [ICSE 2025 listing](https://conf.researchr.org/details/icse-2025/icse-2025-research-track/144/Boosting-Static-Resource-Leak-Detection-via-LLM-based-Resource-Oriented-Intention-Inf); [arXiv 2311.04448v4](https://arxiv.org/html/2311.04448v4)

#### RepoAudit (ICML 2025)

- Agent with initiator / explorer / validator; explores repository on demand, function by function, tracking data-flow facts along feasible paths; validator checks data-flow facts for hallucination and checks path-condition satisfiability — [arXiv 2501.18160](https://arxiv.org/abs/2501.18160)
- 40 true bugs across 15 real-world projects at 78.43% precision; avg cost $2.54 and 0.44 h per project; 185 new bugs in high-profile projects, 174 confirmed or fixed (NPD, memory leak, UAF) — [arXiv 2501.18160](https://arxiv.org/abs/2501.18160)

#### LLM call-graph and type reasoning (Empirical Software Engineering 2025; 24 LLMs incl. GPT-4o)

- Classic tools beat all LLMs on call graphs: Python PyCG 84.9% completeness / 87.3% soundness vs best LLM (mistral-large-2407) 60.3% / 62.6%; Jelly beats LLMs on JS; even GPT-4o struggles with completeness and soundness — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- The reverse holds for type inference: LLMs beat HeaderGen and hybrid HiTyper on Python type inference — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603); benchmarks SWARM-CG / SWARM-JS released — [TU Delft repository](https://repository.tudelft.nl/record/uuid:3da1db02-b4ca-4eba-92ff-9043560209d2)

#### LLM false-positive triage of static analysis alerts (2025-2026)

- Agent frameworks (Aider, OpenHands, SWE-agent; Claude Sonnet 4) filtering CodeQL alerts reach up to 93.3% FP-identification rate in the best configuration; benefit is strongly backbone- and CWE-dependent — [arXiv 2601.22952](https://arxiv.org/html/2601.22952v1)
- Tencent industrial study: hybrid LLM techniques eliminate 94-98% of false positives with high recall; cites prior findings that CodeQL and Infer have >95% false-alarm rates for null-pointer dereference in large projects; documents limitations in industrial settings — [arXiv 2601.18844](https://arxiv.org/html/2601.18844v1)
- ZeroFalse: CWE-specialized, reasoning-oriented prompting over CodeQL alerts improves precision while keeping recall (OWASP Java Benchmark, OpenVuln) — [arXiv 2510.02534](https://arxiv.org/html/2510.02534v1)
- SAST-Genius (Semgrep + fine-tuned LLM): 89.5% precision vs 35.7% for Semgrep and 65.5% for GPT-4 alone across 25 projects — [arXiv 2509.15433](https://arxiv.org/pdf/2509.15433v3)

#### Industry, 2026 frontier models

- Claude Code Security (Feb 2026): Claude re-examines its own findings and tries to disprove them; each finding gets a confidence rating and a multi-stage verification pass before an analyst sees it — [Anthropic](https://www.anthropic.com/news/claude-code-security)
- Opus 4.6 open-source bug finding: each finding validated in a controlled virtualized environment and then by human researchers to weed out hallucinations and false positives — [eSecurity Planet](https://www.esecurityplanet.com/threats/claude-opus-4-6-exposes-hundreds-of-open-source-vulnerabilities/); [SC World](https://scworld.com/news/anthropic-latest-claude-model-finds-more-than-500-vulnerabilities)
- Mythos Preview / Glasswing: >1,000 OSS projects, 23,019 findings; of 1,900 reviewed by external firms, 1,726 confirmed valid (~91%); only 97 patched — bottleneck is triage and remediation, not discovery — [The Decoder](https://the-decoder.com/anthropic-warns-claude-mythos-preview-finds-bugs-faster-than-developers-can-patch-them/) (secondary source)
- OSS Scanner (Oct 2026): pen-testers checked 97 critical/high findings across 48 projects, 1 invalid; only ~6,000 of ~29,000 candidates manually triaged — [Security Affairs](https://securityaffairs.com/200685/ai/claude-helps-secure-open-source-as-anthropic-offers-free-vulnerability-scanning.html) (secondary, fragmentary; verify against Anthropic posts)

### Inferences

- The consistent failure locus for LLMs is *global, transitive, exhaustive* reasoning (call-graph completeness, soundness over all paths); the success locus is *local semantic judgment and naming* (is this API a sink, does this function release a resource, what does this flow mean). This matches the checkpoint's proposed split (static tools ground; LLM resolves dispatch/config/intent) almost exactly.
- Precision in the 65-87% range on benchmarks means that, for the mental-model tool, roughly 1 in 5-8 LLM-inferred steps can be wrong even in well-engineered pipelines (older models). Provenance + anchors are therefore not optional; the tool needs a cheap way to check claims mechanically.
- SemTaint's "fact must bind to an analyzer node" rule is directly transferable: every script step's anchor should resolve to a real symbol / AST node / SCIP occurrence, or the step is rejected or downgraded to `assumed`.
- Demand-driven LLM invocation (TICR, LLMSA lazy prompting, RepoAudit on-demand exploration) is what keeps cost tractable; it maps to the checkpoint's lazy-generation-on-zoom.
- Frontier 2026 models raise raw accuracy (e.g., ~91% validity in Anthropic's externally-reviewed sample), but every production system still wraps them in adversarial self-review plus execution or human validation.

### Gaps

- No found study measuring LLM *pointer/points-to analysis* accuracy directly; call-graph studies are the closest proxy.
- No peer-reviewed re-run of LLMDFA/IRIS/LLMSA on 2026 frontier models; how much the decomposition advantage shrinks with stronger models is unknown.
- Anthropic's 2026 accuracy figures come from secondary press; primary posts not opened in this session.

## 2. Neuro-symbolic patterns: LLM proposes, tools verify

### Takeaway

The proven patterns are: (a) LLM emits *structured facts/specs* consumed by a symbolic engine (IRIS, SemTaint, InferROI, LLMSA); (b) symbolic pre-filter, LLM resolves undecided cases (LLift); (c) LLM-proposed path conditions checked by SMT (LLMDFA, RepoAudit); (d) LLM writes code/scripts instead of answering (LLMDFA); (e) multi-sample self-consistency / voting (SemTaint, QLPro); (f) adversarial self-refutation and execution-based validation (Anthropic). Agentic exploration with code tools is the default integration surface.

### Cited Findings

- LLMDFA has the LLM synthesize scripts that call external parsing/analysis libraries and uses an SMT solver for path feasibility, reducing hallucination versus free-form answers — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754)
- LLMSA separates symbolic (parser) and neural (LLM) relations in one Datalog policy; lazy prompting delays LLM calls until the analysis has narrowed the values of interest — [arXiv 2412.14399](https://arxiv.org/html/2412.14399v1)
- SemTaint: bind-to-analyzer-node validation, 3-run union/majority vote, calibrated confidence rubrics, facts exported as external predicates — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- RepoAudit's validator re-checks data-flow facts and path-condition satisfiability to discard hallucinated bugs — [arXiv 2501.18160](https://arxiv.org/abs/2501.18160)
- LLift only invokes the LLM on cases the static analyzer marked undecided (~40% of UBITect warnings) — [OOPSLA'24](https://www.cs.ucr.edu/%7Ezhiyunq/pub/oopsla24_llift.pdf)
- QLPro uses LLM voting over CodeQL-extracted taint specs — [arXiv 2506.23644](https://www.arxiv.org/pdf/2506.23644)
- Claude Code Security: model attempts to disprove its own findings, attaches a confidence rating; multi-stage verification — [Anthropic](https://www.anthropic.com/news/claude-code-security)
- InferROI ablation: neither LLM inference alone nor static analysis alone matches the combination — [arXiv 2311.04448v4](https://arxiv.org/html/2311.04448v4)
- Agent backbones matter more than scaffolds for FP filtering; gains are CWE-dependent — [arXiv 2601.22952](https://arxiv.org/html/2601.22952v1)

### Inferences

- For the mental-model tool, a natural mapping: tree-sitter/SCIP/LSP (and optionally CodeQL/Joern) produce the *symbolic relations* (call edges where resolvable, def-use, routes, config reads); the LLM produces *neural relations* (resolved dynamic-dispatch targets, which config value applies, branch-condition-to-input-class labels, abstraction names); every neural fact is anchored and re-checked by a symbolic query where possible (e.g., "does a call edge from A to B exist in the index", "does this config key exist").
- Branch conditions on a scenario graph can be stored as constraints and checked for satisfiability (LLMDFA/RepoAudit style) to catch infeasible LLM-proposed scripts.
- Self-consistency (k runs, vote) is a cheap hallucination detector for lazily-generated deep levels: disagreement between runs is itself a signal to mark a step `assumed`.

### Gaps

- No study found that evaluates LLM-generated *behavioral* summaries (as opposed to taint specs) being checked by tools; the transfer is inferred.

## 3. Repository-level understanding: code graphs, retrieval vs long context, benchmarks

### Takeaway

Structure-aware retrieval (code graphs, repo maps) plus agentic navigation beats both naive embedding RAG and "stuff the repo into context". Benchmarks show the bottleneck is multi-hop state propagation, not context length, and popular benchmarks (SWE-bench Verified) are contaminated, so headline scores overstate genuine repo understanding.

### Cited Findings

- LocAgent parses a repo into a directed heterogeneous graph (files, classes, functions; contain/import/invoke/inherit edges) navigated by an agent with multi-hop traversal; with Claude-3.5: 94.16% file-level Acc@5, 87.59% module Acc@10, 77.37% function Acc@10 on SWE-Bench-Lite; fine-tuned Qwen-2.5-Coder-32B reaches 92.70% file-level at ~86% lower cost; introduced Loc-Bench because of SWE-bench contamination risk (ACL 2025) — [arXiv 2503.09089](https://arxiv.org/html/2503.09089); [ACL 2025 PDF](https://preview.aclanthology.org/iwcs-25-ingestion/2025.acl-long.426.pdf)
- In LocAgent's taxonomy, CodexGraph uses a graph database queried via Cypher and RepoGraph uses ego-graph retrieval — [ACL 2025 PDF](https://preview.aclanthology.org/iwcs-25-ingestion/2025.acl-long.426.pdf); CodexGraph — [arXiv 2408.03910](https://arxiv.org/abs/2408.03910); RepoGraph (ICLR 2025, plug-in line-level graph for existing SWE agents) — [arXiv 2410.14684](https://arxiv.org/abs/2410.14684)
- Aider's repo map: tree-sitter extracts definitions/references, a graph-ranking algorithm selects the most relevant symbols to fit a token budget — [Aider docs](https://aider.chat/docs/repomap.html)
- Cursor reports semantic search + grep gives ~12.5% higher accuracy on codebase questions than grep alone (6.5%-23.5% across models) — [Cursor blog](https://cursor.com/blog/semsearch); a vendor (Morph) claims larger gains on 1,000+ file repos (52.3% grep, 66.1% semantic, 81.7% both) — [Morph blog](https://morphllm.com/blog/semantic-search) (vendor, unverified)
- Claude Code and Cline do not index; they rely on agentic search — [Morph blog](https://morphllm.com/blog/semantic-search) (secondary)
- "SWE-Bench Illusion": models name the buggy file from issue text alone (no repo access) with ~76% accuracy on SWE-bench Verified vs ~53% on repos outside SWE-bench, consistent with memorization/contamination — [arXiv 2506.12286](https://arxiv.org/html/2506.12286v4)
- SWE-bench Pro at launch (Sept 2025): best Pass@1 23.3% (GPT-5); private subset shows a further drop — [arXiv 2509.16941](https://arxiv.org/html/2509.16941v1); 2026 aggregator figures conflict wildly (59.1% GPT-5.4 in June 2026 vs ~90% for Claude Opus 5.5 in Oct 2026) and depend on scaffold/split — [Capital and Compute](https://capitalandcompute.net/ai-benchmarks/swe-bench-pro/); [BenchLM](https://benchlm.ai/benchmarks/swe-bench-pro) (aggregators, unverified)
- RepoQA: "search needle function" in long contexts, framed as more realistic than needle-in-a-haystack — [arXiv 2406.06025](https://arxiv.org/pdf/2406.06025)
- Long Code Arena: six project-wide tasks incl. bug localization and module summarization — [arXiv 2406.11612](https://arxiv.org/html/2406.11612v1)
- CoReQA: both long-context models ingesting the whole repo and short-context models with generic retrieval struggle with repo-level QA; calls for more precise retrieval — [arXiv 2501.03447](https://arxiv.org/html/2501.03447v1)
- RepoReasoner (2026): bottleneck is synthesizing and propagating state across multiple hops, not context length; better retrieval helps more than bigger windows — [alphaxiv overview 2607.25996](https://www.alphaxiv.org/overview/2607.25996) (secondary)
- RepoBench: effective cross-file retrieval substantially improves completion — [alphaxiv RepoBench](https://www.alphaxiv.org/benchmarks/university-of-california-san-diego/repobench)

### Inferences

- The tool should own a persistent code graph (SCIP/LSP + tree-sitter + cross-repo boundary links) and expose it to the agent as tools (graph query, go-to-definition, find-references, grep), rather than relying on the agent re-discovering structure each session — this is also the checkpoint's "shared pointer" thesis.
- Because benchmark scores are contaminated, the project should build its own eval on repos *not* in training data (or recent commits), with a hand-written gold script (checkpoint build step 1) as ground truth.
- Multi-hop failure (RepoReasoner) argues for decomposing a script into per-hop generation with explicit hand-off of state (data_in/data_out shapes) rather than asking one prompt to trace an end-to-end request.

### Gaps

- No head-to-head numbers comparing RepoGraph vs CodexGraph vs LocAgent on the same benchmark were found.
- No rigorous public study of Claude Code's agentic search vs indexed retrieval was found; claims are secondary.
- No benchmark specifically evaluates *cross-repo / multi-service* tracing (client fetch -> server route -> queue); this is a gap the tool would be stepping into.

## 4. Abstraction and summarization: hierarchical summaries and faithfulness

### Takeaway

Hierarchical bottom-up summarization (functions -> modules -> repo) is the standard architecture and measurably beats flat approaches, but quality scores are still modest (~65-70% rubric coverage), and code summaries hallucinate at non-trivial rates, especially on unfamiliar or perturbed code. Entity-level tracing (checking every named identifier in a summary against code) is the best-documented detection technique.

### Cited Findings

- CodeWiki: hierarchical documentation where parent modules are synthesized from child docs; CodeWikiBench scores against rubrics derived from official developer docs; CodeWiki 68.79% vs DeepWiki 64.06% quality score (ACL 2026 Findings) — [arXiv 2510.24428](https://arxiv.org/html/2510.24428v5); [ACL Anthology](https://preview.aclanthology.org/ingest-acl/2026.findings-acl.288/)
- RepoAgent aggregates function/class docs into repo structure via dependency analysis (described in CodeWiki) — [arXiv 2510.24428](https://arxiv.org/pdf/2510.24428)
- Agent4cs: multi-agent bottom-up files -> folders -> repo summarization, four metrics for hierarchical summaries, GPT-5 as judge — [arXiv 2607.01425](https://arxiv.org/pdf/2607.01425)
- RepoSummary: feature-oriented summarization; LLM-judge for feature coverage correlates with humans at Spearman 0.590-0.839 — [Moonlight review](https://www.themoonlight.io/en/review/reposummary-feature-oriented-summarization-and-documentation-generation-for-code-repositories) (secondary)
- ETF (ACL 2025): CodeSumEval with 411 summaries from 7 LLMs (~10K entity-level samples); entity tracing (extract entities mentioned in a summary, verify each against code) reaches 73% F1 for hallucination detection — [arXiv 2410.14748](https://arxiv.org/html/2410.14748v4)
- Hallu-Eval / Hallu-Shield (FSE 2026): perturbing code raised Qwen2.5-Coder-7B summary hallucination from 17% to 97%; Hallu-Shield uses an external value model at inference to steer to faithful summaries — [FSE 2026 listing](https://conf.researchr.org/details/fse-2026/fse-2026-research-papers/66/Hallucinations-in-LLM-based-Code-Summarization-Unveiling-Detection-and-Mitigation)
- Code hallucination taxonomy splits knowledge vs faithfulness hallucinations — [arXiv 2404.00971](https://arxiv.org/pdf/2404.00971v3)

### Inferences

- The 17% -> 97% perturbation result implies LLMs lean on *priors from names and familiar patterns*: code that looks like a well-known library but behaves differently (custom forks, unusual config) is exactly where generated behavior will be confidently wrong. Behavior packs for popular libraries are thus higher-risk when the user's version or fork deviates; packs should be keyed by exact version + content hash.
- Entity tracing is cheap to implement on top of SCIP: every identifier, file, route, or config key named in a step title/summary must resolve in the index; unresolved names are hallucination candidates.
- Summaries at higher levels should carry the union of children's assumptions (the checkpoint's "collapsed segment never hides an assumption" rule), since hierarchical summarization compounds child errors.

### Gaps

- No study found that measures hallucination rates in *architecture/intent* extraction specifically (most work is function-level summaries or doc coverage).
- No found evaluation of how frontier 2026 models score on ETF/Hallu-Eval.

## 5. Extracting behavioral models and predicting execution

### Takeaway

Frontier models nearly saturate simple single-function output prediction (CRUXEval ~99%), but are brittle under semantics-preserving perturbation and much weaker at *intermediate-state* and *execution-path* prediction, which is what a script needs. Direct code-to-state-machine/sequence-diagram generation is under-evaluated and shows hallucinated transitions. Tracing a specific path should be grounded by actual execution or symbolic checks wherever possible.

### Cited Findings

- CRUXEval (2024): GPT-4 pass@1 67% (input prediction) / 63% (output prediction) on short Python functions — [arXiv 2401.03065](https://arxiv.org/abs/2401.03065) (superseded generation)
- 2026 preprint: GPT-5.2 scores ~99% on original CRUXEval but drops 20-24 points on perturbed (semantics-preserving transformed) inputs — reported via search for [arXiv 2604.16320](https://arxiv.org/abs/2604.16320) (not opened; verify)
- SX-Bench argues CRUXEval-style single-function benchmarks are saturated and measures step counting across multi-function programs — [arXiv 2508.05193](https://arxiv.org/abs/2508.05193)
- CoRE (ACL 2026 Findings): output-only evaluation is insufficient; tracking intermediate execution states across equivalent implementations lags significantly behind output prediction — [ACL Anthology](https://aclanthology.org/2026.findings-acl.460.pdf)
- REval: four tasks (coverage, program state, execution path, output) plus Incremental Consistency; GPT-4-Turbo avg 75.0%, most open models <50%; execution-path prediction hardest at ~19.4% average — [arXiv 2403.16437](https://arxiv.org/pdf/2403.16437); leaderboard figures from [alphaxiv](https://alphaxiv.org/benchmarks/zhejiang-university/reval) (secondary)
- CodeSense (ICLR 2026): ground truth from real-project test traces (Python, C, Java); clear gap on fine-grained semantic reasoning — [arXiv 2506.00750](https://arxiv.org/abs/2506.00750v2)
- StepCodeReasoner: inserting print-based trace anchors and training on stepwise traces reaches 82.9% on REval vs 77.3% GPT-4o — [HF papers 2605.11922](https://huggingface.co/papers/2605.11922.md)
- Reverse engineering + LLM to produce component state machines from C++: manual scheme-based evaluation (start/end states, completeness, naming, self-transitions, parallelism); hallucinated transitions observed — [arXiv 2511.05165](https://arxiv.org/pdf/2511.05165)
- Code2UML: agentic pipeline converting code IRs into diagram-specific views before generation, 12 repos, 7 diagram types — [alphaxiv 2605.24453](https://www.alphaxiv.org/abs/2605.24453.md)
- MermaidSeqBench evaluates NL-to-sequence-diagram (not code-to-diagram), LLM-as-judge, significant gaps across models — [arXiv 2511.14967](https://arxiv.org/pdf/2511.14967)

### Inferences

- For the "script = one path chosen by binding input classes" design: the LLM is reliable at *labeling* branches with input classes and naming steps, less reliable at *computing* which branch a given concrete input takes deep in a call chain. Prefer: static analysis enumerates branch points and conditions (the scenario graph skeleton); LLM labels conditions as input classes; path selection is done by constraint matching, with the LLM only for conditions the analyzer can't evaluate (marked `assumed`).
- Producing a state machine via an intermediate IR (Code2UML-style) rather than directly from text is the better pattern; the scenario graph itself is that IR.
- Producer 2 (observed traces) is the strongest available verifier for producer 1; even a single test-run trace per scenario can confirm or contradict LLM path predictions (CodeSense shows such traces are collectible at scale).

### Gaps

- No benchmark found for LLM prediction of cross-service request paths or async/concurrent interleavings.
- No quantitative evaluation found of LLM-generated sequence diagrams *from source code* against ground truth.

## 6. Cost, latency, caching and incremental strategies

### Takeaway

Published per-repository costs for well-designed agentic analyses are low (RepoAudit ~$2.54 and ~26 minutes per project on 2024-25 models), and demand-driven invocation cuts LLM calls by ~90%+. Academic work on incremental/cached LLM analysis is thin; industrial practice is content-hash-keyed memoization of per-unit summaries plus re-aggregation.

### Cited Findings

- RepoAudit: $2.54 and 0.44 h per project on average — [arXiv 2501.18160](https://arxiv.org/abs/2501.18160)
- SemTaint's taint-informed callee resolution: unresolved calls 94,909 -> 10,184; agent calls -94.5%; third-party summary candidates -93.2% (6,639 -> 449) — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- LLMSA uses lazy, incremental and parallel prompting — [arXiv 2412.14399](https://arxiv.org/html/2412.14399v1)
- LocAgent: fine-tuned 32B open model matches proprietary localization at ~86% lower cost — [arXiv 2503.09089](https://arxiv.org/html/2503.09089)
- CocoIndex: memoize a function's LLM output while its inputs and code are unchanged; a prompt change invalidates all cached results; per-file LLM calls + one aggregation call per project, re-running only changed files — [CocoIndex blog](https://cocoindex.io/blogs/multi-codebase-summarization/)
- Two-level (file-level then unit-level) hash summary caches are used in OSS tools to avoid redundant LLM calls — [autodev-codebase summary-cache](https://glama.ai/mcp/servers/@anrgct/autodev-codebase/blob/115a11625d55601c312f87d3e1e580c16a3bb8f9/src/cli-tools/summary-cache.ts)
- Anthropic's OSS scanning shows the cost bottleneck at scale shifts to validation: ~6,000 of ~29,000 candidates triaged — [Security Affairs](https://securityaffairs.com/200685/ai/claude-helps-secure-open-source-as-anthropic-offers-free-vulnerability-scanning.html) (secondary)

### Inferences

- Cache key for a generated step/sub-script should include: content hash of the anchored code *and its resolved dependencies' summaries* (compositional, like function summaries), prompt/template version, model ID, and bound assumptions. Invalidation then propagates bottom-up through the summary graph, like incremental static analysis.
- Behavior packs act as a giant shared cache: per-version library summaries computed once in CI amortize cost across users (consistent with the checkpoint's registry idea); SemTaint-style per-package flow summaries are a working precedent.
- The <100 ms expand target is only reachable from stored data; any LLM-in-the-loop expansion will be seconds, so prefetching around focus/playhead (already proposed) is required.

### Gaps

- No academic study found on incremental re-analysis cost for LLM-based program analysis under code changes; vendor claims (e.g., "99% faster incremental") are unverified.
- No public cost figures for multi-repo (microservice) systems.

## 7. Concrete recommendations: division of labor, verification loops, required LLM outputs

### Takeaway

Let static analysis own structure and exhaustiveness (symbols, call edges where resolvable, def-use, branch points, routes, config keys); let the LLM own resolution of what static analysis leaves undecided and all naming/intent/abstraction; require every LLM output to be a structured, anchored, checkable fact; and run mechanical validation (anchor binding, entity tracing, constraint satisfiability, multi-sample agreement, adversarial self-refutation, trace alignment) before a step is shown as `inferred`.

### Cited Findings (evidence base for the recommendations)

- LLMs lose to static tools on call-graph completeness/soundness (60% vs 85%) but win on type inference — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)
- Spec inference is the LLM job with the strongest payoff (IRIS doubles CodeQL detections; SemTaint 73% of wins came from sources/sinks alone) — [arXiv 2405.17238](https://arxiv.org/html/2405.17238v3); [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- Decomposed pipelines beat end-to-end prompting (LLMDFA F1 +0.35 over CoT baseline) — [arXiv 2402.10754](https://arxiv.org/html/2402.10754v2)
- Bind-to-analyzer-node rejection + multi-run voting + calibrated confidence — [arXiv 2601.10865](https://arxiv.org/html/2601.10865)
- SMT path-condition checks reduce false bug paths — [arXiv 2501.18160](https://arxiv.org/abs/2501.18160); [arXiv 2402.10754](https://arxiv.org/abs/2402.10754)
- Entity-level tracing detects summary hallucinations at 73% F1 — [arXiv 2410.14748](https://arxiv.org/html/2410.14748v4)
- Self-refutation and confidence ratings in production (Claude Code Security) — [Anthropic](https://www.anthropic.com/news/claude-code-security)
- Hallucination rises sharply on unfamiliar/perturbed code (17% -> 97%) — [FSE 2026](https://conf.researchr.org/details/fse-2026/fse-2026-research-papers/66/Hallucinations-in-LLM-based-Code-Summarization-Unveiling-Detection-and-Mitigation)

### Inferences (recommendations for the mental-model tool)

- **Division of labor.**
  - Static (authoritative): file/symbol index (SCIP/LSP, tree-sitter), resolvable call edges, imports, def-use within functions, branch points and their syntactic conditions, route tables / gRPC protos / queue topics where declared, config keys and literal defaults, lockfile versions.
  - LLM (proposes, never authoritative): dynamic-dispatch / DI / framework-magic targets (like SemTaint's CallGraph agent), which config value is in effect, mapping branch conditions to named input classes, entity/abstraction naming and grouping, collapse decisions, effect/event-kind tagging, behavior summaries for code below the analyzable floor (runtime/OS/RFC, marked `assumed`).
  - Ask the LLM only on demand: restrict LLM calls to branches/calls on the path of the active scenario (TICR / lazy prompting analog).
- **Required LLM output schema per step (to make hallucinations detectable).**
  - `anchor` that must resolve to a real symbol/AST node at a commit (reject otherwise).
  - `claims[]`: atomic facts, each typed (call-edge, data-flow, branch-condition, config-binding, side-effect) with its own anchor, so each can be checked by a specific query.
  - `basis`: which tool results/code spans were read (evidence citations), separate from `assumptions[]`.
  - `branch_condition` as a machine-readable predicate over named inputs/config/state (checkable for satisfiability and consistency with sibling branches).
  - `confidence` from a calibrated rubric, not free-form.
  - `named_entities[]`: every identifier/route/config key mentioned in titles or summaries (for entity tracing).
- **Verification loop (cheap to expensive).**
  1. Schema + anchor binding (reject unresolvable anchors).
  2. Entity tracing of names in titles/summaries against the index.
  3. Edge cross-check: LLM-claimed call edges compared with the static call graph; where static says "unresolved", accept as `inferred` only with supporting evidence (DI registration, router table), else `assumed`.
  4. Constraint check: branch predicates along a script are jointly satisfiable; sibling branches partition the input class space.
  5. Self-consistency: k samples for deep/lazy levels; disagreement -> downgrade to `assumed` and surface as a fork.
  6. Adversarial refutation pass ("find the line that contradicts this step") for steps that will be shown as `inferred`.
  7. Observed alignment (producer 2): test runs/OTel traces confirm or contradict.
- **Behavior packs.** Generate per exact library version from the actual source (not model priors), validated by the same loop plus the library's own test suite traces; mark generated vs curated vs trace-checked; be extra skeptical of packs for forks/patched versions.
- **Evaluation.** Build a private benchmark on non-public or post-cutoff repos with hand-written gold scripts; report per-claim precision by claim type and assumption rate, not just end-to-end plausibility (benchmark contamination makes public scores unreliable).

### Gaps

- No found end-to-end evaluation of a system that generates *behavioral scenario models* with provenance from code; the recommended loop is assembled from adjacent evidence (taint analysis, summarization, execution reasoning), not measured as a whole.
- No published calibration data for LLM self-reported confidence on program-analysis claims for 2026 models.
