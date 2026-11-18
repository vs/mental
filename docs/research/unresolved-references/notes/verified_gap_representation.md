# Verified: how existing tools represent, count and report unresolved references

State as of 2026-11-14. Every item below was read in the primary source (repository file fetched raw, official docs page, or the paper PDF). Labels: CONFIRMED (quote + URL), REFUTED (what the source says instead), OUTDATED (was true, superseded), NOT FOUND (not in the sources read).

Repository HEADs read: `github/codeql` main @ `20ed6480`, `sourcegraph/scip` main @ `be1d9f76`, `sourcegraph/scip-python` scip @ `51dd05f5`, `joernio/joern` master @ `eb6f3285`, `golang/tools` master @ `cba9b804`, `rust-lang/rust-analyzer` master @ `114dcb8d`, `microsoft/TypeScript` tag `v5.9.2`.

One line per product:

- CodeQL: unresolved call = call with no callee; unresolved type = `ErrorType`; counts both per database; warns (never fails) under 85 % (Java, C#), 50 % / 20 % (Rust).
- Joern: writes placeholder strings into the name itself (`<unresolvedNamespace>`, `<unresolvedSignature>`, `<unknownFullName>`, `ANY`), plus dummy types; no counter found.
- SCIP: no "unresolved" concept in the schema; scip-python emits a file-local symbol documented `[unable to resolve module]`.
- Kythe: `lookup` node for names that cannot be resolved; `tnominal` for types known by name only; `complete` fact is about declaration vs definition, not about unresolved references.
- go/packages: per-package `Errors`, `TypeErrors`, `IllTyped`; `Load` itself does not fail.
- TypeScript: TS2307 / TS7016 error, falls back to `any`.
- rust-analyzer: `{unknown}` type in display, one named diagnostic per unresolved kind.
- clangd: fallback compile command, noted in the log.
- Eclipse JDT: opt-in "recovered" bindings flagged `isRecovered()`.
- Sourcegraph: two documented tiers, precise and search-based, with search-based as fallback.
- GitHub code navigation: current docs describe only a tree-sitter, name-search approach.
- Papers: Cgc/Averroes (one summary node / placeholder library), Sui (median recall 0.884), PyCG (99.2 % / 69.9 %), Jelly (75.9 % to 88.1 %; "percentage of resolved call sites" metric).

## CodeQL: ErrorType, database-quality metrics and thresholds (Java, C#, C/C++, Rust)

### Takeaway

CodeQL counts two things per database, calls with a call target and expressions with a known type, and raises a warning-severity diagnostic on the tool status page when either percentage drops below a threshold. The earlier "5 %" figure is outdated: the threshold was 95 % OK when introduced and is 85 % OK now for Java and C#; Rust uses 50 % (calls) and 20 % (expression types). The scan still "completed successfully"; nothing fails.

### Cited Findings

- CONFIRMED (a) `ErrorType` exists for types the extractor could not extract: "An `ErrorType` is generated when CodeQL is unable to correctly extract a type." — [Type.qll](https://github.com/github/codeql/blob/main/java/ql/lib/semmle/code/java/Type.qll) (line 680-686; `class ErrorType extends RefType, @errortype`)
- CONFIRMED (a) Introduced in java lib 0.3.1: "Added an `ErrorType` class. An instance of this class will be used if an extractor is unable to extract a type, or if an up/downgrade script is unable to provide a type." — [java/ql/lib/CHANGELOG.md](https://github.com/github/codeql/blob/main/java/ql/lib/CHANGELOG.md)
- CONFIRMED (a) Unresolved callee leads to `ErrorType` (java lib 0.8.10): "Java expressions with erroneous types (e.g. the result of a call whose callee couldn't be resolved during extraction) are now given a CodeQL `ErrorType` more often." — [java/ql/lib/CHANGELOG.md](https://github.com/github/codeql/blob/main/java/ql/lib/CHANGELOG.md)
- CONFIRMED (b, historical) The 5 % rule, java lib 2.0.0: "Java build-mode `none` analyses now only report a warning on the CodeQL status page when there are significant analysis problems-- defined as 5% of expressions lacking a type, or 5% of call targets being unknown." — [java/ql/lib/CHANGELOG.md](https://github.com/github/codeql/blob/main/java/ql/lib/CHANGELOG.md). The same entry continues: "Other messages reported on the status page are downgraded from warnings to notes".
- OUTDATED (b) The 5 % threshold was relaxed to 15 % (java queries 1.8.0): "the threshold for reporting database health issues has been lowered from 95% to 85% (if any metric falls below this percentage). These changes are visible on the tool status page." — [java/ql/src/CHANGELOG.md](https://github.com/github/codeql/blob/main/java/ql/src/CHANGELOG.md). Identical wording for C# — [csharp/ql/src/CHANGELOG.md](https://github.com/github/codeql/blob/main/csharp/ql/src/CHANGELOG.md)
- CONFIRMED The implementing query is `java/diagnostic/database-quality` ("Low Java analysis quality", `@kind diagnostic`). Current source: `CallTargetStatsReport::percentageOfOk(msg, value) and threshold = 85 or ExprTypeStatsReport::percentageOfOk(msg, value) and threshold = 85` — [DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED Exact Java metric definitions: OK calls are `count(Call c | exists(c.getCallee()))`, not-OK are `count(Call c | not exists(c.getCallee()))`; a good type is `exists(e.getType()) and not e.getType() instanceof ErrorType`, counted over expressions in source files only. Labels: "calls with call target", "calls with missing call target", "expressions with known type", "expressions with unknown type". — [DatabaseQuality.qll](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQuality.qll)
- CONFIRMED The percentage formula is shared across languages: `value = Stats::getNumberOfOk() * 100.0 / (Stats::getNumberOfOk() + Stats::getNumberOfNotOk())`, key "Percentage of " + ok text — [ReportStats.qll](https://github.com/github/codeql/blob/main/shared/util/codeql/util/ReportStats.qll)
- CONFIRMED The diagnostic fires if any metric is below its threshold, at warning severity (`select d, d.toString(), 1 /* Warning severity */`), and is shown in three places via `key = ["visibilityCliSummaryTable", "visibilityTelemetry", "visibilityStatusPage"] and value = "true"`. — [DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED Message text: "Scanning Java code completed successfully, but the scan encountered issues. This may be caused by problems identifying dependencies or use of generated source code." and "Addressing these issues is advisable to avoid false-positives or missing results." Each metric is printed as `msg + ": " + value.floor() + " % (threshold " + threshold.floor() + " %)"`. — [DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED The raw counts and percentages are also published unconditionally as a metric query (`@id java/telemetry/extraction-information`, `@tags summary telemetry`): `CallTargetStatsReport::numberOfOk`, `numberOfNotOk`, `percentageOfOk` and the same three for `ExprTypeStatsReport`. — [ExtractorInformation.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/ExtractorInformation.ql)
- CONFIRMED C#: same two metrics, both thresholds 85, id `csharp/diagnostic/database-quality`. Unknown type is `UnknownType`; missing-target calls exclude known-legitimate cases (`DelegateLikeCall`, `DynamicExpr`, `nameof`, several property/event patterns). — [csharp DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/csharp/ql/src/Telemetry/DatabaseQualityDiagnostics.ql), [csharp DatabaseQuality.qll](https://github.com/github/codeql/blob/main/csharp/ql/src/Telemetry/DatabaseQuality.qll)
- CONFIRMED C# exclusions were added to stop false alarms: "The excluded property access expressions are expected to have no target callables even in manual or autobuilt databases." — [csharp/ql/src/CHANGELOG.md](https://github.com/github/codeql/blob/main/csharp/ql/src/CHANGELOG.md)
- CONFIRMED Rust: `CallTargetStatsReport::percentageOfOk(msg, value) and threshold = 50 or ExprTypeStatsReport::percentageOfOk(msg, value) and threshold = 20`, id `rust/diagnostic/database-quality`. — [rust DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/rust/ql/src/queries/telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED Rust has a third metric, macro resolution (`MacroCallTargetStats`: "macro calls with call target" / "macro calls with missing call target", OK when `c.hasMacroCallExpansion()`), still reported in telemetry — [rust DatabaseQuality.qll](https://github.com/github/codeql/blob/main/rust/ql/src/queries/telemetry/DatabaseQuality.qll), [rust ExtractorInformation.ql](https://github.com/github/codeql/blob/main/rust/ql/src/queries/telemetry/ExtractorInformation.ql)
- CONFIRMED Rust macro metric was removed from the warning: "The macro resolution metric has been removed from `rust/diagnostic/database-quality`. This metric was found to be an unreliable indicator of database quality in many cases, leading to false alarms on the tool status page." — [rust/ql/src/CHANGELOG.md](https://github.com/github/codeql/blob/main/rust/ql/src/CHANGELOG.md)
- CONFIRMED Rust threshold was tuned down after false alarms: "has been tuned so that it won't trigger on databases that have extracted normally. This will remove spurious messages of \"Low Rust analysis quality\" on the CodeQL status page." — [rust/ql/src/CHANGELOG.md](https://github.com/github/codeql/blob/main/rust/ql/src/CHANGELOG.md)
- CONFIRMED Rust call metric excludes calls that are unresolvable by nature (dynamic calls, non-overloaded operations, raw-pointer derefs) and files "skipped by the compilation". — [rust DatabaseQuality.qll](https://github.com/github/codeql/blob/main/rust/ql/src/queries/telemetry/DatabaseQuality.qll)
- CONFIRMED C/C++: same two stats (unknown type is `ErroneousType`); an unresolved call is approximated: "We assume that calls with an implicit target are calls that could not be resolved. This is accurate in the vast majority of cases" — [cpp DatabaseQuality.qll](https://github.com/github/codeql/blob/main/cpp/ql/src/Telemetry/DatabaseQuality.qll)
- CONFIRMED C/C++ publishes them only as a metric (`@kind metric`, `@id cpp/telemetry/database-quality`, "Metrics that indicate the quality of the database."); the directory `cpp/ql/src/Telemetry` holds no `DatabaseQualityDiagnostics.ql`. — [cpp DatabaseQuality.ql](https://github.com/github/codeql/blob/main/cpp/ql/src/Telemetry/DatabaseQuality.ql), [directory](https://github.com/github/codeql/tree/main/cpp/ql/src/Telemetry)
- CONFIRMED C/C++ further success metrics, each paired with a baseline to form a percentage: "non-error expressions", "expressions with a known type", "calls with an explicit target", "variables with a known type", "functions without errors", "lines of code without errors", "successfully resolved #include directives", plus per-header counts `"Failed to include '" + includeText + "'"`. — [cpp Metrics.qll](https://github.com/github/codeql/blob/main/cpp/ql/src/Telemetry/Metrics.qll)
- CONFIRMED GitHub's stated cause of low quality in `none` mode: "Creating a CodeQL database without a build may produce less accurate results than using autobuild or manual build steps if: The build scripts cannot be queried for dependency information, and dependency guesses are inaccurate." — [docs.github.com](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)

### Inferences

- The design precedent is exact for `mentalc`: two ratios per unit (resolved calls, typed expressions), counts and percentage always published, a threshold that only decides warning versus silence.
- CodeQL had to exclude "unresolvable by nature" sites from the denominator (C# delegates, Rust dynamic calls) and drop a noisy metric (Rust macros). A per-reason classification avoids the same false alarms.
- The threshold moved 95 → 85 within the product's life, and Rust uses 50/20: no universal threshold exists; it is language- and extractor-specific.
- CodeQL never fails the scan on this signal; it is advisory.

### Gaps

- How `ErrorType` propagates through data flow (whether flow stops at an error-typed expression) was not read; only the class doc and two changelog lines were verified.
- Whether a C/C++ threshold diagnostic lives outside `cpp/ql/src/Telemetry` was not checked.
- The version in which the Rust thresholds 50/20 were set was not identified.

## CodeQL extraction diagnostics in general (warnings vs errors, tool status page)

### Takeaway

CodeQL diagnostics are query results of `@kind diagnostic` with a SARIF severity integer (1 warning, 2 error) and visibility attributes; the tool status page shows per-language file coverage and error messages. Detailed GitHub wording on "warning versus error" was only partly read.

### Cited Findings

- CONFIRMED Severity encoding in the Java diagnostics library: "Gets the SARIF severity level that indicates an error." returns 2; "Gets the SARIF severity level that indicates a warning." returns 1. — [DiagnosticsReporting.qll](https://github.com/github/codeql/blob/main/java/ql/src/Diagnostics/DiagnosticsReporting.qll)
- CONFIRMED Java maps extractor severities to user-facing errors: severity 6 → "Frontend errors in file: …", 7 → "Extraction incomplete in file: …", 8 → "Severe error: …", any other severity above 3 in a repo file → "Unknown errors in file: …"; a file is "successfully extracted" only if it has no reportable diagnostic. — [DiagnosticsReporting.qll](https://github.com/github/codeql/blob/main/java/ql/src/Diagnostics/DiagnosticsReporting.qll)
- CONFIRMED Tool status page shows coverage per language: "you can see a percentage total of all the files most recently scanned in your repository, organized by programming language. You can also download detailed language reports in CSV format." — [docs.github.com](https://docs.github.com/en/code-security/code-scanning/managing-your-code-scanning-configuration/about-the-tool-status-page)
- CONFIRMED The CSV report has per-file status: "Which configuration was used to scan each file / The file path / The programming language of the file / Whether the file was successfully extracted" — [docs.github.com](https://docs.github.com/en/code-security/code-scanning/managing-your-code-scanning-configuration/about-the-tool-status-page)
- CONFIRMED CLI option on `codeql database interpret-results`: "`--[no-]print-diagnostics-summary` Print a summary of the analyzed diagnostics to standard output." and "`--[no-]print-metrics-summary` Print a summary of the analyzed metrics to standard output." — [docs.github.com](https://docs.github.com/en/code-security/codeql-cli/codeql-cli-manual/database-interpret-results)
- CONFIRMED In `none` mode some content is silently outside the model, with a warning: "if build-mode is set to none and Kotlin code is found in the repository, the Kotlin code will not be analyzed and a warning will be produced." — [docs.github.com](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)

### Inferences

- CodeQL separates three channels: per-file extraction status, counted metrics (`@kind metric`), and thresholded diagnostics (`@kind diagnostic`). The same split fits `mentalc` (fact, count, gate).

### Gaps

- GitHub's prose definition of "warning" versus "error" on the tool status page, and the "Extraction errors" troubleshooting page, were not read in full.
- The exit-code behaviour of `codeql database analyze` when extraction errors occur was not verified.

## CodeQL Python: no dependency extraction; API graphs and models-as-data

### Takeaway

Since CLI 2.16.0 (January 2024) the Python extractor does not extract dependencies, and since 2.19.2 (October 2024) not even the standard library; GitHub says the effect on results is minimal because library behaviour is modelled. Library calls are referred to by import path through API graphs and extended by data-extension rows.

### Cited Findings

- CONFIRMED CLI 2.16.0 (2024-01-16): "The Python extractor will no longer extract dependencies by default." and "In versions until 2.17.0, it will be possible to restore the old behavior by setting `CODEQL_EXTRACTOR_PYTHON_FORCE_ENABLE_LIBRARY_EXTRACTION_UNTIL_2_17_0=1`." — [codeql-cli-binaries CHANGELOG.md](https://github.com/github/codeql-cli-binaries/blob/main/CHANGELOG.md)
- CONFIRMED CLI 2.19.2 (2024-10-21): "The Python extractor will no longer extract the standard library by default, relying instead on models of the standard library. This should result in significantly faster extraction and analysis times, while the effect on alerts should be minimal." Override: `CODEQL_EXTRACTOR_PYTHON_EXTRACT_STDLIB` set to `1`. — [codeql-cli-binaries CHANGELOG.md](https://github.com/github/codeql-cli-binaries/blob/main/CHANGELOG.md)
- CONFIRMED Preparation in python lib 2.1.1: "Added several models of standard library functions and classes, in anticipation of no longer extracting the standard library in a future release." — [python/ql/lib/CHANGELOG.md](https://github.com/github/codeql/blob/main/python/ql/lib/CHANGELOG.md)
- CONFIRMED Stated effect (2023-07-12): "CodeQL no longer needs to fetch these dependencies in order to analyze a codebase." and "This should improve scan times for Python projects, while having minimal impact on results." — [GitHub changelog](https://github.blog/changelog/2023-07-12-code-scanning-with-codeql-no-longer-installs-python-dependencies-automatically-for-new-users/)
- CONFIRMED Same post: "We plan to deprecate all dependency installation (including for existing users) by the end of 2023." and the opt-out setting `setup-python-dependencies: false`. — [GitHub changelog](https://github.blog/changelog/2023-07-12-code-scanning-with-codeql-no-longer-installs-python-dependencies-automatically-for-new-users/)
- NOT FOUND `CODEQL_EXTRACTOR_PYTHON_DISABLE_LIBRARY_EXTRACTION`: the string does not occur in the CLI changelog or the Python lib/src changelogs. The variables that do occur are `…FORCE_ENABLE_LIBRARY_EXTRACTION_UNTIL_2_17_0` and `…EXTRACT_STDLIB`.
- CONFIRMED API graphs: "API graphs are a uniform interface for referring to functions, classes, and methods defined in external libraries." — [Using API graphs in Python](https://codeql.github.com/docs/codeql-language-guides/using-api-graphs-in-python/)
- CONFIRMED Entry point is the import: "The most common entry point into the API graph will be the point where an external module or package is imported." Example `API::moduleImport("re").getMember("compile").getReturn().getAValueReachableFromSource()`. — [Using API graphs in Python](https://codeql.github.com/docs/codeql-language-guides/using-api-graphs-in-python/)
- CONFIRMED The node stands for the import, not a definition: "This node represents the fact that the re module has been imported rather than a specific location in the program where the import happens." — [Using API graphs in Python](https://codeql.github.com/docs/codeql-language-guides/using-api-graphs-in-python/)
- CONFIRMED Unknown names are first-class: "You can use the getAMember method to access all members, both known and unknown." — [Using API graphs in Python](https://codeql.github.com/docs/codeql-language-guides/using-api-graphs-in-python/)
- CONFIRMED Models-as-data: "Python analysis can be customized by adding library models in data extension files." Extensible predicates: `sourceModel(type, path, kind)`, `sinkModel(type, path, kind)`, `typeModel(type1, type2, path)`, `summaryModel(type, path, input, output, kind)`; also `barrierModel`, `barrierGuardModel`. — [Customizing Library Models for Python](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-python/)
- CONFIRMED Row shape: "The second column is an access path that is evaluated from left to right, starting at the values that were identified by the first column." Status: "Library customization using data extensions is currently in beta and subject to change." — [Customizing Library Models for Python](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-python/)

### Inferences

- An external library entity is identified by (import path, member access path) with no definition behind it. This is a usable identity for a `mentalc` "external, not extracted" target.
- CodeQL treats "library not extracted" as the normal state for Python, not as an error; quality relies on models.

### Gaps

- The type-tracking documentation page was not read; no quote recorded for type tracking.
- The exact CLI version that removed the 2.17.0 escape hatch was not confirmed beyond the 2.16.0 note.
- CodeQL Python has no database-quality diagnostic in the files searched (`python/ql/src/CHANGELOG.md` has no match for "database quality"); the Python telemetry directory itself was not listed.

## Joern: placeholder strings, javasrc2cpg options, pysrc2cpg type recovery

### Takeaway

Joern encodes "unresolved" inside the full name of the call or type using fixed placeholder strings, and a later type-recovery pass may fill in "dummy" types such as `<returnValue>` and `<member>`. For Python, an unresolvable import is turned into a guessed pseudo-path tagged `UnknownMethod` / `UnknownTypeDecl` / `UnknownImport`.

### Cited Findings

- CONFIRMED `ANY`: "Represents an unresolved type, or indicates that one cannot guarantee a single type for the given node." `val Any = "ANY"` — [Defines.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/Defines.scala)
- CONFIRMED Placeholders: `val UnresolvedNamespace = "<unresolvedNamespace>"`, `val UnresolvedSignature = "<unresolvedSignature>"`. "The METHOD_FULL_NAME should be given as \"<unresolvedNamespace>.foo:<unresolvedSignature>(1)\"." — [Defines.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/Defines.scala)
- CONFIRMED Partial resolution keeps what is known and records arity: "If the namespace is known the METHOD_FULL_NAME should be given as \"some.namespace.foo:<unresolvedSignature>(1)\". Thereby the number in parentheses is the number of call arguments." — [Defines.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/Defines.scala)
- CONFIRMED Purpose: "to indicate unresolved static type information. Using them enables the closed source backend to apply policies in a less strict fashion." Scope: "only makes sense for statically typed languages with a package/namespace structure like Java, CSharp". — [Defines.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/Defines.scala)
- CONFIRMED Dynamic languages: "In some languages like Javascript dynamic calls do not provide any statically known method/function interface information. In those cases please use this value." `val DynamicCallUnknownFullName = "<unknownFullName>"`. Also `Unknown = "<unknown>"`, `UnknownField = "<unknownField>"`. — [Defines.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/Defines.scala)
- CONFIRMED `--inference-jar-paths`: "extra jars used only for type information (comma-separated list of paths)" — [javasrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/javasrc2cpg/src/main/scala/io/joern/javasrc2cpg/Main.scala)
- CONFIRMED `--fetch-dependencies`: "attempt to fetch dependencies jars for extra type information. Recommended when scanning Lombok projects" — [javasrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/javasrc2cpg/src/main/scala/io/joern/javasrc2cpg/Main.scala)
- CONFIRMED `--enable-type-recovery` (hidden option): "enable generic type recovery" — [javasrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/javasrc2cpg/src/main/scala/io/joern/javasrc2cpg/Main.scala)
- CONFIRMED `--skip-type-inf-pass` (hidden): "Skip the type inference pass. Results will be much worse, so should only be used for development purposes" — [javasrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/javasrc2cpg/src/main/scala/io/joern/javasrc2cpg/Main.scala)
- CONFIRMED `--no-dummyTypes`: "disable generation of dummy types during type propagation"; `--type-prop-iterations`: "maximum iterations of type propagation"; default `XTypeRecoveryConfig(iterations: Int = 2, enabledDummyTypes: Boolean = true)`. — [XTypeRecovery.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/passes/frontend/XTypeRecovery.scala)
- CONFIRMED A strict mode exists: `--disable-type-fallback` "Disables fallback to wildcard imports, unsound type inferences and the Any type (except where no better information is available)." — [javasrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/javasrc2cpg/src/main/scala/io/joern/javasrc2cpg/Main.scala)
- CONFIRMED Dummy type tokens: `DummyReturnType = "<returnValue>"`, `DummyMemberLoad = "<member>"`, `DummyIndexAccess = "<indexAccess>"`; described as "placeholder dummy values for partially resolved types during the final iteration"; `isDummyType` "Scans the type for placeholder/dummy types." — [XTypeRecovery.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/passes/frontend/XTypeRecovery.scala)
- CONFIRMED Unknown is detected by pattern on the name: `val unknownTypePattern: Regex = s"(i?)(UNKNOWN|ANY|${Defines.UnresolvedNamespace}).*".r`; recovered candidates are written to the `DynamicTypeHintFullName` property (alongside `PossibleTypes`). — [XTypeRecovery.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/passes/frontend/XTypeRecovery.scala)
- CONFIRMED Python imports that do not resolve to code in the CPG: "Here we use heuristics to guess the correct paths, and make the types look friendly for querying" and "For an unresolveable import, create a best-effort path of what could be imported, as well as what kind of entity may be imported." — [PythonImportResolverPass.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/frontendspecific/pysrc2cpg/PythonImportResolverPass.scala)
- CONFIRMED The guesses are typed as `UnknownMethod`, `UnknownTypeDecl` or `UnknownImport` (distinct from `ResolvedMethod`, `ResolvedTypeDecl`, `ResolvedMember`) and seed the symbol table of the Python type recovery. Only internal modules are indexed for resolution (`cpg.typeDecl.isExternal(false)`); "external packages are always fully qualified." — [PythonImportResolverPass.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/frontendspecific/pysrc2cpg/PythonImportResolverPass.scala), [PythonTypeRecovery.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/x2cpg/src/main/scala/io/joern/x2cpg/frontendspecific/pysrc2cpg/PythonTypeRecovery.scala)
- CONFIRMED pysrc2cpg options: `--venvDirs` "Virtual environment directories. If not absolute they are interpreted relative to input-dir."; `--ignoreVenvDir` "Specifies whether venv-dir is ignored. Default to true." — [pysrc2cpg Main.scala](https://github.com/joernio/joern/blob/master/joern-cli/frontends/pysrc2cpg/src/main/scala/io/joern/pysrc2cpg/Main.scala)
- The docs.joern.io Java and Python frontend pages list the same options in a table and contain no text on unresolved calls or placeholders (read through a page-summarising fetch, not raw). — [docs.joern.io/frontends/java](https://docs.joern.io/frontends/java/), [docs.joern.io/frontends/python](https://docs.joern.io/frontends/python/)

### Inferences

- Joern's precedent is "in-band" marking: the hole is a magic string inside the identifier, found again by regex. It keeps partial knowledge (namespace known, signature unknown, argument count) but has no separate fact or reason code.
- Joern distinguishes resolved from guessed imports with separate case classes (`Resolved*` vs `Unknown*`), which is closer to a reason-carrying fact than the string placeholders.
- By default the Python frontend ignores the virtual environment, so calls into external packages are normally guessed, not resolved.

### Gaps

- No Joern counter or report of unresolved calls was found; the repository was not searched exhaustively for one.
- The default value of `--fetch-dependencies` and what javasrc2cpg logs when a type cannot be solved were not read.

## SCIP and scip-python

### Takeaway

SCIP has no schema element for "unresolved". A symbol is either global (scheme, package manager, package name, version, descriptors) or `local`; missing package fields use the placeholder `.`. scip-python falls back to a document-local symbol whose documentation says the module could not be resolved.

### Cited Findings

- CONFIRMED Symbol grammar: "`<symbol> ::= <scheme> ' ' <package> ' ' (<descriptor>)+ | 'local ' <local-id>`" and "`<package> ::= <manager> ' ' <package-name> ' ' <version>`" — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED Empty-field placeholder: "`<manager> ::= any UTF-8, escape spaces with double space. Use the placeholder '.' to indicate an empty value`" (package-name and version: "same as above"). — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED Descriptor suffixes: namespace `/`, type `#`, term `.`, meta `:`, macro `!`, method `(disambiguator).`, type-parameter `[name]`, parameter `(name)`. — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED Local symbols: "Local symbols MUST only be used for entities which are local to a Document, and cannot be accessed from outside the Document." — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED `external_symbols`: "(optional) Symbols that are referenced from this index but are defined in an external package (a separate `Index` message). Leave this field empty if you assume the external package will get indexed separately." — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED `SymbolRole` bitset values: `Definition = 0x1` ("Is the symbol defined here? If not, then this is a symbol reference."), `Import = 0x2`, `WriteAccess = 0x4`, `ReadAccess = 0x8`, `Generated = 0x10`, `Test = 0x20`, `ForwardDefinition = 0x40`. — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED `Occurrence.symbol` is optional ("(optional) The symbol that appears at this position."), and an occurrence can carry diagnostics: "(optional) Diagnostics that have been reported for this specific range." — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED `Diagnostic`: "Represents a diagnostic, such as a compiler error or warning, which should be reported for a document." Fields `severity`, `code`, `message`, `source`, `tags`; `Severity` is `Error = 1`, `Warning = 2`, `Information = 3`, `Hint = 4`. — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED `SyntaxKind` exists as an enum for highlighting ("What syntax highlighting class should be used for this range?"), with members such as `IdentifierLocal = 12`, `IdentifierFunctionDefinition = 16`. — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- CONFIRMED scip-python symbol prefix: `` `scip-python python ${name} ${version} ` `` with empty name or version normalised to `.`. — [ScipSymbol.ts](https://github.com/sourcegraph/scip-python/blob/scip/packages/pyright-scip/src/ScipSymbol.ts)
- CONFIRMED Third-party package name and version come from the environment: "scip-python uses `pip` to attempt to determine the versions and names of the packages available in your environment." Alternative: the `--environment` JSON list of `{name, version, files}`. — [scip-python README](https://github.com/sourcegraph/scip-python/blob/scip/README.md)
- CONFIRMED Standard library is a synthetic package: `new PythonPackage('python-stdlib', versionToString(this.execEnv.pythonVersion), [])`; a source comment shows the result "reference python-stdlib 3.10 `importlib.resources`/__init__:". Membership is decided by a hard-coded list (`Hardcoded.stdlib_module_names.has(firstPart)`). — [treeVisitor.ts](https://github.com/sourcegraph/scip-python/blob/scip/packages/pyright-scip/src/treeVisitor.ts)
- CONFIRMED Unresolved import: "For python packages & modules that we cannot resolve, we'll just make a local for the file and note that we could not resolve this module." The local symbol gets the documentation string `` `(module): ${moduleName} [unable to resolve module]` ``. — [treeVisitor.ts](https://github.com/sourcegraph/scip-python/blob/scip/packages/pyright-scip/src/treeVisitor.ts)
- CONFIRMED The follow-on problem is acknowledged as open: "TODO: The only other question would be what we should do about references to items from this module"; and for names without a declaration: "Without a declaration, it doesn't seem useful to try and add member accesses with locals. You'll just get a new local for every reference because we can't construct what these are." — [treeVisitor.ts](https://github.com/sourcegraph/scip-python/blob/scip/packages/pyright-scip/src/treeVisitor.ts)

### Inferences

- In scip-python the reason for a hole lives in free-text documentation on a local symbol; it is not machine-readable and not counted.
- SCIP's `.` placeholder for an unknown manager, name or version is a precedent for a partially known external identity (package known, version unknown).
- The SCIP `Diagnostic` message could carry an unresolved-reference fact, but the scip-python code path read does not use it for unresolved imports.

### Gaps

- Whether scip-python attaches a SCIP `Diagnostic` anywhere was not checked beyond the unresolved-import path.
- The scip-python README does not describe unresolved-import behaviour; it is documented only in source comments.

## Kythe

### Takeaway

The earlier reading is partly refuted. The `complete` fact (`incomplete`, `complete`, `definition`) describes declarations versus definitions, and `name` nodes are linker identifiers; neither marks an unresolved reference. The schema's real precedents are the `lookup` node (a name whose resolution cannot be completed) and `tnominal` (a type known only by name).

### Cited Findings

- REFUTED `complete` as an "unresolved" marker. The schema says: "incomplete if this is only a declaration; definition if it is a definition." (function, record); for sum: "complete if this is a declaration that is considered usable by value." — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED The three values exist (`incomplete`, `complete`, `definition`) and feed `completedby`: "Declaration A completedby definition B if B fully specifies A." — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- REFUTED `name` nodes as targets for unavailable references. The schema says: "A name specifies an external identifier for a node, typically used for linking." and "A named B if B is an external identifier for A." — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED `lookup` node: "A lookup is a structured name whose resolution cannot be completed without additional context." Fact `text`: "The deferred name to be resolved." — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED `lookup` rationale: "when we are unable to come up with a semantic representation of one or more nodes in a path-structured name, we record this name as a collection of lookup nodes." It names C++ dependent names and Python runtime-dependent resolution. — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED `tnominal`: "A tnominal is a type that may be purely identified by its name." Rule: "If the indexer only has a complete C++ node, or if the node is incomplete, edges should be drawn to a [tnominal] node". — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED `ref/imports`: "A ref/imports B if B is imported at the site of A." Points from anchors toward semantic nodes. — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED "implicit" in Kythe means compiler-generated, not unresolved: "A ref/implicit B if A refers to some previously-defined B, and the expression spanned by A is implicit (e.g., the result of a template instantiation)". — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- CONFIRMED `diagnostic` node and `tagged` edge exist: "A diagnostic is a node with a message concerning some aspect of the related [file] or [anchor]." "A tagged B if B labels A with a [diagnostic] message." (quoted through a page-summarising fetch; the other Kythe quotes were read from the raw page text). — [kythe.io/docs/schema](https://kythe.io/docs/schema/)
- NOT FOUND The words "unresolved", "undefined" (as a reference state) and "unavailable" do not appear in the schema page.

### Inferences

- Kythe's `lookup` node is the closest schema-level precedent for a first-class "deferred name" target: the reference still has a node to point at, carrying the unresolved text.
- Kythe models "known by name only" (`tnominal`) separately from "cannot be resolved" (`lookup`); two reasons, two node kinds.

### Gaps

- How Kythe indexers treat a reference to a target in a compilation unit that was never indexed (a dangling VName with no facts) is not stated on the schema page.

## Language servers and type checkers ("mark the hole")

### Takeaway

Every tool read keeps going after a failed resolution and marks the hole: go/packages with per-package error lists and an `IllTyped` flag, TypeScript with an error plus `any`, rust-analyzer with `{unknown}` and a named diagnostic per kind, clangd with a fallback command, JDT with bindings flagged as recovered.

### Cited Findings

- CONFIRMED go/packages `Errors`: "Errors contains any errors encountered querying the metadata of the package, or while parsing or type-checking its files." — [packages.go](https://github.com/golang/tools/blob/master/go/packages/packages.go)
- CONFIRMED go/packages `TypeErrors`: "TypeErrors contains the subset of errors produced during type checking." `IllTyped`: "IllTyped indicates whether the package or any dependency contains errors. It is set only when Types is set." — [packages.go](https://github.com/golang/tools/blob/master/go/packages/packages.go)
- CONFIRMED go/packages does not fail on package errors: "Errors associated with a particular package are recorded in the corresponding Package's Errors list, and do not cause Load to return an error. Clients may need to handle such errors before proceeding with further analysis." — [packages.go](https://github.com/golang/tools/blob/master/go/packages/packages.go)
- CONFIRMED go/packages error reasons: `ErrorKind` "describes the source of the error, allowing the user to differentiate between errors generated by the driver, the parser, or the type-checker." Values `UnknownError`, `ListError`, `ParseError`, `TypeError`. — [packages.go](https://github.com/golang/tools/blob/master/go/packages/packages.go)
- CONFIRMED TypeScript TS2307 (category Error): "Cannot find module '{0}' or its corresponding type declarations." — [diagnosticMessages.json v5.9.2](https://github.com/microsoft/TypeScript/blob/v5.9.2/src/compiler/diagnosticMessages.json)
- CONFIRMED TypeScript TS7016 (category Error): "Could not find a declaration file for module '{0}'. '{1}' implicitly has an 'any' type." TS7006: "Parameter '{0}' implicitly has an '{1}' type." — [diagnosticMessages.json v5.9.2](https://github.com/microsoft/TypeScript/blob/v5.9.2/src/compiler/diagnosticMessages.json)
- CONFIRMED TypeScript `noImplicitAny` (quoted through a page-summarising fetch): "In some cases where no type annotations are present, TypeScript will fall back to a type of `any` for a variable when it cannot infer the type." and "Turning on `noImplicitAny` however TypeScript will issue an error whenever it would have inferred `any`". — [tsconfig noImplicitAny](https://www.typescriptlang.org/tsconfig/noImplicitAny.html)
- CONFIRMED rust-analyzer `unresolved-import`: "This diagnostic is triggered if rust-analyzer is unable to resolve a path in a use declaration." — [rust-analyzer book, diagnostics](https://rust-analyzer.github.io/book/diagnostics.html)
- CONFIRMED rust-analyzer `unresolved-macro-call`: "This diagnostic is triggered if rust-analyzer is unable to resolve the path to a macro in a macro invocation." `unresolved-extern-crate`: "triggered if rust-analyzer is unable to discover referred extern crate." `unresolved-module`: "unable to discover referred module." Also listed: `unresolved-assoc-item`, `unresolved-associated-type`, `unresolved-field`, `unresolved-ident`, `unresolved-method`. — [rust-analyzer book, diagnostics](https://rust-analyzer.github.io/book/diagnostics.html)
- CONFIRMED rust-analyzer `unlinked-file`: "This diagnostic is shown for files that are not included in any crate, or files that are part of crates rust-analyzer failed to discover. The file will not have IDE features available." — [rust-analyzer book, diagnostics](https://rust-analyzer.github.io/book/diagnostics.html)
- CONFIRMED rust-analyzer admits false positives in the diagnostic source: "This currently results in false positives in the following cases: - `cfg_if!`-generated code in libstd … - proc macros and/or proc macro generated code" — [unresolved_import.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/ide-diagnostics/src/handlers/unresolved_import.rs)
- CONFIRMED rust-analyzer `{unknown}`: the type formatter writes `write!(f, "{{unknown}}")` for display and `_` for source code; the display-kind doc says "Showing `{unknown}` or not qualifying paths is fine here." — [hir-ide display.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/hir-ide/src/display.rs)
- CONFIRMED rust-analyzer `type-must-be-known`: "This diagnostic is triggered when rust-analyzer cannot infer some type." — [type_must_be_known.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/ide-diagnostics/src/handlers/type_must_be_known.rs)
- CONFIRMED rust-analyzer invariant: "parsing never fails, the parser produces (T, Vec<Error>) rather than Result<T, Error>." — [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
- CONFIRMED clangd fallback: "If no compilation database is found, a very simple command like clang foo.cc is used. For a real project this will often fail to find #includes, but it allows clangd to work on toy examples without configuration." — [clangd compile commands](https://clangd.llvm.org/design/compile-commands)
- CONFIRMED clangd marks a borrowed command: "If a command has been “borrowed” from another file, this is noted when the compile command is logged. (... with command inferred from foo/bar.cc)." and "As it’s purely a filename-based heuristic it occasionally goes wrong." — [clangd compile commands](https://clangd.llvm.org/design/compile-commands)
- CONFIRMED JDT `ASTParser.setBindingsRecovery`: "Requests that the compiler should perform bindings recovery. When bindings recovery is enabled the compiler returns incomplete bindings. Default to false." — [ASTParser javadoc](https://help.eclipse.org/latest/topic/org.eclipse.jdt.doc.isv/reference/api/org/eclipse/jdt/core/dom/ASTParser.html)
- CONFIRMED (with correction) `isRecovered()` is declared on `IBinding`, so `ITypeBinding` inherits it: "Return whether this binding is created because the bindings recovery is enabled. This binding is considered to be incomplete. Its internal state might be incomplete." — [IBinding javadoc](https://help.eclipse.org/latest/topic/org.eclipse.jdt.doc.isv/reference/api/org/eclipse/jdt/core/dom/IBinding.html)

### Inferences

- The common pattern has three parts: continue, mark the node (flag or sentinel type), and attach a reason from a closed list (`ErrorKind`, a diagnostic code).
- go/packages is the clearest "per unit" precedent: errors and an ill-typed flag live on each package, the flag is transitive over dependencies, and the caller decides whether to stop.
- rust-analyzer's own notes show that unresolved diagnostics from a tool that does not see the full build are often false positives; this argues for distinguishing "extractor could not see it" from "does not exist".

### Gaps

- TypeScript's handbook text on the type given to an import from an unresolved module was not found on the pages read; the behaviour is evidenced by the TS7016 message only.
- The rust-analyzer `{unknown}` line was matched by search and a line-level read, not a full read of the function that owns it.

## Sourcegraph and GitHub code navigation labels

### Takeaway

Sourcegraph documents two tiers, precise and search-based, with search-based as the automatic fallback, and says reference lists contain both kinds. The "syntactic" tier and a UI badge were not found in the docs pages read. GitHub's current docs describe a single tree-sitter, name-search approach and do not mention a "precise" label.

### Cited Findings

- CONFIRMED Sourcegraph fallback: "Sourcegraph automatically uses Precise Code Navigation whenever available, and Search-based Code Navigation is used as a fallback when precise navigation is not available." — [Precise Code Navigation](https://sourcegraph.com/docs/code-navigation/precise-code-navigation.md)
- CONFIRMED Sourcegraph search-based definition: "It uses a mix of text search and syntax-level heuristics (no language-level semantic information) for fast, performant searches across large code bases." — [Code Navigation](https://sourcegraph.com/docs/code-navigation.md)
- CONFIRMED Sourcegraph precise definition: "Uses compile-time information to provide users with accurate cross-repository navigation experience across the entire code base." — [Code Navigation](https://sourcegraph.com/docs/code-navigation.md)
- CONFIRMED Mixed result lists: Find references "lists all references, definitions, and implementations at the bottom, including precise and search-based results". — [Code Navigation](https://sourcegraph.com/docs/code-navigation.md)
- CONFIRMED Search-based mechanics: "Find references: Performs a case-sensitive word-boundary cross-repository plain text search for the given symbol". — [Search-based Code Navigation](https://sourcegraph.com/docs/code-navigation/search-based-code-navigation.md)
- NOT FOUND A third "syntactic" tier, and a hover badge or tooltip labelling a result as precise or search-based, are not in the three Sourcegraph pages read (overview, precise, search-based). A search-engine summary asserted a hover badge; it was not recorded because no primary page confirmed it.
- CONFIRMED GitHub: "Code navigation uses the open source tree-sitter library." and "GitHub has developed a code navigation approach based on the open source tree-sitter library that searches all definitions and references across a repository to find entities with a given name." — [Navigating code on GitHub](https://docs.github.com/en/repositories/working-with-files/using-files/navigating-code-on-github)
- NOT FOUND On the GitHub page as fetched on 2026-11-14 the words "precise", "search-based" and "stack graphs" do not occur; no result label is documented there.

### Inferences

- Sourcegraph's model is a provenance tier per result (compiler-accurate versus heuristic), mixed in one list. For `mentalc` this maps to a confidence or provenance attribute on a resolved edge, separate from the unresolved fact.

### Gaps

- Sourcegraph API field names for result provenance were not looked up.
- GitHub's earlier "precise" versus "search-based" labelling (stack graphs) could not be confirmed from the current docs page; an archived version was not consulted.

## Call-graph research on unanalysed libraries

### Takeaway

Ali and Lhoták show that one summary node (Cgc) or a generated placeholder library (Averroes) for all unanalysed library code gives application call graphs that are sound against dynamic graphs and close to whole-program precision, under the separate compilation assumption. Measured recall of practical analyses is well under 100 %: 0.884 median (Java, Sui et al.), 69.9 % (PyCG), 75.9 % rising to 88.1 % (Jelly).

### Cited Findings

- CONFIRMED Cgc, separate compilation assumption: "all of the library classes are developed separately from the application classes. In particular, all of the library classes can be compiled in the absence of the application classes." — [Ali & Lhoták, ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Cgc, single summary node: "It assumes that all library methods are reachable, and uses a single summary “method” to represent them." and "no edges are created to represent calls within the library." — [ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Cgc, what is still needed from the library: "the algorithm uses structural information (i.e., method signatures and field names) of each library class that is referenced in an application class, as well as its superclasses and superinterfaces." — [ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Cgc soundness: "the call graphs generated by Cgc soundly include all of the call edges that were dynamically observed at run time by *J, except for one single call edge from the application to the library in the lusearch benchmark". — [ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Cgc precision, application edges: "Cgc generates a median of 41 extra ApplicationCallGraphEdges (min: 0, max: 2656, median: 40.5) when compared to Doop and a median of 5 extra … when compared to Spark. Both medians are negligible as they represent 2.42% and 0.13%". — [ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Cgc imprecision is concentrated in callbacks: extra LibraryCallBackEdges are "72.9% as opposed to only 2.53%" of the median for Doop and Spark; "the majority of LibraryCallBackEdges generated by Cgc are spurious compared to Doop." Most frequent spurious targets: `clone`, `<init>`, `toString`, `equals`, `remove`, `hashCode`. — [ECOOP 2012](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf)
- CONFIRMED Averroes: "a tool that generates a placeholder library that overapproximates the possible behaviour of an original library." It is "typically in the order of 80 kB of class files (comparatively, the Java standard library is 25 MB)." — [Ali & Lhoták, ECOOP 2013](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop13.pdf)
- CONFIRMED Averroes cost: "improves the analysis time of whole-program call graph construction by a factor of 4.3x to 12x, and reduces memory requirements by a factor of 8.4x to 13x." — [ECOOP 2013](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop13.pdf)
- CONFIRMED Averroes quality: "The call graphs built with Averroes and existing whole-program frameworks are as precise and sound as those built with Cgc." Finding 5: "Averroes matches and slightly exceeds Cgc in both precision and soundness." Largest difference in library callback edges versus Cgc: "The biggest difference is 2% of the edges, in the chart benchmark." — [ECOOP 2013](https://plg.uwaterloo.ca/~olhotak/pubs/ecoop13.pdf)
- CONFIRMED Reif et al.: "the current practice of using call graph algorithms designed for applications to analyze libraries leads to call graphs that, at the same time, lack relevant call edges and contain unnecessary edges." and "the proposed call graph algorithms reduce the number of call edges up to 30% when compared to existing approaches." Two assumptions are named, open-package (OPA) and closed-package (CPA). — [Reif et al., FSE 2016 preprint](https://sse.cs.tu-dortmund.de/storages/sse-cs/r/Publications/Preprints/reh_16-callgraphs.pdf)
- CONFIRMED Sui et al.: "the median recall is 0.884 suggesting that standard static analyses have significant gaps" on "a set of 31 real-world Java programs"; with reflection support "the median observed is 0.935", "but it comes with a hefty performance penalty". — [Sui et al., ICSE 2020 preprint](https://ecs.wgtn.ac.nz/foswiki/pub/Main/JensDietrich/recall.pdf)
- CONFIRMED Sui et al., unit of recall is reachable methods, not edges: "the static analysis typically misses around 11% of the known reachable methods." Recall against built-in tests was lower "(median 0.859)" than against generated tests "(median 0.904)". — [ICSE 2020 preprint](https://ecs.wgtn.ac.nz/foswiki/pub/Main/JensDietrich/recall.pdf)
- CONFIRMED Sui et al., causes: "the main sources of unsoundness are not reflective method invocations, but objects allocated or accessed via native methods, and invocations initiated by the JVM, without matching call sites in the program under analysis." — [ICSE 2020 preprint](https://ecs.wgtn.ac.nz/foswiki/pub/Main/JensDietrich/recall.pdf)
- CONFIRMED PyCG macro benchmark (Table IV, five projects): average precision 99.2 %, average recall 69.9 %; Pyan 74.4 % / 20.6 %, Depends 98.7 % / 14.4 %. "Recall results show that on average, 69.9% of all call edges are successfully retrieved." — [Salis et al., ICSE 2021, arXiv 2103.00587](https://arxiv.org/pdf/2103.00587)
- CONFIRMED PyCG on code it cannot see: "we can only analyze modules for which their source code has been provided. When a function—for which its code definition is not available—is called, our method will add an edge to the function, but no edges stemming from that function will ever be added, and its return value will be ignored." — [arXiv 2103.00587](https://arxiv.org/pdf/2103.00587)
- CONFIRMED PyCG on built-ins: "attribute calls that depend on a specific built-in type (e.g., list.append()) are not resolved". — [arXiv 2103.00587](https://arxiv.org/pdf/2103.00587)
- CONFIRMED Jelly approximate interpretation: "On average the use of approximate interpretation leads to 55.1% more call edges, 21.8% more reachable functions, 17.7% more resolved call sites, and only 1.5% fewer monomorphic call sites." on "141 real-world Node.js-based JavaScript applications and libraries". — [Laursen, Xu, Møller, PLDI 2024](https://cs.au.dk/~amoeller/papers/approx/paper.pdf)
- CONFIRMED Jelly recall: "For 36 JavaScript projects where dynamic call graphs are available, average analysis recall is improved from 75.9% to 88.1% with a negligible reduction in precision." and "In one case, recall increases from 40.1% to 98.0%". Precision "is reduced by only 1.5%." — [PLDI 2024](https://cs.au.dk/~amoeller/papers/approx/paper.pdf)

### Inferences

- A single "library" summary target per unanalysed dependency is a published, measured design: sound for application edges, with imprecision concentrated in callbacks from the library into commonly overridden methods.
- The Cgc approach still needs signatures of referenced library classes. A pure source-only extractor without dependency metadata has less than Cgc assumed.
- Recall figures are not comparable across papers: Sui measures reachable methods, PyCG measures edges against a hand-made graph, Jelly measures edges against a dynamic graph.

### Gaps

- Reif et al.: only the abstract and introduction were read closely; per-library numbers were not extracted.
- The PyCG paper was read from arXiv, the Sui and Reif papers from author preprints; page numbers in the published versions may differ.

## Tools or papers that publish a "resolution rate" and whether any gates on it

### Takeaway

CodeQL is the only tool found that computes a per-database resolution rate, publishes it, and applies a threshold; the threshold produces a warning, not a failure. Jelly defines "percentage of resolved call sites" as a research metric. No tool was found that fails a build on a resolution rate.

### Cited Findings

- CONFIRMED CodeQL publishes "Percentage of calls with call target" and "Percentage of expressions with known type" per database (key built as "Percentage of " + ok text) and compares them with thresholds 85/85 (Java, C#) and 50/20 (Rust). — [ReportStats.qll](https://github.com/github/codeql/blob/main/shared/util/codeql/util/ReportStats.qll), [java DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQualityDiagnostics.ql), [rust DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/rust/ql/src/queries/telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED CodeQL does not gate: the message opens "Scanning Java code completed successfully, but the scan encountered issues." and the result row is selected with severity 1 (warning). — [java DatabaseQualityDiagnostics.ql](https://github.com/github/codeql/blob/main/java/ql/src/Telemetry/DatabaseQualityDiagnostics.ql)
- CONFIRMED CodeQL C/C++ publishes an include resolution rate: baseline "#include directives" is `count(Include i) + count(CannotOpenFileError e)`, success metric "successfully resolved #include directives", and a per-header count `"Failed to include '" + includeText + "'"`. — [cpp Metrics.qll](https://github.com/github/codeql/blob/main/cpp/ql/src/Telemetry/Metrics.qll)
- CONFIRMED Jelly's metric definition: "Percentage of resolved call sites. A call site is considered resolved if the analysis found at least one call edge for it. Some call sites are unresolved because they involve callbacks in unused library code, but for most call sites a good call graph analysis should be able to find one or more call edges, so this metric gives an indication of the analysis recall". — [PLDI 2024](https://cs.au.dk/~amoeller/papers/approx/paper.pdf)
- CONFIRMED Jelly's paired precision proxy: "Percentage of monomorphic call sites. A call site is considered monomorphic if the analysis has found at most one call edge for it". — [PLDI 2024](https://cs.au.dk/~amoeller/papers/approx/paper.pdf)
- CONFIRMED GitHub's per-file coverage signal: the tool status page shows "a percentage total of all the files most recently scanned in your repository, organized by programming language." — [docs.github.com](https://docs.github.com/en/code-security/code-scanning/managing-your-code-scanning-configuration/about-the-tool-status-page)
- NOT FOUND A tool or paper that fails the build or exits non-zero when a resolution rate is below a threshold. go/packages leaves the decision to the client ("Clients may need to handle such errors before proceeding with further analysis."). — [packages.go](https://github.com/golang/tools/blob/master/go/packages/packages.go)

### Inferences

- If `mentalc` fails the build on a resolution rate it goes further than every precedent read; the precedents warn and continue. CodeQL's history (threshold relaxed, metric removed, exclusions added) shows the false-alarm risk of a hard gate.
- Jelly's observation that some call sites are legitimately unresolved (callbacks in unused library code) and CodeQL's exclusion lists both support counting by reason and gating only on reasons that indicate an extractor defect.
- CodeQL counts per database (and per language), not per file or module; a per-unit count in `mentalc` would be finer-grained than the precedent.

### Gaps

- Joern, SCIP indexers and Kythe were not found to publish any resolution rate; none of the three was searched exhaustively for one.
- Other candidates (Semgrep, Infer, Soot/OPAL "unresolved" statistics, WALA) were out of the scope searched.
