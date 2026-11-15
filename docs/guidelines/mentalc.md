# `mentalc`: guidelines

**Status: Accepted by the owner on 2026-11-14.** Distilled from [mentalc_architecture.md](../research/implementation-guidelines/notes/mentalc_architecture.md) and [mentalc_llm_loop_mcp.md](../research/implementation-guidelines/notes/mentalc_llm_loop_mcp.md), with parts of [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md). The cross-cutting rules in [README.md](README.md) and [testing.md](testing.md) apply too.

## One job

`mentalc` compiles one repository at one commit into a complete model (spec 6). It owns:
- the pipeline of spec 6.2, in one function;
- tasks, prompts and answer types;
- the LLM backends and the task protocol (`task next`, `task submit`);
- the checks of spec 6.7 that need no interpreter, and repairs;
- the answer cache and the call ledger;
- the CLI and the MCP stdio server;
- the packs and prompts, as data embedded in the binary.

**It never:** runs the model (it does not depend on `mentalvm`); writes a partial model; extracts facts itself (that is `extract`); lets an LLM drop an obligation, decide feasibility or invent an ID (spec 6.4); calls a live LLM in a test.

## Module layout

```
mentalc/
  src/
    main.rs           thin: parse args, call run, report the error chain           [edge: stdio, exit]
    lib.rs            pub fn run(cli: Cli) -> anyhow::Result<ExitStatus>
    cli.rs            clap derive; human and JSON rendering                         [edge: stdout]
    config.rs         mentalc.toml -> Config (role -> backend and LLM, repair cap)
    repo.rs           RepoSnapshot: reads files, runs `docker compose config`       [edge: fs, process]
    build.rs          the pipeline: one function, phases in spec order
    phase/            detect.rs vocabulary.rs group.rs compile.rs mechanisms.rs
                      link.rs load.rs close.rs levels.rs  (each a pure function; LLM phases have plan and apply)
    task.rs           Task, RenderedTask, next_task and accept (the build state machine)
    answer.rs         answer types (deny_unknown_fields) and schema derivation
    prompt.rs         embedded templates, rendering, prompts.lock check
    check/            coverage.rs guard.rs composition.rs agreement.rs machine.rs atomicity.rs
    repair.rs         failure list -> repair task
    cache.rs          content-addressed answer store                                [edge: fs]
    ledger.rs         calls.jsonl and raw answers by hash                           [edge: fs]
    backend.rs        trait Backend
    backend/          api.rs agent_cli.rs replay.rs                                 [edge: network, process]
    mcp.rs            serve(reader: impl BufRead, writer: impl Write)
  packs/              hand-written TOML packs (spec 7), embedded with include_str!
  prompts/            <role>/<name>.md templates, embedded with include_str!
  prompts.lock        template -> (version, blake3 hash)
  tests/              cli.rs  mcp.rs  report_jobs.rs  fixtures/<repo>/answers/
```

