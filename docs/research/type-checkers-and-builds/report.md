# How type checkers learn a project's structure from its build

**Question.** How do type checkers and language tools decide what a project is: which files belong together, which module name each file has, which Python version and which installed packages apply, and how a repository splits into units that are checked separately? Do they read build files, run the build tool, guess, or take a model that the build system exports? What follows for Mental's build model ([spec 6.10](../../specification.md#610-the-build-model-first))?

**Date:** 2026-11-17. Read from the tools' own documentation and, for Buck, from the source of its export script. Some long pages came back partly as summaries; claims that rest only on a summary, or on my reading rather than the page's text, are marked *unverified*.

## Per-product summary

- **Pyright / basedpyright:** one project per config file; *execution environments* give subtrees their own root, extra paths and Python version, but share one installed environment; adds `src/` by guess; runs the interpreter only when it cannot find site-packages.
- **mypy:** one config, one environment; maps paths to module names by walking up `__init__.py` files or from explicit bases; runs the target interpreter to list installed packages.
- **ty:** one project per config found by walking up; guesses roots (`.`, `src/`, `python/`, `<name>/<name>`); one environment found from `VIRTUAL_ENV`, Conda, `.venv`, `PATH`; can ask uv for the workspace root and environment.
- **Pyrefly:** guesses an import root like the others, but also takes a **source database from a build system**: Buck2 through a query script, any tool through a custom command, Bazel through an aspect with one check per target, Pants through a plugin with one run per partition.
- **rust-analyzer:** runs `cargo metadata` and builds a **crate graph**; runs build scripts and proc macros by default; for other build systems, reads `rust-project.json` that the build system writes.
- **gopls / go/packages:** runs `go list` to get packages; other build systems plug in through `GOPACKAGESDRIVER`; one build per module, workspace or platform.
- **TypeScript:** a `tsconfig.json` is a project; projects reference each other and see each other through emitted `.d.ts` files; module resolution imitates the target runtime or bundler, without running it.
- **clangd:** reads `compile_commands.json`, which CMake, Bear or a Bazel extractor writes; one compile command per translation unit; headers borrow a command by guess.
- **JVM tools (jdtls, IntelliJ, Metals):** run the build tool to import the project (the Gradle Tooling API "always uses the Gradle daemon"); Metals and others speak the **Build Server Protocol**, whose central record is a *build target*.

## 1. Python type checkers: one config, one environment, and a guess at the roots

### What a project is

All four start from a config file found by walking up from the working directory, and treat the directory it sits in as the project root.

