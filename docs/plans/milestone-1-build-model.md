# Implementation plan: milestone 1, resolve a simple project

**Status:** proposed, 2026-11-18. **Specifies:** [stage 1](../specification.md#5-stage-1-the-build-model) without its Docker readers, and the first half of [stage 2](../specification.md#6-stage-2-from-code-to-facts): parse, list sites, resolve. **Rules it follows:** the [guidelines](../guidelines/README.md) and the [testing playbook](../guidelines/testing.md). **Changes to the specification it needs:** [section 9](#9-what-changes-in-the-specification).

## 1. Goal

`mentalc build` takes a plain Python project, finds its dependencies, parses every file and says what every reference points to. There is no Docker, no compose file, no outside process, no pack and no LLM.

**Done when:**

1. `mentalc build` in `examples/report-jobs`, with no environment on disk, writes 1 build unit (`reports/unit:app`) and 11 pins equal to the hand-written reference in `examples/report-jobs/model/`, reports the one gap `environment-missing` with the command that makes the environment, and exits 1.
2. With the environment made by that command, it parses the five files, gives every reference site exactly one outcome, resolves every `report.status = …` write receiver to `Report`, writes the resolution report, and exits 0.
3. In a copy with one package removed from the environment, it reports `module-missing` for that package, names the fix, and exits 1.
4. Building twice gives the same bytes.
5. Every gate of the guidelines is green on the three crates.

## 2. Scope

| In | Out (and where it goes) |
|---|---|
| In `model`: semantic IDs, slugs, diagnostics, canonical JSONL, anchors, three record kinds: `build_unit`, `pin`, `site` | every other record kind; guards |
| In `extract`: readers for `pyproject.toml`, requirements files with exact pins, `uv.lock`, `.python-version`, an environment's `pyvenv.cfg`; parse with ruff; list sites; resolve with ty | `pylock.toml`, Poetry and PDM locks; obligations, effects, entry points (the facts milestone) |
| In `mentalc`: `mentalc.toml`, reading the working tree, `mentalc build`, `--json`, exit codes, the resolution report | `mentalc init`, acceptances, the reachability part of the gate (it needs entry points), the LLM loop, MCP |
| | Dockerfiles, compose files, artifacts, launches, configuration inputs: not in this plan; see [section 9](#9-what-changes-in-the-specification) |

## 3. Decisions to confirm

| # | Question | Proposal | Why |
|---|---|---|---|
| D1 | What is a build unit? | A directory with a `pyproject.toml` (slug from its project name), or one declared in `mentalc.toml` as `[unit.<slug>]` with `dir`. Nothing else is searched for | report-jobs has no `pyproject.toml`; a declared unit is one table, and nothing is guessed |
| D2 | Where do roots come from? | `roots` in `mentalc.toml`; else the package location in `pyproject.toml`; else `.` and `src/` when they hold Python files, with the warning `root-by-convention` | the compose build context is gone as a source |
| D3 | In which order is the Python version taken? | `mentalc.toml`; `.python-version`; the environment's `pyvenv.cfg`. The record says which source won. `requires-python` is recorded and never used as the version | a range is not a version; the environment is what references resolve against |
| D4 | How are tests hermetic without installed packages? | Resolver tests use tiny environments checked in as fixtures: a `pyvenv.cfg` and a few stub packages. One CI job makes the real report-jobs environment with `uv` and runs the acceptance of goal 2 and 3; locally it runs when `RUN_SLOW_TESTS` is set | fast tests with no network; the real path is still run on every pull request |
| D5 | Who reads files for ty? | ty reads the roots and the environment from disk itself, read-only, in `extract/src/python/resolve.rs`, as one named edge with a row in the Exceptions table | it is the setup the experiment measured; feeding ty an in-memory copy of site-packages is work with no user-visible gain |
| D6 | How is a site identified? | `<unit>/site:<module>-<function>-<kind>-<name>`, with the site's order among equal ones in its function appended when two collide; never a line number | the rule the specification already uses for moves |
| D7 | Which sites? | import, decorator, callee, base class, write receiver. Escapes and handler types arrive with the facts milestone | these five are what the experiment measured, so the numbers can be compared |
| D8 | Is every site in the hand-written reference? | No. The reference holds the unit and the pins. Sites are checked by the conservation oracle ([X8](../guidelines/extract.md#x8-conservation)), by a table of counts per kind and outcome, and by named assertions (the write receivers) | about 190 site records by hand would be copied from the tool's output, which is a snapshot, not a specification |
| D9 | What fails the build? | A missing input (`environment-missing`, `python-version-unknown`, `lock-missing`) and a missing module. Every other unresolved site is listed in the resolution report and fails nothing yet | which gaps count needs entry points, which do not exist yet |

## 4. Records

| Kind | Fields |
|---|---|
| `build_unit` | `id`; `dir`; `roots`: each a path and how it is known (`configured`, `declared`, `convention`); `python`: version and its source; `requires_python`; `lock`: path and format; `environment`: path |
| `pin` | `id`; `package` (normalized name); `version`; `marker` when the lock gives one; `origin`: registry, a git commit, or a path |
| `site` | `id`; `kind`; `outcome`: `resolved` with a definition (module and qualified name), `external` with a package and an access path, or `unresolved` with a reason and, when another site is its cause, that site; `checker`: name and version |

Every record has an anchor (file, lines, quote) in `anchors.jsonl`, and a `source`.

## 5. Diagnostics

| Code | Severity | Means | Fix text |
|---|---|---|---|
| `python-version-unknown` | error | no source of D3 states a version | add a `.python-version` file |
| `environment-missing` | error | a unit has no environment | the command for its lock format, for example `uv venv && uv pip install -r requirements.lock`; `mentalc` does not run it |
| `lock-missing` | error | a unit has no lockfile | the lock command of its tool |
| `lock-unpinned` | error | a requirements line is a range or an include. Hashes and markers are read | pin it |
| `lock-version-unknown` | error | a `uv.lock` schema version this build does not read | the versions it does read |
| `module-missing` | error | an import names a module that is neither under a root nor in the environment | install the package, or add the root |
| `root-by-convention` | warning | no root was configured or declared | declare it |
| `syntax-error` | warning | a file does not parse; the recovered tree is used | — |

An unreadable file or a bad `mentalc.toml` is a tool failure: exit code 2.

## 6. Layout

```text
model/    src/  id.rs  diagnostic.rs  jsonl.rs  anchor.rs  record.rs  record/build.rs  record/site.rs  testkit.rs
extract/  src/  lib.rs
                build.rs  build/pyproject.rs  build/lock.rs  build/python.rs    strings in, records and diagnostics out
                python.rs  python/parse.rs  python/sites.rs                     one file in, sites out
                python/resolve.rs                                               sites + unit -> outcomes [edge: ty reads]
mentalc/  src/  main.rs  cli.rs  lib.rs  config.rs  repo.rs  build.rs
examples/report-jobs/mentalc.toml   name = "reports"; [unit.app] dir = "src", roots = ["."], python = "3.12"
examples/report-jobs/model/         the reference: build_unit and pin
```

Dependencies: `model` has `serde` and `serde_json`; `extract` adds `toml`, `thiserror` and the ruff and ty crates at the commit the experiment used; `mentalc` adds `clap` and `anyhow`. Dev: `insta`, `proptest`, `insta-cmd`, `tempfile`.

## 7. Steps

Each step is one pull request that leaves every gate green, with its tests in the same change.

| # | Step | Ends with |
|---|---|---|
| 0 | Two spikes, before any production code. Coverage: does branch coverage count the panic side of an assertion. ty in the workspace: add the pinned crates to `extract`, move the [experiment's harness](../research/type-checkers-as-libraries/experiments/tyharness/) setup behind one function, and run every gate (lockfile allowlist, vulnerability scan, build time) | both answers written into the guidelines; a go or no-go on ty in process |
| 1 | `model`: `id`, `diagnostic`, `jsonl`, `anchor` | ID tables, round-trip properties |
| 2 | `model::record`: `build_unit`, `pin`; the report-jobs reference written by hand | the format snapshot; the fields written into the specification |
| 3 | `extract::build`: the readers and the function that assembles a unit | fixtures per format and per error code; a requirements file with hashes and markers; a `uv.lock` |
| 4 | `mentalc`: `config`, `repo`, `build`, `--json`, exit codes | goal 1: report-jobs writes its unit and 11 pins and reports `environment-missing` |
| 5 | `extract::python`: `parse` and `sites`; the `site` record | the conservation oracle green on every fixture and on report-jobs |
| 6 | `extract::python::resolve` | every outcome and every reason reached by a fixture; the write receivers of report-jobs resolve to `Report` |
| 7 | `mentalc`: stage 2 in the build, the resolution report, the CI job that makes the environment | goals 2, 3 and 4 |
| 8 | Measure on rq and full-stack-fastapi-template at the experiment's commits; compare resolved rates with the experiment's (88.5% of calls over five projects) | a findings entry; differences explained or fixed |

## 8. Risks

- **ty's API is unstable and partly private.** The experiment already works around it (`Self` and narrowed types). Step 0 decides before anything is built on it; the fallback is Pyrefly's export.
- **ty's dependency tree** (242 crates) meets the lockfile allowlist and the vulnerability scan for the first time in step 0.
- **Site identity (D6) is new.** Two sites with the same module, function, kind and name are common (`s.commit()` twice); the order suffix makes IDs change when a call is inserted before them. Step 5 measures how many sites need it.
- **report-jobs fails its own build until its environment exists.** That is goal 1, by design.

## 9. What changes in the specification

Not applied yet. The specification and the other documents still describe the earlier design; each change below is written into them only when the owner confirms it.

- **Stage 1 shrinks to build units and pins.** The Dockerfile and compose readers, the one outside process and its time limit, the records `artifact`, `launch` and `config_input`, and replica counts leave version one.
- **Milestones 1 and 2 merge** into this one. Requirements R1 and R2 become one, with the acceptance of [section 1](#1-goal).
