# Implementation plan: milestone 1, resolve a simple project

**Status:** decisions settled with the owner and written into the specification, 2026-11-18. **Specifies:** [stage 1](../specification.md#5-stage-1-the-build-model) and the first half of [stage 2](../specification.md#6-stage-2-from-code-to-facts): parse, list sites, resolve. **Rules it follows:** the [guidelines](../guidelines/README.md) and the [testing playbook](../guidelines/testing.md).

## 1. Goal

`mentalc build` takes a standard Python project, finds its dependencies, parses every file and says what every reference points to. There is no Docker, no compose file, no outside process, no pack, no LLM and no Mental-specific project file.

A standard project is what `uv init` makes: a `pyproject.toml`, a `uv.lock`, a `.python-version` file, and a `.venv` after `uv sync`. `examples/report-jobs` becomes one.

**Done when:**

1. `mentalc build` in `examples/report-jobs`, with no `.venv`, writes 1 build unit and one pin per package in `uv.lock` other than the project itself, equal to the hand-written reference in `examples/report-jobs/model/`, reports the one gap `environment-missing` with its fix (`uv sync`), and exits 1.
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
| D5 | Which sites? | All seven kinds of the specification: import, decorator, callee, base class, write receiver, escape, handler type | complete from the start |
| D6 | What fails the build? | A missing input (`project-missing`, `lock-missing`, `python-version-unknown`, `environment-missing`) and a missing module. Every other unresolved site is listed in the resolution report and fails nothing yet | which gaps count needs entry points, which do not exist yet |
| D7 | How is a site identified? | By names: the qualified name of the enclosing scope (function, class body or module), the site's kind and the name it refers to as written. Sites are grouped by scope, kind and the name's slug without trailing digit-only segments; every site in a group of two or more gets its order in the group, counted from 0, so `s.commit_0()` cannot collide with a counter. The three `s.commit()` calls of `generate_report` are `site:app-tasks-generate-report-callee-s-commit-0` to `-2`. Never a line number | a change outside the scope changes no ID, and neither does a moved line unless it holds one of a group of identical sites; Semgrep, rust-analyzer and Infer identify things the same way |
| D8 | Is every site in the hand-written reference? | Yes: every site record of report-jobs is written by hand before the code | the reference is a specification of the whole output |
| D9 | How do tests get an environment? | Resolver tests use tiny environments checked in as fixtures: a `pyvenv.cfg` and a few stub packages. One CI job runs `uv sync` in report-jobs and runs the acceptance of goals 2 and 3; locally it runs when `RUN_SLOW_TESTS` is set | fast tests with no network; the real path still runs on every pull request |

## 4. Records

| Kind | Fields |
|---|---|
| `build_unit` | `id`; `dir`; `roots`; `python`: version and its source; `requires_python`; `lock`: path and schema version; `environment`: path |
| `pin` | `id`; `package` (normalized name); `version`; `origin`: registry, a git commit, or a path |
| `site` | `id`; `kind`; `outcome`: `resolved` with a definition (module and qualified name), `external` with a package and an access path, or `unresolved` with a reason and, when another site is its cause, that site; `checker`: name and version |

Every record has an anchor (file, lines, quote) in `anchors.jsonl`, and a `source`.

## 5. Diagnostics

| Code | Severity | Means | Fix text |
|---|---|---|---|
| `project-missing` | error | the directory has no `pyproject.toml` | add one, for example with `uv init` |
| `lock-missing` | error | the project has no `uv.lock` | `uv lock` |
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
| 0 | Two spikes, before any production code. Coverage: does branch coverage count the panic side of an assertion. ty in the workspace: add the pinned crates to `extract`, move the [experiment's harness](../research/type-checkers-as-libraries/experiments/tyharness/) setup behind one function, and run every gate (lockfile allowlist, vulnerability scan, build time) | both answers written into the guidelines; a go or no-go on ty in process |
| 1 | Done: report-jobs is a standard uv project. `pyproject.toml` names it `reports`, with Python 3.12 or later and its 11 dependencies as lower bounds (SQLAlchemy kept below 2.1, whose default PostgreSQL driver is psycopg 3, not the psycopg2 the example uses); `uv.lock` (41 packages) replaces `requirements.lock`; `.python-version` says 3.12. No Python file changed | `uv sync` works, every module of the example imports, and the lock has no known vulnerability |
| 2 | `model`: `id`, `diagnostic`, `jsonl`, `anchor` | ID tables, round-trip properties |
| 3 | `model::record`: `build_unit`, `pin`; the reference for them written by hand | the format snapshot |
| 4 | `extract::build`: the readers and the function that assembles a unit | fixtures per file and per error code |
| 5 | `mentalc`: `repo`, `build`, `--json`, exit codes | goal 1 |
| 6 | The `site` record; the site reference of report-jobs written by hand; `extract::python`: `parse` and `sites` | every call, decorator and import of the syntax tree appears exactly once, on every fixture and on report-jobs |
| 7 | `extract::python::resolve` | every outcome and every reason reached by a fixture; outcomes equal the reference |
| 8 | `mentalc`: stage 2 in the build, the resolution report, the CI job that runs `uv sync` | goals 2, 3 and 4 |
| 9 | Measure on rq and full-stack-fastapi-template at the experiment's commits; compare resolved rates with the experiment's (88.5% of calls over five projects) | a findings entry; differences explained or fixed |

## 8. Risks

- **ty's API is unstable and partly private.** The experiment already works around it. Step 0 decides before anything is built on it; the fallback is Pyrefly's export.
- **ty's dependency tree** (242 crates) meets the lockfile allowlist and the vulnerability scan for the first time in step 0.
- **Escapes and handler types are unmeasured.** The experiment covered the other five kinds; how well ty answers these two is first seen in step 7.
- **The site reference is large.** The experiment counted about 190 sites in report-jobs for five kinds. Writing each by hand, with its outcome, before the code is the biggest single piece of step 6, and its outcomes are a prediction of what ty answers.
- **Counters shift.** Inserting an identical reference earlier in a function renumbers the later ones. Step 6 counts how many sites need a counter.
- **report-jobs fails its own build until `uv sync` is run.** That is goal 1, by design.
