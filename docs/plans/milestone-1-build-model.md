# Implementation plan: milestone 1, the build model

**Status:** proposed, 2026-11-17. **Milestone:** 1 of the [specification](../specification.md#142-milestones). **Specifies:** [stage 1](../specification.md#5-stage-1-the-build-model). **Rules it follows:** the [guidelines](../guidelines/README.md) and the [testing playbook](../guidelines/testing.md). **Format rules:** [specification, section 7](../specification.md#7-the-model).

## 1. Goal

The first thing that runs: `mentalc build` reads a repository's build files and writes the build model.

**Done when:**

1. `mentalc build` in `examples/report-jobs` writes, under `.mental/model/`, records equal to the hand-written reference in `examples/report-jobs/model/` ([T6](../guidelines/testing.md#t6-the-golden-is-never-a-snapshot)):
   - 1 build unit, `reports/unit:app`, with the root `src/` marked as declared;
   - 11 pins, one per line of `requirements.lock`;
   - 3 artifacts: the built image, `redis:7.4`, `postgres:16`;
   - 5 launches: `api`, `worker` (2 replicas), `beat`, `redis`, `postgres`;
   - 12 configuration inputs.
2. It reports exactly three gaps, each with its fix, and exits with code 1:
   - `dockerfile-missing`: the compose file says `build: .` and `src/` has no Dockerfile;
   - `python-version-unknown`: nothing states the Python version;
   - `environment-missing`: no environment to resolve references against, with the command that makes one.
3. Building twice gives the same bytes.
4. Every gate of the guidelines is green on the new crates.

It is the first step because every later stage is set up from its output, it needs no parser, checker, pack or LLM, and it exercises the whole path once: read files, write records, report diagnostics, exit codes.

## 2. Scope

| In | Out (and where it goes) |
|---|---|
| The workspace and three crates: `model`, `extract`, `mentalc` | `mentalvm` (milestone 6) |
| In `model`: semantic IDs, slugs, diagnostics, canonical JSONL, the five build record kinds, anchors | the other record kinds (each arrives with the milestone that first writes it); guards and the partition check (milestone 4) |
| In `extract`: the readers of stage 1 and the function that assembles a build model from them | parsing Python (milestone 2) |
| In `mentalc`: reading the working tree, running `docker compose config`, `mentalc build`, `--json`, exit codes | `mentalc init`, the gate, the LLM loop, MCP |
| Readers: compose JSON, `pyproject.toml`, `requirements` files, `uv.lock`, `pylock.toml`, `.python-version`, Dockerfile, an environment's `pyvenv.cfg` | Poetry and PDM locks, Kubernetes manifests, Procfiles |
| `mentalc.toml`: `name`, roots, and the environment path per unit | acceptances (milestone 2) |

## 3. Decisions to confirm

These are this plan's proposals. Each is written into the specification when confirmed.

| # | Question | Proposal | Why |
|---|---|---|---|
| D1 | Where do the readers live? | In `extract`, as a module `build/`. They take file contents as strings and return records and diagnostics. `mentalc` does all I/O | keeps the accepted rule that `extract` does no I/O; every reader is testable from a string |
| D2 | How do tests avoid needing Docker? | The compose reader takes the JSON that `docker compose config` printed. Tests use stored JSON. One CI job with Docker re-renders the stored files and fails on a difference | tests stay fast and hermetic; drift in Docker's output is still caught |
| D3 | How does a launch command name its module? | Launcher rows, as pack data: `uvicorn <module>:<attr>`, `celery -A <module> …`, `python -m <module>`, `gunicorn <module>:<attr>`. No launcher name appears in Rust | the accepted rule that library names live in packs; root inference needs the module a command names |
| D4 | Is a configuration input per launch? | Yes: `reports/launch:worker/cfg:database-url`. The same variable set on three launches is three records | two launches can set one variable differently; Load needs the value per launch |
| D5 | Are values stored? | Yes, as written in the repository's files, with interpolation off. A reference to a shell variable stays a reference and is marked unset | Link needs hosts and names (`DATABASE_URL`); nothing from the developer's shell enters the model |
| D6 | In which order is the Python version taken? | `mentalc.toml`; the Dockerfile's base image; `.python-version`; the lower bound of `requires-python`; the environment's `pyvenv.cfg`. The record says which source won | what runs in production ranks above what a developer has installed |
| D7 | Which compose file? | The one named in `mentalc.toml`; else the single `compose.yaml`, `compose.yml`, `docker-compose.yaml` or `docker-compose.yml` nearest the root. Two candidates at the same depth are an error naming both | no guessing |
| D8 | What is the system's slug? | `name` in `mentalc.toml`, else the compose `name`, else the directory name. report-jobs gets a `mentalc.toml` with `name = "reports"` | IDs in every document already say `reports/…` |
| D9 | Does a gap stop later stages? | In this milestone there are none. The build writes the records it can, reports every gap, and exits 1 if any gap is an error | the gate arrives with milestone 2 |

## 4. Records

The five kinds, with IDs as in the [specification](../specification.md#72-semantic-ids). Field names are fixed by the format snapshot in step 4.

| Kind | Fields |
|---|---|
| `build_unit` | `id`; `dir`; `roots`: each a path and how it is known (`configured`, `declared`, `convention`); `python`: version and its source; `lock`: path and format; `environment`: path; `uses`: other units |
| `pin` | `id`; `package` (normalized name); `version`; `origin`: registry, a git commit, or a path |
| `artifact` | `id`; `form`: `image` or `wheel`; `origin`: `built` or `outside`; for a built image: `unit`, `context`, `dockerfile`, `base`, `unread` (build steps not read); for an outside image: `ref` |
| `launch` | `id`; `artifact`; `command` as a list; `module` and `attr` when a launcher row matches; `workdir`; `replicas`; `publishes` (ports); `after` (launches it is started after) |
| `config_input` | `id`; `name`; `value` or `unset`; `source`: the compose file, `.env`, or the Dockerfile |

Every record has an anchor to the file and lines it came from, and `source: extractor:build`.

## 5. Diagnostics

Codes are a closed enum, written in kebab-case. A diagnostic has a code, the record's ID, a message, related IDs and an anchor; severity is fixed per code; output is sorted and never stops at the first error. Each diagnostic names the record, the file, and the fix.

| Code | Severity | Means | Fix text |
|---|---|---|---|
| `dockerfile-missing` | error | a build context has no Dockerfile | add one, or name it in the compose file |
| `python-version-unknown` | error | no source of D6 states a version | the cheapest source: a `.python-version` file |
| `environment-missing` | error | a unit has no environment | the command for its lock format, for example `uv venv && uv pip install -r requirements.lock`; `mentalc` does not run it |
| `lock-missing` | error | a unit has dependencies and no lockfile | the lock command of its tool |
| `lock-unpinned` | error | a requirements line is a range, an include or an option | pin it, or export a `pylock.toml` |
| `lock-version-unknown` | error | a lockfile's schema version is not one this build reads | the versions it does read |
| `compose-ambiguous` | error | D7 found two candidates | name one in `mentalc.toml` |
| `root-ambiguous` | error | a launch's module is found under two roots | list the roots in `mentalc.toml` |
| `root-by-convention` | warning | no root was declared; `.` or `src/` was assumed | declare it |
| `build-step-unread` | warning | `setup.py`, a `RUN` line or a build script exists and was not read | — |
| `config-unset` | warning | a compose value refers to a variable that neither the file nor `.env` sets | set it in `.env` |

A failure of `docker compose config` itself (a non-zero exit, or no answer within its time limit of 60 s, after which it is killed; [C26](../guidelines/mentalc.md#c26-the-one-process-has-a-time-limit)), or an unreadable file, is a tool failure: exit code 2, no diagnostics file.

## 6. Layout and size

```
Cargo.toml            the workspace
model/      src/  id.rs  diagnostic.rs  jsonl.rs  record.rs  record/build.rs  anchor.rs  testkit.rs
extract/    src/  lib.rs
                  build.rs            assemble(inputs) -> (BuildModel, Vec<Diagnostic>)
                  build/compose.rs    compose JSON -> launches, artifacts, configuration inputs
                  build/pyproject.rs  pyproject.toml -> units, requires-python, workspace members, path dependencies
                  build/lock.rs       requirements, uv.lock, pylock.toml -> pins
                  build/docker.rs     Dockerfile -> base image, workdir, default command, unread steps
                  build/python.rs     .python-version, pyvenv.cfg, a base image tag -> a version
                  build/roots.rs      units + launches + file list -> source roots
                  pack.rs             launcher rows, from a string
mentalc/    src/  main.rs  cli.rs     [edge: stdio]
                  lib.rs              run(cli) -> ExitStatus
                  repo.rs             [edge: file system, docker compose]
                  build.rs            snapshot -> model files as bytes
            packs/launchers.toml      embedded with include_str!
examples/report-jobs/mentalc.toml     name = "reports"
examples/report-jobs/model/           the reference: the five build kinds
```

`repo.rs` is the only file that touches the disk or runs a process ([G4](../guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)): it reads the inputs, runs `docker compose config` and writes `.mental/model/`. `build.rs` calls `extract::build::assemble` and returns the files to write, so it has no I/O. `lib.rs` holds `run` so `main.rs` stays thin ([G12](../guidelines/README.md#g12-exit-codes-and-a-thin-main)). The launcher pack lives in `mentalc/packs/` with every other pack, and `extract::pack` parses the string `mentalc` passes in.

Dependencies: `model` has `serde` and `serde_json`; `extract` adds `toml`; `mentalc` adds `clap` and `anyhow`. Budget: about 1,600 lines of non-test code.

`repo.rs` reads only files whose names the readers ask for, skips `.git`, `.venv`, `node_modules` and `.mental`, and never follows a link out of the repository.

## 7. Steps

Each step is one pull request that leaves every gate green, with its tests in the same change.

| # | Step | Ends with |
|---|---|---|
| 0 | The workspace, generated by alpha, with the three crates empty; lint settings; the dependency allowlist test; a one-hour spike on whether branch coverage counts the panic side of an assertion as an untaken branch (if it does, invariants become typed errors where the code can recover) | empty crates passing every gate |
| 1 | `model::id`: `Slug`, `slugify`, `Id`, and the kinds this milestone needs (root, unit, pin, artifact, launch, configuration input) | ID tables and properties green |
| 2 | `model::diagnostic`: `Diagnostic`, `Severity`, `Code` with the codes of section 5; the completeness test that every code has a fixture | — |
| 3 | `model::jsonl`: canonical read and write, `format.json`, strict reading; `anchor` | round-trip properties |
| 4 | `model::record::build`: the five kinds; the report-jobs reference for them, written by hand in `examples/report-jobs/model/` from section 1 and the example's files | the format snapshot, which freezes the fields of section 4; the reference passes `check` |
| 5 | `extract::build::compose` and `pack` with launcher rows (`mentalc/packs/launchers.toml`) | the stored JSON of report-jobs gives 5 launches, 3 artifacts, 12 configuration inputs; fixtures for profiles, `env_file`, a variable reference, `replicas` |
| 6 | `extract::build::lock` and `pyproject` | fixtures per format and per error code; a uv workspace with two members and a path dependency |
| 7 | `extract::build::docker` and `python` | fixtures: a multi-stage file (the last stage counts), `CMD` in both forms, a `RUN` line listed as unread |
| 8 | `extract::build::roots` and `assemble` | report-jobs as strings in, the records and three gaps of section 1 out |
| 9 | `mentalc`: `lib.rs`, `repo.rs`, `build`, `--json`, exit codes; `examples/report-jobs/mentalc.toml` | the acceptance of section 1, compared with the reference of step 4 and run against the real directory with the stored compose JSON, and in the Docker CI job against the real command |
| 10 | Measure: run `mentalc build` on full-stack-fastapi-template, Mealie, Redash and rq; compare launches with `docker compose config` and pins with `uv export --frozen`; record the differences and the gaps as findings | a findings entry; reader fixes go back through steps 5 to 8 |

## 8. Tests

| What | How |
|---|---|
| IDs, diagnostics, JSONL | accept and reject tables with exact messages; a property that any accepted ID prints back unchanged; `write(load(write(m))) == write(m)`; a test per strictness rule (extra field, `null`, `[]`, wrong version); one fixture per diagnostic code and a test that fails if a code has none |
| Each reader | a table of small inputs to expected records, through one `check(input, expected)` helper; one fixture per error code; arbitrary bytes give a diagnostic, never a panic |
| Assembly | report-jobs, plus fixtures for: no compose file, two units, a root by convention, a module under two roots |
| The command | few snapshots: the text output and the `--json` output on report-jobs, and the exit code in each of the three cases (clean, gaps, tool failure) |
| Determinism | two builds of one input are byte-equal |

## 9. Risks

- **Docker's JSON changes shape between versions.** The reader reads only the fields it uses and rejects a shape it does not know with a clear error naming the Docker Compose version. The Docker CI job catches drift.
- **Root inference is the one judgment in this milestone.** It is confined to `roots.rs`, every root says how it is known, and step 10 measures it on four real projects before milestone 2 relies on it.
- **Lock formats without a specification** (`uv.lock`). Read by schema version; an unknown version is an error, never a best effort (F-U10).
- **report-jobs fails its own build.** After this milestone the example has three gaps by design. Before milestone 2 the owner decides how they are closed: a Dockerfile with a pinned base image closes two, and an environment made by the printed command closes the third. Adding a Dockerfile changes no line of the application and none of the four bugs.

## 10. After this milestone

Milestone 2 sets the type checker up from these records: per build unit, its roots, Python version and environment. Its plan is written when this one is done, and starts with a spike on running ty's crates over a read-only file source supplied by `mentalc`.
