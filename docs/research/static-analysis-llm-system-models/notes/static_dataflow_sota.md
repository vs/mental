# Static data-flow and interprocedural analysis: state of the art (Nov 2026) and what to reuse for scenario graphs and behavior packs

Scope note: these notes cover classical and modern static analysis (IFDS/IDE, SPDS, sparse value-flow, abstract interpretation, Datalog, call graphs for dynamic languages, incremental analysis), compositional and summary-based techniques, library modeling, industrial tools, soundness limits, path-sensitivity, and recommendations for the "mental model" tool (scenario graphs, scripts, behavior packs; see `docs/brainstorm/checkpoint-01.md`). Sources marked "(foundational, not re-fetched this session)" are well-known primary references cited by DOI or official page; their content was not re-read during this session.

## 1. State of the art in interprocedural data-flow analysis (IFDS/IDE, SPDS, sparse/demand-driven, abstract interpretation, Datalog, pointer analysis, dynamic-language call graphs, incremental)

### Takeaway
The core algorithms are mature and mostly from 1995–2019 (IFDS/IDE, SPDS/Boomerang, sparse value-flow like Pinpoint, Datalog points-to like Doop). Work from 2019 to 2026 has been about scaling them (sparsification, disk-assisted solving, better call graphs) and about making dynamic languages tractable with soundy, hint-assisted call graphs (Jelly for JS/TS, PyCG/Jarvis for Python). The weakest link for TypeScript/Python web systems is still the call graph: published recall is about 70–90%, not 100%.