Structural and anchor checks come from `model` ([M12](model.md#m12-checking-is-one-pure-pass-that-never-stops-early)); phase 2 is `extract::extract`.

## Dependency allowlist

| Kind | Crates |
|---|---|
| workspace | `model`, `extract` |
| normal | `serde`, `serde_json`, `toml` (config, packs), `blake3` (cache keys, prompt hashes), `clap` (derive), `thiserror` (typed errors of cache, ledger, backends), `anyhow` (context at the binary layer), `ureq` (blocking HTTP with rustls, `api` backend), `schemars` (answer schemas) |
| dev | `insta` (with `json`, `filters`, `redactions`), `insta-cmd`, `proptest`, `tempfile`; `model` with the `testkit` feature |

Explicitly not: `tokio` or any async runtime, `rmcp`, `reqwest`, `jsonschema`, `salsa`, `comemo`, `mockall`, `assert_cmd`, `wiremock` ([G16](README.md#g16-banned-weight)).

## Rules

### Pipeline

#### C1. The pipeline is one function
- **Rule:** `build()` calls the phases of spec 6.2 in order as plain function calls over typed values; there is no `Phase` trait, registry or plugin system.
- **Why:** the data flow is visible in about 40 lines and matches the spec's Reads and Writes columns.
- **Source:** [typst `compile_impl`][typst-compile]; [jaq's explicit pipeline][jaq-lib]; [rust-analyzer "functions over objects"][ra-style-fn].
- **Enforced by:** reviewer; a test that asserts the phase order reported by `build --json`.

#### C2. Phases are pure and take only what they read
- **Rule:** every phase is a free function `fn phase(inputs: &...) -> (Output, Vec<Diagnostic>)` whose parameters are exactly its "Reads" column; the repository arrives as an in-memory `RepoSnapshot` loaded once by `repo.rs`. No phase receives a context object holding everything.
- **Why:** each phase can be tested alone from a fixture string, and its dependencies are visible in its signature.
- **Source:** [rust-analyzer ground state, architecture.md L25-L33][ra-arch-ground]; [typst `World`][typst-world].
- **Enforced by:** clippy `disallowed-methods` ([G4](README.md#g4-io-and-ambient-inputs-only-at-named-edges)); reviewer.

#### C3. LLM phases split into plan and apply
- **Rule:** an LLM phase is two pure functions, `plan(inputs) -> Vec<Task>` and `apply(inputs, &Answers) -> (Output, Vec<Diagnostic>)`; only the driver between them looks up the cache and asks the backend. Bottom-up compilation iterates plan and apply level by level along the call graph.
- **Why:** LLM steps stay snapshot-testable with canned answers, and no async code is needed.
- **Source:** derived from [rust-analyzer's pure core][ra-arch-core]; no studied project has an LLM inside its pipeline.
- **Enforced by:** G4's lint keeps backends out of phases; reviewer.

#### C4. One judge for every backend
- **Rule:** the LLM core is a state machine with two operations, `next_task() -> Option<Task>` and `accept(task_id, raw: &str) -> Verdict` (`Accepted | Repair(Task) | Failed(Error)`). All checking, repair, caching and ledger writes happen inside `accept`; backends never call them. Commands are stateless: `task next` and `task submit` rerun the pipeline from the repository plus `.mental/` until the first cache miss, so no session file exists.
- **Why:** the pull backend inverts control, every backend must be judged identically (spec 6.5), and nothing can drift between calls.
- **Source:** [rust-analyzer "stateless a-la HTTP", architecture.md L241-L245][ra-arch-stateless]; [SysMoBench in-memory attempt counter][smb-server] (counterexample).
- **Enforced by:** a test that every backend goes through `accept`; the pull-protocol property test ([C23](#c23-the-pull-protocol-is-tested-at-three-layers)).

### Backends

#### C5. The backend is synchronous
- **Rule:** `trait Backend { fn caps(&self) -> Caps; fn complete(&self, task: &RenderedTask) -> Result<RawAnswer, BackendError>; }`, implemented only by the push backends (`api`, `local`, `agent-cli`, `replay`). The `agent` backend implements nothing: it is the CLI and MCP surface over `next_task` and `accept`.
- **Why:** 15 calls per build need no concurrency, and an async runtime would add dozens of crates. Accepted and written into spec 6.5 ([accepted changes](README.md#accepted-specification-changes)).
- **Source:** [rmcp `Cargo.toml`, tokio mandatory][rmcp-cargo]; spec 6.4 cost estimate (F-S10).
- **Enforced by:** tidy tests `banned_crates` and `no_async`.

#### C6. A small blocking API client
- **Rule:** the `api` backend sends the raw Messages API through `ureq`: model IDs come from config, never from code; `temperature`, `top_p` and `top_k` are never sent; only 408, 409, 429, 5xx, 529 and transport errors are retried, with exponential backoff, jitter, `retry-after` and a maximum attempt count; `stop_reason` `refusal` or `max_tokens` is a `BackendError`, never a repair; a missing key is an error that names the variable to set.
- **Why:** there is no official Rust SDK, and clust shows a typed wrapper's bloat and staleness; repairing a refusal wastes calls; sampling parameters are rejected by the current API version.
- **Source:** [Codex retry policy][codex-retry]; [clust's hard-coded model enum][clust-model]; Anthropic claude-api reference (`shared/tool-use-concepts.md`, `shared/error-codes.md`).
- **Enforced by:** insta snapshot of the serialized request; unit tests of response parsing on recorded bodies (success, refusal, `max_tokens`, 429 with `retry-after`, 529); unit tests of the retry classification as a pure function.

### Answers and prompts

#### C7. Answer types are the schema and the validator
- **Rule:** answer types are Rust structs with `#[serde(deny_unknown_fields)]`, and deserialization is the structural validator. The JSON Schema sent to the LLM is derived from the same types with schemars, then passed through one `Transform` that rejects recursion, numeric and length keywords and objects without `additionalProperties: false`, and rewrites `oneOf` to `anyOf`. Constraints a schema can't express are checks. No JSON Schema validator crate.
- **Why:** one source of truth for shape; the Anthropic structured-output subset rejects recursion and constraint keywords; a validator crate costs 64 crates for what serde already does.
- **Source:** [schemars `schema_exprs.rs`][schemars-exprs]; Anthropic claude-api reference, structured outputs.
- **Enforced by:** test over every role's schema (the subset check passes, an insta snapshot exists); property test that random JSON never panics the deserializer and an unknown field is always rejected.

#### C8. Prompts are versioned data
- **Rule:** templates live in `prompts/<role>/<name>.md`, are embedded with `include_str!` and declare a version; `prompts.lock` pins `(version, blake3)` per template.
- **Why:** prompts are reviewable data, and the version in the ledger must mean something.
- **Source:** [Codex `prompts/src/compact.rs`][codex-prompts]; spec 6.6.
- **Enforced by:** unit test that fails when a template's hash changes without a version bump or when the lock is stale; insta snapshot of each template rendered on a fixture.

### Checks and repair

#### C9. Checks are pure and re-run on every cache hit
- **Rule:** each check of spec 6.7 is a pure function over the model returning diagnostics; checks run on fresh answers and again on every cache hit.
- **Why:** a stricter check then turns exactly the now-failing stored answers into repairs, so the check version needs no place in the cache key.
- **Source:** [ruff caches only clean results][ruff-diagnostics]; the LLM note, section 4.
- **Enforced by:** test that a cached answer that now fails a check yields a repair, not acceptance.

#### C10. A repair names every failure
- **Rule:** a repair task carries the original inputs, the previous raw answer and, per failure, the check, record ID, rule, evidence (for an anchor: the quote, the file and the closest actual lines) and what passed. It asks for the whole answer again under the same schema.
- **Why:** the model can fix only what it is shown; global checks (coverage, guard algebra) need a whole answer to re-check uniformly.
- **Source:** [Aider's failed-block message, `editblock_coder.py` L41-L123][aider-edit].
- **Enforced by:** insta snapshot of a repair task per check kind; unit test that every failure variant renders non-empty evidence.

#### C11. Repairs are capped and the build fails closed
- **Rule:** repairs per task are capped in config (default 3); on exhaustion the build fails with every failure attached. There is no fail-open path and no "model with gaps".
- **Why:** fail rather than degrade (spec 6.5); Specula's stop gate fails open and is the counterexample.
- **Source:** [Aider `max_reflections = 3`][aider-loop]; [SysMoBench correction loop][smb-method]; [Specula `stop_gate.py`][specula-gate].
- **Enforced by:** limit tests at 2, 3 and 4 failed repairs ([T13](testing.md#t13-test-every-limit-at-its-edges)); a mutant that skips the cap must be caught.

### Cache and ledger

#### C12. The cache key is content, never the LLM
- **Rule:** `key = blake3(canonical_json({cache_format, role, template name, template hash, answer-schema hash, sorted input-slice hashes}))`. Code enters by the hash of the exact slice placed in the prompt, callee summaries by their own content hash; the LLM identity never enters.
- **Why:** unrelated edits and model changes invalidate nothing (spec 6.6), a forgotten version bump can't serve stale answers, and an unchanged callee summary stops recompilation above it (early cutoff, as Salsa's backdating and ninja's `restat` do, without revision counters).
- **Source:** [ruff `cache_key.rs`, `Hash` is not portable][ruff-cachekey]; [Salsa `backdate.rs`][salsa-backdate]; [ninja `graph.cc` L225-L262][ninja-graph].
- **Enforced by:** property tests (key independent of map order; a byte change outside every slice leaves the key unchanged, inside a slice changes it); a golden key for a fixed input as an inline snapshot.

#### C13. The cache keeps paid answers safe
- **Rule:** the cache is a hand-written store of one JSON file per accepted answer at `.mental/cache/<k[0..2]>/<k>.json`, written by temp file and rename. A corrupt entry is reported and treated as a miss; nothing is evicted by age; in `check` mode and with `--offline`, a miss exits 2.
- **Why:** an entry is a paid, non-deterministic judgment that defines the model; Salsa is 27.7k lines with persistence still a prototype and solves an in-memory problem.
- **Source:** [ruff atomic write, `cache.rs` L157-L199][ruff-cache-write]; [Salsa `Cargo.toml` and CHANGELOG][salsa-cargo]; ruff's silent reset and 30-day eviction ([`cache.rs` L100-L133][ruff-cache-evict]) are deliberately not copied.
- **Enforced by:** cache unit tests (round trip, corrupt file is a reported miss, a failure between write and rename leaves no partial file); CLI test that an offline miss exits 2.

#### C14. Every recompile is explained
- **Rule:** `build --json` lists each cache miss with the input hash that changed.
- **Why:** spec 6.8's promise ("a one-line fix recompiles exactly these units") becomes observable and testable.
- **Source:** [ninja `-d explain`, `graph.cc`][ninja-graph].
- **Enforced by:** CLI snapshot of the explanation; the one-line-fix test ([C24](#c24-the-golden-report-jobs-build)).

#### C15. The ledger is append-only and crash-safe
- **Rule:** every call, failed and repair calls included, appends exactly one complete JSON line to `calls.jsonl` (one `write_all`) before its records are used; raw answers are stored by content hash, not inline; a torn last line is detected when the ledger is opened. Repair counts are derived from the ledger (`repair_of`), not kept elsewhere.
- **Why:** audit and LLM comparison (spec 6.6), and crash safety for the pull backend.
- **Source:** [Codex rollout recorder][codex-rollout]; Aider `--llm-history-file`.
- **Enforced by:** property test with crash and reopen ([C23](#c23-the-pull-protocol-is-tested-at-three-layers)); test that the ledger line count equals the number of backend calls; each ledger line deserializes into a `deny_unknown_fields` struct.

### MCP and CLI

#### C16. A hand-written, synchronous, dual-era MCP server
- **Rule:** `mentalc mcp` is line-delimited JSON-RPC over `serve(impl BufRead, impl Write)`, sequential, returning on end of input. It answers both eras: 2025-11-25 (`initialize`, `notifications/initialized`, `ping`) and 2026-07-28 (`server/discover`, `_meta` protocol version, `resultType`, `ttlMs`, `cacheScope`); unknown methods get `-32601`, malformed lines `-32700`, bad parameters `-32602`. All MCP code and version constants live in `mcp.rs`.
- **Why:** zero added crates against rmcp's 61 and an async runtime, stable against three SDK majors in five months, testable in memory.
- **Source:** [rmcp `Cargo.toml`][rmcp-cargo] and [CHANGELOG][rmcp-changelog]; [MCP 2026-07-28 changelog][mcp-changelog]; [MCP stdio transport][mcp-stdio]; [MCP versioning][mcp-versioning].
- **Enforced by:** insta transcript tests per era built from the spec's own examples; a subprocess test of clean exit on end of input; a manual Claude Code and Codex smoke check at each release (reviewer checklist).

#### C17. A fixed tool list over shared core functions
- **Rule:** the tools are `mental_task_next`, `mental_task_submit(task_id, answer)` and optionally `mental_build_status`, in that order. The per-task answer schema travels inside the task, never as a dynamic `inputSchema`. Check failures, unknown or stale `task_id`s and exhausted repairs return a normal result with `isError: true` and the repair text; only protocol faults are JSON-RPC errors. The CLI and MCP call the same core function per operation.
- **Why:** MCP forbids `tools/list` varying as a side effect, recommends server-minted handles, and says clients feed `isError` results back to the model; sharing the core keeps both surfaces thin.
- **Source:** [MCP tools, `tools.mdx` L60-L66, L690-L785][mcp-tools]; [SysMoBench returned errors as plain text][smb-server] (counterexample).
- **Enforced by:** snapshot of `tools/list`; test that every check failure maps to `isError: true`.

#### C18. A typed CLI with two renderers
- **Rule:** the CLI is clap derive; every command has `--json`, which prints the model's own JSON (diagnostics as `model::Diagnostic` lines), and otherwise prints human text; only `cli.rs` and `main.rs` print. `mentalc.toml` is parsed once into a `deny_unknown_fields` `Config` that is passed explicitly and has no `Default`.
- **Why:** one schema for agents and scripts; ruff's 12 output formats are bloat here.
- **Source:** [ruff JSON view `render/json.rs`][ruff-json]; [rust-analyzer config rules, style.md L406-L445][ra-style-config].
- **Enforced by:** clippy `print_stdout`, `print_stderr` with `#[expect]` only in `cli.rs` and `main.rs`; CLI snapshots ([C22](#c22-cli-snapshots-are-few-and-filtered)).

### Tests (beyond [testing.md](testing.md))

#### C19. Phase tests go through one check helper with a fixture DSL
- **Rule:** each phase's `tests.rs` has one `#[track_caller] fn check(fixture: &str) -> String` that builds typed inputs from a fixture string (`//- path/to/file.py` headers for a miniature repository, `//- answers/<role>/<unit>.json` blocks for canned answers), runs only that phase and returns a stable text rendering, which each test compares with its own inline insta snapshot. Expected check diagnostics are written next to the code they anchor (`# mental: stranded-state`).
- **Why:** refactoring touches one helper, not a hundred tests.
- **Source:** [rust-analyzer fixture DSL][ra-fixture]; [rust-analyzer data-driven tests, architecture.md L416-L436][ra-arch-tests]; [typst inline annotations][typst-tests].
- **Enforced by:** reviewer.

#### C20. Tests never call a live model
- **Rule:** tests use the `replay` backend (answers looked up by cache key in `tests/fixtures/<repo>/answers/`, an error on any miss) or a scripted backend with hand-written answers, including invalid answers for every check kind (bad anchor, missing obligation, overlapping guards, unknown field, invented ID). Recording is an explicit step that needs an LLM key and fails without one.
- **Why:** sampling parameters are rejected, so recorded answers are the only source of determinism; every repair path must be covered without a model.
- **Source:** [Specula deterministic fixture adapter][specula-adapter]; [SysMoBench `ExistSpecAdapter`][smb-exist]; [Codex snapshot normalization][codex-normalize].
- **Enforced by:** CI has no API key and no network; 100% diff coverage on `check/`, `repair.rs`, `cache.rs`, `ledger.rs`.

#### C21. Backend failures are swept with a countdown
- **Rule:** a `Countdown<B: Backend>` wrapper fails call k (once, or every call from k on) for every k up to the dry-run call count, with each error kind (malformed answer, timeout, rate limit, refusal, missing key). After each run the build either failed with a clear error or succeeded after a repair, the cache and ledger are consistent, and a clean rerun gives a byte-identical model.
- **Why:** SQLite's fault-injection method, at the one external seam `mentalc` already has.
- **Source:** [SQLite testing][sqlite-testing]; [wasmtime `OomTest`][wt-oom].
- **Enforced by:** test helper `for_each_failure` ([T12](testing.md#t12-inject-faults-through-seams-that-already-exist)).

#### C22. CLI snapshots are few and filtered
- **Rule:** one `insta_cmd` snapshot per command and mode (`init`, `build`, `check`, `diff`, `task next`, `task submit`, each with and without `--json`) on a temp copy of a fixture tree, recording exit code, stdout and stderr, with filters for the temp path and the version.
- **Why:** typed messages rarely break at the protocol layer, so heavy tests stay few.
- **Source:** [ruff `CliTest`][ruff-cli]; [rust-analyzer test boundaries, architecture.md L390-L407][ra-arch-tests].
- **Enforced by:** `tests/cli.rs`.

#### C23. The pull protocol is tested at three layers
- **Rule:** (1) a property test drives `next_task` and `accept` through generated operation sequences (`Next`, `Submit(valid or invalid)`, `SubmitStale`, `SubmitTwice`, `Reopen`, `CrashMidWrite`) and checks: the final model equals an uninterrupted run's, every ledger line is well formed, each accepted task appears once, repairs never exceed the cap, a stale or duplicate submit changes nothing; (2) a few subprocess tests run `env!("CARGO_BIN_EXE_mentalc")` for `task next` and `task submit`; (3) MCP transcripts run through `serve` in memory.
- **Why:** crash and resume are where pipelines break; every command being a fresh process gives "attach after kill" by construction.
- **Source:** [Specula fault-injection e2e suite][specula-faults].
- **Enforced by:** the tests; mutation testing on `task.rs` and `mcp.rs`.

#### C24. The golden report-jobs build
- **Rule:** `mentalc build --offline` on `examples/report-jobs` with the committed answers succeeds; the produced model matches the hand-written reference at `examples/report-jobs/model/` under a written comparison rule; the four findings of spec 6.9 are asserted by `Code` and anchor; a second test applies the one-line fix and asserts that exactly the expected units recompile. The test also runs on a release build.
- **Why:** spec 6.9 and 6.8 become regression tests; the reference is a specification, so tooling never rewrites it ([T6](testing.md#t6-the-golden-is-never-a-snapshot)).
- **Source:** spec 6.9, 11.4 and F-S10; [SQLite as-delivered testing][sqlite-testing].
- **Enforced by:** `tests/report_jobs.rs`, in debug and release CI jobs.

#### C25. Golden tasks show every key change
- **Rule:** the rendered tasks for report-jobs (prompt text, input anchors and hashes, answer schema) are insta snapshots with volatile values normalized.
- **Why:** a diff in a golden task is exactly the signal that cache keys changed.
- **Source:** [Codex context snapshots with normalization][codex-normalize].
- **Enforced by:** insta in CI.

## Open questions for the owner

- **Comparison rule for C24.** The reference model is hand-written; LLM-written prose (display names, case summaries) will differ. The comparison needs a written rule, for example "same records, IDs, references, obligations and findings; prose fields excluded".
- **Optional properties in answer schemas.** It is unverified whether Anthropic strict structured outputs accept properties missing from `required`, which decides whether answer structs may use `Option<T>`.
- **Recording answers.** Re-recording after a prompt change needs a real build with an LLM; decide who runs it and how the new answers are reviewed.

[ra-arch-ground]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L25
[ra-arch-core]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L451
[ra-arch-stateless]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L241
[ra-arch-tests]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L390
[ra-style-fn]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L350
[ra-style-config]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L406
[ra-fixture]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/test-utils/src/fixture.rs#L1
[typst-compile]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst/src/lib.rs#L100
[typst-world]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/lib.rs#L63
[typst-tests]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/tests/README.md#L128
[jaq-lib]: https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/src/lib.rs#L1
[ruff-cachekey]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_cache/src/cache_key.rs#L1
[ruff-cache-write]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L157
[ruff-cache-evict]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L100
[ruff-diagnostics]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/diagnostics.rs#L365
[ruff-json]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/render/json.rs#L47
[ruff-cli]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/tests/cli/main.rs
[salsa-cargo]: https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/Cargo.toml#L20
[salsa-backdate]: https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/src/function/backdate.rs#L15
[ninja-graph]: https://github.com/ninja-build/ninja/blob/1a8fee23135e57cda12d47aedb19f795353fa13c/src/graph.cc#L225
[aider-loop]: https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/base_coder.py#L932-L944
[aider-edit]: https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/editblock_coder.py#L41-L123
[specula-gate]: https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/stop_gate.py
[specula-adapter]: https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/e2e/fixtures/single_session_adapter.py
[specula-faults]: https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/e2e/test_single_session_faults.py
[smb-method]: https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tla_eval/methods/direct_call/method.py
[smb-server]: https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tools/submit_spec/src/submit_spec/server.py
[smb-exist]: https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tla_eval/models/exist_spec_adapter.py
[codex-prompts]: https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/prompts/src/compact.rs
[codex-normalize]: https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/tests/common/context_snapshot/normalize.rs
[codex-retry]: https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/codex-client/src/retry.rs
[codex-rollout]: https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/rollout/src/recorder.rs
[clust-model]: https://github.com/mochi-neko/clust/blob/81fda7c644a6e01971c378e53f4d0afdb873a9b9/src/messages/claude_model.rs
[schemars-exprs]: https://github.com/GREsau/schemars/blob/4791bb7352555d339e45658ca09c092ed9fa1f5d/schemars_derive/src/schema_exprs.rs
[rmcp-cargo]: https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/Cargo.toml
[rmcp-changelog]: https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/CHANGELOG.md
[mcp-changelog]: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/changelog.mdx
[mcp-stdio]: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/basic/transports/stdio.mdx
[mcp-versioning]: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/basic/versioning.mdx
[mcp-tools]: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/server/tools.mdx
[sqlite-testing]: https://www.sqlite.org/testing.html
[wt-oom]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oom.rs