| | Config files | Several projects in one repository |
|---|---|---|
| **Pyright** | `pyrightconfig.json`, else `[tool.pyright]` in `pyproject.toml`; `extends` for inheritance | each root of a multi-root editor workspace may have its own config; within one config, **execution environments** ([configuration.md](https://github.com/microsoft/pyright/blob/main/docs/configuration.md)) |
| **mypy** | `mypy.ini`, `.mypy.ini`, `pyproject.toml`, `setup.cfg`, in that order per directory; "there is no merging of configuration files" | none documented; one config per run, per-module sections only change checking options ([config_file](https://mypy.readthedocs.io/en/stable/config_file.html)) |
| **ty** | `ty.toml`, else `[tool.ty]` in the nearest `pyproject.toml`; reads `project.name` and `requires-python` even without a `[tool.ty]` table | not documented; `overrides` by file glob change only rules and analysis options, not the environment ([reference](https://docs.astral.sh/ty/reference/configuration/)) |
| **Pyrefly** | `pyrefly.toml`, else `[tool.pyrefly]`; also stops at `setup.py`, `mypy.ini`, `pyrightconfig.json`, and migrates a mypy or Pyright config in memory | each file argument uses its nearest config; `sub-config` by glob (*unverified* which keys); a build system's per-target configs ([configuration](https://pyrefly.org/en/docs/configuration/)) |

**Pyright's execution environments** are the nearest thing to units: each has a `root`, its own `extraPaths`, `pythonVersion` and `pythonPlatform`, and a file belongs to "the first one whose root directory contains that file" (from a summary of the page, *unverified* verbatim). All environments share one set of installed packages.

### From a file to a module name

- **mypy** crawls up `__init__.py` files and takes the highest one; with `explicit_package_bases`, the base is the nearest parent that is in `MYPYPATH`, `mypy_path` or the working directory ([running mypy](https://mypy.readthedocs.io/en/stable/running_mypy.html#mapping-paths-to-modules)).
- **The others** search a list of roots, first match wins. The roots are guessed when not given:

| | Roots when nothing is configured |
|---|---|
| **Pyright** | the execution environment's root, or the workspace root; its extra paths; then the local `src` directory, "if no execution environment is configured"; last, the importing file's directory and its parents within the workspace ([import-resolution.md](https://github.com/microsoft/pyright/blob/main/docs/import-resolution.md)) |
| **ty** | `.` always; before it, `./src`, `./<project-name>` (only if `./<project-name>/<project-name>` exists) and `./python`, each only if it is not itself a package ([modules](https://docs.astral.sh/ty/modules/)) |
| **Pyrefly** | `src/` next to the config, else the parent if the config's directory has an `__init__.py`, else the config's directory; turned off by `disable-search-path-heuristics`, which the docs recommend for monorepos ([import resolution](https://pyrefly.org/en/docs/import-resolution/)) |

None of them reads a compose file, a Dockerfile or a service command. So report-jobs, whose root `src/` is implied only by the compose build context and `uvicorn app.api:app`, resolves under Pyright, ty and Pyrefly only because they guess `src/`.

### The Python environment and version

| | How the environment is found | Runs the interpreter? | Python version |
|---|---|---|---|
| **Pyright** | `venvPath` + `venv`, else the interpreter the editor selected, else `python` on `PATH` | "If no site-packages directory can be found, Pyright attempts to run the configured Python interpreter and ask it for its search paths" | config, else the interpreter's |
| **mypy** | the interpreter running mypy, or `python_executable` | yes, to collect installed packages (the page says "inspect"; that it runs it is inferred from `no_site_packages` also disabling "searching for a usable Python executable", *unverified*) | `python_version`, else the running interpreter's |
| **ty** | `VIRTUAL_ENV`, a non-`base` Conda or Pixi environment, `.venv` in the project, `base` Conda, `python3`/`python` on `PATH`; or `environment.python` | not stated in the docs; in our [type-checker experiment](../type-checkers-as-libraries/report.md#experiment-pyrefly-against-ty-on-five-projects) it read `pyvenv.cfg` and site-packages as files | lower bound of `requires-python`, else the environment's metadata, else 3.15 ([python-version](https://docs.astral.sh/ty/python-version/)) |
| **Pyrefly** | `python-interpreter-path` (default `which python3`, then `python`), a find command, or a Conda environment by name | yes, it filters the interpreter's `sys.path`, unless `skip-interpreter-query`, which gives hard-coded defaults and an empty site-packages list | config, else the interpreter's, else 3.13 |

All of them warn about editable installs that use import hooks: "Pyright will be unable to find the corresponding source files", because finding them needs Python code to run. Only `.pth` files with plain paths work.

**uv.** Only ty mentions it: with `TY_UV=1`, "ty also uses uv to discover the workspace root and its Python environment" ([environment](https://docs.astral.sh/ty/reference/environment/)). None of the four reads `uv.lock` or `[tool.uv.workspace]` to split a repository into members.

## 2. Pyrefly and build systems: the build system hands over a source database

Pyrefly is the only Python checker with a documented build-system interface, and the interface is a model of the build, not the build files:

- **Buck2.** With `[build-system] type = "buck"`, Pyrefly runs `buck2 bxl … prelude//python/sourcedb/pyrefly.bxl:main @<argfile>`. The script asks Buck for the targets that own a file and their transitive dependencies, and emits, per target, `srcs` (module name → files), `deps`, `buildfile_path`, `python_version` and `python_platform` (hard-coded to `"linux"`, with a TODO) ([pyrefly.bxl](https://github.com/facebook/buck2/blob/main/prelude/python/sourcedb/pyrefly.bxl)).
- **Any other tool.** With `type = "custom"`, Pyrefly runs a `command` and reads JSON: a `root`, and a `db` mapping each target to its `srcs`, `deps`, `python_version`, `python_platform`, optional `config`, or an `alias`. Files outside the build file's directory count as generated. The docs warn that build-system support "is currently unstable" ([configuration](https://pyrefly.org/en/docs/configuration/)).
- **Per-target settings.** A target can name one of several `configs`; "A file's build system config replaces its sub-configs".
- **Bazel.** No `pyrefly.toml`: an aspect over `py_library`, `py_binary` and `py_test` gives each target "one `pyrefly bazel-check` action that checks the target's own sources … with its dependencies available for import resolution". The importable files are the target's transitive sources from `PyInfo`, so **an undeclared dependency is not importable** ([Bazel](https://pyrefly.org/en/docs/bazel/), [rules_pyrefly integration](https://github.com/facebook/rules_pyrefly/blob/main/docs/integration.md)).
- **Pants.** A community plugin runs Pyrefly once per partition of targets "grouped by resolve and interpreter constraints", with search paths from Pants source roots and the heuristics turned off ([Pants](https://pyrefly.org/en/docs/pants/)).

Whether a Buck or custom target sees only its declared `deps` is not documented (*unverified*).

## 3. Other languages: the build system exports a project model

| | Unit of analysis | Where the model comes from | Runs the build? |
|---|---|---|---|
| **rust-analyzer** | a **crate** with its root file, edition, `cfg` flags, environment and dependencies; one crate can appear several times with different `cfg`s, each with its own analysis ([architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)) | `cargo metadata`; for Buck, Bazel and others, a `rust-project.json` listing every crate, written by the build system or by a "discover" command that prints it as JSON lines ([non-Cargo projects](https://rust-analyzer.github.io/book/non_cargo_based_projects.html)) | yes: build scripts and proc macros run by default ("Run build scripts (`build.rs`) for more precise code analysis"), can be turned off ([configuration](https://rust-analyzer.github.io/book/configuration.html)) |
| **gopls / go/packages** | a **package** under one build configuration (GOOS, GOARCH, tags); one build per module, `go.work` workspace or platform; operations run in "the default build for that file" ([workspace](https://github.com/golang/tools/blob/master/gopls/doc/workspace.md)) | `go list`; other build systems through a driver named in `GOPACKAGESDRIVER`, which reads a JSON request on stdin and prints packages as JSON ([go/packages](https://pkg.go.dev/golang.org/x/tools/go/packages)) | runs `go list`, which reads the build and does not compile |
| **TypeScript** | a **project**, one `tsconfig.json`; projects reference each other, and "importing modules from a referenced project will instead load its output declaration file (`.d.ts`)" ([project references](https://www.typescriptlang.org/docs/handbook/project-references.html)) | the tsconfig files themselves; `moduleResolution` "should be set to match the module resolver used by the target runtime or bundler" ([modules reference](https://www.typescriptlang.org/docs/handbook/modules/reference.html)) | no; resolution is TypeScript's own imitation of the runtime or bundler |
| **clangd** | a **translation unit** with its own compile command | `compile_commands.json` from CMake, Bear or a Bazel extractor; otherwise `compile_flags.txt`; otherwise "a very simple command like `clang foo.cc`"; a header borrows the command of a file that includes it or by "a filename-based heuristic" ([compile commands](https://clangd.llvm.org/design/compile-commands), [format](https://clang.llvm.org/docs/JSONCompilationDatabase.html)) | no; something else produces the database |
| **JVM: jdtls, IntelliJ, Metals** | a **build target** or source set with sources, dependencies and a classpath | the build tool. Buildship and IntelliJ use the Gradle Tooling API, which "always uses the Gradle daemon" and fails "if the target project's configuration script fails to evaluate" ([Tooling API](https://docs.gradle.org/current/userguide/tooling_api.html)); Metals exports sbt, Maven, Gradle or Mill builds to Bloop ([Metals](https://scalameta.org/metals/docs/build-tools/overview)) | yes |

**The Build Server Protocol** (BSP 2.2.0, "not an approved standard") is the shared form of this idea. A *build target* has an ID, a base directory, tags (`library`, `application`, `test`, …), language IDs and direct `dependencies`. Separate requests return its sources, dependency sources and resources. Language data hangs off it by `dataKind`, and `python` is one of the kinds (its fields not read). A client starts the server from a `.bsp/*.json` file in the workspace ([specification](https://build-server-protocol.github.io/docs/specification), [server discovery](https://build-server-protocol.github.io/docs/overview/server-discovery)). That file is a command line committed to the repository.

## 4. What they have in common

- **Three ways in.**
  1. **Guess:** the Python checkers, clangd's fallback, TypeScript's inferred projects.
  2. **Run the build tool:** rust-analyzer, gopls, the JVM tools, and the Python checkers when they ask the interpreter for `sys.path`.
  3. **Take a model the build system exports:** `rust-project.json`, `compile_commands.json`, `GOPACKAGESDRIVER`, BSP, Pyrefly's source database.

  Where a language has several build systems, the third way wins: each build system writes the model, and the checker reads one format.
- **The exported model is small and much the same everywhere:** a list of units, each with its files (often with module names), its direct dependencies, its language settings (Python version, edition, `cfg`, compile flags, GOOS) and, in some, the build file that declares it.
- **Python checkers have the weakest unit concept.** In Python, a project is one config with one environment and one Python version. Splitting it into units with separate environments comes only from a build system (Pyrefly with Buck, Bazel or Pants) or from running the checker several times.
- **Visibility differs.** A Bazel or BSP target sees only its declared dependencies. A Python checker without a build system sees everything installed in the environment.
- **Execution is common and mostly unguarded.** rust-analyzer runs build scripts and proc macros, the Gradle Tooling API runs the build's configuration scripts, and Pyright, mypy and Pyrefly run the project's interpreter. A `.bsp` file in a repository names a command the client starts. None of the pages read mentions a trust prompt.

## 5. What this means for Mental's build model

- **Mental's build model is the same kind of record these tools take as input.** A build unit with its roots, Python version, environment and dependencies on other units carries what Pyrefly's custom source database, a `rust-project.json` crate or a BSP build target carries.
- **It could feed the checkers directly.**
  - *Pyrefly:* `mentalc` could act as a custom build system. It would print the `db` (per build unit: module names to files, `deps`, `python_version`, `python_platform`) and so turn Pyrefly's guessing off. This interface is unstable, so the Pyrefly version would be pinned.
  - *ty:* ty has one environment per run, so a repository with several build units with different environments means one ty run per unit, as the Pants plugin does for Pyrefly.
  - In both cases the roots come from the build model, for example `src/` in report-jobs from the compose build context and `uvicorn app.api:app`, not from a `src/` guess.
- **Not asking the interpreter is possible.** `skip-interpreter-query` with an explicit site-packages path for Pyrefly, and reading `pyvenv.cfg` for ty, both worked in the type-checker experiment. The build model names the environment, so `mentalc` never needs to run Python to learn `sys.path`.
- **The Python checkers don't tell which dependency each unit declared.** If Mental wants a unit to see only its declared dependencies, as Bazel targets do, the build model has to say so and the checker has to be given a matching search path. This is a choice, not something the checkers already provide.
- **A standard format exists if one is wanted.** BSP's build target, with Python as a `dataKind`, is the closest thing to a shared format for a build model. Reading or writing it is optional. It is not needed for version one.

## Not read

The BSP Python extension, tsserver's choice of project for an open file, the clangd configuration reference, how m2e imports Maven projects, and the source of Pyrefly's build-system client.