### Cited Findings
**IFDS/IDE and descendants**
- IFDS (Reps, Horwitz, Sagiv, POPL 1995) reduces interprocedural, finite, distributive subset problems to graph reachability over an "exploded supergraph", giving context-sensitive (realizable-path) results in polynomial time. IDE generalizes it to environment-transformer value domains (Sagiv, Reps, Horwitz 1996). These are the theoretical basis of FlowDroid, Heros, PhASAR and the core of IFDS-style taint engines. (foundational, not re-fetched this session) — [IFDS, POPL'95](https://dl.acm.org/doi/10.1145/199448.199462)
- FlowDroid (PLDI 2014) is the canonical IFDS taint analysis: context-, flow-, field- and object-sensitive, with a precise Android lifecycle model. It is still the baseline new taint work compares against in 2026. (foundational, not re-fetched this session) — [FlowDroid, PLDI'14](https://dl.acm.org/doi/10.1145/2594291.2594299)
- SPDS (Späth, Ali, Bodden, POPL 2019) encodes calls and returns as one CFL-reachability problem and field stores and loads as another, then "synchronizes" the two. This avoids k-limited access paths, which "do not scale very well when used to analyze programs with recursive data structures". It approximates only where both problems are satisfied but not along the same path. SPDS underlies the demand-driven pointer analysis Boomerang and the IDEal typestate framework. — [Paderborn record of POPL'19 paper](https://ris.uni-paderborn.de/record/20539); [Späth dissertation](https://digital.ub.uni-paderborn.de/hs/content/structure/3030984)
- Sparsification of IFDS: SparseDroid (ASE 2019) propagates each fact directly to its next possible use instead of through every program point. It averaged a 22.0x speedup over FlowDroid on 40 Android apps, and FlowDroid did not finish 9 of them within 5 h and 220 GB. FlowDroidSP (2019) reports up to 85.5% of IFDS propagation is unrelated, with a 4.8x speedup and 61.5% less memory (18.1x and 76.1% with pruning). — [SparseDroid ASE'19](https://2019.ase-conferences.org/details/ase-2019-papers/1/Performance-Boosting-Sparsification-of-the-IFDS-Algorithm-with-Applications-to-Taint-); [FlowDroidSP](https://crad.ict.ac.cn/EN/Y2019/V56/I3/480)
- DiskDroid (CGO 2021) ran 19 apps that need 10–128 GB of RAM in FlowDroid in under 10 GB, and was 8.6% faster. — [DiskDroid](https://archive.cps-vo.org/node/84845)
- "Pick Your Call Graphs Well" (SOAP@PLDI 2025): precise context-sensitive call graphs significantly reduce IFDS runtime and improve precision, and there is a sweet spot between call-graph construction time and total runtime. — [SOAP 2025](https://pldi25.sigplan.org/details/SOAP-2025-papers/1/Pick-Your-Call-Graphs-Well-On-Scaling-IFDS-Based-Data-Flow-Analyses)
- TnFix (ECOOP 2026) targets over-tainting caused by k-limiting. It analyzes 7 of 36 Android apps that FlowDroid cannot finish within 3 hours. — [ECOOP 2026, LIPIcs](https://drops.dagstuhl.de/storage/00lipics/lipics-vol372-ecoop2026/html/LIPIcs.ECOOP.2026.10/LIPIcs.ECOOP.2026.10.html)

**Sparse value-flow and path-sensitive analysis**
- Pinpoint (PLDI 2018) splits high-precision points-to analysis into precise local data dependence plus memoized, deferred interprocedural analysis. It slices on demand only the interprocedural dependence and path-feasibility queries a check needs and solves them with an SMT solver. It checks MySQL (~2 MLOC) in 1.5 h with a 14.3–23.6% false-positive rate and found 40+ real bugs. — [Pinpoint PLDI'18 PDF](https://www.cse.ust.hk/~charlesz/papers/pinpoint.pdf); [PLDI'18 listing](https://pldi18.sigplan.org/details/pldi-2018-papers/42/Pinpoint-Fast-and-Precise-Sparse-Value-Flow-Analysis-for-Million-Lines-of-Code)

**Abstract interpretation frameworks**
- Infer.AI is Infer's abstract-interpretation framework. A checker becomes interprocedural by defining a summary type and using callee summaries in its transfer functions (example: the SIOF checker). — [Infer absint framework docs](https://fbinfer.com/docs/absint-framework)
- Frama-C/Eva (sound abstract interpretation for C) is still used in 2026 as the precise back end in LLM-assisted pipelines, for example in "Evident" (arXiv, June/Oct 2026). — [arXiv 2606.15122](https://arxiv.org/abs/2606.15122)

**Datalog-based analysis**
- Doop expresses points-to and call-graph analyses for Java as declarative Datalog. JackEE (PLDI 2020) builds on Doop because Datalog rules suit framework modeling. — [JackEE summary (acolyer)](https://blog.acolyer.org/2020/09/25/jackee); [Doop repo with JackEE mode](https://github.com/kanaksad/doop); (foundational, not re-fetched: [Doop OOPSLA'09](https://dl.acm.org/doi/10.1145/1640089.1640108); [Soufflé](https://souffle-lang.github.io/))
- CodeQL is a Datalog-family query language (QL) over an extracted relational database. Its library models are "extensible predicates" filled from YAML/JSON data extensions (see section 2). — [CodeQL JS library-model docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)

**Call graphs for dynamic languages**
- JavaScript/TypeScript: Jelly (Aarhus) is a flow-insensitive control-flow and points-to analysis for Node.js JS/TS that uses access paths to track library usage. "Approximate interpretation" (PLDI 2024) is a forced-execution dynamic pre-analysis that emits hints about dynamic property accesses. On 36 projects with dynamic call graphs it raised call-graph recall from about 75.9% to 88.1% at a small precision cost, and it was evaluated on 141 Node.js apps and libraries. Jelly can also build dynamic call graphs through NodeProf to measure unsoundness. — [PLDI'24 paper page](https://cs.au.dk/~amoeller/papers/approx); [PLDI'24 PDF](https://www.cs.au.dk/~amoeller/papers/approx/paper.pdf); [Jelly GitHub](https://www.github.com/cs-au-dk/jelly)
- Related JS work: automatic root-cause quantification of missing call-graph edges (ECOOP 2022) and indirection-bounded call graphs (ECOOP 2024, the basis of Jelly's `--max-indirections`). — [JS call-graph papers index](https://javascript-papers.readthedocs.io/en/latest/callgraph.html)
- Python: PyCG reports ~99.2% precision and ~69.9% recall. That figure comes from a third-party listing of the ICSE 2021 paper. — [PyCG listing](https://arxiv.deeppaper.ai/papers/2103.00587v1)
- Jarvis improves on PyCG by 84% in precision and at least 20% in recall for application-centered call-graph construction (later arXiv versions; v1 claimed 10%). The gain comes mainly from covering more language features: PyCG misses context managers and built-in types. — [Jarvis arXiv v5](https://arxiv.org/abs/2305.05949v5); [v1](https://arxiv.org/abs/2305.05949v1)
- PyCG is archived and unmaintained. (Not verified this session; the report writer should check before recommending it.)
- Go: `golang.org/x/tools/go/callgraph/vta` implements Variable Type Analysis over a type-propagation graph. It refines an initial call graph and is sound only if that graph is sound, "modulo reflection and unsafe". The package is marked experimental. — [vta package docs](https://pkg.go.dev/golang.org/x/tools/go/callgraph/vta)
- LLMs versus static call-graph tools (Empirical Software Engineering, 2025): 24 LLMs were evaluated on Python and JavaScript with new benchmarks SWARM-CG and SWARM-JS. Static tools (PyCG, Jelly) "consistently outperform" LLMs. On Python, PyCG reached about 84.9% completeness against about 60.3% for the best LLM (figure from a secondary summary). LLMs mishandle indirect imports through intermediate modules but sometimes catch dictionary-update calls PyCG misses, and they do much worse on JS than on Python. — [arXiv 2410.00603](https://arxiv.org/pdf/2410.00603); [EMSE article](https://link.springer.com/article/10.1007/s10664-025-10704-3)

**Incremental analysis (industrial)**
- CodeQL incremental PR analysis timeline:
  - May 2025: up to 20% faster for JS/TS, Java, Ruby and Python, with the most improved JS/TS scans 58% faster. Results are unchanged.
  - Sep 2025: extended to all CodeQL languages; Go about 20% faster, C#/C++ about 5%.
  - Mar 2026: a database for the changed code is combined with a cached database of the whole codebase ("overlay"), for the default query suite only.
  - Jun 2026: CodeQL CLI 2.25.5+ supports incremental analysis in third-party CI.
  - The docs describe diff-informed analysis (CLI 2.21.0+) and overlay analysis (CLI 2.23.8+) with "up to 10x" shorter scans (GitHub's own figure).
  - Sources: [May 2025](https://github.blog/changelog/2025-05-28-incremental-security-analysis-makes-codeql-up-to-20-faster-in-pull-requests); [Sep 2025](https://github.blog/changelog/2025-09-23-incremental-security-analysis-with-codeql-is-now-available-for-all-languages/); [Mar 2026](https://github.blog/changelog/2026-03-24-faster-incremental-analysis-with-codeql-in-pull-requests/); [Jun 2026](https://github.blog/changelog/2026-06-10-incremental-analysis-for-go-c-c-and-codeql-cli); [docs](https://docs.github.com/en/enterprise-server@3.19/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis)
- "Incrementalizing Production CodeQL Analyses" is a GitHub/academic paper on incremental Datalog evaluation for CodeQL. — [arXiv 2308.09660](https://arxiv.org/abs/2308.09660v1); [GitHub Next project](https://githubnext.com/projects/incremental-codeql/)
- Semgrep redesigned its interfile engine in v1.158.0, reporting a 20–40% speedup (from release notes as relayed by search; not opened directly). — [Semgrep releases](https://github.com/returntocorp/semgrep/releases)

### Inferences
- Theory is not the bottleneck for our use. A caller-specific query ("what does this handler do with `req.body.password`?") maps naturally onto demand-driven IFDS/SPDS or sparse value-flow. The real bottlenecks are call-graph recall in TS/Python and library/framework models.
- Cache by content hash, and at function granularity. Compositional summaries plus overlay-style incremental databases (the CodeQL 2026 design) match the plan to cache deeper levels by the content hash of the code they cover.
- Hybrid call graphs are the 2024–2026 consensus for dynamic languages: static analysis with hints (Jelly's approximate interpretation) or with LLM-resolved edges (SemTaint, section 2). LLM-only call graphs are measurably worse.

### Gaps
- I found no 2023–2026 head-to-head benchmark of TypeScript call graphs (Jelly vs. CodeQL JS vs. TS-compiler-based tools) on typed TS code. Most JS studies target untyped Node.js.
- I did not verify current maintenance status of PyCG, PhASAR, Heros or Doop in 2026.
- I did not find recent primary sources on Soufflé performance or on incremental Datalog beyond CodeQL.

## 2. Compositional and summary-based analysis: how library summaries are represented, generated, versioned and composed; failure modes

### Takeaway
Every production tool composes per-function or per-API summaries, but summary formats differ sharply:
- **CodeQL models-as-data (MaD):** declarative rows: type, access path, input → output, kind, provenance. The closest existing precedent for a "behavior pack" registry.
- **Infer bi-abduction and Pulse:** pre/post (disjunctive) specifications; richer, closer to path-sensitive behavior.
- **Pysa:** type-stub-like `.pysa` annotations.
- **FlowDroid/StubDroid:** generated access-path summaries.

Since 2024, summaries are increasingly LLM-generated (CodeQL `ai-generated` provenance, IRIS, SemTaint). The main failure modes are missing models (silent false negatives), over-approximate "fuzzy" models (false positives), and version drift.

### Cited Findings
**Representations**
- CodeQL JS/TS MaD format:
  - Rows go into extensible predicates: `sourceModel(type, path, kind)`, `sinkModel(type, path, kind)`, `summaryModel(type, path, input, output, kind)`, `typeModel(type1, type2, path)`, `barrierModel`, and `barrierGuardModel(type, path, acceptingValue, kind)`.
  - The `type` column is an npm package name, `global`, a qualified name, or `file:<path>`.
  - Access-path tokens: `Member[name]`, `ReturnValue`, `Argument[n]`, `Parameter[n]`, `ArrayElement`, `MapValue`, `Awaited`, `Instance`, `AnyMember`, `Fuzzy`, `WithArity[n]`, `WithStringArgument[n=value]`, decorator tokens, and `GuardedRouteHandler`.
  - Example summary: `["global","Member[decodeURIComponent]","Argument[0]","ReturnValue","taint"]`.
  - Kind is `taint` or `value` (value-preserving).
  - "Library customization using data extensions is currently in beta"; `Fuzzy` is approximate; `GuardedRouteHandler` over-approximates route handlers.
  - Source: [CodeQL docs: customizing library models for JavaScript](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)
- CodeQL provenance and precedence:
  - Provenance has the form `{origin}-{verification}`, e.g. `df-generated` (from the data-flow model generator), `df-manual` (generator output checked by a human), `manual`, `ai-manual`.
  - Neutral models declare "no summary / not a source or sink". Manual neutral models override generated summaries.
  - An older doc version said generated models are ignored when the library's source code is available.
  - Sources: [Java/Kotlin customization guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-java-and-kotlin); [ExternalFlow module](https://codeql.github.com/codeql-standard-libraries/java/semmle/code/java/dataflow/ExternalFlow.qll/module.ExternalFlow.html); [PR #16552](https://github.com/github/codeql/pull/16552/files)
- CodeQL MaD also exists for Go, C#, Ruby, C/C++ and Python. — [Go guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-go); [C# guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-csharp); [Ruby ModelsAsData](https://codeql.github.com/codeql-standard-libraries/ruby/codeql/ruby/frameworks/data/ModelsAsData.qll/module.ModelsAsData.html)
- Infer bi-abduction (POPL 2009) infers pre/post specs bottom-up, without calling contexts, and composes them at call sites by solving frame and anti-frame (abduction) problems. It was deployed at Facebook scale. (foundational, not re-fetched: [Calcagno et al., POPL'09](https://dl.acm.org/doi/10.1145/1480881.1480917))
- Infer represents summaries as pre/post specs and computes them "without looking at every call site", making each summary as general as possible in one pass. — [Facebook engineering post on Infer](https://code.facebook.com/posts/1537144479682247/finding-inter-procedural-bugs-at-scale-with-infer-static-analyzer/)
- Pulse:
  - Infer's current interprocedural memory-safety and taint engine. Its foundation is incorrectness separation logic (Pulse-X, OOPSLA 2022: 15 new bugs in OpenSSL).
  - "Errors are only reported when all conditions on the erroneous path are true regardless of input." Parameter-dependent errors are kept as *latent* in the summary and become manifest at a caller that satisfies the conditions.
  - A summary is a list of disjuncts, and applying it in a caller's context can yield contradictions that prune disjuncts.
  - Unknown functions get optimistic assumptions ("scramble" reachable state), which "may cause false negatives as well as false positives".
  - Languages: C/C++/ObjC, Java, Hack; Erlang and Rust experimental; no Python, C# or Swift.
  - Sources: [Pulse checker docs](https://fbinfer.com/docs/checker-pulse); [PulseInterproc odoc](https://fbinfer.com/odoc/next/infer/Pulselib/PulseInterproc/index.html); [Pulse-X OOPSLA'22 PDF](https://discovery.ucl.ac.uk/id/eprint/10146384/1/3527325.pdf)
- Pysa (Meta, Python):
  - `taint.config` (JSON) declares sources, sinks, features and rules. `.pysa` model files use Python annotation syntax, e.g. `django.http.request.HttpRequest.COOKIES: TaintSource[UserControlled]`.
  - Third-party libraries need stubs whose names match the originals.
  - `pyre validate-models` checks models without a run.
  - Third-party models live under `stubs/third_party_taint`.
  - Sources: [Pysa basics](https://pyre-check.org/docs/pysa-basics/); [Pysa tips](https://pyre-check.org/docs/pysa-tips); [shipping rules/models](https://pyre-check.org/docs/pysa-shipping-rules-models/); [Meta engineering post](https://engineering.fb.com/security/pysa/)
- StubDroid (ICSE 2016) automatically generates context- and flow-sensitive library summaries for FlowDroid from library bytecode as access-path transfer rules. It avoids reanalyzing the JDK/Android framework per app. (foundational, not re-fetched: [StubDroid ICSE'16](https://dl.acm.org/doi/10.1145/2884781.2884816))

**Generation (manual, mined, LLM)**
- CodeQL ships `df-generated` models from a data-flow "model generator". The VS Code model editor is the recommended way to write models by hand, saved as YAML in a model pack. — [Java/Kotlin guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-java-and-kotlin); [model editor](https://docs.github.com/ru/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-vs-code/use-the-model-editor)
- A github/codeql PR adds YAML models produced by a tool called `codeql-mads-via-llm`, tagged `ai-generated` with "do not edit" headers (Apache Avro). This is evidence of LLM-generated MaD in the official repo, but it is a PR diff, not documentation. — [github/codeql PR #21754](https://github.zuike.workers.dev/github/codeql/pull/21754/files) (mirror URL)
- Community tooling `codeql-summarize` generates MaD files from CodeQL databases and can run as a GitHub Action. — [advanced-security/codeql-summarize](https://www.github.com/advanced-security/codeql-summarize)
- IRIS (ICLR 2025): an LLM infers CWE-specific source/sink specs for library APIs, CodeQL runs the taint analysis, and a second LLM pass triages paths. On CWE-Bench-Java (120 real vulnerabilities) CodeQL alone found 27 and IRIS+GPT-4 found 55, with about 5 points better false discovery rate, plus 4 new vulnerabilities. — [arXiv 2405.17238](https://arxiv.org/html/2405.17238v3); [ICLR proceedings](https://proceedings.iclr.cc/paper/2025/hash/582d4e27fa24168f3af1f4582655034b-Abstract-Conference.html)
- SemTaint (arXiv Jan 2026): multi-agent LLM plus static analysis extracts per-npm-package taint specs, including sources, sinks, *call edges static analysis cannot resolve*, and *library flow summaries*. Fed to CodeQL, these let it detect 106 of 162 vulnerabilities it previously missed, plus 4 new ones. — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- LLMDFA (NeurIPS 2024): LLM pipeline of source/sink extraction, per-function dataflow summaries via few-shot CoT, and path-feasibility checks in which the LLM writes code that calls an SMT solver. Results: 87.10% precision and 80.77% recall on average; 74.63% precision and 60.24% recall on TaintBench Android malware. — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754v2); [NeurIPS poster](https://neurips.cc/virtual/2024/poster/95227)
- LATTE (binary taint) uses an LLM to summarize external library functions; first submitted 2023, revised January 2025. — [arXiv 2310.08275](https://arxiv.org/abs/2310.08275)
- AdaTaint (arXiv Nov 2025) interleaves LLM-inferred specs with static analysis "while ensuring symbolic validation" (snippet only). — [arXiv 2511.04023](https://arxiv.org/pdf/2511.04023)

**Failure modes (documented)**
- Missing specs mean missed vulnerabilities. Writing them by hand is laborious and leaves gaps; this is the stated motivation of IRIS. — [IRIS](https://arxiv.org/html/2405.17238v3)
- Over-approximate models (`Fuzzy`, `GuardedRouteHandler`) cause false positives; type-based models fail without type annotations. — [CodeQL JS docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)
- Unknown or unmodeled callees: Pulse's optimistic havoc gives both false positives and false negatives. — [Pulse docs](https://fbinfer.com/docs/checker-pulse)
- Generated summaries can be wrong. CodeQL needs neutral models and a manual-over-generated precedence rule to suppress bad generated summaries. — [Java/Kotlin guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-java-and-kotlin)
- Format instability: CodeQL MaD and threat models are "in beta and subject to change". — [CodeQL JS docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)

### Inferences
- CodeQL MaD is the best-documented, multi-language, *data-only* summary format. Its design choices carry over directly to behavior packs:
  - access paths relative to a call
  - `taint` vs. `value` kinds
  - provenance with a verification state
  - neutral rows ("we checked, nothing happens")
  - precedence of manual over generated
- MaD summaries describe flows only. They have no branching on input classes, no states and no effects. For scenario graphs, the extra semantics should come from Pulse-style disjunctive pre/post summaries: each disjunct carries a path condition over the inputs, and latent conditions bind at the caller. In our terms, an unbound latent condition is exactly an "assumption".
- Versioning: none of the tools I examined publishes per-library-*version* summary packs. CodeQL bundles models in versioned query/model packs per CodeQL release, not per library version. This is an open niche for our registry.
- The 2024–2026 trend (IRIS, SemTaint, `ai-generated` MaD) validates the "LLM writes summaries, static engine checks and composes them" split that we plan.

### Gaps
- I did not find primary documentation of how GitHub validates `ai-generated` CodeQL models, or their measured accuracy.
- I did not find a study measuring how often library summaries go stale across library versions.
- StubDroid's and Pysa's current maintenance and versioning practices were not verified this session.

## 3. Industrial tools: cross-file, cross-language and cross-service support, with precision/recall evidence

### Takeaway
Cross-file, cross-procedure taint within one language and one repo is solved industrially for Java, JS/TS, Go, Python and C# (CodeQL, Semgrep Pro for most of these, Pysa for Python, Joern via CPG). Cross-language and cross-service flow (HTTP client to route, gRPC, queues) is not solved by any mainstream SAST tool I examined. It is handled by research architecture-recovery tools using keyword and config heuristics. On real vulnerabilities, recall is low: only 12.7% detected by seven open-source Java SAST tools in FSE 2023.

### Cited Findings
- **CodeQL**
  - Whole-program interprocedural data flow per language database, with MaD library models (section 2).
  - Incremental and overlay PR analysis in all languages by 2026 (section 1).
  - Each database is per language and per codebase; I found no official cross-language or cross-repo flow feature.
  - Sources: [CodeQL JS models docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/); [incremental analysis docs](https://docs.github.com/en/enterprise-server@3.19/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis)
- **Semgrep Pro**
  - The product page lists interfile analysis for C, C++, Go, Java, Kotlin and JS/TS. Interprocedural (single-file) analysis is available for all languages and is "experimental".
  - Rules need `options: interfile: true` and a `semgrep --pro` run.
  - Later releases added C# and experimental Gosu interfile taint (v1.166.0).
  - Python interfile support could not be confirmed from an official source.
  - Sources: [Semgrep Pro Engine product page](https://Semgrep.dev/products/pro-engine); [feature definitions](https://docs.semgrep.dev/references/feature-definitions); [Pro engine examples](https://docs.semgrep.dev/semgrep-code/semgrep-pro-engine-examples); [birth of Pro Engine](https://semgrep.dev/blog/2023/the-birth-of-semgrep-pro-engine/)
- **Joern** builds a Code Property Graph (AST, CFG and PDG layers). A 2025 paper documents its data-flow engine: a data-dependence graph built on the CPG, plus a query engine computing source–sink flows.
  - Official docs list C/C++, Java, JavaScript, Python and x86/x64 binaries.
  - Frontends also include `jssrc2cpg` (JS/TS), `javasrc2cpg`, `php2cpg`, `jimple2cpg` and `abap2cpg`; wrappers mention Go, Kotlin and C#.
  - Maturity varies by language.
  - Sources: [Joern docs](https://docs.joern.io/); [arXiv 2506.06247](https://arxiv.org/html/2506.06247v1); [cpggen](https://pypi.org/project/cpggen/1.0.6)
- **Infer/Pulse** covers C/C++/ObjC, Java and Hack, with Erlang/Rust experimental. It does not cover Python, TS or Go. — [Pulse docs](https://fbinfer.com/docs/checker-pulse)
- **Pysa** is Python only and relies on Pyre types and `.pysa` models for frameworks like Django. — [Pysa basics](https://pyre-check.org/docs/pysa-basics/)
- **Precision/recall evidence**
  - FSE 2023 (seven free/open-source Java SAST tools chosen from 161): tools do well on synthetic benchmarks, but "only 12.7% of real-world vulnerabilities can be detected", and 70.9% stay undetected even when all tools are combined. Benchmark: 165 CVEs. — [FSE'23 listing](https://2023.esec-fse.org/details/fse-2023-research-papers/21/-Remote-Comparison-and-Evaluation-on-Static-Application-Security-Testing-SAST-Tool); [Java_CVE_Bench](https://github.blesdmm.dns-dynamic.net/MarkLee131/Java_CVE_Bench)
  - EASE 2024 (Esposito et al.): SAST tools show "high Precision while falling short in Recall". — [arXiv 2403.09219](https://arxiv.org/pdf/2403.09219)
  - IRIS: CodeQL alone detected 27 of 120 real Java CVEs. — [IRIS](https://arxiv.org/html/2405.17238v3)
- **LLM-based false-positive triage**: an industrial empirical study exists (arXiv 2601.18844). "Evident" (2026) shows that LLM-decides filters miss 27.5–45.0% of confirmed bugs with GPT-5.5. Using the LLM only to build a harness for a sound analyzer (Frama-C/Eva) filtered 48.8–63.8% of false alarms with zero missed bugs on 200 Android kernel-driver warnings. — [arXiv 2606.15122](https://arxiv.org/abs/2606.15122); [arXiv 2601.18844](https://arxiv.org/pdf/2601.18844)
- **Cross-service**
  - Code2DFD (JSS 2023) extracts security-annotated dataflow diagrams from Java microservice code and config files by keyword evidence. It handles message-broker flows via queue/exchange configuration and emits PlantUML, JSON and a *traceability file* linking each DFD element to code evidence. — [arXiv 2304.12769](https://arxiv.org/abs/2304.12769v1); [code2DFD repo](https://scriptagc.wasmer.app/https_GitHub_com/tuhh-softsec/code2DFD/tree/0af4b1683f611eecdb13b1c457801a5904aad150)
  - A 2024 comparison of static microservice architecture-recovery tools (e.g., MicroGraal partial topology graphs) notes inter-service connections are often found only dynamically. Other approaches (Bushong and Das; Abdelfattah and Cerny) extract HTTP endpoints and client calls, then match signatures across services. — [arXiv 2412.08352](https://arxiv.org/pdf/2412.08352)
- **Google**: analyses run in Tricorder inside code review, and checks are kept only if their effective false-positive rate stays low (<10% is the cited bar). (foundational, not re-fetched: [Lessons from Building Static Analysis Tools at Google, CACM 2018](https://cacm.acm.org/magazines/2018/4/226371-lessons-from-building-static-analysis-tools-at-google/fulltext))

### Inferences
- For our multi-repo boundary linking (`fetch('/api/login')` to the route, gRPC protos, queue topics), no off-the-shelf SAST gives us edges. The practical approach is Code2DFD-style: extract endpoint and client-call facts per repo (CodeQL/Semgrep queries or tree-sitter), match them by URL template, proto service/method or topic name, and record each match as an `inferred` link with evidence. Every link should keep a traceability anchor, as Code2DFD does.
- Low SAST recall on real CVEs comes mostly from missing models and frameworks. Our tool will show *flows*, not flag vulnerabilities, so precision on what *is* shown matters more than recall. But missing edges become silently missing branches, which argues for explicit "unknown callee" markers in scenario graphs.
- The Evident result is a strong design rule: let the LLM propose context, summaries and harnesses, and let a precise analyzer or check decide. Never let the LLM's narration certify that a branch is impossible.

### Gaps
- I found no public precision/recall numbers for Semgrep Pro interfile, Snyk Code (DeepCode) or SonarQube's cross-file taint on independent benchmarks; vendor claims only.
- I found no primary source on cross-service or cross-language taint in Google's or Meta's internal tools (Meta's Zoncolan for Hack and Pysa for Python are per-language).
- Joern's current frontend list and data-flow precision were not verified against the repo.

## 4. Soundness vs. soundiness: blind spots and framework modeling (Spring, Express, Django)

### Takeaway
Practical analyses are "soundy" by design: they handle most features conservatively but knowingly skip reflection, `eval`, dynamic property access, `unsafe`, and framework-driven control. The biggest blind spot for web systems is inversion of control: dependency injection, routing, decorators and middleware. Raw analyzers see only a small fraction of enterprise code without framework models (JackEE: about 14.5% to 58% in-app reachability for Doop). The state of the art is explicit, per-framework models as Datalog rules, MaD rows or `.pysa` files, plus dynamic hints for dynamic languages.

### Cited Findings
- The Soundiness manifesto (CACM 2015, Livshits et al.) says analyses are typically "soundy": conservative for most features but deliberately omitting a known subset (reflection, eval, etc.). Skipping these affects most real programs. It calls for documenting unsoundness and comparing results empirically against observed behavior. — [CACM full text](https://cacm.acm.org/magazines/2015/2/182650-in-defense-of-soundiness/fulltext); [PDF](https://www.doc.ic.ac.uk/~livshits/papers/pdf/cacm15.pdf)
- Enterprise Java (JackEE, PLDI 2020): Soot, WALA and Doop out of the box yield very low coverage on enterprise apps. JackEE adds framework rules (Spring, Struts2, JAX-RS, Servlets, EJB) in Datalog on Doop. Slides report about 58% average in-app reachability against about 14.5% for plain Doop. Caches and maps are a second major obstacle. — [acolyer summary](https://blog.acolyer.org/2020/09/25/jackee); [PLDI'20 listing](https://pldi20.sigplan.org/details/pldi-2020-papers/66/Static-Analysis-of-Enterprise-Applications-Haven-t-We-Solved-This-Problem-Yet-); [slides](https://www.scribd.com/presentation/491732108/Static-Analysis-of-Java-Enterprise-Applications); [Doop/JackEE repo](https://github.com/kanaksad/doop)
- Jasmine (ASE 2022) found that Soot, Doop and WALA only partially support Spring dependency injection and AOP, and proposes Spring-specific modeling. — [Jasmine PDF](https://weihang-wang.github.io/papers/ASE2022-Jasmine.pdf)
- JS dynamic property access is the main source of missed call edges. Approximate interpretation (forced execution producing hints) raises recall from about 76% to 88%. Jelly also uses access paths to model library and framework usage. — [PLDI'24](https://cs.au.dk/~amoeller/papers/approx)
- Express-style middleware: CodeQL's MaD has a `GuardedRouteHandler` token for route handlers behind middleware, documented as over-approximating. Decorator tokens (`DecoratedClass`, `DecoratedParameter`, `DecoratedMember`) model decorator-driven frameworks such as NestJS/Angular-style DI. — [CodeQL JS docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)
- Django: Pysa models request attributes (`HttpRequest.COOKIES`, `GET`) as `TaintSource[UserControlled]` via `.pysa` stubs. — [Pysa basics](https://pyre-check.org/docs/pysa-basics/); [Meta engineering](https://engineering.fb.com/security/pysa/)
- Python dynamic features: PyCG's false negatives come from unsupported features (context managers, built-in types); Jarvis recovers some of them. — [Jarvis](https://arxiv.org/abs/2305.05949v5)
- Go: VTA is sound only modulo reflection and `unsafe`, and only if its initial call graph is sound. — [vta docs](https://pkg.go.dev/golang.org/x/tools/go/callgraph/vta)
- Unknown or unmodeled code: Pulse havocs the reachable state, causing false positives and false negatives. — [Pulse docs](https://fbinfer.com/docs/checker-pulse)
- Android lifecycle and callbacks were modeled explicitly in FlowDroid. This is the standard precedent for event-loop and callback IoC. (foundational, not re-fetched: [FlowDroid](https://dl.acm.org/doi/10.1145/2594291.2594299))
- SemTaint uses an LLM to resolve call edges that static analysis cannot (JS npm packages). — [arXiv 2601.10865](https://arxiv.org/abs/2601.10865)

### Inferences
- Framework packs must contain *slots for user code* (route-handler slot, middleware chain, DI-provided bean, ORM hooks). This matches JackEE's rules and CodeQL's `GuardedRouteHandler` and decorator tokens, and confirms the checkpoint's "frameworks invert control, so their graphs have slots for user code."
- Async and event loops (Node promises, Python asyncio, Go goroutines and channels) are modeled in CodeQL only through tokens like `Awaited`, with no scheduling or interleaving semantics. For parallel lanes and races we must rely on platform packs and LLM hypotheses marked `assumed`.
- Config-driven behavior (env vars, YAML, feature flags) is outside every tool's data-flow model. It matches our design to resolve it with constant propagation plus config reading, recording an `assumed` binding when unresolved.

### Gaps
- I found no 2024–2026 empirical study quantifying how much framework IoC (Express, NestJS, FastAPI, Django) costs TS/Python call-graph recall.
- I found no primary sources on static modeling of async scheduling (event-loop ordering, goroutine interleavings) in production tools.

## 5. Which techniques give "branches on input classes" (path conditions / input partitions) rather than just reachability?

### Takeaway
Most production data-flow tools (IFDS taint, CodeQL, Semgrep, Pysa, Joern) answer reachability ("can tainted X reach sink Y?") and are path-insensitive or only lightly guard-aware. Path conditions over inputs come from:
- Infer Pulse's disjunctive summaries with latent conditions,
- SMT-checked sparse value-flow (Pinpoint),
- symbolic execution,
- LLM pipelines that offload feasibility to solvers (LLMDFA).

The disjunct-plus-condition shape of Pulse summaries is the closest existing analogue to "a graph per public function keyed by input classes".

### Cited Findings
- Pulse summaries are disjunctive. Errors are reported only when "all conditions on the erroneous path are true regardless of input". Conditions on parameters are kept as *latent* issues and discharged at call sites, and summary application in a caller's context prunes contradicting disjuncts. — [Pulse docs](https://fbinfer.com/docs/checker-pulse); [PulseInterproc](https://fbinfer.com/odoc/next/infer/Pulselib/PulseInterproc/index.html)
- Pulse-X is based on incorrectness (under-approximate) separation logic, whose triples assert that every reported path is reachable. This is the foundation for "no false positives" path reports. — [Pulse-X OOPSLA'22](https://discovery.ucl.ac.uk/id/eprint/10146384/1/3527325.pdf); [Raad et al. 2020](https://link.springer.com/chapter/10.1007/978-3-030-53291-8_14)
- Pinpoint: sparse value-flow plus on-demand path-feasibility queries solved by SMT, at multi-MLOC scale. — [Pinpoint PDF](https://www.cse.ust.hk/~charlesz/papers/pinpoint.pdf)
- LLMDFA: per-function summaries from the LLM; path feasibility checked by LLM-synthesized scripts that call an automated theorem prover. — [arXiv 2402.10754](https://arxiv.org/abs/2402.10754v2)
- IDE (environment transformers) can track values such as constants and typestate along paths, but it is still a meet-over-paths analysis, not per-path conditions. IDEal (built on SPDS) applies this to typestate. — [Späth dissertation](https://digital.ub.uni-paderborn.de/hs/content/structure/3030984)
- CodeQL's `barrierGuardModel(type, path, acceptingValue, kind)` treats a value as sanitized on the branch where a check returns the accepting value. It is branch-aware for guards, but only as a barrier, not as partitions. — [CodeQL JS docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)
- "Evident" (2026) uses Frama-C/Eva (abstract interpretation over value domains) in LLM-built harnesses to rule out error paths soundly, constraining how harnesses initialize inputs so that bug-triggering inputs are not excluded. — [arXiv 2606.15122](https://arxiv.org/abs/2606.15122)

### Inferences
- Recommended scenario-graph node shape: a function or pack summary as a set of disjuncts `{guard over input classes, effects/outputs, callees, latent obligations}`. Binding input classes at a call site prunes disjuncts (as Pulse does), and unresolved guards become assumptions.
- Input classes ("expired token", "user not found") are human-meaningful predicates over guards found in code. Static analysis can give the guard expressions (branch conditions on parameters, return-value checks), but naming and grouping them into classes is an LLM task. A solver (Z3) or abstract interpreter can then check disjointness or feasibility where the guards are simple (constants, enums, nullness, string-literal comparisons).
- Keep summaries abstract to avoid path explosion: cap disjuncts per summary and merge disjuncts with identical observable effects. Pulse and its imitators cap disjuncts, but I found no authoritative number for Infer's own cap.

### Gaps
- I found no production tool for TS/Python/Go that emits path-condition-partitioned function summaries. Pulse does not support those languages.
- I did not research symbolic execution engines for JS/Python (e.g., ExpoSE, CrossHair) or concolic testing in 2024–2026. Those would need a separate search.

## 6. Concrete recommendations: grounding tools and the summary representation for "behavior packs"

### Takeaway
Use a layered grounding stack:
- **Navigation:** tree-sitter plus SCIP/LSP for symbols.
- **Facts:** CodeQL, or Jelly/Jarvis-class call graphs where CodeQL is weak, for call graphs, data-flow paths and framework entry points. Semgrep for cheap cross-repo boundary-fact extraction.
- **Composition:** the LLM composes and names input classes on top of those facts.

Model behavior packs on CodeQL MaD (data-only rows, access paths, kinds, provenance, neutral rows, manual-over-generated precedence), extended with Pulse-style disjunctive cases (guards over input classes plus latent obligations) and explicit IoC slots.

### Cited Findings (supporting evidence for recommendations)
- MaD shows that a declarative, data-only, multi-language summary format with provenance works at industrial scale and accepts LLM-generated entries (`ai-generated`, `ai-manual`). — [CodeQL Java/Kotlin guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-java-and-kotlin); [ExternalFlow](https://codeql.github.com/codeql-standard-libraries/java/semmle/code/java/dataflow/ExternalFlow.qll/module.ExternalFlow.html); [LLM-generated MaD PR](https://github.zuike.workers.dev/github/codeql/pull/21754/files)
- LLM spec inference plus static engine beats either alone: IRIS 27 to 55 CVEs; SemTaint 106 of 162 previously missed. — [IRIS](https://arxiv.org/html/2405.17238v3); [SemTaint](https://arxiv.org/abs/2601.10865)
- Static call graphs beat LLM call graphs for Python and JS. — [EMSE 2025 / arXiv 2410.00603](https://arxiv.org/pdf/2410.00603)
- Hint-assisted static analysis improves JS call-graph recall to about 88%. — [Approximate interpretation PLDI'24](https://cs.au.dk/~amoeller/papers/approx)
- LLM final-decision filtering drops real bugs, while LLM-as-context plus precise analysis does not. — [Evident, arXiv 2606.15122](https://arxiv.org/abs/2606.15122)
- Framework models dramatically change coverage (JackEE). — [acolyer](https://blog.acolyer.org/2020/09/25/jackee)
- Traceability from extracted architecture elements to code evidence is practical (Code2DFD traceability file). — [Code2DFD](https://arxiv.org/abs/2304.12769v1)
- Incremental, overlay-style re-analysis on changed code is production-ready in CodeQL (2025–2026). — [GitHub changelog Mar 2026](https://github.blog/changelog/2026-03-24-faster-incremental-analysis-with-codeql-in-pull-requests/)

### Inferences (recommendations)

**Per model part, what to reuse:**

| Model part | Grounding technique / tool | Notes |
| --- | --- | --- |
| Entities, symbols, anchors | tree-sitter + SCIP/LSP indexes (as planned) | Fast, multi-language, stable anchors for `anchor:` fields. |
| Inputs (entry points) | CodeQL framework libraries (`RemoteFlowSource`-style sources, route handlers), Semgrep rules for routes/CLI/queue consumers; Pysa models for Django | Framework entry points are where SAST invests most; reuse rather than rebuild. |
| Call edges inside a repo | CodeQL call graph (Java, Go, C#, Python, JS/TS); Jelly with `--approx` for Node/TS; Jarvis-class for Python; `go/callgraph` VTA for Go | Mark static edges `inferred`; LLM-resolved edges (SemTaint style) `assumed` unless confirmed. |
| Data flow within a step (`data_in`/`data_out`) | CodeQL data-flow/taint path queries; Joern data-flow engine as an open-source alternative | Use value-preserving vs. taint distinction for "message changes form". |
| Branches on input classes | Guard extraction from CFG/CodeQL `Guard`s + LLM naming + Z3/abstract-interpretation feasibility for simple guards; Pulse-like disjunct pruning at call sites | No off-the-shelf TS/Python/Go tool gives this; build a thin layer. |
| Cross-repo/service links | Code2DFD-style endpoint/client/topic extraction per repo + matching by URL template, proto method, topic | Record `inferred` with evidence; unmatched calls become unwired ports. |
| Incrementality | Function-level summaries keyed by content hash; CodeQL overlay/diff-informed analysis | Matches lazy, cached zoom levels. |
| Library behavior | Behavior packs (below); fall back to analyzing fetched dependency source with same pipeline | Like CodeQL's precedence: curated pack > generated pack > on-the-fly analysis. |

**Behavior pack representation (proposed, derived from MaD + Pulse):**
- Key: `ecosystem/package@version` + callable path in MaD-like access-path syntax (`Member[sign].Argument[2].Member[algorithm]`).
- Rows/cases per public callable:
  - `cases[]`, each with `guard` (predicate over named input classes or access-path constraints, e.g. `Argument[2].Member[algorithm] in {"HS256"}`), `effects` (event kinds from our fixed vocabulary), `flows` (MaD-style input → output with `value|taint`), `calls_down` (links to lower packs), and `latent` (obligations the caller must satisfy, which become assumptions if unbound).
  - A `default` case.
  - `neutral: true` for callables checked to have no relevant behavior.
- IoC `slots` (e.g., `route_handler`, `middleware[]`, `di_provider`), mirroring `GuardedRouteHandler`, `DecoratedParameter` and JackEE rules.
- `provenance`: CodeQL's `{origin}-{verification}` scheme (`df-generated`, `ai-generated`, `ai-manual`, `manual`, plus our `trace-checked`), with precedence manual > verified-generated > generated, and neutral overriding generated.
- Bounded size: cap cases per callable and merge cases with identical observable effects, to keep summaries abstract and avoid path explosion.
- Validation in CI per version: run the pack's claimed flows against the library's own source with the same static pipeline (CodeQL/Joern), and against its test suite traces where available. A mismatch demotes the pack's provenance.

**Rules of engagement for the LLM:**
1. The LLM proposes summaries, input-class names, unresolved call edges and boundary matches. Static analysis or a solver confirms what it can. Anything unconfirmed is `assumed`.
2. Never let LLM narration prune a branch as infeasible (Evident result). Only a solver or analyzer prunes. Otherwise show the fork.
3. Expose unsoundness explicitly: unknown callee, reflection/eval/dynamic import, unmodeled framework slot. Each becomes a visible marker on the scenario graph, never a silent gap.

### Gaps
- I found no benchmark of how well LLM-generated summaries keyed by input classes match observed traces. The checkpoint's producer 2 (trace alignment) could itself serve as that evaluation.
- I did not research behavior summaries for RFC state machines (TLS, TCP, HTTP/2) by protocol-state-machine extraction or model learning. That falls outside this topic and should be covered separately.
- I did not research TypeScript-specific type-driven analysis (TS compiler API, stack graphs, SCIP-typescript) for precision on typed code; worth a follow-up.
