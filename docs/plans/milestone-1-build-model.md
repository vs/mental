# Implementation plan: milestone 1, resolve a simple project

**Status:** decisions settled with the owner and written into the specification, 2026-11-18. **Specifies:** [stage 1](../specification.md#5-stage-1-the-build-model) and the first half of [stage 2](../specification.md#6-stage-2-from-code-to-facts): parse, list sites, resolve. **Rules it follows:** the [guidelines](../guidelines/README.md) and the [testing playbook](../guidelines/testing.md).

## 1. Goal

`mentalc build` takes a standard Python project, finds its dependencies, parses every file and says what every reference points to. There is no Docker, no compose file, no outside process, no pack, no LLM and no Mental-specific project file.

A standard project is what `uv init` makes: a `pyproject.toml`, a `uv.lock`, a `.python-version` file, and a `.venv` after `uv sync`. `examples/report-jobs` becomes one.

**Done when:**

1. `mentalc build` in `examples/report-jobs`, with no `.venv`, writes 1 build unit and one pin per package in `uv.lock` other than the project itself, equal to the hand-written reference in `examples/report-jobs/model/` without its `environment`, reports the one gap `environment-missing` with its fix (`uv sync`), and exits 1.
2. After `uv sync`, it parses the five files and writes site records equal to the hand-written reference: every reference site has exactly one outcome, and every `report.status = …` write receiver resolves to `Report`. It writes the resolution report and exits 0.
3. In a copy with one package removed from the environment, it reports `module-missing` for that package, names the fix, and exits 1.
4. Building twice gives the same bytes.
5. Every gate of the guidelines is green on the three crates.

## 2. Scope

| In | Out (and where it goes) |
|---|---|
| In `model`: semantic IDs, slugs, diagnostics, canonical JSONL, anchors, three record kinds: `build_unit`, `pin`, `site` | every other record kind; guards |
| In `extract`: readers for `pyproject.toml`, `uv.lock`, `.python-version` and an environment's `pyvenv.cfg`; parse with ruff; list all seven kinds of site; resolve with ty | requirements files, `pylock.toml`, Poetry and PDM locks; obligations, effects, entry points (the facts milestone) |
| In `mentalc`: reading the working tree, `mentalc build`, `--json`, exit codes, the resolution report | `mentalc.toml`, `mentalc init`, acceptances, the reachability part of the gate (it needs entry points), the LLM loop, MCP |
| | Dockerfiles, compose files, artifacts, launches, configuration inputs, replica counts: [Later](../specification.md#151-later) |

## 3. Decisions

Settled with the owner on 2026-11-18.

| # | Question | Decision | Why |
|---|---|---|---|
| D1 | What is a project? | What Python already calls one: a directory with a `pyproject.toml`. It is the build unit; the system and the unit take the project's name. Mental needs no file of its own to read a project. A directory without one is the gap `project-missing` | nobody would write a Mental-specific definition; the example is made standard instead |
| D2 | Where are the source roots? | The two standard layouts: `src/` when the project has one, else the project directory | report-jobs already keeps its code in `src/app/` |
| D3 | Which Python version? | `.python-version`; else the environment's `pyvenv.cfg`. The record says which source won. `requires-python` is recorded and never used as the version | a range is not a version |
| D4 | Who reads files for ty? | ty reads the roots and the environment from disk itself, read-only, inside `extract/src/python/resolve.rs`; it is not an edge of our own code | it is the setup the experiment measured |
| D5 | Which sites? | All seven kinds of the specification: import, decorator, callee, base class, write receiver, escape, handler type. Six come from the syntax tree; an escape needs resolution to be told apart, so step 7 finds it (owner choice, 2026-11-19) | complete from the start |
| D6 | What fails the build? | A missing input (`project-missing`, `lock-missing`, `python-version-unknown`, `environment-missing`) and a missing module. Every other unresolved site is listed in the resolution report and fails nothing yet | which gaps count needs entry points, which do not exist yet |
| D7 | How is a site identified? | By names: the qualified name of the enclosing scope (function, class body or module), the site's kind and the name it refers to as written. Sites are grouped by scope, kind and the name's slug without trailing digit-only segments; every site in a group of two or more gets its order in the group, counted from 0, so `s.commit_0()` cannot collide with a counter. The three `s.commit()` calls of `generate_report` are `site:app-tasks-generate-report-callee-s-commit-0` to `-2`. Never a line number | a change outside the scope changes no ID, and neither does a moved line unless it holds one of a group of two or more sites; Semgrep, rust-analyzer and Infer identify things the same way |
| D8 | Is every site in the hand-written reference? | Yes: every site record of report-jobs is written by hand before the code | the reference is a specification of the whole output |
| D9 | How do tests get an environment? | Resolver tests use tiny environments checked in as fixtures: a `pyvenv.cfg` and a few stub packages. One CI job runs `uv sync` in report-jobs and runs the acceptance of goals 2 and 3; locally it runs when `RUN_SLOW_TESTS` is set | fast tests with no network; the real path still runs on every pull request |

## 4. Records

| Kind | Fields |
|---|---|
| `build_unit` | `id`; `dir`; `roots`; `python`: version and its source; `requires_python`; `lock`: path and schema version; `environment`: path |
| `pin` | `id`; `package` (normalized name); `version`; `origin`: registry, a git commit, or a path |
| `site` | `id`; `kind` (step 6); `outcome` (step 7): `resolved` with a definition (module and qualified name), `external` with a package and an access path, or `unresolved` with a reason and, when another site is its cause, that site; `checker`: name and version |

Every record has an anchor (file, lines, quote) in `anchors.jsonl`, and a `source`.

## 5. Diagnostics

| Code | Severity | Means | Fix text |
|---|---|---|---|
| `project-missing` | error | the directory has no `pyproject.toml` | add one, for example with `uv init` |
| `project-invalid` | error | the `pyproject.toml` does not read, has no `[project]` name, or its name has no ASCII letter | a `[project]` table with a `name`, as `uv init` writes it |
| `lock-missing` | error | the project has no `uv.lock` | `uv lock` |
| `lock-invalid` | error | the `uv.lock` does not read, or holds a package it cannot pin: one with no fixed version, a name with no ASCII letter, a subdirectory of a `url` archive, or a source of another form than `registry`, `git` with a commit, `url`, `path`, `directory`, `editable` or `virtual` | `uv lock`; for a dynamic version, a static one; for a name with no letter, another name; for a subdirectory, a built archive or a local path |
| `lock-ambiguous` | error | the `uv.lock` locks one package more than once, such as at one version per platform | one version for every platform: a tighter bound, or `environments` under `[tool.uv]` |
| `lock-version-unknown` | error | a `uv.lock` schema version this build does not read | the versions it does read |
| `python-version-unknown` | error | no source of D3 states a version | add a `.python-version` file |
| `environment-missing` | error | the project has no `.venv` | `uv sync`; `mentalc` does not run it |
| `module-missing` | error | an import names a module that is neither under a root nor in the environment | `uv sync`, or add the dependency |
| `syntax-error` | warning | a file does not parse; the recovered tree is used | — |

An unreadable file is a tool failure: exit code 2.

## 6. Layout

```text
model/    src/  id.rs  diagnostic.rs  jsonl.rs  anchor.rs  record.rs  record/build.rs  record/site.rs  testkit.rs
extract/  src/  lib.rs
                build.rs  build/pyproject.rs  build/lock.rs  build/python.rs    strings in, records and diagnostics out
                python.rs  python/parse.rs  python/sites.rs                     one file in, sites out
                python/resolve.rs                                               sites + unit -> outcomes (ty reads)
mentalc/  src/  main.rs  cli.rs  lib.rs  repo.rs  build.rs
examples/report-jobs/   pyproject.toml  uv.lock  .python-version  src/app/
examples/report-jobs/model/   the reference: build_unit, pin, site
```

Dependencies: `model` has `serde` and `serde_json`; `extract` adds `toml`, `thiserror` and the ruff and ty crates at the commit the experiment used; `mentalc` adds `clap` and `anyhow`. Dev: `insta`, `proptest`, `insta-cmd`, `tempfile`.

## 7. Steps

Each step is one pull request that leaves every gate green, with its tests in the same change.

| # | Step | Ends with |
|---|---|---|
| 0 | Done: two spikes, before any production code. Coverage: assertions add no line or branch to the report, and the error side of a `?` counts only when it starts the first code region of its line ([testing playbook](../guidelines/testing.md#branch-coverage-of-assertions-settled)). ty in the workspace: go, see [section 9](#9-the-ty-spike) | both answers written into the guidelines; a go on ty in process |
| 1 | Done: report-jobs is a standard uv project. `pyproject.toml` names it `reports`, with Python 3.12 or later and its 11 dependencies as lower bounds (SQLAlchemy kept below 2.1, whose default PostgreSQL driver is psycopg 3, not the psycopg2 the example uses); `uv.lock` (41 packages) replaces `requirements.lock`; `.python-version` says 3.12. No Python file changed | `uv sync` works, every module of the example imports, and the lock has no known vulnerability |
| 2 | Done: `model`: `id`, `diagnostic`, `jsonl`, `anchor`. IDs take only the kinds `unit`, `pin` and `site` so far; a diagnostic has `id`, `code`, `at` and `message`, and the `field` and `related` of [M13](../guidelines/model.md#m13-diagnostic-codes-are-closed-and-complete) come with the checker | ID tables, round-trip properties |
| 3 | Done: `model::record`: `build_unit`, `pin`; the reference for them written by hand, in `examples/report-jobs/model/` as `build_unit.jsonl`, `pin.jsonl` and `anchors.jsonl`. The reference is the build after `uv sync` ([goal 1](#1-goal) compares without `environment`). The format version and `format.json` come with `mentalc` writing `.mental/` in step 5 | the format snapshot |
| 4 | Done: `extract::build`: one reader each for `pyproject.toml`, `uv.lock` and the Python version, and `build_model`, which assembles the unit from the files' contents. Three gaps were added ([section 5](#5-diagnostics)): `project-invalid`, `lock-invalid` and `lock-ambiguous`, and the origin `url` for an archive at a URL. `path`, `directory`, `editable` and `virtual` sources are all the origin `path`. A package from a subdirectory of a `url` archive gets no pin yet (`lock-invalid`) | fixtures per file and per error code (`extract/tests/data/err/<code>/`); report-jobs equals its reference |
| 5 | `mentalc`: `repo`, `build`, `--json`, exit codes | goal 1 |
| 6 | Done: the `site` record with `id`, `kind` and `source`; report-jobs' 133 sites and their anchors written by hand (41 imports, 71 callees, 7 decorators, 4 base classes, 8 write receivers, 2 handler types); `extract::python`: `parse`, `sites`, and `list_sites`, which numbers each group of sites across the build unit. Each site's kind, scope and name were written by hand; a throwaway script only formatted them as JSON lines and copied the quoted lines. The outcomes move to step 7, so that the form of an external target is settled where ty's answers are at hand. Fixtures in `extract/tests/fixtures/<category>/` carry `# $ kind=slug` on the line each site starts | every call, decorator, import, base, attribute write and handler type of the syntax tree appears exactly once, on every fixture and on report-jobs |
| 7 | The `outcome` and `checker` of the `site` record, and report-jobs' outcomes written by hand before the code; escapes; `extract::python::resolve` | every outcome and every reason reached by a fixture; outcomes equal the reference |
| 8 | `mentalc`: stage 2 in the build, the resolution report, the CI job that runs `uv sync` | goals 2, 3 and 4 |
| 9 | Measure on rq and full-stack-fastapi-template at the experiment's commits; compare resolved rates with the experiment's (88.5% of calls over five projects) | a findings entry; differences explained or fixed |

## 8. Risks

- **ty's API is unstable.** The spike used only public items, but a ruff bump can move them; [X3](../guidelines/extract.md#x3-bump-ruff-on-purpose) makes each bump its own change. The fallback is still Pyrefly's export.
- **Escapes and handler types are unmeasured.** The experiment covered the other five kinds; how well ty answers these two is first seen in step 7.
- **The site reference is large.** The experiment counted about 190 sites in report-jobs for five kinds. Writing each by hand, with its outcome, before the code is the biggest single piece of step 6, and its outcomes are a prediction of what ty answers.
- **One package locked twice.** `uv.lock` can hold one package at two versions, one per platform marker; their pins would get the same ID, and a version may not go into an ID. Step 4 made it the gap `lock-ambiguous`, which fails the build: a project whose lock forks by platform cannot be built until a later milestone gives a pin more than one version.
- **Counters shift.** Inserting an identical reference earlier in a function renumbers the later ones. In report-jobs, 22 of the 133 sites carry a counter, in 7 groups: the 10 `Column(…)` calls of the two model classes, the 3 `s.commit()` calls and 3 `report.status` writes of `generate_report`, and 2 each of `HTTPException`, `click.ClickException` and `func.now`.
- **report-jobs fails its own build until `uv sync` is run.** That is goal 1, by design.

## 9. The ty spike

Done on 2026-11-19. The pinned crates were added to `extract`, and one function opened a ty database for a project and resolved every call of a file with ty's "go to definition". The code is on the local branch `spike/ty-in-process`, for step 7 to start from; it is not merged.

| Gate | Result |
|---|---|
| Builds on the workspace toolchain | yes, on the pinned Rust 1.99.1 and on 1.100.0; ruff needs 1.98 at commit `4228f4c` |
| Coverage on nightly-2026-11-10 | yes; the report holds only `extract`'s own files |
| `Cargo.lock` | 244 packages; of the banned list, only `salsa` and `rand`, both through ty ([G16](../guidelines/README.md#g16-banned-weight)) |
| Vulnerability scan | no known vulnerability; the ruff crates themselves come from git, which the scanner cannot match |
| `cargo machete`, clippy pedantic | clean, once the review removed `ty_python_core`, a direct dependency the code did not use |
| Clean build of `extract` | debug 45 s, release 81 s, debug with optimized dependencies 138 s; a rebuild after a change about 1 s |
| Speed of a resolver test | fixture project: 241 ms on debug, 49 ms with optimized dependencies |
| Speed on report-jobs, every call of the five files | debug 9.6 s, optimized dependencies 0.9 s, release 0.75 s |
| Ambient input | ty's own system reads `PYTHONPATH`, `VIRTUAL_ENV`, `CONDA_PREFIX` and a user `ty.toml`, and can run `uv`; a wrapper of about 70 lines shuts all of it off. Two variables are read straight from the process, past the wrapper: `TY_CONSTRAINT_SET_ORDER` and `RAYON_NUM_THREADS`; setting them to `reverse` and `1` changed no outcome ([X5](../guidelines/extract.md#x5-pure-stages-one-module-each)) |

What step 7 inherits:

- **Optimized dependencies in tests** ([T19](../guidelines/testing.md#t19-a-speed-budget)): without them, one resolver test on report-jobs breaks the 1 s budget.
- **Several targets for one site.** "Go to definition" returns every overload: `print(...)` and `"a".upper()` each gave two definitions in `builtins.pyi`. An outcome in the specification names one definition or one access path, so step 7 decides how several targets become one outcome: for example the first one, or `unresolved` with the reason `ambiguous`.
- **Options as a struct.** ty's `Options` and `EnvironmentOptions` have public fields: the roots, the environment and the version go in as `RelativePathBuf::cli` and `RangedValue::cli` values, the way ty's own command line builds them. No TOML text and no escaping.
- **`OsSystem::new` asserts an absolute path.** `mentalc` passes an absolute project directory, or the build panics.
- **A fixture environment needs no Python.** ty accepted a `pyvenv.cfg` whose `home` does not exist, so the tiny environments of D9 are plain files.
