# Unresolved references in static analysis: impact and mitigation

Research date: 2026-11-14. Older studies are dated inline. "Search summary" after a source means I saw the claim in a search-result summary of that page and did not open the page itself; treat those as leads to verify. Everything else was read from the page or from Mental's own docs.

## 0. Grounding: which parts of Mental's model depend on reference resolution

### Takeaway

Almost every structural record in the model sits downstream of the extractor's `resolve` step: ports (via slots), effects, state-variable write sites, call-graph edges, compile-unit ordering and wires. Mental already has the right primitive (`Resolved | External | Unresolved(Reason)` with a closed reason enum and a separate count), but the spec does not yet say when an unresolved fact fails the build, when it becomes an assumption, and how it propagates into "complete model" claims.

### Cited Findings

- The extractor's "Resolve names" step produces the symbol table, import graph and call graph with "Mental's own resolver, driven by the type rows of packs"; "What it cannot resolve is recorded as `unresolved` with a reason, never guessed" — /Users/vs/workspace/mental/docs/specification.md §6.2.1
- A call-graph edge is a distinct (caller, callee) pair where both functions are in the repository; a call that reaches a library through a pack becomes an effect or a slot; "A call that cannot be resolved is listed as `unresolved` and counted apart" — /Users/vs/workspace/mental/docs/specification.md §6.2.1
- Every lookup returns `Resolved(id) | External(qualified_name) | Unresolved(Reason)`; `Reason` is a closed enum: `not_found`, `wildcard_import`, `dynamic_attr`, `eval`, `dynamic_import`, `ambiguous`, `no_pack`, `not_converged`; each `Unresolved` becomes a fact — /Users/vs/workspace/mental/docs/guidelines/extract.md (line 87)
- "No heuristic edges, no name-similarity matching, no 'most likely' type." A binding's type comes only from its own statement (assignment, `with ... as`, annotation) combined with pack type rows; anything else is `unresolved` — /Users/vs/workspace/mental/docs/guidelines/extract.md (line 99)
- Records that depend on resolution, by phase (all from /Users/vs/workspace/mental/docs/specification.md §6.2, §6.2.1, §6.2.2, §6.4, §7):
  - **Entry points / provided ports**: found by framework packs through slots (`Decorates`, `WithArg` rows). The decorator or registration call must resolve to the pack's type for the port to exist at all.
  - **Effects**: "per call that reaches outside", from extractor plus packs ("the pack says `session.commit()` writes"). The receiver's type must resolve to a pack type.
  - **State variables and their write sites**: extractor plus the ORM pack. Lifecycle machines, the machine check and the atomicity check are computed from the write sites.
  - **Obligations**: branch points are syntactic, but effect obligations depend on effect resolution; the coverage check is only as complete as the obligation list.
  - **Compile units**: compiled "bottom-up along the call graph", with callee summaries substituted for callee code. A missing edge means a missing summary.
  - **Group**: components are boxes from "packages and call-graph clusters".
  - **Link**: wires are matched by key (task name, URL path, `DATABASE_URL` host); `match` is `exact`, `heuristic` or `assumed-env`; "What cannot be resolved becomes an assumption".
  - **Close**: required ports with no wire get a stub.
- Already-recorded uncertainty vocabulary: provenance `inferred | attested | assumed | observed | hypothetical`; wire `match: exact | heuristic | assumed-env`; anchor `status: exact | moved | broken`; interpreter stuck outcome "Model gap: a message arrived that no case handles, or a call could not be resolved" — /Users/vs/workspace/mental/docs/specification.md §4.2, §4.4, §4.5, §8.5
- "`mentalc` output is complete. A build either produces a full model or fails; it never leaves gaps for another tool to fill" and "a degraded model would later be read as complete" — /Users/vs/workspace/mental/docs/specification.md §3, §6.5
- Mental chose not to use a SCIP indexer or language server because they "would bring Node or a Python environment, against running locally with nothing else installed" — /Users/vs/workspace/mental/docs/specification.md §6.2.1
- Mental's own finding F-R1: static call graphs are more complete than LLM-made ones, "about 85% against 60% on Python" — /Users/vs/workspace/mental/docs/findings.md (this matches the PyCG 84.9% vs mistral-large 60.3% "complete" figures in section 5 below)

### Inferences

- The worst failure for Mental is not a missing call edge but a **missing port, effect or write site**, because those are silent: an unresolved decorator means no provided port, hence no behavior, hence no finding, and nothing downstream knows something is absent. A missing write site makes a lifecycle machine look cleaner than the code and can hide exactly the races the tool exists to find.
- Mental's dependency on resolution is unusual: it needs library symbols resolved only up to **the pack's type rows**, not to library source. So the "external library unresolved" problem becomes "no pack" (`no_pack`) or "receiver type not inferable", which is narrower than whole-program analysis.
- There is a tension to settle in the spec: "a build either produces a full model or fails" versus "each `Unresolved` becomes a fact". The evidence below suggests a threshold-plus-location rule (as CodeQL does) rather than either extreme.

### Gaps

- The spec does not define which unresolved reasons are errors, which are diagnostics, and which become `assumption` records, nor a per-unit resolution-rate metric.

## 1. Which facts are lost or corrupted when references are unresolved

### Takeaway

Measured recall of static call graphs is well below 100% even with a full build (median 0.884 for Java in 2020; about 70% for PyCG on real Python projects in 2021), and removing the build or the library code makes it worse in specific, predictable ways: missing call targets, missing types, missing generated code, and colliding class names. Framework-driven code is the extreme case, where conventional analyses reach only a small fraction of application code.

