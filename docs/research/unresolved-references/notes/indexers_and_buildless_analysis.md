# Indexers and buildless analysis: how tools cope with unbuildable or partially resolvable code (state as of November 2026)

Reading note: findings marked "(search summary)" come from a search-result summary of the cited page, not from a full read of the page. Findings marked "(unverified)" could not be confirmed against a primary source. Inferences are mine.

## CodeQL: build modes, buildless extraction, accuracy and limits

### Takeaway
GitHub has moved CodeQL's default from "observe a real build" to `build-mode: none` for Java (2024), C# (2024), C/C++ and Rust (GA October 2025), because requiring a working build was the main obstacle to enabling scanning at scale. Buildless mode does not ignore dependencies: it queries Maven/Gradle or restores NuGet packages without compiling, guesses when it cannot, and loses generated code. GitHub publishes one hard number: over 70% success with no manual intervention across more than 10,000 C/C++ repositories.

### Cited Findings
- Three build modes exist: `none`, `autobuild`, `manual`. GitHub's own comparison table rates analysis success without user configuration as "Yes" for `none`, "Variable" for `autobuild`, "No" for `manual`; completeness as "Generated code not analyzed" for `none`; accuracy as "Good" for `none` and `autobuild`, "Best" for `manual`. — [GitHub Docs: CodeQL code scanning for compiled languages](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)
- Per-language modes: C/C++ `none`/`autobuild`/`manual` (default setup uses `none`); C# all three (default `none`); Java all three (default `none`); Kotlin `autobuild`/`manual` only; Go `autobuild`/`manual` only; Swift `autobuild`/`manual` only (macOS only); Rust `none` only. — [GitHub Docs: build options for compiled languages](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- Java `none`: "CodeQL will attempt to run Gradle or Maven to extract accurate dependency information (but not to invoke a build)." On version clashes "more recent dependency versions are preferred." If build scripts cannot be queried, "dependency guesses (based on Java package names) are inaccurate." A private Maven registry defined for the organization is used. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- Java `none` accuracy caveats: code generated during the build is not analyzed; CodeQL uses the highest JDK version required by any build script, so files needing older JDKs may be only partially analyzed; multiple dependency versions and colliding class names (several files defining the same fully qualified class) reduce accuracy. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- General statement: "Creating a CodeQL database without a build may produce less accurate results" when "the build scripts cannot be queried for dependency information, and dependency guesses are inaccurate" or "the repository normally generates code during the build process." — [GitHub Docs: compiled languages](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)
- Kotlin: "Kotlin analysis requires a build", so repositories containing Kotlin get `autobuild` in default setup; if Kotlin is added to a repository already on `none`, the Kotlin code is not analyzed and a warning is produced. — [GitHub Docs: compiled languages](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)
- C# `none`: "CodeQL restores dependencies and generates a few additional source files, to give more accurate results." Dependencies come from `*.csproj`, `*.sln`, `nuget.config`, `project.assets.json`; private NuGet feeds are used if configured. Generated global usings and `.cshtml`-to-`.cs` conversion "significantly increase the correctness of the CodeQL database." "When there are problems restoring dependencies or compiling the source code, this can affect the accuracy"; colliding class names can cause missing call targets. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- C/C++ `none`: "CodeQL will infer C/C++ compilation units through source file extensions"; compiler flags and include paths are inferred from the codebase. Accuracy drops when code relies on custom macros or has many external dependencies; making dependencies available and running on the target platform helps. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- Rust `none`: "CodeQL uses rust-analyzer to compile and run build scripts (build.rs files) and compile macro code, but does not invoke a full build." Requires `Cargo.toml` or `rust-project.json`, plus `rustup` and `cargo`. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- Autobuild heuristics: Java checks the root for Gradle, then Maven, then Ant, then direct subdirectories, and errors if several subdirectories have build files. Go tries `make`, `ninja`, `./build`, `./build.sh`, then falls back to `go.mod`-based dependency install, then a GOPATH rearrangement, and finally extracts all Go code. — [GitHub Docs: build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)
- Timeline: Java buildless public beta March 2024 (CodeQL 2.16.5, default for newly configured Java repositories). — [GitHub changelog, 2024-03-26](https://github.blog/changelog/2024-03-26-codeql-can-scan-java-projects-without-a-build/). Java and C# GA 28 August 2024; GitHub says it "validated that analysis results were as accurate as the previous methodology" and that the change gives "higher setup success and adoption rates", with no figures. — [GitHub changelog, 2024-08-28](https://github.blog/changelog/2024-08-28-codeql-code-scanning-can-analyze-java-and-c-codebases-without-needing-a-build-ga/)
- C/C++ buildless: public preview June 2025 (CLI 2.21.4). — [GitHub changelog, 2025-06-03](https://github.blog/changelog/2025-06-03-codeql-can-be-enabled-at-scale-on-c-c-repositories-in-public-preview-using-build-free-scanning/). GA 14 October 2025 with CLI 2.23.3 and GHES 3.20: during the preview "over 10,000 repositories" were enabled "with a success rate above 70%" and no manual intervention; one customer enabled and scanned over 1,400 repositories in under 48 hours. — [GitHub changelog, 2025-10-14](https://github.blog/changelog/2025-10-14-codeql-scanning-rust-and-c-c-without-builds-is-now-generally-available)
- Lombok: in August 2023 GitHub announced improved CodeQL support for Java code using Project Lombok (build-based extraction). — [GitHub changelog, 2023-08-31](https://github.blog/changelog/2023-08-31-code-scanning-with-codeql-improves-support-for-java-codebases-that-use-project-lombok/)

### Inferences
- The 70% figure measures "scan completed", not detection accuracy. GitHub gives no precision or recall comparison between `none` and build-based modes in any source read here; "Good" versus "Best" is the only public grading.
- The design pattern is worth copying: ask the build tool for the dependency graph without compiling (cheap, usually works), fall back to name-based guessing, and surface diagnostics about what could not be resolved, instead of failing the whole extraction.
- Languages that need compiler-level semantics that are hard to reimplement (Kotlin, Swift) and Go (where the toolchain is the resolver) still require a build in CodeQL. That is a signal about where buildless resolution is hardest.
- Lombok-generated members under `build-mode: none` fall under the "generated code not analyzed" class of limitation; the docs do not say so explicitly.

### Gaps
- No published success rate for Java or C# `autobuild` versus `none`. GitHub only says setup success is higher.
- No primary statement found on how Lombok behaves under `build-mode: none`.
- The exact Java dependency-guessing mechanism (how package names map to artifacts) is not documented in the pages read.

## SCIP and LSIF indexers: what each needs from the build, auto-indexing, cross-repository symbols

### Takeaway
SCIP indexers are compiler-accurate because they run inside or on top of the real compiler or type checker, so each needs a resolvable dependency environment and, for JVM and C/C++, a working compile. Sourcegraph's answer to a failed or absent index is a silent fallback to search-based (ctags plus text search) navigation. Cross-repository linking rests on symbol strings that embed package manager, package name and version.

### Cited Findings
- Sourcegraph lists these indexers as generally available: scip-go (Go), scip-typescript (TypeScript, JavaScript), scip-clang (C, C++, CUDA), scip-java (Java, Kotlin, Scala), rust-analyzer (Rust), scip-python (Python), scip-ruby (Ruby), scip-dotnet (C#, Visual Basic). — [Sourcegraph Docs: precise code navigation](https://sourcegraph.com/docs/code-navigation/precise-code-navigation)
- Fallback: Sourcegraph "automatically uses Precise Code Navigation whenever available"; search-based navigation is the fallback otherwise. For "complex or authenticated builds" Sourcegraph recommends running the indexer in the customer's CI, reusing existing build configuration and secrets. — [Sourcegraph Docs: precise code navigation](https://sourcegraph.com/docs/code-navigation/precise-code-navigation)
- scip-java "is implemented as a Java compiler plugin that runs as part of your regular compilation in the build tool" (SemanticDB plugin, then conversion to SCIP). `scip-java index` works automatically only for Gradle, Maven and sbt; other build tools need manual configuration in two phases (compile with the plugin, then `index-semanticdb`). Kotlin goes through a separate SemanticDB compiler plugin (scip-kotlin). (search summary) — [scip-java: getting started](https://sourcegraph.github.io/scip-java/docs/getting-started.html), [scip-java: manual configuration](https://sourcegraph.github.io/scip-java/docs/manual-configuration.html)
- scip-clang requires a JSON compilation database (`compile_commands.json`, from CMake export or Bear for Make builds) and is run from the project root. (search summary) — [sourcegraph/scip-clang](https://github.com/sourcegraph/scip-clang)
- scip-python (built on Pyright) uses `pip` to determine names and versions of packages in the active environment; the virtual environment should be activated first, or an `--environment` JSON file supplied. (search summary) — [scip-python README (fork mirror)](https://github.com/codean-io/scip-python)
- Auto-indexing inference: Lua "recognizers" request file patterns from a repository and return index job configurations (indexer image, root, `local_steps` for dependency setup, `indexer_args`, outfile, requested environment variables). Go and Java/Scala/Kotlin have default inference. The Java job runs `scip-java index --build-tool=auto`. Defaults cap inference at 100 jobs per repository. Private artifact repositories need credentials passed as environment variables. (search summary) — [Sourcegraph Docs: inference configuration](https://sourcegraph.com/docs/code-navigation/inference-configuration.md)
- SCIP symbol grammar: `<symbol> ::= <scheme> ' ' <package> ' ' (<descriptor>)+ | 'local ' <local-id>`, where package is `<manager> <package-name> <version>`. Descriptors form "a fully qualified name for the symbol" unique "across the package". Local symbols "MUST only be used for entities which are local to a Document". — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- SCIP `Index.external_symbols`: "Symbols that are referenced from this index but are defined in an external package (a separate `Index` message)." "Leave this field empty if you assume the external package will get indexed separately. If the external package won't get indexed for some reason then you can use this field to provide hover documentation for those external symbols." — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)

### Inferences
- A reference in SCIP always has a symbol, even when its definition lives in a package that was never indexed: the symbol names (manager, package, version, descriptor path). So an external reference is "resolved to an identity" without the target's source being present. This is exactly the shape Mental's extractor could use for external dependencies: record the edge to a named external entity, mark it as external, and stop.
- That only works because the indexer had the dependency's type information (classpath, node_modules, site-packages) at index time. With no dependency environment the compiler-based indexers either fail (scip-java: compile error means no SemanticDB for that unit) or emit degraded results (Pyright and tsc resolve unknown imports to `Unknown`/`any`, so no symbol is emitted for members reached through them).
- Sourcegraph's operational experience matches GitHub's: auto-inferred builds work for simple cases, and anything with private registries or custom build logic is pushed to the customer's CI.

### Gaps
- No Sourcegraph-published rate of auto-indexing success or failure.
- Could not read the scip-java page directly (404 on the canonical URL at fetch time); the findings come from search summaries of versioned mirrors (0.8.x to 0.12.x).
- Not verified per tool: behaviour of scip-typescript, scip-go, scip-dotnet, scip-ruby and rust-analyzer's SCIP output when dependencies are missing. From general knowledge (unverified here): scip-typescript needs `tsconfig.json` and installed `node_modules`; scip-go needs a loadable module (`go list` must work); scip-dotnet needs a restorable solution; scip-ruby is built on Sorbet; rust-analyzer needs `cargo metadata` to succeed.
- How Sourcegraph matches a package-version symbol to the repository that defines it is not specified in `scip.proto`.

## Kythe and Glean: capturing the exact compiler invocation

### Takeaway
Kythe and Glean sit at the "build required" extreme: they record each real compiler invocation as a hermetic compilation unit and index that. This gives exact results inside a monorepo with a uniform build system, and it is the approach GitHub and Sourcegraph both moved away from for arbitrary repositories.

### Cited Findings
- Kythe extraction under Bazel uses the extra-action mechanism: `extract_kzip` "is a Bazel extra action that records the compilation for each spawn action"; per-action `.kzip` files are collected from `bazel-out` and merged. (search summary) — [Kythe Bazel extractors (pkg.go.dev)](https://beta.pkg.go.dev/kythe.io@v0.0.67/kythe/go/extractors/bazel)
- For C++ outside Bazel, the Kythe `cxx_extractor` is run in place of the compiler for each entry of `compile_commands.json` (CMake path). (search summary) — [Kythe extractors (pkg.go.dev)](https://pkg.go.dev/kythe.io@v0.0.67/kythe/go/extractors)
- Glean is Meta's system for "collecting, deriving, and querying facts about source code"; indexing runs centrally on a shared fleet, not on developer machines. — [Engineering at Meta: Indexing code at scale with Glean, 2024-12-19](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/)
- The open-source Glean README lists native indexers for C++/C, Hack, Haskell and JavaScript/Flow, and SCIP/LSIF-based ingestion for Rust, Go, TypeScript, Java, Python and .NET. (search summary) — [Glean README (Hackage)](https://hackage.haskell.org/package/glean-0.1.0.0/src/README.md)

### Inferences
- Glean's reuse of SCIP and LSIF for most languages means it inherits those indexers' build requirements; Meta did not build a separate way around the build.
- Both systems presuppose an organization that controls its build. Neither offers a degraded mode for code that does not compile.

### Gaps
- How Glean's C++ indexer obtains compile commands (Buck integration or compilation database) was not found in a primary source.
- No published numbers on the fraction of Google or Meta code that fails extraction.

## Tools that deliberately avoid the build: Semgrep, Joern, stack graphs, search-based navigation

### Takeaway
Buildless tools trade resolution depth for coverage: they always produce something, and they resolve names by syntax, scope rules or text match. Almost none publish precision or recall for reference resolution. GitHub's stack graphs, the most ambitious attempt at precise buildless name resolution, was archived on 9 September 2025.

### Cited Findings
- Stack graphs design goal: code navigation should "Just Work" "without requiring any configuration from the repository owner", without "tapping into a build process or other CI job". Each file is analyzed in isolation into a graph of definitions, references and scope edges; graphs are merged at query time and a binding is a valid path from reference to definition. The post lists open problems: dataflow-dependent resolution and Java inheritance with generics. — [GitHub Blog: Introducing stack graphs, 2021-12-09](https://github.blog/open-source/introducing-stack-graphs)
- Stack-graph-based precise navigation shipped for Python first (December 2021); other languages kept search-based results. — [GitHub Blog: Precise code navigation for Python](https://github.blog/news-insights/product-news/precise-code-navigation-python-code-navigation-pull-requests/)
- `github/stack-graphs` was archived by its owner on 9 September 2025; the README says it is no longer supported or updated by GitHub and suggests forking. (search summary, read through a web-archive mirror) — [github/stack-graphs (archived snapshot)](https://web.archive.org/web/20260601045401/https:/github.com/github/stack-graphs); corroborated by [ecosyste.ms package record](https://packages.ecosyste.ms/api/v1/registries/crates.io/packages/tree-sitter-stack-graphs-java)
- GitHub's search-based navigation extracts symbols with tree-sitter in a syntax-aware way; 2021 coverage listed C#, Python, Go, Java, JavaScript, TypeScript, PHP, Protocol Buffers, Ruby, Rust. (search summary, secondary) — [InfoQ, December 2021](https://www.infoq.com/news/2021/12/GitHub-code-navigation-search)
- Sourcegraph search-based navigation is "built-in code navigation provided by search-based heuristics": go-to-definition is a symbol search (Universal Ctags), find-references is a case-sensitive, word-bounded plain-text search across repositories, optionally narrowed by file extension and, for some languages, by imports at the top of the file. — [Sourcegraph Docs: search-based code navigation](https://sourcegraph.com/docs/code-navigation/search-based-code-navigation)
- Joern "employs a fuzzy parser, allowing code to be imported even if a working build environment cannot be supplied"; for C the parser must work even when header declarations are missing. (search summary) — [Joern documentation (readthedocs)](https://joern.readthedocs.io/)
- Joern's Java source frontend (`javasrc2cpg`, built on JavaParser) has options that show how it handles missing types: `--fetch-dependencies` ("Attempts to fetch dependencies jars for extra type information"), `--inference-jar-paths` (extra jars for type information only), `--jdk-path`, `--delombok-mode` (`no-delombok | default | types-only | run-delombok`), `--enable-type-recovery` ("Enable generic type recovery"), `--no-dummyTypes` ("Disables the generation of dummy types during type propagation"), and `--skip-type-inf-pass` ("Results will be much worse, so should only be used for development"). — [Joern Docs: Java frontend](https://docs.joern.io/frontends/java/)
- Joern has per-language type recovery passes for dynamic languages (for example `JavaScriptTypeRecovery`) and a research subsystem, JoernTI, for type inference robust to missing code. (search summary) — [JoernTI paper, arXiv 2310.00673](https://arxiv.org/pdf/2310.00673.pdf)
- Semgrep Community Edition is single-file: "classes in Java are frequently defined across files (interfile), which is beyond the scope of Semgrep CE's analysis"; Semgrep Pro adds cross-file type inference, class hierarchy, constant propagation and taint. (search summary) — [Semgrep Docs: Java support](https://semgrep.dev/docs/languages/java)
- Semgrep cross-file analysis runs on full scans only, uses more time and memory, resolves names differently from CE, has documented gaps (certain CommonJS export patterns), and can fall back to single-file analysis so CI scans complete. (search summary) — [Semgrep Docs: cross-file analysis](https://semgrep.dev/docs/semgrep-code/semgrep-pro-engine-intro)

### Inferences
- Joern is the closest prior art to what Mental needs: parse without a build, attempt to fetch dependency jars only for type information, then run a type-propagation pass that invents "dummy" types for receivers it cannot resolve so calls still get a (guessed) full name. Joern itself says results are "much worse" without the type inference pass, which is a direct statement that unresolved types leave big gaps.
- Text-search find-references (Sourcegraph, ctags) has high recall for distinctive names and low precision for common ones (`get`, `run`, `id`); no vendor quantifies this.
- Stack graphs' archival suggests that hand-written per-language scope rules were too costly to maintain against the alternatives. GitHub gave no public reason in the sources read, so this is an inference.

### Gaps
- No precision or recall figures found for stack graphs, GitHub search-based navigation, Sourcegraph search-based navigation, Semgrep name resolution, or Joern call resolution.
- No primary Semgrep statement found on how Pro resolves types defined in third-party dependencies whose source is absent.
- Whether GitHub.com still serves stack-graph-based Python navigation after the repository was archived was not determined.
- The archive status was read through a mirror, not on github.com directly.

## Code-graph tools for LLM agents

### Takeaway
Agent-oriented tools split into three groups: tree-sitter name matching (Aider, Potpie, most "code graph" MCP servers), embeddings with no reference resolution at all (Cursor), and delegation to a language server (Serena, Claude Code LSP plugins). Only the third group resolves references precisely, and it inherits the language server's need for a working project setup.

### Cited Findings
- Aider's repo map parses files with tree-sitter tag queries, builds a graph of which files reference identifiers defined in which other files, and ranks with PageRank personalized by identifiers mentioned in the chat. (search summary, secondary) — [Agent Patterns: repository map pattern](https://www.agentpatterns.ai/context-engineering/repository-map-pattern/)
- Serena's `find_referencing_symbols` locates a symbol by name path and file, "then using LSP to find references at its location"; Serena claims support for over 40 languages through language servers, with capabilities limited by each server. (search summary) — [Serena (oraios/serena) tool listing](https://glama.ai/mcp/servers/oraios/serena/tools/find_referencing_symbols)
- Claude Code exposes language-server operations (go to definition, find references) through LSP plugins; the language server binary must be installed separately. A reported issue: `goToDefinition` does not follow `jdt://` URIs into Java jar dependencies. (search summary, secondary sources; verify names against official docs) — [Claude Code issue mirror: jdt:// URIs](https://claudeissues.com/issue/38453-bug-lsp-gotodefinition-should-support-jdt-uris-for-java-jar-dependency-navigatio)
- Cursor indexing: a Merkle tree of file hashes detects changes; changed files are chunked and embedded; embeddings and metadata are stored remotely. It is semantic search, not a reference graph. — [Cursor Blog: secure codebase indexing](https://cursor.com/blog/secure-codebase-indexing)
- Potpie parses a repository into a knowledge graph in Neo4j with files, functions, classes, imports and call relationships. — [potpie-ai/potpie](https://github.com/potpie-ai/potpie). Use of tree-sitter is stated only by a third-party summary (unverified).
- Greptile "builds a complete graph of your repository" and at review time looks up a changed function's direct calls, imports, accessed variables and callers. — [Greptile Docs: graph-based codebase context](https://greptile.com/docs/how-greptile-works/graph-based-codebase-context). A tree-sitter AST pipeline with generated docstrings is described only by secondary sources (unverified).

### Inferences
- None of these tools publish reference-resolution accuracy. The tree-sitter group matches a call to a definition by identifier name, so two methods named `save` in different classes are conflated or both linked.
- External dependencies are mostly invisible to this group: tree-sitter tools index only repository files, so a call into a library is an identifier with no definition. LSP-backed tools can resolve into dependencies only if the dependencies are installed and the server can open them.
- The market gap Mental is looking at is real: precise tools need a build, buildless tools are name-based, and no agent-facing tool reports what fraction of references it failed to resolve.

### Gaps
- Aider's own documentation was not read directly; the description is from a secondary write-up.
- Claude Code's official LSP documentation was not read; tool and marketplace names differ between secondary sources.
- No evaluation found comparing agent task success with name-based versus precise graphs.

## Academic work on partial programs, missing dependencies and call graph recall

### Takeaway
Research shows that types in incomplete Java can be recovered heuristically with around 90% success on snippets, and that even fully built, whole-program static call graphs miss a meaningful share of real calls: median recall 0.884 for Java, about 70% for Python (PyCG), and 76% rising to 88% for JavaScript (Jelly). Missing references are therefore a property of static analysis in general, not only of buildless analysis.

### Cited Findings
- PPA (Dagenais and Hendren, OOPSLA 2008) is a framework for partial type inference that recovers declared types in partial Java programs, reported with a low error rate even when only one class is available. — [PPA paper (McGill Sable)](https://www.sable.mcgill.ca/ppa/oopsla2008.pdf). JCoffee's authors note that PPA works only with Java 1.4 and "is now deprecated". — [JCoffee, arXiv 2009.05090](https://ar5iv.labs.arxiv.org/html/2009.05090)
- JCoffee (2020) uses compiler error feedback to synthesize missing declarations: it made 8,220 of 9,133 BigCloneBench snippets compilable (just over 90%) within 10 compiler iterations, where fewer than 1% compiled beforehand; on a 30-snippet sample 93.3% of inferred types matched a manual reference fix; mean 1.64 seconds per snippet. — [JCoffee, arXiv 2009.05090](https://ar5iv.labs.arxiv.org/html/2009.05090)
- Sui et al. (ICSE 2020), "On the Recall of Static Call Graph Construction in Practice": median recall 0.884 on real-world Java programs against an oracle from high-coverage test executions; the main sources of unsoundness were objects allocated or accessed through native methods and JVM-initiated invocations, not ordinary reflection. (search summary) — [ICSE 2020 programme entry](https://2020.icse-conferences.org/details/icse-2020-papers/47/On-the-Recall-of-Static-Call-Graph-Construction-in-Practice)
- Helm et al. (ISSTA 2024), "Total Recall? How Good Are Static Call Graphs Really?" re-examines static call graph recall for Java using fuzzing-derived dynamic baselines on four programs. (search summary) — [Preprint (TU Dortmund)](https://sse.cs.tu-dortmund.de/storages/sse-cs/r/Publications/Preprints/dyncg-issta-2024.pdf)
- PyCG (ICSE 2021): precision about 99.2%, recall about 69.9% on its macro-benchmark; missing edges are attributed to the approach's limits and unsupported Python features. — [PyCG, arXiv 2103.00587](https://arxiv.org/pdf/2103.00587)
- Jelly, approximate interpretation (Laursen, Xu, Møller, PLDI 2024): on 36 JavaScript projects with dynamic call graphs, "average analysis recall is improved from 75.9% to 88.1% with a negligible reduction in precision". (search summary) — [Reducing Static Analysis Unsoundness with Approximate Interpretation](https://cs.au.dk/~amoeller/papers/approx)
- Jelly, indirection-bounded analysis (ECOOP 2024): a faster variant with "only 5% reduction in recall and almost identical precision" relative to the baseline. (search summary) — [Indirection-bounded call graph analysis](https://cs.au.dk/~amoeller/papers/bounded/paper.pdf)
- A JavaScript study notes that CodeQL, Semgrep and SonarQube "exclude dependencies from being analyzed by default". (search summary) — [arXiv 2601.10865](https://arxiv.org/pdf/2601.10865)

### Inferences
- Two different kinds of gap should be kept apart in Mental's model: (a) the reference target is outside the analyzed code (external dependency), which is an identity problem and solvable by naming the external entity; (b) the target is inside the code but cannot be determined statically (dynamic dispatch, reflection, dependency injection, dynamic language features), which costs 10% to 30% recall even with a perfect build.
- JCoffee-style and PPA-style inference shows that a missing dependency does not make a file unanalysable: the shape of the missing type can be reconstructed from its uses well enough to compile about 90% of snippets. The reconstructed types are placeholders, though, with no link to the real library entity.
- Python and JavaScript recall figures are measured against dynamic call graphs from tests, so they understate true recall gaps where tests do not reach.

### Gaps
- SnR (snippet repair with import inference) figures were not found; no number recorded.
- No PPA accuracy figure extracted from the primary PDF.
- JavaParser symbol solver failure rates on projects with missing jars: no study found.
- No study found that directly measures call graph recall as a function of the fraction of missing library dependencies. Ali and Lhoták, "Application-Only Call Graph Construction" (ECOOP 2012) and Reif et al., "Call Graph Construction for Java Libraries" (FSE 2016) are the standard references, and their results were not retrieved.

## How often repositories build out of the box

### Takeaway
Out-of-the-box builds fail often: 38% of about 7,200 GitHub Java projects in 2016, 59% in a 2020 replication, and roughly half of the top 200 Java projects in 2017, with dependency resolution the leading cause. LLM agents in 2025 and 2026 reach 70% to 86% on curated benchmarks and much less on hard or multi-language sets, at the cost of minutes to an hour per repository.

### Cited Findings
- Sulír and Porubän (PLATEAU 2016, JDK 8): of 7,264 GitHub Java projects with a root build file, 38.05% failed to build. Maven failed 34.2%, Ant 55.6%. Error categories: dependency-related 39.2%, Java compilation 22.1%, documentation generation 6.4%, missing or invalid configuration properties 4.7%, missing files 3.7%. — [A Quantitative Study of Java Software Buildability, arXiv 1712.01024](https://ar5iv.labs.arxiv.org/html/1712.01024)
- Sulír et al. 2020 replication on a new project set with JDK 11: the proportion of failed projects rose to 59%. — [Local Software Buildability across Java Versions (registered report), arXiv 2408.11544](https://ar5iv.labs.arxiv.org/html/2408.11544). Dataset: 7,264 build logs. — [Large-Scale Dataset of Local Java Software Build Results, MDPI Data 2020](https://www.mdpi.com/2306-5729/5/3/86)
- The same registered report cites CI build failure rates of 17.9% (Kerzazi et al.) and, at Google, about 37% for C++ and 29.7% for Java (Seo et al.). — [arXiv 2408.11544](https://ar5iv.labs.arxiv.org/html/2408.11544)
- Hassan et al. (ESEM 2017), top 200 Java projects on GitHub with default build commands: about half failed. Search summaries give inconsistent counts: "99 of 200 cannot be built" in one reading, "86 projects with build failures" of which 52 could be fixed by command extraction or version reverting in another. A 2022 replication (Yasi and Qin) reports 157 projects (52%) failing against 49% for the earlier paper. (search summary; exact count unverified) — [Hassan et al., ESEM 2017](https://foyzulhassan.github.io/files/ESEM17_Full_Paper.pdf); [Yasi and Qin replication record](https://katalog.lib.cas.cz/KNAV/EdsRecord/edsndl,edsndl.UPSALLA1.oai.DiVA.org.miun.45928)
- Tufano et al. (JSEP 2017), 100 Java projects: broken (non-compilable) snapshots occur in 96% of projects, mainly from dependency resolution problems; on average only 38% of a project's change history is currently compilable. (search summary) — [There and back again: Can you compile that snapshot?](https://microsoft.com/en-us/research/?p=771955)
- Python snippets: DockerizeMe (2019) removed import errors for 892 of 2,891 Gistable gists that still failed with `ImportError` after baseline inference. (search summary) — [DockerizeMe, arXiv 1905.11127](https://ar5iv.labs.arxiv.org/html/1905.11127)
- Python libraries: in a PyPI study, PyEGo achieved the highest pass rate, 65.0%; Pipreqs and DockerizeMe covered only 20% to 50% of library releases; about 86% of DockerizeMe failures were missing libraries for direct imports. (search summary) — [Less is More? Configuration issues in the PyPI ecosystem, arXiv 2310.12598](https://arxiv.org/pdf/2310.12598)
- ExecutionAgent (2024/2025): built and ran test suites for 33 of 50 projects (66%) across 14 languages; average 74 minutes and USD 0.16 in LLM cost per project; 7.5% deviation from ground-truth test results. — [ExecutionAgent, arXiv 2412.10133](https://arxiv.org/abs/2412.10133)
- Repo2Run (NeurIPS 2025): built executable environments for 361 of 420 Python repositories (86.0%). — [Repo2Run, arXiv 2502.13681](https://arxiv.org/html/2502.13681v3)
- RepoLaunch (2026): build success of about 70% across nine languages, 78% in a later version. (search summary) — [arXiv 2603.05026](https://arxiv.org/pdf/2603.05026v2)
- Multi-Docker-Eval (December 2025): across 40 repositories, the best model's fail-to-pass success is at most 37.7%. (search summary) — [Multi-Docker-Eval, arXiv 2512.06915](https://arxiv.org/html/2512.06915v3)
- SetupBench (2025): OpenHands reached 38.9% to 57.4% on repository setup tasks. (search summary) — [SetupBench, arXiv 2507.09063](https://arxiv.org/abs/2507.09063v1)

### Inferences
- The older Java studies and GitHub's 70% C/C++ buildless figure point the same way: for an arbitrary repository, expect a working build somewhere between 40% and 70% of the time without manual help. An extractor that requires a build will fail on a large minority of repositories.
- The dominant failure cause is dependency resolution (39% of failures in 2016, also named first by Tufano). That is the same input a buildless extractor needs for type information. So "ask the package manager for dependencies without compiling" fails less often than a full build but shares its largest failure mode (dead repositories, private registries, snapshot versions).
- Agent-driven setup is now a viable fallback (66% to 86%), but at tens of minutes per repository it suits a one-off deep extraction, not routine indexing. For a local-first product running through the user's coding agent, the user's own machine usually already has a working build environment, which sidesteps the public-repository problem.
- Buildability decays with time: the 2020 replication's higher failure rate and Tufano's 38%-of-history figure both show that old commits are far less buildable than HEAD. Any plan to extract history should assume buildless extraction.

### Gaps
- Gradle-specific failure rate from Sulír and Porubän not extracted.
- Hassan et al. exact figures need checking against the PDF.
- No SWE-bench official figure for environment setup success; SWE-rebench (31% on Python repositories) and EnvBench (best 6.69%, 22 of 329 on hard repositories) figures appeared only in third-party summaries without a confirmed primary URL, so they are unverified.
- No recent (2024 to 2026) large-scale study of out-of-the-box buildability for non-Java ecosystems (npm, Cargo, Go) was found; the registered report for Java across JDK 6 to 23 had no results in the version read.
