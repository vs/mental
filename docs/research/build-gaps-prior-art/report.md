# How PyCharm, Pyright and ty handle the gaps of F-B9

**Question.** Mealie and Redash do not build because of three gaps ([F-B9](../../findings.md#b-mentalc-build-on-outside-projects)): files under a root that are no module, dependency groups that `uv sync` does not install, and a call of a subscripted generic class. How do PyCharm, Pyright and ty handle the same cases, and what can Mental reuse?

**Date:** 2026-11-25. Read from source: PyCharm's Python plugin in [intellij-community](https://github.com/JetBrains/intellij-community/tree/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python) at `2f38595` (2026-11-25, Apache 2.0), [Pyright](https://github.com/microsoft/pyright/tree/2981f9f1ce41be5c15667da1f7a3cfda4f245089) at `2981f9f` (2026-11-20), and ty 0.0.105 at the ruff commit `mentalc` pins, `4228f4c`. Runtime behavior was checked against uv 0.13.9 and alembic 1.20.1, the version Mealie locks. `mentalc build` was run again on Mealie at `04ba6b7` to see which of its imports fail. A claim that rests on my reading of the code, not on a comment or a run, is marked *reading*.

## Per-product summary

- **PyCharm:** resolves each import from the importing file's folder first, a relative import always and an absolute one too in project files by default, so no module name is needed; a file with no importable name resolves but gives its definitions no qualified name. Syncs with `uv sync --all-packages`, so with uv's default groups, and does not ask for group packages to be installed. Can hand type checking to ty or Pyrefly; its own engine is the default.
- **Pyright:** resolves a relative import from the importing file's folder, and an absolute import that fails from that folder and each one above it. Names a file with invalid characters by its dotted path anyway, to keep types apart. Reads no build file.
- **ty:** names a file only when every part of its path under a search path is an identifier, and a relative import needs that name. An absolute import falls back to the folders above the file, so the scripts' imports resolve while the package they import cannot resolve its own. With uv metadata, it checks that package code imports only declared dependencies; groups count only outside package code.

## 1. Files that are no module

### The two cases, and what Python does

- **A script folder.** Mealie's `mise.toml` runs `uv run python dev/code-generation/main.py`. Python puts the script's folder, `dev/code-generation`, first on `sys.path`, so `utils` is a package there and `utils/__init__.py` imports its siblings with `from .route import …`. From the repository root, the path `dev/code-generation/utils` is no module name: `code-generation` holds a hyphen.
- **Migrations loaded by path.** Alembic loads `mealie/alembic/versions/2022-02-21-19.56.24_6b0f5f32d602_initial_tables.py` with `importlib.util.spec_from_file_location`. The module's name is the file name with every non-word character replaced by `_`, `2022_02_21_19_56_24_6b0f5f32d602_initial_tables_py`, and it never goes into `sys.modules` ([`load_python_file`](https://github.com/sqlalchemy/alembic/blob/rel_1_20_1/alembic/util/pyfiles.py)). `mealie/alembic/` and `versions/` have no `__init__.py`.

So Python itself gives both a name, but neither name follows from the path under the repository's root. The script's comes from how it is started; the migration's comes from the library that loads it.

### What `mentalc` does today

The run on Mealie gives exit 1 with 19 `module-missing` errors. All 19 are the relative imports in `utils/__init__.py` and `utils/open_api_parser.py`, such as `from .route import HTTPRequest`. The absolute imports in the scripts, such as `from utils import log` in `main.py`, do not fail: they are `not_found`. Of the 4,140 sites in `mealie/alembic/versions/`, 368 are `not_found`. Both come from ty's module resolver, through two calls `mentalc` makes:

- **A missing module** is an import whose module ty's "go to definition" cannot find ([`no_answer`](../../../extract/src/python/resolve.rs)).
- **A `not_found` target** is a definition ty did find, in a file `file_to_module` gives no name ([`target`](../../../extract/src/python/resolve.rs)). This is the migrations' case: ty resolves the call to a function in the same file, and `mentalc` cannot name it.

### ty

- **A file's name.** [`file_to_module`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_module_resolver/src/resolve.rs#L394) relativizes the path against each search path and keeps it only if [every part is an identifier](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_module_resolver/src/path.rs#L169). If none fits, it tries one more base: the [nearest folder above with a `pyproject.toml` or `ty.toml`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_module_resolver/src/resolve.rs#L695). For Mealie that is the root again, so `utils/__init__.py` and every migration get no name.
- **A relative import** is resolved from the importing file's module name: `from .route` in `utils` becomes `utils.route` ([`relative_module_name`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_module_resolver/src/module_name.rs#L549)). With no name, it fails with `UnknownCurrentModule`. This is the source of the 19 errors.
- **An absolute import** that the search paths cannot find falls back to "desperate" search paths: [every folder above the importing file that has no `__init__.py`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_module_resolver/src/resolve.rs#L626), up to the first-party root. The code comments that this "reflects pytest's default mode". `dev/code-generation` has no `__init__.py`, so `from utils import log` in `main.py` finds `utils`.
- **The asymmetry.** The fallback for absolute imports is not used when naming a file. So `main.py` reaches `utils`, but `utils` cannot reach `.route`. *Reading:* if `file_to_module` also tried the folders of the absolute fallback, `utils/__init__.py` would be `utils` under `dev/code-generation`, and the 19 imports would resolve. That is a change to ty, worth an issue upstream; it would not name the migrations, whose file names are no identifiers.

### Pyright

- **A relative import** is resolved from the importing file's folder, by walking up one folder per leading dot and resolving the rest there ([`_resolveRelativeImport`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/importResolver.ts#L2167)). The importing file needs no module name, so `from .route import …` resolves in `dev/code-generation/utils/`.
- **An absolute import** that fails on the search paths is tried in the importing file's folder, then in each folder above it, up to the execution root ([`resolveImportInternal`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/importResolver.ts#L518)). This is the same fallback as ty's, without the `__init__.py` condition.
- **A file's name.** Pyright joins the path under the best search path with dots and, if a part is no identifier, keeps the name anyway: "We allow illegal module names (e.g. names that include "-" in them) because we want a unique name for each module even if it cannot be imported" ([`program.ts`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/program.ts#L2060), [`_getModuleNameForImport`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/importResolver.ts#L1067)). `utils/route.py` becomes `dev.code-generation.utils.route`. The migration becomes `mealie.alembic.versions.2022-02-21-19.56.24_6b0f5f32d602_initial_tables`, *reading* [`_getModuleNameInfoFromPath`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/importResolver.ts#L2587), whose dots inside the file name can no longer be told from the dots between folders.

### PyCharm

- **Resolution is by syntax tree element, not by name.** PyCharm parses with its own parser into PSI trees and resolves a reference to the element it names. A name is needed only to print or to import.
- **A relative import** is resolved in the importing file's folder ([`resolveWithRelativeLevel`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/src/com/jetbrains/python/psi/resolve/PyResolveImportUtil.kt#L253)), as in Pyright.
- **An absolute import** in a project file is also tried in the importing file's folder, before the roots ([`ResolveImportUtil`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/src/com/jetbrains/python/psi/resolve/ResolveImportUtil.java#L63)). The switch is the registry key `python.explicit.namespace.packages`, on by default: "Require marking namespace packages explicitly, treat regular directories as implicit source roots" ([registry](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/resources/intellij.python.psi.impl.xml#L981)). For `main.py`, that is Python's own rule for a script.
- **A file's name** is the shortest path under a root whose parts are all identifiers ([`PathChoosingVisitor`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/src/com/jetbrains/python/psi/resolve/QualifiedNameFinder.java#L305)). The roots are the source roots a user marks, the content roots and the interpreter's paths. With none, a class or function in the file has no qualified name: [`getQualifiedNameInner`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/src/com/jetbrains/python/psi/resolve/QualifiedNameFinder.java#L263) returns `null`. Navigation still works, since it goes by element.
- **The user closes the gap.** For a script folder, the remedy PyCharm offers is to mark it as a source root, which makes `utils.route` a name.

### What Mental could take

| Option | Precedent | Fixes | Costs |
|---|---|---|---|
| **Resolve a relative import from the file's folder** | Pyright, PyCharm | the 19 `module-missing` | it is ty that resolves imports; this is ty's change to make, or the upstream issue above |
| **Name a file with no module name by its path** | Pyright keeps the path; PyCharm and ty give no name | the 368 migration targets become `resolved` | a name no import can use; it must not look like a module name. A dotted path is ambiguous, as Pyright's is; the path with `/` is not |
| **Take roots from how code runs** | Python itself; PyCharm's source roots, set by hand | the script folder becomes a root, and `utils.route` a real name, as at runtime | needs the entry point (`uv run python dev/code-generation/main.py` in `mise.toml`), which milestone 2 reads |
| **Let the loader name loaded-by-path files** | alembic's own `load_python_file` | each migration gets the name alembic gives it | a rule per library: work for an alembic pack, not for the build model |
| **Fail only where it matters** | the gate decided in F-B5: code that an entry point reaches | dev scripts stop failing the build | needs entry points, milestone 2 |

None of the three tools fails on these files. Each one either guesses, or leaves a name out and goes on. What fails the build is Mental's own decision.

## 2. Dependency groups that `uv sync` leaves out

### What uv does

`uv sync` installs the project's dependencies and its default groups: `dev`, unless `[tool.uv] default-groups` says otherwise. It installs no extras unless asked (`--extra`, `--all-extras`), other groups only with `--group` or `--all-groups`, and `--no-default-groups` leaves the default ones out. Redash sets `default-groups = []`, so `uv sync` installs no group. `mentalc` pins every group and every extra of the project ([specification](../../specification.md#5-stage-1-the-build-model)), so its pins include packages that are not in `.venv`. Its fix, "run `uv sync`", then changes nothing.

`uv workspace metadata` (uv 0.13.9, schema `preview`) prints the members, the workspace and every group with its dependencies. On Redash it lists `dev`, `all-ds` and `ldap3` as group nodes. It does not say which groups are default, and it has no module owners.

### ty

- **It reads uv when asked.** *Reading:* with `TY_UV=1`, ty runs `uv workspace metadata` ([command](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_project/src/uv/command.rs#L53)) and reads members, the environment, each project's dependencies, extras and groups, and which distribution owns each module, when uv supplies it. It is off by default, and `mentalc` does not turn it on.
- **Groups count only outside package code.** An import is allowed when the importing project declares the imported module's distribution in `project.dependencies` or `project.optional-dependencies`. A group dependency is allowed only in a file that is not the project's package code, such as a test or a script ([`is_direct_dependency`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_python_semantic/src/dependency.rs#L116), [`missing_dependency`](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_python_semantic/src/dependency.rs#L170)). Package code is a file whose module the project's distribution owns, or a file under its editable root.
- **The message names the declaration.** "Import of `x` requires a direct dependency on `y`", with "Declare `y` in `project.dependencies` or `project.optional-dependencies` in your `pyproject.toml`" and, for a group, "Dependency groups are only available to non-package files" ([imports.rs](https://github.com/astral-sh/ruff/blob/4228f4c2168d50dc60df1aec705a1438c75a4888/crates/ty_python_semantic/src/types/infer/builder/imports.rs#L94)). This checks what is declared, not what is installed.

### Pyright

It reads no `pyproject.toml` dependencies and no lock. An import resolves against the interpreter's `site-packages` or fails with `Import "x" could not be resolved`, with no fix named.

### PyCharm

- **Sync is plain uv.** Its sync runs `uv sync --all-packages`, every workspace member with uv's default groups ([`UvLowLevel`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-uv/backend/src/com/jetbrains/python/sdk/uv/impl/UvLowLevel.kt#L391)). Its package tree shows every group (`uv tree --frozen --all-groups`).
- **Groups are not required.** The inspection "Package requirements not satisfied" checks only the packages in no group ([`filterToMainPackages`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/src/com/jetbrains/python/inspections/requirement/DeclaredButNotInstalledPackagesChecker.kt#L30)), and offers "Sync project" and "Ignore requirement".

### What Mental could take

- **Pins follow what `uv sync` installs**: the project's dependencies and the default groups (`default-groups`, else `dev`), no extras. This is uv's own rule, PyCharm's environment, and the reading of uv the specification already follows for workspaces. A group or extra left out is not pinned.
- **A missing module that a left-out group would provide names the group.** For Redash: `uv sync --group dev` for `pytest`. Finding the group needs the package that provides the module. The lock does not list modules, and uv 0.13.9 prints no module owners, so for a package that is not installed it is a guess by name. Where the guess fails, the fix can list the groups left out.
- **ty's split between package code and the rest** is a ready rule for which imports must be satisfied. Redash's 9 unguarded driver imports in `redash/` come from the group `all_ds`; by ty's rule, package code that relies on a group is a declaration problem, which a fix can name as such.
- **Extras have the same gap.** `mentalc` pins every extra, which `uv sync` does not install. Not seen yet on the five projects.

## 3. A subscripted class as callee

`HttpRepo[A, B, C](…)` is called 19 times in Mealie and comes out `dynamic`.

- **ty** types the callee `HttpRepo[A, B, C]` as a `GenericAlias`. `mentalc` already takes a `GenericAlias` callee as the class it names ([`called`](../../../extract/src/python/resolve.rs)), but never gets there: the walker gives a subscript callee no place to ask ty ([`asked`](../../../extract/src/python/sites.rs)), and a site with nothing to ask is `dynamic`.
- **Pyright** takes the type of any callee expression, a subscript included ([`getTypeOfCall`](https://github.com/microsoft/pyright/blob/2981f9f1ce41be5c15667da1f7a3cfda4f245089/packages/pyright-internal/src/analyzer/typeEvaluator.ts#L6530)), and a class type is a call of the class.
- **PyCharm** does the same: [`getCalleeType`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-psi-impl/src/com/jetbrains/python/psi/impl/PyCallExpressionHelper.kt#L157) works on any callee expression and turns a class type into its constructor.

**What Mental could take:** all three go by the callee's type, not its syntax. A subscript callee that ty types as a class is a call of that class. One that it types otherwise, as in `handlers[0]()`, stays `dynamic`. The change is in the walker; ty already answers.

## 4. Relying on these tools

- **PyCharm's resolver is not reusable as code.** It is Java and Kotlin on the IntelliJ platform, with its own parser and syntax tree. What it offers are rules, and its rules here favor never failing: guess a root, or leave a name out. JetBrains itself now lets PyCharm hand type checking to ty or Pyrefly ([`PyTypeEngineId`](https://github.com/JetBrains/intellij-community/blob/2f38595b1d1f40c92a281cfa8285ae1d88b5e4b0/python/python-type-engine/common/src/PyTypeEngineApi.kt)), with its own engine as the default.
- **On gaps 1 and 3 the tools mostly agree.** An absolute import falls back to the folders above the file in all three, and a call goes by the callee's type in all three. Pyright and PyCharm also resolve a relative import by folder; ty alone needs the file's name. That one difference is ty's gap, and an upstream issue is the cheapest fix.
- **On gap 2, uv is the authority, not an IDE.** PyCharm and ty both defer to uv: PyCharm runs it, ty reads its metadata. Mental's build model already reads uv as uv does; pins should too.
- **Runtime is the ground truth the tools approximate.** A script's folder and a loader's name are what Python actually uses. Both need to know how the code is started, which is what milestone 2's entry points and packs add.

## Not read

mypy, which names files by walking up `__init__.py` files ([type checkers and builds](../type-checkers-and-builds/report.md#from-a-file-to-a-module-name)); how PyCharm marks source roots when it opens a uv project; ty's issue tracker, for an existing report of the relative-import asymmetry; uv's source for `workspace metadata`, and whether a later uv prints module owners or default groups.