### Cited Findings

- (2020) Sui et al., "On the Recall of Static Call Graph Construction in Practice" (ICSE 2020): 31 real-world Java programs, median recall 0.884 against dynamic baselines; the main sources of unsoundness were objects allocated or accessed through native methods and JVM-initiated invocations with no call site, rather than reflection (search summary) — [ICSE 2020](https://2020.icse-conferences.org/details/icse-2020-papers/47/On-the-Recall-of-Static-Call-Graph-Construction-in-Practice)
- (2024) "Total Recall? How Good Are Static Call Graphs Really?" (ISSTA 2024) benchmarks eight call-graph implementations in four frameworks on four real-world Java programs against dynamic baselines built with fixed entry points, manual extension and coverage-guided fuzzing (search summary; I could not extract its numbers) — [ISSTA 2024](https://2024.issta.org/details/issta-2024-papers/10/Total-Recall-How-Good-Are-Static-Call-Graphs-Really-)
- (2019) Judge (Reif et al., ISSTA 2019): soundness-relevant language features are common in modern Java bytecode and support for them varies widely across Soot, WALA, DOOP and OPAL, so graphs from the "same" algorithm in different frameworks are not comparable (search summary) — [ISSTA 2019](https://conf.researchr.org/event/issta-2019/issta-2019-technical-papers-judge-identifying-understanding-and-evaluating-sources-of-unsoundness-in-call-graphs)
- (2018) Reif et al., SOAP 2018: precision of call-graph algorithms is well understood, recall is not (search summary) — [SOAP 2018](https://2018.ecoop.org/details/SOAP-2018-papers/3/Systematic-Evaluation-of-the-Unsoundness-of-Call-Graph-Construction-Algorithms-for-Ja)
- (2021) PyCG (ICSE 2021): on its macro-benchmark of real projects, average precision about 99.2% and recall about 69.9%; the missed edges come from design limits and unsupported features (search summary) — [PyCG, arXiv 2103.00587](https://arxiv.org/pdf/2103.00587)
- (2023) Jarvis claims at least 20% higher recall and 84% higher precision than PyCG on 135 micro-benchmark programs and 6 real applications (authors' numbers, search summary) — [Jarvis, arXiv 2305.05949](https://arxiv.org/html/2305.05949v5)
- (2024) On the PyCG micro-benchmark, PyCG is complete on 107 of 126 cases (84.9%) and sound on 110 (87.3%). On the JavaScript SWARM-JS benchmark, Jelly is complete on 49 of 126 (38.8%) and sound on 85 (67.4%); TAJS produced errors and empty output on 102 of 126 cases because it supports only ECMAScript 3 — [Venkatesh et al., arXiv 2410.00603v2](https://arxiv.org/html/2410.00603v2)
- (2024) Five static JavaScript call-graph tools agreed on only about 60% of call edges on 26 SunSpider programs and 6 Node.js modules; each found valid edges the others missed (search summary) — [Static JavaScript Call Graphs: a Comparative Study, arXiv 2405.07206](https://arxiv.org/html/2405.07206v1)
- (2024) Adding approximate interpretation (running code fragments to learn dynamic property accesses) to Jelly on 141 Node.js applications and libraries gave 55.1% more call edges, 21.8% more reachable functions and 17.7% more resolved call sites. These are gains over the purely static baseline, which shows how much the static baseline was missing (search summary) — [Reducing Static Analysis Unsoundness with Approximate Interpretation, PLDI 2024](https://pldi24.sigplan.org/details/pldi-2024-papers/48/Reducing-Static-Analysis-Unsoundness-with-Approximate-Interpretation)
- (2020) JackEE (PLDI 2020): for Java enterprise applications, framework-aware analysis achieved more than 4 times the completeness of conventional techniques (search summary). One slide deck gives average in-application reachability of 58% with JackEE against 14.48% with plain Doop and near zero with Soot and WALA; this figure is secondary (slides, not the paper) — [PLDI 2020](https://pldi20.sigplan.org/details/pldi-2020-papers/66/Static-Analysis-of-Enterprise-Applications-Haven-t-We-Solved-This-Problem-Yet-), [slides on Scribd](https://www.scribd.com/presentation/491732108/Static-Analysis-of-Java-Enterprise-Applications)
- What a no-build extraction loses, according to CodeQL's own documentation for Java `build-mode: none` — [GitHub Docs: CodeQL build options and steps for compiled languages](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-build-options-and-steps-for-compiled-languages):
  - It "may produce less accurate results" when build scripts cannot be queried, so "dependency guesses (based on Java package names) are inaccurate".
  - Code generated during the build is not analysed.
  - Only one version of a dependency is used ("usually chooses the newer version"); files needing another JDK are "partially analyzed".
  - Colliding class names "may cause missing method call targets, which has an impact on dataflow analysis".
  - Kotlin is not supported without a build.
  - For C/C++ without a build, compilation flags and include paths are inferred; accuracy drops when "the code depends heavily on custom macros/defines" or "has many external dependencies".
- In CodeQL's Java no-build mode, expressions whose callee could not be resolved are given an `ErrorType`, and the result of an unresolved call propagates as an erroneous type (search summary of the Java library changelog mirror) — [CodeQL Java CHANGELOG mirror](https://source-hub.etos-pro.ru/github/codeql/src/commit/addef2f1713ed270a56855304e46599e2c2b9d03/java/ql/lib/CHANGELOG.md)
- Joern's data-flow engine depends on calls having been resolved "to some degree"; for dynamic languages this needs an expensive type-recovery pass, external methods may have no stub, and fewer than 2 iterations give no interprocedural type recovery (search summary) — [Joern blog: How Interprocedural Data-flow Works in Joern (2024)](https://joern.io/blog/interproc-dataflow-2024), [x2cpg frontend passes javadoc](https://www.javadoc.io/static/io.joern/x2cpg_3/1.1.1403/io/joern/x2cpg/passes/frontend.html)
- Semgrep's cross-file analysis documents a concrete resolution gap: CommonJS functions defined and assigned to an export later are not tracked (search summary) — [Semgrep docs: cross-file analysis](https://semgrep.dev/docs/semgrep-code/semgrep-pro-engine-intro)

### Inferences

- **External-only unresolved** (library symbols missing, repository symbols fine) loses: the type of anything returned by a library call, hence every method call on that value (a cascade, shown by CodeQL's `ErrorType` propagation); overload selection among in-repository methods whose arguments have library types; members inherited from library base classes (a call to `self.save()` on a subclass of a library class looks like `not_found`); callbacks the library makes into repository code (inverted edges, see section 2); and effect classification. In Mental these are the pack-covered facts: effects, slots, state-variable writes.
- **In-repository unresolved** (no classpath or module graph at all) additionally loses: which of several same-named classes or functions is meant (the "colliding class names" case), re-exports and barrel files, wildcard imports, generated sources, conditional compilation, and multi-module edges whose module boundaries are defined only in the build configuration.
- Recall figures of 70 to 88% with a full build mean an extractor cannot treat "everything resolved" as "everything found". Resolution rate and recall are different metrics; only the first can be measured without ground truth.
- Tool disagreement (60% edge agreement across JavaScript tools) means a single tool's graph should carry its own coverage statement rather than be presented as the call graph.

### Gaps

- I found no study that isolates call-graph recall with library code present versus absent on the same programs with a dynamic baseline. The closest are JackEE (framework modelling present versus absent) and CodeQL's qualitative no-build caveats.
- No published figure for the accuracy difference between CodeQL build and no-build modes on the same corpus was found; GitHub's changelog posts announce the feature (beta March 2024, generally available 28 August 2024) without a comparison — [GitHub changelog 2024-08-28](https://github.blog/changelog/2024-08-28-codeql-code-scanning-can-analyze-java-and-c-codebases-without-needing-a-build-ga/)
- No measured data found for specific constructs named in the question (extension functions, implicit receivers, generics) under missing dependencies.

## 2. Dynamic and non-static wiring that a perfect build still cannot resolve

### Takeaway

Dependency injection, annotation routing, reflection and string-keyed messaging are not resolution failures; they are facts that live in framework semantics, and every serious tool recovers them with declarative framework models rather than with deeper analysis. This is the same idea as Mental's packs and slots, and the evidence says it is the single largest lever on completeness for framework-based code.

### Cited Findings

- (2020) JackEE layers a framework-independent vocabulary (components, entry points, injection) over a per-framework mapping written as Datalog rules on Doop; for example Spring's `@Controller` marks a controller and entry-point class and `@RequestMapping` an entry-point method (search summary) — [the morning paper on JackEE](https://blog.acolyer.org/2020/09/25/jackee)
- (2022) Jasmine (ASE 2022) extends Soot to model Spring core features (dependency injection, AOP), evaluated on Spring micro-benchmarks and 18 real-world Spring programs, improving Soot- and Doop-based tools (search summary) — [Jasmine, ASE 2022](https://conf.researchr.org/details/ase-2022/ase-2022-research-papers/26/Jasmine-A-Static-Analysis-Framework-for-Spring-Core-Technologies)
- When Joern cannot resolve a method internally, it looks the method up in default and user-supplied "semantic definitions" (summaries of how data flows through an external method) (search summary) — [Joern blog (2024)](https://joern.io/blog/interproc-dataflow-2024)
- Mental's packs follow CodeQL's models-as-data: rows name a package, type or function through a closed access-path grammar, with `Decorates` and `WithArg` added for slots — /Users/vs/workspace/mental/docs/specification.md §7
- Mental's link phase already treats string-keyed wiring as key matching: "a task name, a URL path against a route, a `DATABASE_URL` host against a compose service", with guesses marked `heuristic` — /Users/vs/workspace/mental/docs/specification.md §6.2.2
- (2024) Approximate interpretation recovers dynamic property accesses in JavaScript by executing fragments, a non-static technique (search summary) — [PLDI 2024](https://pldi24.sigplan.org/details/pldi-2024-papers/48/Reducing-Static-Analysis-Unsoundness-with-Approximate-Interpretation)
- (2020) For Java, reflection was not the dominant measured source of missed edges; native allocation and JVM-initiated calls were (search summary) — [ICSE 2020](https://2020.icse-conferences.org/details/icse-2020-papers/47/On-the-Recall-of-Static-Call-Graph-Construction-in-Practice)

### Inferences

- A useful split for Mental's reason enum: **unresolvable-by-construction** (`eval`, `dynamic_attr`, `dynamic_import`, monkey patching) versus **resolvable-with-a-model** (injection, routes, task names, `no_pack`) versus **resolvable-with-more-input** (`not_found` because a dependency or stub is absent). Only the third is helped by fetching dependencies; the second is helped only by packs; the first should become an explicit assumption or stub.
- String-keyed edges (routes, topics, task names, bean names) are best recorded as key matches with the key as evidence, which Mental already does with `match: exact`.

### Gaps

- I did not retrieve documentation for CodeQL's Spring/DI framework models or for IntelliJ's Spring bean resolution in this session; claims about them are omitted.
- No measured recall figure for Angular, NestJS, Guice or Dagger wiring was found.

## 3. Representing what could not be resolved

### Takeaway

Mature tools do three things instead of dropping an unresolved reference: label each result with how it was obtained (precise, syntactic, search-based), keep an explicit error or unknown node so the gap propagates visibly, and raise an analysis-level warning when the unresolved share crosses a threshold. CodeQL's threshold is concrete: 5% of expressions without a type or 5% of call targets unknown.

### Cited Findings

- CodeQL Java no-build mode reports a warning on the tool status page only when there are "significant analysis problems", defined as 5% of expressions lacking a type or 5% of call targets being unknown (search summary of the Java library changelog mirror; verify against the official changelog) — [CodeQL Java CHANGELOG mirror](https://source-hub.etos-pro.ru/github/codeql/src/commit/addef2f1713ed270a56855304e46599e2c2b9d03/java/ql/lib/CHANGELOG.md)
- CodeQL keeps unresolved callees in the database as expressions of `ErrorType` rather than omitting them (search summary, same source) — [CodeQL Java CHANGELOG mirror](https://source-hub.etos-pro.ru/github/codeql/src/commit/addef2f1713ed270a56855304e46599e2c2b9d03/java/ql/lib/CHANGELOG.md)
- Sourcegraph uses a priority order: precise navigation (from an indexer, compiler-accurate, cross-repository) when available; a "syntactic" tier when no precise data exists; search-based navigation as the fallback, which can match unrelated same-named symbols (search summary) — [Sourcegraph docs: code navigation](https://sourcegraph.com/docs/code-navigation/syntactic-code-navigation)
- (2021) GitHub introduced "precise" code navigation for Python built on stack graphs, alongside "search-based" results for other cases; stack graphs need no per-repository build configuration (search summary; I found nothing confirming current coverage in 2026) — [GitHub blog 2021-12-09](https://github.blog/2021-12-09-precise-code-navigation-python-code-navigation-pull-requests/), [Stack graphs: Name resolution at scale, arXiv 2211.01224](https://arxiv.org/abs/2211.01224v1)
- (2008) PPA (Partial Program Analysis for Java) attaches a type to every AST node even when the declaration is inaccessible and represents unknown declarations with a universal placeholder type (`UNKNOWNP.UNKOWN`); it states that disambiguating a partial program is undecidable, so the output may contain errors — [PPA project page](https://www.sable.mcgill.ca/ppa/)
- (2026) iResolveX annotates indirect call edges with confidence scores so that downstream users choose their own precision/recall trade-off (binary analysis, ML not LLM; search summary) — [iResolveX, arXiv 2601.17888](https://arxiv.org/abs/2601.17888)
- GRAPHIA ranks candidate targets for unresolved JavaScript calls: correct target first in over 42% of unresolved cases and in the top five in 72% (search summary of an aggregator listing; primary paper not opened) — [aitopics.org call-graph tag](https://aitopics.org/tag/call%20graph)
- PyCG is the counter-example: when it cannot resolve a call it returns with no record — /Users/vs/workspace/mental/docs/research/implementation-guidelines/report.md (citing PyCG `cgprocessor.py` L126-160)
- ruff's semantic model returns an explicit outcome enum for name reads (`Resolved`, `ImplicitGlobal`, `WildcardImport`, `UnboundLocal`, `NotFound`) and keeps unresolved references — /Users/vs/workspace/mental/docs/research/implementation-guidelines/notes/extract_static_analysis.md

### Inferences

- A three-part representation fits Mental's existing vocabulary without new concepts:
  1. **Per-reference fact**: `unresolved` with reason (exists), plus the syntactic call shape (name, arity, receiver expression) and, where cheap, a **candidate set** of same-named in-repository definitions recorded as candidates, never as edges. This keeps the "no heuristic edges" rule while preserving what a later resolver or the LLM needs.
  2. **Per-unit metric**: resolved, external and unresolved counts per function, compile unit and module, by reason. This is what CodeQL's 5% rule is computed from.
  3. **Build-level gate**: a threshold, failing with exit code 1 and a diagnostic that names the modules and the missing input (which package has no pack, which import was not found).
- "Fail rather than degrade" maps well to *where* the unresolved reference sits rather than only *how many* there are: an unresolved decorator on a function, an unresolved receiver on a call inside a handler that writes state, or an unresolved callee on a path to a port are each model-changing; an unresolved call to a logging helper is not. A location-weighted rule is stricter where it matters than a flat 5%.
- An unresolved callee inside a compiled behavior can be carried honestly as a `call` node wired to a stub with every declared outcome and provenance `assumed`; the interpreter already has "model gap" for the run-time side.
- Provenance on edges (Sourcegraph's precise / syntactic / search-based) corresponds to Mental's wire `match` field; the same field on call facts would let agent-proposed resolutions (section 5) be stored without being confused with extractor facts.

### Gaps

- Joern's placeholder strings for unresolved call targets (I recall `<unresolvedNamespace>` and `<unresolvedSignature>` in `methodFullName`) could not be confirmed from a source in this session.
- SCIP's handling of unresolved or local symbols and CodeQL's per-language extraction-diagnostics queries were not retrieved.
- The 5% thresholds come from a changelog mirror seen only as a search summary.

## 4. Recovery without a full build

### Takeaway

The best-documented no-build design is CodeQL's: query the build tool for dependency information without compiling, fall back to guessing dependencies from package names, and warn when quality drops. Partial-program inference recovers most but not all types (about 91% correct with 2.7% wrong in the classic 2008 study), which makes it a source of candidates rather than of facts under Mental's rules.

### Cited Findings

- Tier 1, ask the build tool without building: CodeQL's Java no-build mode will "run Gradle or Maven to extract accurate dependency information (but not to invoke a build)"; each root project file is queried; an organisation's private Maven registry is used when defined — [GitHub Docs: CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-build-options-and-steps-for-compiled-languages)
- Tier 1 fallback: when build scripts cannot be queried, CodeQL guesses dependencies from Java package names, and says these guesses can be inaccurate — [GitHub Docs: CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-build-options-and-steps-for-compiled-languages)
- Tier 2, restore from manifests and lockfiles: CodeQL's C# no-build mode restores dependencies using "multiple heuristics and strategies" with project, solution, NuGet and lock files as inputs, and generates extra source files (global usings, Razor views) that "significantly increase the correctness of the CodeQL database" — [GitHub Docs: CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-build-options-and-steps-for-compiled-languages)
- Tier 2/3 in Joern's Java source frontend: `--inference-jar-paths` supplies jars used only for type information; a fetch-dependencies flag attempts to download dependency jars for type information; skipping the type-inference pass makes results "much worse" (search summary) — [Joern docs: Java frontend](https://docs.joern.io/frontends/java/)
- Tier 4, heuristic resolution: Joern has a proposed change titled "JavaSrc2Cpg: Infer type by Namespace and arg/parameter size" (status unconfirmed; search summary) — [joernio/joern pull requests](https://www.github.com/joernio/joern/pulls)
- Tier 4, ranking: Sourcegraph's search-based navigation is name matching and can return unrelated same-named symbols (search summary) — [Sourcegraph docs](https://sourcegraph.com/docs/code-navigation/syntactic-code-navigation)
- Tier 5, partial-program inference: (2008) PPA recovered on average 91.2% correct types with 2.7% erroneous types on four large open-source systems when analysing a single class without its dependencies — [PPA project page](https://www.sable.mcgill.ca/ppa/)
- Tier 5: (2020) JCoffee makes partial Java snippets compilable by using compiler feedback to declare missing identifiers and stub classes (search summary) — [JCoffee, arXiv 2009.05090](https://arxiv.org/pdf/2009.05090)
- Tier 5: (2023) JoernTI learns type inference for JavaScript to improve data-flow analysis; 1,093 inferred types were manually graded (search summary) — [Learning Type Inference for Enhanced Dataflow Analysis, arXiv 2310.00673](https://arxiv.org/pdf/2310.00673)
- No-build name resolution: stack graphs resolve references with name-binding rules and no per-repository build configuration (search summary) — [arXiv 2211.01224](https://arxiv.org/abs/2211.01224v1)
- Semgrep describes scanning Java without lockfiles through a beta "Dynamic Dependency Resolution" feature (this is for dependency scanning; search summary) — [Semgrep docs: Java](https://docs.semgrep.dev/languages/java.md)
- Mental's current plan for infrastructure already uses a renderer of this kind: `docker compose config` — /Users/vs/workspace/mental/docs/specification.md §6.2.1

### Inferences

- What each tier recovers, mapped to Mental:

| Tier | Recovers | Does not recover | Runs repository code? |
|---|---|---|---|
| 1. Query build tool (dependency graph, source sets, module layout) | exact versions, module boundaries, source roots, generated-source locations | generated code itself; anything dynamic | yes for Gradle and often Maven (build scripts and plugins execute); no for pure-data manifests |
| 2. Lockfile plus fetch artifact or stub | exact API surface of dependencies at pinned versions | transitive versions absent from a lockfile; private registries without credentials | no (download only), if install scripts are not run |
| 3. API surface only (class signatures, `.d.ts`, typeshed stubs, pack type rows) | return types, inherited members, overload sets | library behavior, callbacks into user code (needs a pack) | no |
| 4. Name and arity heuristics | candidate targets | certainty; wrong when names collide | no |
| 5. Partial-program inference | most local types (about 91% in PPA, 2008) | a few percent are wrong, and wrong silently | no |

- For Mental, tier 3 is already the design ("pack type rows"); tiers 2 and 3 can be combined by generating type rows mechanically from stubs or signatures of the lockfile-pinned version, which keeps the result deterministic and declarative. Tiers 4 and 5 produce candidates, which Mental's rule forbids as edges but which are useful inputs to a verified resolver.
- Tier 1 conflicts with "works with nothing else installed" and with safety (section 7) whenever the build tool executes repository code; lockfiles and static manifests avoid both.
- CodeQL's two-step (query, else guess from package names, and warn) is a degrade-with-warning policy. Mental's stated principle argues for the stricter variant: query or pinned lookup, else an explicit `not_found` with the package named, and a build failure when the affected share or location crosses the gate.

### Gaps

- Not fetched this session, so stated here without citation: `go list -json`, `cargo metadata`, `mvn dependency:build-classpath`, the Gradle Tooling API and `tsconfig.json` resolution are the usual tier-1 interfaces. From background knowledge, `go list` and `cargo metadata` read manifests without running build scripts, while Gradle configuration executes build scripts and Maven can execute extensions and plugins; this should be verified against current documentation before the report relies on it.
- No measurement found of how much each tier recovers on a common corpus. Rustdoc JSON, typeshed coverage and `.d.ts` availability rates were not researched.
- Newer partial-program work surfaced only as titles (a constraint-based approach at ISSTA 2026 and "Incompleter" for Python, reported as 37% higher F1 than LExecutor); not verified — [ISSTA 2026 paper PDF (not opened)](https://cs.uwaterloo.ca/~cnsun/public/publication/issta26-a/issta26-a.pdf)

## 5. An LLM coding agent as the resolver of last resort

### Takeaway

Agents can set up environments, but not reliably: published success rates range from about 7% to 86% depending on language, benchmark difficulty and whether the agent gets execution feedback, at minutes to over an hour per repository. For direct resolution, LLMs are strong at local type inference (over 90% exact match on a micro-benchmark) and weak at whole call graphs (about 60% complete against 85% for PyCG), so the defensible role is to propose a resolution for a specific unresolved site that a deterministic checker then confirms.

### Cited Findings

Environment and build setup:

- (2025) ExecutionAgent (Bouzenia and Pradel, ISSTA 2025): successfully ran test suites for 33 of 50 projects; test results deviated from ground truth by 7.5%; average 74 minutes and about $0.16 of LLM cost per project; generic LLM-generated scripts managed 5 of 50 and AutoGPT 4 of 50 (search summary) — [arXiv 2412.10133](https://arxiv.org/abs/2412.10133v2)
- (2025) Repo2Run: built executable Docker environments for 361 of 420 Python repositories (86.0%), which the authors report as 77.0 percentage points above SWE-agent (authors' numbers; search summary) — [arXiv 2502.13681](https://arxiv.org/html/2502.13681v4)
- (2025) EnvBench (JetBrains Research; 329 Python and 665 JVM repositories chosen to exclude those a simple deterministic script can configure): the best approach configured 29.47% of JVM and 6.69% of Python repositories, using GPT-4o and GPT-4o-mini; success is checked by compilation (JVM) and a static missing-imports check (Python); approaches without error feedback commonly produce wrong setup scripts (search summary) — [EnvBench, arXiv 2503.14443](https://arxiv.org/pdf/2503.14443)
- (2025) SetupBench (93 instances): OpenHands succeeded on 38.9 to 57.4% of repository-setup tasks and 20.0 to 53.3% of local database configuration tasks (search summary) — [SetupBench, arXiv 2507.09063](https://arxiv.org/abs/2507.09063v1)
- (December 2025) Multi-Docker-Eval: overall success of current models is low (fail-to-pass at most 37.7%), with environment construction the main bottleneck (search summary) — [arXiv 2512.06915](https://arxiv.org/html/2512.06915v3)
- These benchmarks differ in scope and scoring, so the rates are not directly comparable; Repo2Run's 86% is Python-only on its own benchmark — [Repo2Run](https://arxiv.org/html/2502.13681v4), [EnvBench](https://arxiv.org/pdf/2503.14443)

Type inference and call resolution:

- (2024) On the TypeEvalPy micro-benchmark (860 annotations), exact matches: gpt-4o 806 (93.7%), mistral-large-2407 804 (93.5%), gpt-4o-mini 783 (91.0%), llama3.1-70b 770 (89.5%), against HeaderGen 563 (65.5%) and HiTyper 367 (42.7%) — [Venkatesh et al., arXiv 2410.00603v2](https://arxiv.org/html/2410.00603v2)
- (2024) On call graphs in the same study: PyCG complete on 107 of 126 cases (84.9%), sound on 110 (87.3%); the best LLM (mistral-large-2407) complete on 76 (60.3%), sound on 79 (62.6%); gpt-4o complete on 63, sound on 75. The LLMs missed edges through multi-level returns and indirect imports. On JavaScript, mistral-large was complete on 51 of 126 (40.5%) and sound on 54 (42.8%) against Jelly's 49 and 85. The study covered 24 LLMs — [arXiv 2410.00603v2](https://arxiv.org/html/2410.00603v2)
- The same study's TypeEvalPy benchmark was expanded from 860 to 77,268 annotations by auto-generation; mistral-large surpassed gpt-4o there (no figures retrieved) — [arXiv 2410.00603](https://arxiv.org/abs/2410.00603)

Hybrid static plus LLM:

- (January 2026) SemTaint runs CodeQL's JavaScript call-graph builder and hands only the unresolved call edges on vulnerability paths of interest to an LLM "CallGraph Agent", to limit cost; no precision or recall figures were visible (search summary) — [SemTaint, arXiv 2601.10865](https://arxiv.org/abs/2601.10865)
- (January 2026) iResolveX: a conservative analysis gives a high-recall candidate set and a learned step assigns confidence (search summary) — [arXiv 2601.17888](https://arxiv.org/abs/2601.17888)
- Mental's own rule set already encodes the division: the LLM never removes a branch or decides feasibility (F-R3: 27.5 to 45% of confirmed bugs lost when an LLM filters), and LLM-written specifications fed to a static engine double what it finds (F-R2: 27 to 55 known vulnerabilities) — /Users/vs/workspace/mental/docs/findings.md

### Inferences

- LLM type inference at about 90 to 94% exact match is in the same band as PPA's 91% (2008): good enough to propose, not good enough to assert. The remaining 6 to 10% would be silent errors in write sites and effects if accepted unverified.
- Deterministic verification of an agent-proposed resolution is straightforward when the proposal is shaped as a checkable claim:
  - **Proposed target symbol**: accept only if the symbol exists at the stated location (Mental's anchor check: the quote must be found in the file), the name matches the call site, the arity and keyword arguments are compatible, and, for a method, the proposed receiver type defines or inherits it. If the target is in the candidate set from section 3, the agent is choosing among extractor-found candidates, not inventing.
  - **Proposed type of a binding**: accept only with an anchor to the statement or signature that yields it (a return annotation, a constructor call, a stub entry), re-derived by the extractor from that anchor.
  - **Proposed missing dependency or stub**: accept only if the named package and version appear in the lockfile and the proposed type rows can be regenerated mechanically from the fetched artifact or stub; the agent's text is never the source of the rows.
  - **Proposed environment setup**: verified by its effect, not its transcript: re-run the extractor and require the unresolved count to fall, with the newly resolved references each passing the checks above.
- Accepted proposals should carry `source: llm` with the ledger call, a distinct provenance or `match` value, and pass through the existing `task next` / `task submit` and repair loop; they fit the `agent` backend because the agent may "read more code with its own tools".
- Cost framing for a local-first tool: environment setup at tens of minutes per repository with 7 to 86% success is a poor default step; per-site resolution of a bounded list of unresolved references (the SemTaint pattern) is cheap and bounded, and a failure leaves an explicit `unresolved` fact.
- Hallucination risk is asymmetric: a hallucinated edge can be rejected by the checks above; a hallucinated "this cannot be resolved, ignore it" must not be accepted at all, consistent with F-R3.

### Gaps

- Installamatic and TypeGen were not retrieved; no figures for them are given.
- No 2026 study was found that measures precision and recall of LLM resolution of individual unresolved call sites in Python with a deterministic verifier; SemTaint is the nearest and reports no such figures in what I saw.
- All LLM results above use 2024-era models (GPT-4o class). Results for 2026 models were not found and are likely better, but that is unverified.
- A Paderborn thesis evaluating 26 LLMs on call graphs for Python, JavaScript and Java surfaced but its numbers were not read — [Paderborn digital library](https://digital.ub.uni-paderborn.de/hs/download/pdf/8212050)

## 6. Reusing what the developer's machine already has

### Takeaway

By 2026 the developer's coding agent can commonly reach a live, fully resolved index: Claude Code exposes a language-server tool through plugins, JetBrains IDEs ship a built-in MCP server, and Serena wraps language servers or the JetBrains engine. These give resolution answers without `mentalc` shipping an indexer, but they depend on the user's environment being set up, so they suit the role of an optional oracle for verification rather than a required input.

### Cited Findings

- Serena is an MCP server for symbol-level retrieval and editing with two backends: language servers (described as supporting over 40 languages) and a paid JetBrains plugin that uses the IDE's analysis engine. Tools include find-referencing-symbols, find-declaration and find-implementations; the JetBrains backend adds type hierarchy (search summary of directory listings) — [Serena on mcpservers.org](https://mcpservers.org/servers/oraios/serena)
- Starting with version 2025.2, IntelliJ IDEA includes an integrated MCP server that external clients such as Claude Code and Codex can use; tools are individually enabled under Settings | Tools | MCP Server | Exposed Tools; `get_symbol_info` returns Quick Documentation content for the symbol at a position (search summary) — [IntelliJ IDEA help: MCP server](https://www.jetbrains.com/help/idea/2026.2/mcp-server.html)
- Find-references and go-to-definition were not among the built-in JetBrains MCP tool names in what I saw; a third-party "IDE Index MCP Server" plugin exposes them (search summary) — [jetbrains-index-mcp-plugin listing](https://claudemarket.ai/mcp/hechtcarmel/jetbrains-index-mcp-plugin)
- Claude Code has a built-in LSP tool with operations including goToDefinition, findReferences, hover, documentSymbol, workspaceSymbol, goToImplementation and call hierarchy (incoming and outgoing calls). Servers are registered by plugins through an `lspServers` field in `plugin.json`; the language-server binary must be installed by the user; official marketplace plugins such as `pyright-lsp` and `typescript-lsp` exist. Early versions had a startup race reported as fixed in v2.1.0 (search summary of third-party repositories and an issue thread, not the official docs page) — [anthropics/claude-code issue #15619](https://github.com/anthropics/claude-code/issues/15619), [boostvolt/claude-code-lsps](https://github.com/boostvolt/claude-code-lsps)
- Joern's `--inference-jar-paths` is an example of feeding already-present jars to a source-level analysis purely for types (search summary) — [Joern docs: Java frontend](https://docs.joern.io/frontends/java/)
- IntelliJ's own guidance notes that an IDE opened in safe mode has not resolved dependencies, so its index is incomplete in that state (search summary) — [IntelliJ IDEA help: Project security](https://www.jetbrains.com/help/idea/2025.3/project-security.html)
- Mental's `agent` backend already lets the agent "read more code with its own tools", and "no task depends on it" — /Users/vs/workspace/mental/docs/specification.md §6.5

### Inferences

- Two ways to reuse local state without new infrastructure:
  1. **Read-only caches as an artifact source** for tier 2/3: an existing virtualenv's `site-packages` (including `py.typed` packages and installed stubs), `node_modules` with `.d.ts`, `~/.m2`, the Gradle and Go module caches. Reading files executes nothing. The version found must equal the lockfile's version, otherwise it is a different input and should be rejected or flagged.
  2. **The agent's language-server or IDE tools as an oracle** inside a resolution task: the agent asks go-to-definition for an unresolved site and submits the target; `mentalc` verifies it deterministically as in section 5. The oracle's answer is then evidence the agent used, not something the model depends on.
- Making a model depend on an IDE index or language server breaks determinism (the model is "a function of the code, the compiler version, the packs and the LLM answers") unless the answer is captured in the ledger and re-checked on each cache hit, which the existing cache rules already require.
- A cache-derived resolution should record where the artifact came from (path, version, hash), in the spirit of `packs.lock`.

### Gaps

- The official Claude Code documentation page for code-intelligence plugins was not opened; details above are from third-party sources.
- No tool was found that documents building a code graph specifically from local build caches with lockfile verification; CI-artifact reuse was not researched.
- Serena details come from directory listings rather than its repository documentation.

## 7. Safety of running build tools from an untrusted repository

### Takeaway

Asking a build tool for its resolved model usually means executing repository-controlled code, and both major IDE families treat that as requiring explicit trust. For a tool that is supposed to run without human involvement, the safe default is to read only declarative inputs (lockfiles, manifests, caches) and never to run installs or build scripts; the JavaScript ecosystem moved the same way in 2025 to 2026 by turning install scripts off by default.

### Cited Findings

- VS Code opens an unfamiliar folder in Restricted Mode, which disables workspace settings that point to executables, tasks, debugging, and extensions that have not opted in to Workspace Trust; the documentation notes that Workspace Trust cannot stop a malicious extension from ignoring it (search summary) — [VS Code docs: Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)
- (February 2026) A VS Code team member is reported as saying Restricted Mode is for browsing and navigation, not active development, since development needs execution and therefore trust (secondary source; search summary) — [Visual Studio Magazine 2026-02-03](https://visualstudiomagazine.com/articles/2026/02/03/vs-code-team-member-blames-user-trust-decisions-in-repo-based-attacks.aspx)
- IntelliJ IDEA's safe mode disables the import for Gradle, Maven and sbt projects, "so no build scripts are executed and no dependencies are resolved"; trusted locations can be configured to load projects automatically (search summary) — [IntelliJ IDEA help: Project security](https://www.jetbrains.com/help/idea/2025.3/project-security.html)
- (2021) JetBrains states that building or running a Maven or Gradle project from the command line carries the same security risks as importing it into the IDE (search summary) — [IntelliJ IDEA 2020.3.3 release post](https://blog.jetbrains.com/idea/2021/03/intellij-idea-2020-3-3/)
- The Shai-Hulud npm worm spread through malicious preinstall and postinstall scripts, stealing credentials and infecting packages the victim could publish (secondary sources; search summary) — [Semgrep blog 2026: RIP post/preinstall scripts](https://semgrep.dev/blog/2026/rip-npm-postinstall-scripts-npm-v12-default-change)
- (2026) npm 12 is reported to turn lifecycle scripts off by default, with preinstall, install and postinstall running only for allow-listed packages; the older `ignore-scripts` setting has no allow-list and overrides the new behaviour. One report expected npm 12 around July 2026; I could not confirm it has shipped (secondary sources; search summary) — [The Register 2026-06-10](https://theregister.com/devops/2026/06/10/github-pulls-pin-on-npms-auto-run-scripts/5253453), [SecurityWeek](https://www.securityweek.com/npm-12-will-change-script-execution-behavior-to-prevent-supply-chain-attacks/amp/)
- pnpm disables dependency build scripts by default behind an allow-list (`onlyBuiltDependencies` in v10; sources disagree on the exact setting name in v11). Blanket `ignore-scripts` breaks packages such as esbuild, sharp, node-gyp dependencies, Cypress and Playwright (secondary sources; search summary) — [Mondoo blog 2026](https://mondoo.com/blog/npm-supply-chain-security-package-manager-defenses-2026)
- CodeQL's Java no-build mode does run Gradle or Maven to obtain dependency information, in GitHub's hosted runners rather than on a developer machine — [GitHub Docs: CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-build-options-and-steps-for-compiled-languages)
- The environment-setup agents in section 5 (Repo2Run, ExecutionAgent) work inside Docker containers (search summary) — [Repo2Run](https://arxiv.org/html/2502.13681v4)
- Mental's design rule that packs "are declarative data. They never execute" is the same boundary applied to library models — /Users/vs/workspace/mental/docs/specification.md §7

### Inferences

- A safety ladder for `mentalc`, from no execution upward:
  1. Parse lockfiles and static manifests; read existing caches. Nothing executes.
  2. Download pinned artifacts or stubs from a registry and read them as data (no install scripts, no `setup.py`). Network access, no code execution.
  3. Run a manifest-only tool query. Safe only for tools that do not evaluate repository code; needs per-tool verification.
  4. Run build-tool configuration (Gradle, Maven with extensions, `setup.py`, `build.rs`, procedural macros, npm installs with scripts). This executes repository code and should require the same explicit trust decision an IDE asks for, or a container.
- Steps 1 and 2 are compatible with "no human involvement"; step 4 is not, unless the user has already expressed trust (for example by having built the project, which is also when the caches in section 6 exist).
- When `mentalc` runs inside a coding agent, the agent's own permission system is the trust prompt. An agent-driven environment repair therefore inherits human approval or the agent's sandbox, and `mentalc` itself can stay execution-free: it only verifies the result.
- A build that needs step 4 and lacks trust should fail with a clear error naming the missing input and the command the user could run, consistent with "fail rather than degrade".

### Gaps

- Whether rust-analyzer's handling of `build.rs` and procedural macros in untrusted workspaces has changed by 2026 was not found; no source retrieved for Rust-specific risks.
- No source retrieved on sandboxing features of coding agents (for example container or OS-level sandboxes) as a mitigation.
- The npm 12 and pnpm 11 details rest on secondary reporting and should be checked against official release notes.
- Which tier-1 queries are free of repository code execution (`go list`, `cargo metadata`, Maven without extensions) was not verified this session.
