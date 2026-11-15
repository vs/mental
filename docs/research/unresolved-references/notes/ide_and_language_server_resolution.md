# How IDEs and language servers obtain a resolved project model (state as of November 2026)

Reading guide. "Fetched" means the page was read in this session. "Search snippet" means the claim comes from a search-result summary of the linked page and was not read in full. Claims I know from prior knowledge but did not re-verify are listed only under Gaps, marked "unverified this session".

Per-tool summary, one line each:

- IntelliJ IDEA + Gradle: runs the real Gradle build's configuration phase through the Tooling API (daemon, build scripts, plugins execute); all-or-nothing until Gradle 9.7 + IDEA 2026.2 "resilient sync".
- IntelliJ IDEA + Maven: its own importer process reads the POMs with a chosen JDK; generated sources are found by folder convention or by running Maven up to a chosen phase.
- JetBrains Bazel plugin: runs Bazel aspects (a real `bazel build` of metadata) plus `bazel query`; GA since July 2025.
- CLion / clangd: need a per-file compile command (CMake, `compile_commands.json`); with none, clangd falls back to `clang foo.cc` and includes mostly do not resolve.
- PyCharm / pyright: need a Python environment with dependencies installed; pyright can run without one, unresolved things become `Unknown`.
- gopls: `go list` via `go/packages`; near zero-config since v0.15; other build systems plug in through `GOPACKAGESDRIVER`.
- rust-analyzer: `cargo metadata` plus running `build.rs` and proc macros by default; non-Cargo builds supply `rust-project.json`.
- tsserver: `tsconfig.json`/`jsconfig.json` or an "inferred project" for loose files; needs `node_modules` installed, falls back to automatic type acquisition for JS.
- Eclipse JDT LS: embeds m2e (Maven) and Buildship (Gradle Tooling API); needs Java 25 to run.
- Roslyn / Visual Studio: MSBuild design-time build, which evaluates the project and runs resolve targets but does not compile.
- Metals: exports the build to Bloop or talks BSP to sbt/Mill/Bazel.
- Kotlin LSP (official): alpha; Gradle and Maven JVM import, no KMP yet.
- CodeQL (not an IDE, the closest analogue to a static extractor): `build-mode: none` for Java, C#, C/C++, Rust, with a documented accuracy cost.

## 1. How do JetBrains IDEs import projects, and what does each import execute?

### Takeaway
JetBrains IDEs do not parse build files themselves for Gradle and Bazel: they run the build tool and ask it for a model, which executes arbitrary build logic and downloads dependencies. That is why opening a project is gated by a trust prompt, and why a broken build historically meant no model at all.

### Cited Findings
- Gradle sync is done through the Gradle Tooling API: "IntelliJ IDEA ... uses the Tooling API for importing your Gradle project and running tasks"; so does Eclipse Buildship. (fetched) — [Gradle Tooling API docs](https://docs.gradle.org/current/userguide/tooling_api.html)
- The Tooling API "always uses the Gradle daemon", is "wrapper aware", and "can download and install the appropriate Gradle version, similar to the wrapper"; it can query "the project hierarchy and the project dependencies, external dependencies", source directories and tasks. (fetched) — [Gradle Tooling API docs](https://docs.gradle.org/current/userguide/tooling_api.html)
- The IDE asks Gradle for typed models through a build action that runs inside the Gradle process; this requires configuring projects, build scripts, included builds and `buildSrc`. (fetched) — [Gradle blog: Resilient Sync](https://blog.gradle.org/resilient-sync)
- "When you open a Gradle project the synchronization is done automatically." "IntelliJ IDEA cannot reload only a part of your project, it reloads the whole project including modules and dependencies." On re-import IDEA "considers the Gradle configuration as a single source of truth" and drops dependencies added by hand in Project Structure. (fetched) — [IntelliJ IDEA help: Work with Gradle projects](https://www.jetbrains.com/help/idea/work-with-gradle-projects.html)
- Maven: the importer runs with a selectable JDK (Internal JRE by default, a JDK, or JAVA_HOME) and its own VM options (default `-Xmx512m`). A "Phase to be used for folders update" setting picks a Maven phase to run so plugins that add sources at that phase are seen; "Generated sources folders" can be auto-detected, set to `target/generated-sources`, its subdirectories, or not detected. Sources/docs download is optional. (fetched) — [IntelliJ IDEA help: Maven. Importing](https://www.jetbrains.com/help/idea/maven-importing.html)
- Maven "Enable fast import", on by default, imports through the Workspace Model API; IDEA does not reuse Maven build results and compiles from scratch; generated sources are searched only in `target/generated-sources` and its subdirectories. (search snippet, 2023.2 and 2023.3 help pages; older practice that may have shifted) — [IntelliJ IDEA 2023.2 help: Maven. Importing](https://jetbrains.com/help/idea/2023.2/maven-importing.html)
- Bazel: the new JetBrains Bazel plugin reached general availability in July 2025 for IntelliJ IDEA, PyCharm and GoLand, adding Python and Go targets; Google's original plugin is being deprecated and JetBrains maintains both. (search snippet) — [JetBrains blog: Bazel plugin GA](https://blog.jetbrains.com/idea/2025/07/bazel-ga-release/)
- Bazel "phased sync" (also "query sync") is an experimental mode that lets the developer start working sooner; in 2025.1 it shards sync to lower peak memory at the cost of total sync time. (search snippet) — [JetBrains blog: Introducing the new Bazel plugin](https://blog.jetbrains.com/idea/2024/12/introducing-the-new-bazel-plugin-eap-for-intellij-idea/); [What's new in Bazel plugin 2025.1](https://blog.jetbrains.com/idea/2025/04/what-s-new-in-bazel-plugin-2025-1/)
- Bazel sync runs aspects over the workspace to collect per-target metadata, and uses `bazel query` (plus `bazel mod graph` for Bzlmod) to load aspects only for rulesets in use. (search snippet, secondary source, VirtusLab's Bazel book) — [VirtusLab Bazel book: IDE support](https://bazel.virtuslab.com/book/3~5~1/)
- CLion, compilation database: `compile_commands.json` "lets CLion detect project files and extract all the necessary compiler information". Ways to produce it: CMake `CMAKE_EXPORT_COMPILE_COMMANDS=ON` (Makefile and Ninja generators only), clang `-MJ`, `ninja -t compdb`, compiledb for Make, Bear / intercept-build (capture compiler calls during a real build). A compilation-database project has no build/run/debug data, and changing toolchain means regenerating the file. (fetched) — [CLion help: Compilation database](https://www.jetbrains.com/help/clion/compilation-database.html)
- CLion still uses the older "Bazel for CLion" plugin; C/C++ in the new plugin is a stated goal. (search snippet) — [JetBrains blog: Bazel for CLion](https://blog.jetbrains.com/clion/2025/07/bazel-for-clion/)
- PyCharm: "To work with your Python code in PyCharm, you need to configure at least one Python interpreter." It detects conda, Poetry and uv installations (manual fallback if detection fails), finds pipenv on `PATH`, and in Workspace mode can "automatically detect and configure interpreters for each workspace member or subproject". (fetched) — [PyCharm help: Configure a Python interpreter](https://www.jetbrains.com/help/pycharm/configuring-python-interpreter.html)

### Inferences
- For Gradle and Bazel, "import" is a partial run of the build, not a read of a declarative file. Nothing short of executing the build logic yields the true classpath, because the classpath is computed by code (plugins, convention plugins, `buildSrc`).
- Maven sits in the middle: the POM is declarative enough for an importer to resolve dependencies without running the lifecycle, but anything a plugin generates (sources, extra source roots) needs either a convention guess or a real phase run. JetBrains exposes both as settings, which is evidence that neither is reliably right.
- A static extractor that wants IDE-grade resolution on the JVM has to do what the IDE does: have a working JDK, network access to the repositories, and permission to run the build tool.

### Gaps
- I did not find a JetBrains primary source in this session that spells out, step by step, what Gradle sync executes (init scripts injected, which model builders, whether tasks run). The Gradle blog describes it at a high level only.
- I did not verify the Android Studio specifics (build variants, AGP models, the variant selected at sync time deciding which source sets resolve). The Gradle blog says only that Android Studio support for resilient sync comes later "as its platform catches up" — [Gradle blog: Resilient Sync](https://blog.gradle.org/resilient-sync).
- Rider, GoLand and WebStorm import were not researched from JetBrains sources. Unverified this session: Rider loads solutions through MSBuild and NuGet restore; GoLand uses the `go` tool's module data; WebStorm indexes `node_modules` and uses the TypeScript service.
- Unverified this session: IDEA's annotation processor settings are derived from the build model for Maven/Gradle, and generated code appears only after a build has produced it.

## 2. Where do JetBrains IDEs need a human, and what happens when sync fails?

### Takeaway
A human is still needed for trust, for the toolchain (JDK, interpreter), for credentials, and for anything generated by the build. Until 2026 a single broken Gradle script left the IDE with no model at all; partial models on failure arrived only with Gradle 9.7 and IntelliJ IDEA 2026.2.

### Cited Findings
- Trust gate: in Safe Mode "for a project that contains any build tool, the importing process is disabled. So, no build scripts are executed and no dependencies are resolved." Trusting the project is what lets the IDE resolve plugins and add dependencies. (fetched) — [IntelliJ IDEA help: Project security](https://www.jetbrains.com/help/idea/project-security.html)
- Classic Gradle sync is all-or-nothing: a failure configuring any part of the build (one subproject, or a shared convention plugin in an included build) fails the whole sync, because "a throwing build action discards everything it had already collected". Without models, build scripts turn red and lose completion. (fetched) — [Gradle blog: Resilient Sync](https://blog.gradle.org/resilient-sync)
- Resilient sync: `BuildController.fetch(...)` (added in Gradle 9.3.0) returns the model plus failures instead of throwing; Gradle 9.4 completed the daemon side; Gradle 9.7 marks the sync failed while still streaming usable partial models. Requires Gradle 9.7+ and IntelliJ IDEA 2026.2+. If nothing configures, Gradle returns a base script model with the standard Gradle API. No other Tooling API IDE (Eclipse, NetBeans, VS Code Gradle) has committed to adopting it. (fetched) — [Gradle blog: Resilient Sync](https://blog.gradle.org/resilient-sync)
- Early field reports against resilient sync: slow imports with Gradle 9.7.0 (issue IDEA-392538, workaround is registry key `gradle.use.resilient.model.fetch`), and a sync failure "Class is not registered: ...GradleModelFetchFailureState" on IDEA 2026.2.1. (search snippets of user forums; not confirmed by JetBrains) — [Kotlin Slack archive](https://slack-chats.kotlinlang.org/t/34437464/anyone-else-having-issues-syncing-projects-in-intellij-2026-); [JetBrains support forum](https://intellij-support.jetbrains.com/hc/en-us/community/posts/35241899816850/comments/35294084660242)
- On failure IDEA shows "the detailed list of the occurred problems" in the Build tool window. (fetched) — [IntelliJ IDEA help: Work with Gradle projects](https://www.jetbrains.com/help/idea/work-with-gradle-projects.html)
- Human choices in Maven import: JDK for the importer, the phase used for folder updates, generated-sources folder policy. (fetched) — [IntelliJ IDEA help: Maven. Importing](https://www.jetbrains.com/help/idea/maven-importing.html)
- Human choices in PyCharm: install Python, select or create the interpreter, point to the executable if it is not listed, fix an "Invalid environment" warning. (fetched) — [PyCharm help: Configure a Python interpreter](https://www.jetbrains.com/help/pycharm/configuring-python-interpreter.html)

### Inferences
- The human steps cluster into four kinds, and each maps to something a headless extractor also lacks: (1) consent to execute build code, (2) a toolchain of the right version, (3) secrets for private repositories, (4) a first build to produce generated sources.
- The 2026 arrival of resilient sync shows that the industry's best-resourced IDE had no partial-model story for Gradle until this year. It is new and still rough, and it covers only projects on Gradle 9.7+, so most existing repositories do not benefit.

### Gaps
- No primary source found this session on private-repository credentials (settings.xml, `gradle.properties`, credential helpers) as an import failure cause, though it is commonly reported. Unverified this session.
- Unverified this session: what IDEA shows when sync fails on an older Gradle (my understanding is that it keeps the previous model if one exists, else shows files without module structure and marks external symbols unresolved).
- Maven profiles and Android build variants as human choices were not confirmed from a fetched page.

## 3. How do language servers obtain the project model? What is BSP?

### Takeaway
Every mainstream language server delegates to the ecosystem's build tool, and the ones with a single dominant, declarative-enough tool (Go, Rust with Cargo, TypeScript) get close to zero-config. The ones without (C/C++, Python, JVM) need either an artefact produced by a real build or a prepared environment.

### Cited Findings
Go (gopls):
- "gopls will typically work when you open a Go file anywhere in your workspace." Since v0.15.0 it infers builds from open files and creates a separate build scope for files constrained to another GOOS/GOARCH. Before v0.15.0 it "would often fail in mysterious ways" ("No packages found"). (fetched) — [gopls: Setting up your workspace](https://go.dev/gopls/workspace)
- Limits: gopls "is currently unable to guess build flags that include arbitrary user-defined build constraints"; these must be set through `build.buildFlags` / `build.env`. Files importing "C" fail when the target does not match the default toolchain (`CGO_ENABLED=0`). References search only the default build of the current file. (fetched) — [gopls: Setting up your workspace](https://go.dev/gopls/workspace)
- `go/packages` delegates to the go command by default ("Load passes most patterns directly to the underlying build tool"). For other build systems you install a driver and set `GOPACKAGESDRIVER`; the driver gets a JSON `DriverRequest` on stdin and returns a `DriverResponse`. The rules_go wiki documents the Bazel driver. (fetched) — [go/packages](https://pkg.go.dev/golang.org/x/tools/go/packages)

Rust (rust-analyzer):
- `cargo.buildScripts.enable` defaults to `true`: "Run build scripts (`build.rs`) for more precise code analysis." `procMacro.enable` defaults to `true` and implies build scripts. `cargo.sysroot` defaults to `"discover"` via `rustc --print sysroot`. `cargo.noDeps` (default `false`) skips fetching dependency metadata. `cargo.features` defaults to `[]`; `cfg.setTest` defaults to `true`. The default build-script command uses `cargo check`. (fetched) — [rust-analyzer book: Configuration](https://rust-analyzer.github.io/book/configuration.html)
- "rust-analyzer does not require Cargo." Other build systems describe the project in `rust-project.json` (crates, deps, cfgs, `sysroot`, `is_proc_macro`, `proc_macro_dylib_path` pointing at a compiled `.so`), supplied as a file at the root, through `linkedProjects`, or generated on the fly by a `workspace.discoverConfig` command. (fetched) — [rust-analyzer book: Non-Cargo based projects](https://rust-analyzer.github.io/book/non_cargo_based_projects.html)
- rules_rust ships a `discoverConfig` integration for Bazel that watches BUILD, BUILD.bazel and MODULE.bazel; a generated `rust-project.json` "is effectively a build artifact". (search snippet) — [rules_rust: rust-analyzer discoverConfig integration](https://pigweed.googlesource.com/third_party/github/bazelbuild/rules_rust/+/4d399f5ee60728737d8f1a60f2da198364b2c87c)

Python (pyright; Pylance and basedpyright share this resolver):
- Resolution order for absolute imports: configured `stubPath`; workspace code (execution root, extra paths, local `src`); site-packages of the environment (stub packages, inline stubs, `py.typed`, then library `.py` code if `useLibraryCodeForTypes`); stdlib typeshed; third-party typeshed bundled with pyright; last, the importing file's directory and parents. (fetched) — [pyright docs: Import resolution](https://github.com/microsoft/pyright/blob/main/docs/import-resolution.md)
- Environment discovery: `venvPath` + `venv` (discouraged), then `python.pythonPath` / the Python extension's picker, then "As a fallback, use the default Python environment" (whatever `python` is on the shell). Pyright does not require an environment if local files and stubs cover every import. (fetched) — [pyright docs: Import resolution](https://github.com/microsoft/pyright/blob/main/docs/import-resolution.md)
- Editable installs that use import hooks (the setuptools default) "are not usable by Pyright and other static analysis tools"; path-based `.pth` files are needed. Hatchling and PDM use `.pth` by default. (fetched) — [pyright docs: Import resolution](https://github.com/microsoft/pyright/blob/main/docs/import-resolution.md)

TypeScript (tsserver):
- Three project kinds: configured (a `tsconfig.json`/`jsconfig.json` defines root, files, options), external (host supplies files and options; "Currently VS is the only consumer"), inferred (for a loose file with no config "in the current directory or any parent directories"; the server pulls in files reached by imports and triple-slash references, with default options). Precedence: "configured projects > external projects > inferred projects". (fetched) — [TypeScript wiki: Standalone Server (tsserver)](https://github.com/microsoft/TypeScript/wiki/Standalone-Server-(tsserver))
- Automatic type acquisition: "TypeScript will provide types for your node_modules automatically using the DefinitelyTyped set of @types definitions"; it applies to JavaScript projects and needs npm available. (search snippet) — [TSConfig reference: typeAcquisition](https://www.typescriptlang.org/tsconfig/typeAcquisition.html); [VS Code docs: Working with JavaScript](https://code.visualstudio.com/docs/nodejs/working-with-javascript)

C/C++ (clangd):
- "Generally, clangd first obtains a basic compile command from a compilation database and then applies some tweaks to it." With `--query-driver` it can ask the real compiler for its default include paths and target; this "must be explicitly enabled as we're executing otherwise unknown binaries". clangd adds its own `-resource-dir` and on macOS sets `-isysroot`. (fetched) — [clangd design: Compile commands](https://clangd.llvm.org/design/compile-commands)

Java (Eclipse JDT LS):
- Built on M2Eclipse ("which provides Maven support") and Buildship ("which provides Gradle support"); lists "Gradle project support (with experimental Android project import support)", "Annotation processing support (automatic for Maven projects)", "Standalone Java files support". "The language server requires a runtime environment of Java 25 (at a minimum) to run"; compiles projects from Java 1.8 through 25. (fetched) — [eclipse.jdt.ls README](https://github.com/eclipse-jdtls/eclipse.jdt.ls/blob/main/README.md)

Kotlin:
- The official Kotlin LSP is "currently in the Alpha state"; "Build system support for JVM projects: Gradle, Maven, experimental Android Gradle Plugin support"; "Support for Kotlin Multiplatform (KMP) projects is coming in the future releases"; partially closed-source. (fetched) — [Kotlin/kotlin-lsp](https://github.com/Kotlin/kotlin-lsp)

C# (Roslyn in Visual Studio):
- Visual Studio gets sources, references and options from MSBuild "via a so-called design-time build (DTB)". "Design-time builds occur during project load, and in response to project changes." "No compilation occurs, and outputs aren't modified." Targets include `ResolveAssemblyReferencesDesignTime`, `ResolveProjectReferencesDesignTime2`, NuGet collection targets, and `CompileDesignTime`, which hands command-line arguments, `<Compile>` and `<Analyzer>` items to the language service. Results are cached in `.vs`. (fetched) — [dotnet/project-system: Design-time builds](https://github.com/dotnet/project-system/blob/main/docs/design-time-builds.md)

Scala (Metals) and BSP:
- Metals imports sbt, Maven, Gradle and Mill builds "directly from the language server without the need for running custom steps in the terminal" by exporting to Bloop; Bazel is also automatic; it requires sources compiled with the `semanticdb-scalac` plugin. It works with any build tool that speaks BSP. (fetched) — [Metals docs: Build tools overview](https://scalameta.org/metals/docs/build-tools/overview)
- BSP servers: bazel-bsp (JetBrains; Java, Scala, Kotlin), Bloop (sbt, Gradle, Maven, Mill), Mill (built in since 0.9.3), sbt (since 1.4.0), scala-cli, cargo-bsp (work in progress), Microsoft's build-server-for-gradle (Java). BSP clients: IntelliJ Scala plugin, IntelliJ-BSP, Metals, scala-cli, Microsoft's vscode-gradle, bsp.nvim (alpha). (fetched) — [BSP: Implementations](https://build-server-protocol.github.io/docs/overview/implementations)

### Inferences
- BSP is real but JVM/Scala-centred. Outside Scala its only notable uses are JetBrains' Bazel work and Microsoft's Gradle server for VS Code Java; there is no BSP path for Python, JS/TS, C/C++, Go or C#. It is not a universal interchange point.
- rust-analyzer's defaults show that even the "easy" ecosystems execute project code to resolve references: build scripts and proc macros are compiled and run. An extractor that refuses to execute code will miss everything proc-macro-generated.
- The JDT LS requirement of Java 25 to run, separate from the project's JDK, is a concrete example of toolchain friction for an embedded analyser.

### Gaps
- Could not fetch the vscode-java wiki page on "invisible projects" (folders without build files, where source roots and `lib/*.jar` are inferred). Unverified this session.
- Did not verify the current Roslyn LSP / C# Dev Kit behaviour outside Visual Studio (automatic `dotnet restore`, MSBuildWorkspace). Unverified this session.
- Did not verify TypeScript project references, or the state of the native TypeScript 7 (Go port) language service.
- Did not confirm from a fetched page whether rust-analyzer runs build scripts at project load; the config page documents the setting and a rebuild-on-save option but not the trigger. A search snippet of older source mentions running `cargo check` at launch to find `OUT_DIR` — [rust-analyzer cargo_workspace.rs, old commit](https://git.joshthomas.dev/language-servers/rust-analyzer/src/commit/d387bfdc4abc47ff62f1dffe7d41ce2bb11e7a00/crates/project_model/src/cargo_workspace.rs).

## 4. What interchange points hand a resolved model to an analysis tool without re-running the build?

### Takeaway
There is no cross-ecosystem standard. Each ecosystem has one machine-readable "ask the build tool" call, and all of them still invoke the build tool (and usually need dependencies downloaded); only `compile_commands.json` and `rust-project.json` are plain files that can be produced once and consumed later.

### Cited Findings
- `compile_commands.json`: produced by CMake export, `ninja -t compdb`, clang `-MJ`, compiledb, or by intercepting a real build with Bear / intercept-build. (fetched) — [CLion help: Compilation database](https://www.jetbrains.com/help/clion/compilation-database.html)
- Gradle Tooling API: any program can embed it, query `IdeaProject` / `EclipseProject` models or custom models through build actions (`getModel`/`findModel`/`fetch`); it starts a daemon and can download the wrapper's Gradle version. An IDE can still get these models when part of a composite build fails. (fetched) — [Gradle Tooling API docs](https://docs.gradle.org/current/userguide/tooling_api.html)
- Go: `go/packages` with `GOPACKAGESDRIVER` is the one API that covers both `go list` and Bazel/other build systems. (fetched) — [go/packages](https://pkg.go.dev/golang.org/x/tools/go/packages)
- Rust: `rust-project.json` is the documented hand-off format for non-Cargo builds, including the path to compiled proc-macro dylibs. (fetched) — [rust-analyzer book: Non-Cargo based projects](https://rust-analyzer.github.io/book/non_cargo_based_projects.html)
- C#: the design-time build's `CompileDesignTime` target yields exactly the compiler command line, compile items and analyzers, without compiling. (fetched) — [dotnet/project-system: Design-time builds](https://github.com/dotnet/project-system/blob/main/docs/design-time-builds.md)
- BSP: a protocol between a build server and a client; implementations listed in section 3. (fetched) — [BSP: Implementations](https://build-server-protocol.github.io/docs/overview/implementations)
- Bazel: IDE plugins get per-target metadata by running aspects, with `bazel query` for the target graph. (search snippet, secondary) — [VirtusLab Bazel book: IDE support](https://bazel.virtuslab.com/book/3~5~1/)

### Inferences
- "Without re-running the build" is mostly unattainable: the realistic goal is "without compiling". Gradle model fetch, MSBuild design-time build, `go list`, `cargo metadata` and Bazel aspects all evaluate the build definition but skip compilation.
- For an extractor that runs inside the user's coding agent on the user's machine (the "Mental" setting), the cheapest route is to reuse what the developer's environment already has: an existing `compile_commands.json`, an already-populated `~/.m2` / Gradle cache / `node_modules` / virtualenv, rather than building a fresh environment.

### Gaps
- Not verified this session, from prior knowledge: `cargo metadata --format-version 1` (resolved package graph as JSON), `go list -json -deps`, `mvn dependency:build-classpath -Dmdep.outputFile=...`, MSBuild binary logs (`-bl`, readable with the MSBuild Structured Log libraries, containing the csc command line), `bazel aquery` (action graph with full command lines), and `bazel cquery`. I believe these all exist and work as described, but no page was fetched for them.
- No source found on SCIP/LSIF indexers (scip-java, scip-typescript, etc.) as a hand-off point; this may be covered by another researcher.

## 5. What degrades gracefully when dependencies are missing?

### Takeaway
Tools split into two camps: those that keep going with holes (pyright, tsserver, go/packages, clangd, CodeQL's no-build mode) and those whose model is all-or-nothing at import time (classic Gradle sync). The graceful ones mark the hole explicitly as an unknown type rather than guessing a target.

### Cited Findings
- pyright: when a type cannot be inferred it is "Unknown", a special form of Any, which "allows Pyright to optionally warn when types are not declared and cannot be inferred". For unannotated functions it uses "call-site return type inference", limited to a small recursion depth. (fetched) — [pyright docs: Type inference](https://github.com/microsoft/pyright/blob/main/docs/type-inference.md)
- pyright has a last-resort resolution step that looks in the importing file's directory and its parents; and can analyse with no environment at all if stubs and local files suffice. (fetched) — [pyright docs: Import resolution](https://github.com/microsoft/pyright/blob/main/docs/import-resolution.md)
- go/packages returns partial results with errors attached per package: "Errors contains any errors encountered querying the metadata of the package, or while parsing or type-checking its files"; "IllTyped indicates whether the package or any dependency contains errors"; "Clients may need to handle such errors before proceeding with further analysis." (fetched) — [go/packages](https://pkg.go.dev/golang.org/x/tools/go/packages)
- clangd: "If no compilation database is found, a very simple command like `clang foo.cc` is used", which often fails to find includes. For a header with no entry it uses the command of an open file that includes it, or will "pick some entry whose filename most closely matches that of the header"; "As it's purely a filename-based heuristic it occasionally goes wrong." (fetched) — [clangd design: Compile commands](https://clangd.llvm.org/design/compile-commands)
- tsserver: a file with no config still gets an inferred project with default options and follows its imports. (fetched) — [TypeScript wiki: Standalone Server (tsserver)](https://github.com/microsoft/TypeScript/wiki/Standalone-Server-(tsserver))
- Roslyn/VS: when a design-time build fails, VS shows "An error occurred during a design-time build. Some features may not work correctly"; effects include incomplete IntelliSense and files opening in the Miscellaneous Files project. (fetched) — [dotnet/project-system: Design-time builds](https://github.com/dotnet/project-system/blob/main/docs/design-time-builds.md)
- Gradle: partial models on failure only from Gradle 9.7 + IDEA 2026.2; before that, nothing. (fetched) — [Gradle blog: Resilient Sync](https://blog.gradle.org/resilient-sync)
- CodeQL `build-mode: none` builds a database from source with no build; GitHub's docs list it for Java, C#, C/C++ and Rust, and it is the default-setup mode for Java (unless Kotlin is present, because "Kotlin analysis requires a build"), C#, C/C++ and Rust. It "may produce less accurate results" if "The build scripts cannot be queried for dependency information, and dependency guesses are inaccurate" or "The repository normally generates code during the build process". Accuracy is rated "Good" for `none` and `autobuild`, "Best" for `manual`; with `none`, generated code is not analysed. (fetched) — [GitHub docs: CodeQL code scanning for compiled languages](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/codeql-code-scanning-for-compiled-languages)

### Inferences
- CodeQL is the strongest precedent for the reader's problem: GitHub, with every incentive toward precision, made "no build, guess dependencies" the default for Java, C#, C/C++ and Rust because requiring a working build blocked too many repositories. The wording "build scripts ... queried for dependency information" implies a tiered approach: ask the build tool if possible, guess otherwise.
- The common pattern across graceful tools is a three-level result per reference: resolved, unresolved-but-known-to-be-external (the import names a package that is not present), and unknown. An extractor that records which level each edge is at loses much less than one that drops unresolved edges.
- Intra-project references resolve without any dependencies in every tool surveyed; what is lost is types flowing through third-party APIs (a method call on a value returned by a library function). So gaps concentrate on edges whose receiver type comes from a dependency, not on project-to-project edges with explicit imports.
- Kotlin is the outlier: both CodeQL and the alpha state of the official LSP indicate that Kotlin has no good no-build path today.

### Gaps
- No primary source fetched for JetBrains' own degraded behaviour (PSI is error-tolerant and project-local symbols still resolve by index when libraries are missing). I believe this is true but it is unverified this session.
- No source fetched for the JDT/ECJ "recovered bindings" mechanism, or for how tsserver types an unresolved module import (my understanding: an error plus implicit `any`). Unverified this session.
- No quantitative measure found of how many references remain unresolved in any of these degraded modes; CodeQL publishes only the qualitative Good/Best rating.

## 6. Quantitative evidence on import failure and no-manual-step success

### Takeaway
I found no published numbers on IDE sync failure rates. The nearest evidence is academic work on whether open-source Java projects build with their default command: a consistent 30 to 60% fail without manual intervention, mostly on dependency resolution. So "projects just build from sources nowadays" is not supported for the JVM.

### Cited Findings
- Sulír and Porubän tried to build "over 7,200 open source Java projects" from GitHub; "more than 38%" of builds failed; dependency-related errors were the largest category. (fetched abstract; study from 2016, older practice) — [A Quantitative Study of Java Software Buildability, arXiv 1712.01024](https://arxiv.org/abs/1712.01024)
- A 2026 Empirical Software Engineering paper that built 2,500 projects across JDK versions reports failure rates of 30 to 44% from Java 8 onward, peaking at JDK 17, with about 32% of projects failing on all JDKs. (search snippet only; the page is behind a login redirect and could not be read) — [Springer, s10664-026-10806-6](https://link.springer.com/article/10.1007/s10664-026-10806-6); preprint of the related report: [Local Software Buildability across Java Versions, arXiv 2408.11544](https://arxiv.org/abs/2408.11544v1)
- The registered report on the same topic states that prior studies found "about 40–60%" of such build attempts fail. (search snippet) — [arXiv 2408.11544](https://arxiv.org/abs/2408.11544v1)
- A 2022 study of the top 300 Java projects on GitHub found about 89% of build failures could be resolved manually; main causes were test failures (49%) and file setup (24%). (search snippet; I could not identify which listed URL it comes from, likely the CEUR paper) — [CEUR-WS Vol-3237 paper](https://ceur-ws.org/Vol-3237/paper-fon.pdf)
- gopls's own docs concede that before v0.15.0 it "would often fail in mysterious ways". (fetched) — [gopls: Setting up your workspace](https://go.dev/gopls/workspace)

### Inferences
- A full build is a stricter bar than IDE import (import needs dependency resolution and configuration, not compilation or tests). Since dependency errors are the top failure category and test failures are a large share in the top-300 study, the import failure rate should be lower than the build failure rate but well above zero. I cannot put a number on it.
- Popular, maintained projects fare much better than the long tail; the reader's target (a team's own active repository, on the developer's machine, with caches warm) is the best case. The worst case is a cold, headless checkout of someone else's old repository.
- By ecosystem, expected no-human success from cited behaviour: Go and Cargo-based Rust highest (single tool, lockfiles, near zero-config), TypeScript high once `npm install` has run, Python dependent on an environment being present, JVM dependent on JDK version and repository access, C/C++ dependent on a compilation database that usually only a real build produces.

### Gaps
- No JetBrains, Google (Android Studio) or Microsoft telemetry on sync or import failure rates was found. JetBrains and Google are known to track sync success internally, but I found nothing public.
- No study found measuring "fraction of repositories that open with fully resolved references and zero manual steps" for any IDE or language server.
- No comparable buildability studies were found in this session for Python, JS/TS, Go, Rust, C/C++ or C#. Execution-based benchmark work (for example SWE-bench-style environment setup studies) likely has numbers for Python but was outside what I could cover.
- The 2026 Springer figures need confirmation from the paper itself.
