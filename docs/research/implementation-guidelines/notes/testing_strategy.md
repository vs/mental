# Testing strategy for Mental's four Rust crates: what the best-tested projects do

Sources were shallow-cloned and read in place (nothing built or run). Commits read:

- **SQLite**: C database whose test code is 590 times its library code. `sqlite/sqlite` @ `33fd659` (2026-11-14), plus the testing page.
- **rust-analyzer**: Rust IDE engine whose tests are data-driven text snapshots. `rust-lang/rust-analyzer` @ `114dcb8` (2026-11-13).
- **ruff**: Python linter in Rust. Uses fixture files, snapshots per rule, fuzzers and an ecosystem diff in CI. `astral-sh/ruff` @ `b140af2` (2026-11-13).
- **cargo-mutants**: Rust mutation tester. It tests itself against 51 small Rust trees with insta snapshots. `sourcefrog/cargo-mutants` @ `6c720ae` (2026-10-19).
- **proptest**: Rust property testing, with shrinking, persisted regressions and state-machine tests. `proptest-rs/proptest` @ `70cf37d` (2026-10-10).
- **toml / toml_edit**: format library with a compliance suite, round-trip proptests and a round-trip fuzzer. `toml-rs/toml` @ `563cb1a` (2026-11-06).
- **wasmtime**: Wasm runtime. Has differential fuzzing against reference interpreters and an OOM injector modeled on SQLite's. `bytecodealliance/wasmtime` @ `c52bc6d` (2026-11-14).
- **TigerBeetle**: Zig database (ideas only). Uses deterministic simulation (VOPR), assertions kept on in production, exhaustive generators and coverage marks. `tigerbeetle/tigerbeetle` @ `a2a3c70` (2026-11-10).
- **stateright**: Rust model checker that tests its own search with exact state counts. `stateright-rs/stateright` @ `ab8c8be` (2025-07-27).
- **turmoil**: Rust deterministic network and filesystem simulator. Read but mostly does not transfer, because mentalvm is itself the simulator. `tokio-rs/turmoil` @ `720d7bc` (2026-11-11).

Mental context comes from `docs/specification.md` sections 3, 4.9, 6.7, 8.2–8.9 and 11.4. Mental is a Cargo workspace with four crates: `model`, `extract`, `mentalc` and `mentalvm`. The acceptance target is `examples/report-jobs`, checked against a hand-written reference model.

## Which SQLite practices transfer to Rust, and how?

### Takeaway
Almost every SQLite practice has a cheap Rust equivalent, with three exceptions: TH3's scale, the 200-item manual release checklist, and continuous fuzzing farms.

The core that carries over:
- 100% branch coverage measured by a tool;
- countdown fault injection (fail the N-th call, raise N until the operation succeeds), run in transient and persistent modes;
- boundary tests at, below and above each limit;
- invariants asserted in code;
- a regression test before every fix;
- running the suite on the as-delivered build.

In Rust the countdown runs over a trait the product already has, the LLM backend. Mutation testing (cargo-mutants) replaces SQLite's assembly-level branch-flipping script.

### Cited findings
- SQLite's library is 155.8 KSLOC and its test code is 92,053 KSLOC, "590 times as much". The TCL suite has 51,445 distinct test cases. TH3 has 50,362, runs about 2.4 million test instances, and achieves "100% branch test coverage (and 100% MC/DC test coverage)" — [SQLite testing][sqlite-testing]
- **OOM procedure:** an instrumented malloc is "rigged to fail after a certain number of allocations". The test fails the first allocation, checks the error is handled, increments the counter and repeats until the operation completes with no failure. Each sweep runs twice: once failing one time (transient) and once failing every call after the first (persistent) — [SQLite testing][sqlite-testing]
- The injector is a countdown, `faultsimStep()`. It decrements `iCountdown`, fails once the countdown reaches zero, counts `nFail` and `nBenign`, and turns itself off after `nRepeat` failures — [sqlite src/test_malloc.c L65-86][sqlite-malloc]
- "Benign" faults are failures the code may absorb, such as a hash-table resize. They are marked with begin/end hooks so the harness knows a silent recovery is allowed there — [sqlite src/fault.c][sqlite-fault]
- **I/O errors:** a custom VFS fails after N operations, with N advanced in a loop in both modes. After the injector is turned off, `PRAGMA integrity_check` confirms nothing was corrupted. Compound tests stack one fault on top of another, for example an OOM "while trying to recover from a prior crash" — [SQLite testing][sqlite-testing]
- **`testcase(X)`** is a no-op in release builds. In coverage builds it is `if( X ){ sqlite3CoverageCounter += __LINE__; }`, so a boundary such as `a==b` must be seen both true and false. The source has 1184 uses — [sqlite src/sqliteInt.h L492-511][sqlite-int]; [SQLite testing][sqlite-testing]
- **`ALWAYS(X)` / `NEVER(X)`** wrap defensive conditions that should never fire. They assert in debug builds, are hard-coded to `1`/`0` in coverage builds so the unreachable side is not counted, and pass the value through in release builds. "All three test runs should yield exactly the same result" — [sqlite src/sqliteInt.h L547-563][sqlite-int]; [SQLite testing][sqlite-testing]
- **Mutation testing:** a script turns each branch in the assembly into an unconditional jump or a no-op and checks that some test fails. Branches that only exist for speed are tagged `/*OPTIMIZATION-IF-TRUE*/` or `/*OPTIMIZATION-IF-FALSE*/` so the script skips them — [SQLite testing][sqlite-testing]
- **Regression rule:** a bug "is not considered fixed until new test cases that would exhibit the bug have been added" — [SQLite testing][sqlite-testing]
- The core has 6754 `assert()`s. They are compiled in only with `SQLITE_DEBUG`, which makes SQLite about three times slower — [SQLite testing][sqlite-testing]
- **As-delivered testing:** the gcov-instrumented run is described as "a test of the test suite". The suite is then run again on the delivery build, and both runs must give the same output. Tests also run with optimizations disabled and must produce the same results — [SQLite testing][sqlite-testing]
- **Malformed input:** corrupt database files must produce `SQLITE_CORRUPT` and nothing worse. `fuzzcheck` replays "a few thousand 'interesting' cases" taken from billions of fuzz runs on every `make test` — [SQLite testing][sqlite-testing]
- "Static analysis has not been helpful in finding bugs in SQLite" — [SQLite testing][sqlite-testing]
- Wasmtime ports SQLite's OOM method to Rust. `OomTest` first does a dry run that counts allocations (`OomState::Counting`), then for each `i in 0..total_allocs` injects an OOM at allocation `i`. A `post_oom_func` phase then checks "that nothing was left half-registered in shared state". The module cites SpiderMonkey's `oomTest()` as its model — [wasmtime crates/fuzzing/src/oom.rs][wt-oom]
- cargo-llvm-cov: `--branch` means "Enable branch coverage. (unstable)" and "requires nightly". Code can be excluded with `#[coverage(off)]`, which should be gated by `cfg(coverage_nightly)`, a cfg the tool sets on nightly. Thresholds include `--fail-under-lines` and `--fail-uncovered-lines`. The README does not mention diff coverage. "Currently, when using `--lcov` flag, only line coverage is available on Codecov" — [cargo-llvm-cov README][llvm-cov]
- TigerBeetle keeps its assertions on in production: "Assertions downgrade catastrophic correctness bugs into liveness bugs. Assertions are a force multiplier for discovering bugs by fuzzing" — [TIGER_STYLE.md L105-150][tb-style]
- TigerBeetle also asks for "pair assertions" (check the same property at two code paths, such as before writing and after reading). It asks to assert both the positive and the negative space, and says "tests must test exhaustively, not only with valid data but also with invalid data, and as valid data becomes invalid" — [TIGER_STYLE.md][tb-style]

### Inferences: how each SQLite practice maps to Rust for Mental

| SQLite practice | Rust equivalent for Mental | Cost |
|---|---|---|
| 100% branch coverage (gcov, TH3) | `cargo llvm-cov --branch` on a pinned nightly, gated on the diff (already the alpha kit gate) | low |
| `testcase(a==b)` boundary marks | Not needed as macros. cargo-mutants already mutates `<` to `==`, `>` and `<=`, and `<=` to `>` ([cargo-mutants src/visit.rs L802-808][cm-visit]). A missing boundary test therefore shows up as a missed mutant. | free |
| `ALWAYS`/`NEVER` defensive checks | `debug_assert!` for invariants. Where code must recover in release, return a typed internal error instead of writing a "can't happen" branch. Check empirically whether llvm-cov counts the panic side of `assert!` as an untaken branch (see Gaps). | low |
| Countdown OOM / I/O injection | A countdown wrapper around the `LlmBackend` trait and the small fs facade, swept `k = 0..` until the operation succeeds, in transient and persistent modes, like wasmtime's `OomTest`. Do not inject into the allocator: Mental is not an embedded library and allocation failure aborts in Rust. | low |
| `integrity_check` after a fault | After each injected failure, assert the post-condition: the cache and ledger stay consistent and a re-run gives a byte-identical model (spec 3: "a build either produces a full model or fails"). | low |
| Crash/power-loss VFS | Only for files mentalc writes (cache, ledger, `.mental/model`): write to a temp file then rename. Test it by failing between the write and the rename. Skip a full crash simulator; it would be bloat. | low |
| Branch-flipping mutation script | `cargo mutants --in-diff`, with `exclude_re` or `#[mutants::skip]` playing the role of the OPTIMIZATION tags | low |
| A test before each bug fix | Reviewer rule plus PR template. Optionally a CI check that a `fix` commit touches a `tests.rs` or `tests/` file. | free |
| Test the as-delivered build | Run the report-jobs acceptance test with `--release`. Run the mentalvm suite (or at least the replay tests) on the WebAssembly target. | medium |
| `fuzzcheck` (replay saved fuzz finds) | Commit minimized fuzz crashes and proptest regressions as ordinary fixtures, run by `cargo test` | free |
| Bloat for a one-person project | TH3-scale suites; MC/DC (the `--mcdc` flag exists but is unstable); OSS-Fuzz or 24/7 fuzzing; 200-item manual checklists; running under several compilers and endiannesses | skip |

### Gaps
- I did not verify how rustc's branch coverage handles `assert!`/`debug_assert!` expansions, or `?` desugaring. If they count as branches, 100% branch coverage needs either a convention (invariant checks live in `debug_assert!` and coverage runs in release-with-debug-assertions-off?) or `#[coverage(off)]` helpers. Settle this with a 20-line spike before fixing the rule.
- No tool for diff branch coverage was confirmed. The cargo-llvm-cov README does not cover diff coverage, and lcov output reaches Codecov as lines only. A small script that intersects `git diff` hunks with llvm-cov JSON branch regions may be needed. Not researched.

## How do top Rust projects structure tests so they stay fast, readable and data-driven?

### Takeaway
The pattern that recurs is "a test is input text plus expected output text". Small cases are inline strings run through one `check(input, expect![..])` helper per feature. Larger cases are fixture files next to expected files, walked by one loop. The harness checks cross-cutting invariants on every fixture for free. Tests concentrate on a few stable API boundaries.

On snapshot tools: expect-test (inline, update with `UPDATE_EXPECT=1`, no extra tool) suits small outputs. insta or snapbox suit separate `.snap` files with a review tool. Either way, CI rejects orphaned snapshots.

### Cited findings
- rust-analyzer: "testing is based on *snapshot tests*: a test is a piece of input text, usually Rust code, and some output text". It uses `expect-test` plus a custom fixture DSL. Inside fixtures, `$0` marks a cursor and `// ^^^ label` annotates the line above. Expected output is regenerated with `UPDATE_EXPECT=1` — [r-a docs/book/src/contributing/testing.md][ra-testing]
- rust-analyzer's fixture mini-language writes several files and crates in one string: `//- minicore: flag1, flag2`, `// /lib.rs crate:foo deps:bar` — [r-a testing.md][ra-testing]
- "Architecture Invariant: rust-analyzer tests do not use libcore or libstd. All required library code must be a part of the tests. This ensures fast test execution" — [r-a architecture.md L412-414][ra-arch]
- rust-analyzer tests three boundaries. The outermost LSP boundary has "heavy" tests that read real files and run only when `RUN_SLOW_TESTS` is set; the project keeps them few because "in a statically typed language, it's hard to make an error in the protocol itself". Most tests sit at `ide` (a Rust API) and `hir` — [r-a architecture.md L390-410][ra-arch]
- The slow-test gate is `skip_slow_tests()`. It skips locally unless `CI` or `RUN_SLOW_TESTS` is set, and writes a cookie file when the tests do run — [r-a crates/test-utils/src/lib.rs L397-405][ra-tu]
- Style rules from rust-analyzer:
  - **Minimal Tests:** snippets "should be minimal", written as unindented raw strings, because that is "less to read", "easier to understand what exactly is tested" and gives "less time to run test".
  - **Marked Tests:** use `cov_mark::hit!/check!`, one mark per test and per condition, so each branch has "the canonical test".
  - **No `#[should_panic]`:** "explicitly check for `None`, `Err`".
  - **No `#[ignore]`:** "assert the wrong behavior and add a fixme".

  [r-a style.md L108-178][ra-style]
- These rules are enforced by a tidy test inside `cargo test`. `xtask/src/tidy.rs` has `#[test] fn test()`, which scans every `.rs` file for `#[should_panic]` without a FIXME on the line before, trailing whitespace, docs, Cargo.toml hygiene and licenses. It also matches `cov_mark::hit!` against `check!` so no mark is left untested — [r-a xtask/src/tidy.rs][ra-tidy]
- "Most of the things, including formatting, are checked by `cargo test`. If `cargo test` passes locally, that's a good sign that CI will be green" — [r-a contributing/README.md][ra-readme]
- **Data-driven directory tests:** `parse_ok` and `parse_err` loop over `TestCase::list("parser/ok")` and compare each result with `expect_file![case.rast]`. The `ok` test also asserts there are no errors and the `err` test that there are some. `panic_context` names the file that failed — [r-a crates/parser/src/tests.rs L55-78][ra-parser-tests]
- **Inline tests:** the grammar code carries `// test let_stmt` comments with a one-line program. A codegen step extracts them into `test_data/parser/inline`, so the test sits next to the branch it covers — [r-a crates/parser/src/grammar/expressions.rs L117-126][ra-expr]; [r-a xtask/src/codegen/parser_inline_tests.rs][ra-inline]
- Generated files are checked by `ensure_file_contents`. If the file differs, it rewrites it and fails, printing "run `cargo test` locally and commit the updated files" on CI — [r-a test-utils/src/lib.rs L480-510][ra-tu]
- ruff rule tests are one `#[test_case(Rule::X, Path::new("B024.py"))]` line per fixture. Each fixture under `resources/test/fixtures/<linter>/` produces one insta `.snap` file (75 in `flake8_bugbear/snapshots`) holding rendered diagnostics — [ruff crates/ruff_linter/src/rules/flake8_bugbear/mod.rs][ruff-bugbear]
- **Free invariant checks in ruff's harness:** `test_contents` applies the fixes repeatedly and panics with "Failed to converge after {} iterations" if they don't settle. It also panics if a fix leaves a syntax error that was not in the source — [ruff crates/ruff_linter/src/test.rs L244-360][ruff-test]
- ruff parser fixture tests (`datatest_stable`) run `validate_tokens` and `validate_ast` on every valid and invalid fixture. These check node ranges against token boundaries — [ruff crates/ruff_python_parser/tests/fixtures.rs][ruff-parser-fx]
- ruff CI runs `cargo insta test --all-features --unreferenced reject --test-runner nextest`, so orphaned snapshots fail the build — [ruff .github/workflows/ci.yaml L475][ruff-ci]
- ruff is moving rule tests to "mdtest": Markdown files mixing prose, Python blocks and `# error` / `# snapshot` comments. It advises "Prefer using `# error` for the bulk of your assertions because it helps to keep the tests short" — [ruff CONTRIBUTING.md L264-280][ruff-contrib]
- cargo-mutants tests itself by copying one of 51 tiny Rust trees from `testdata/`, such as `factorial`, `well_tested`, `missing_test` or `hang_when_mutated`. Manifests are named `Cargo_test.toml` so cargo ignores them. `copy_of_testdata` copies a tree so concurrent tests don't collide. The CLI output is snapshotted with insta — [cargo-mutants testdata/README.md][cm-testdata]; [cargo-mutants tests/util/mod.rs][cm-util]
- toml_edit runs the shared `toml-test` compliance corpus through `toml_test_harness::EncoderHarness`. It walks `tests/fixtures/valid/*.toml` against sibling `.json` files — [toml crates/toml_edit/tests/encoder_compliance.rs][toml-compl]
- toml_writer uses a `#[track_caller] fn t(decoded, expected)` helper that computes every encoding variant into one struct and compares it with `snapbox::assert_data_eq!`. One helper call per behavior — [toml crates/toml_writer/tests/string.rs][toml-writer]
- **Golden state counts:** stateright tests its checker by asserting exact `unique_state_count()` values (`256 * 256`, `12`, `55`) and the exact action path BFS finds. A symmetry test asserts "9 states without symmetry reduction" and 6 with it, under both DFS and BFS — [stateright src/checker/bfs.rs L442-475][sr-bfs]; [stateright src/checker/dfs.rs L560-586][sr-dfs]
- TigerBeetle's `marks.zig` ties a log line in production code to one test that hits it. The stated goal is "traceability", to "trace production code to a single minimal hand-written test which explains why the code needs to exist" — [tigerbeetle src/testing/marks.zig][tb-marks]

### Inferences: the playbook for layers, fixtures and snapshots
**Test layers and where each lives:**

| Layer | Where | What | Speed |
|---|---|---|---|
| Unit | `src/<module>/tests.rs` (`#[cfg(test)] mod tests;`) | one behavior per test through a module-local `check(input, expect)` helper | < 10 ms each |
| Crate API | `<crate>/tests/*.rs` | public behavior. For readers, a directory walk over `tests/data/{ok,err}/` | < 100 ms each |
| Property / differential | `tests.rs` (internal) or `tests/props.rs` | round-trips and reference-oracle comparisons (see later sections) | 256 cases by default |
| CLI and MCP transcript | `mentalc/tests/cli.rs` | a handful of "heavy" tests (rust-analyzer's term): run the real binary on a temp copy of a fixture tree, like cargo-mutants' `copy_of_testdata`, and snapshot stdout and the files written | gated as slow locally, always run in CI |
| Acceptance | `tests/report_jobs.rs` (workspace-level or in `mentalc`) | the whole pipeline on `examples/report-jobs` with a scripted LLM backend, compared against the hand-written reference model | under 5 s; also run in release |
| Tidy | `xtask` or `tests/tidy.rs` in one crate | source-level lint tests (rules below) | < 1 s |
| Fuzz | `fuzz/` (cargo-fuzz, its own workspace) | readers only. Saved crashes go back into `tests/data/` as fixtures | outside `cargo test` |

**Fixture conventions:**
- Inline raw strings when the input is under about 30 lines. Otherwise use a file pair `name.<ext>` plus `name.expected`.
- One concept per fixture, minimized as rust-analyzer requires.
- The layout of `tests/data/ok/` and `tests/data/err/` carries meaning: `ok` asserts no diagnostics, `err` asserts at least one. This is rust-analyzer's `parse_ok`/`parse_err` split.
- For models, a fixture is a tiny `.mental/model/` directory of sorted JSONL. Build it from a small text DSL, like rust-analyzer's multi-file fixture, so that a test reads as one string.

**Snapshot policy:**
- Use `expect-test` everywhere: inline `expect![[...]]` for small outputs, `expect_file!` for large ones. It needs no extra tool (`UPDATE_EXPECT=1`), which matches "compact, no bloat". insta only adds a review UI that a solo maintainer doesn't need.
- Never let a snapshot update the golden. The hand-written `examples/report-jobs` reference model is a golden, not a snapshot: it is compared with plain `assert_eq!` plus a readable diff, never with `expect_file!`.
- CI runs with `UPDATE_EXPECT` unset and fails on any difference. A tidy test fails on orphaned `.expected` files, the equivalent of ruff's `--unreferenced reject`.
- Snapshot only human-readable renderings: sorted JSONL, rendered diagnostics, traces one step per line. Never `Debug` output of internal structs, which churns on refactors.

**Readability:**
- One behavior per test, named after the behavior.
- A test body is `check(input, expected)`. All plumbing lives in the helper.
- Use `#[track_caller]` on helpers.
- No `#[should_panic]` and no `#[ignore]` (rust-analyzer's rules, enforced by tidy).
- Let the harness assert the global invariants (as ruff does for fix convergence and syntax validity), so individual tests stay short.

**Coverage marks (`cov_mark`):** optional. With a 100% diff branch-coverage gate and diff mutants, marks add production lines for little extra benefit. Use them only where a branch is subtle and several tests could plausibly cover it, so a reader can jump from the code to its canonical test.

### Gaps
- I did not compare expect-test and insta on CI speed or compile time. The recommendation of expect-test rests on its smaller footprint and rust-analyzer's long use of it, not on measurement.

## How to inject failures (file I/O, HTTP, LLM backend errors) without adding production complexity?

### Takeaway
Put each fault at a seam the product already needs and inject it there.
- For Mental the seam is the `LlmBackend` trait: the spec already makes the backend a setting with `agent`, `api`, local and headless variants. A `Scripted` backend replays recorded answers, and a `Countdown<B>` wrapper fails the k-th call.
- File I/O stays at the edge. A pure core works on in-memory text, with a thin read and write shell, so I/O faults come down to a few error paths at the boundary.
- mentalvm has no I/O at all. Its faults (drop, duplicate, timeout) are model-level `env` choices that the search enumerates, which is Mental's own deterministic simulation.

### Cited findings
- SQLite injects faults through pluggable interfaces it already ships: `sqlite3_config(SQLITE_CONFIG_MALLOC,...)` and the VFS. No test-only branches are added to core logic — [SQLite testing][sqlite-testing]; [sqlite src/test_malloc.c][sqlite-malloc]
- wasmtime's OOM injection is a test `GlobalAlloc` (`OomTestAllocator`) with a thread-local state machine `OutsideOomTest → Counting → OomOnAlloc → DidOom`. Production code is unchanged — [wasmtime crates/fuzzing/src/oom.rs][wt-oom]
- TigerBeetle's VOPR: "all non-deterministic parts of the system are stubbed out. This includes the clock, network, and disk operations". A seed sets the fault parameters (drop, reorder, partition, corrupt reads and writes), and "the seed and Git commit hash can be used to replay back the exact simulation" — [tigerbeetle docs/internals/vopr.md][tb-vopr]
- The simulator also hosts specific hand-written scenario tests "that are hard or slow to replicate through random simulation" (`src/vsr/replica_test.zig`) — [tigerbeetle docs/internals/vopr.md][tb-vopr]
- turmoil injects "latency, drops, partitions, crashes, torn writes" into simulated network and fs crates that replace `tokio::net` and `std::fs`, "under manual control or a seeded RNG". `FsConfig` has `block_size` for torn writes and `sync_probability` — [turmoil README.md][turmoil-readme]; [turmoil crates/turmoil-fs/src/lib.rs][turmoil-fs]
- cargo-mutants' `error_values` makes functions that return `Result` produce `Err(...)`. This flags code where "there are _only_ tests for the error cases and not for the Ok case", and the reverse — [cargo-mutants book/src/error-values.md][cm-error]

### Inferences
- **The LLM seam.** `trait LlmBackend { fn call(&mut self, task: &Task) -> Result<Answer, BackendError>; }` exists anyway (spec 6.5). Test implementations live under `#[cfg(test)]` or in a dev-only `testkit` module:
  - `Scripted`: answers keyed by task hash, loaded from a fixture file. The spec 6.6 call ledger (`calls.jsonl`) can serve as that fixture, so recorded real runs turn into deterministic tests at no extra cost.
  - `Countdown { inner, fail_at: k, mode: Once | Persistent, error }`: SQLite's procedure.
  - A sweep helper `for_each_failure(|backend| build(backend))` that counts calls in a dry run (like wasmtime's `Counting` phase), then runs `k = 0..n` in both modes. After each run it asserts that the build either failed with a clear error or succeeded after a repair. The cache, ledger and output must stay consistent, and a clean re-run must produce a byte-identical model.
- **Error variants to inject:**
  - malformed answer (bad JSON, schema mismatch);
  - an answer that fails a check, which should trigger a repair task, then exhaust the repairs and fail the build (spec 6.7);
  - timeout;
  - rate limit;
  - missing credentials, which must fail with a clear error and never degrade (the "fail rather than degrade" memory).
- **File I/O:** don't add an `Fs` trait. Keep `model::read(&str)` and `model::write(&Model) -> String` pure, with fs calls only in `mentalc`'s command layer, so I/O errors become a few `?` sites. Test those with real temp directories: a missing file, a read-only directory, and a corrupted cache entry (truncate it and check it is detected and rebuilt). This keeps production code free of test seams. An `Fs` trait would be bloat until a failure appears that a temp dir can't reproduce.
- **HTTP (`api` backend):** test the request/response mapping as pure functions (`fn to_request(&Task) -> Request`, `fn parse_response(&[u8]) -> Result<Answer>`). HTTP failures reach the build as `BackendError`s, which the countdown already covers. Don't add a mock HTTP server to the default suite.
- **MCP stdio server:** write the handler as `fn handle(state, line: &str) -> Vec<String>` and test it with transcript fixtures: request lines in, response lines expected. Add one heavy test that spawns the binary over pipes.
- **mentalvm:** faults are part of the model semantics (spec 8.2, 8.3: `drop`, `duplicate`, `timeout`, stub answers as `env` choices). No injection framework is needed. The search is the simulator, and testing it is about the correctness of the search (next sections).
- Don't use mocking crates (mockall and similar). A hand-written 20-line `Scripted` backend is easier to read and has no macros.

### Gaps
- Mental's backend trait does not exist yet. Whether `call` is sync or async changes the countdown wrapper's shape; wasmtime's has to handle both.

## How to use mutation testing so it is cheap in CI and meaningful?

### Takeaway
Run `cargo mutants --in-diff git.diff` on every PR and fail on any missed mutant. Use a `mutants` cargo profile with no debug info, `--in-place` in CI, `error_values` set for the crate's error type, and skip doctests. Skips must be visible in code (`#[mutants::skip]` plus a reason comment) or listed with a reason in `.cargo/mutants.toml`.

Diff runs miss tests weakened elsewhere: the diff is matched against the code under test only, so a diff that only changes tests runs no mutants. Add an optional periodic full run. A fast suite matters most, because cost is (number of mutants) × (build plus test time).

### Cited findings
- `--in-diff` "tests only mutants that overlap with regions changed in the diff". Caution from the docs: "edits in one region cause code in a different region or a different file to no longer be well tested", and "a diff that only deletes or changes test code won't cause any mutants to run" — [cargo-mutants book/src/in-diff.md][cm-indiff]
- The example workflow runs `git diff origin/${{ github.base_ref }}.. | tee git.diff` then `cargo mutants --no-shuffle -vV --in-diff git.diff`, and uploads `mutants.out` — [cargo-mutants examples/workflows/in-diff.yml][cm-wf]
- CI recommendations: use `--in-place` to avoid copying the tree; install with `taiki-e/install-action`; GitHub annotations appear automatically — [cargo-mutants book/src/ci.md][cm-ci]
- Self-use: cargo-mutants' own `pr-mutants` job runs `cargo mutants -vV --in-diff git.diff --timeout=500 --build-timeout=500 --exclude=windows.rs --exclude=console.rs` — [cargo-mutants .github/workflows/tests.yml L286-330][cm-tests-wf]
- cargo-mutants' own `.cargo/mutants.toml`: `error_values = ["::anyhow::anyhow!(\"mutated!\")"]`, `exclude_globs` for console and Windows code, `exclude_re = ["delete ! in LockFile::acquire_lock" # causes a hang]`, `profile = "mutants"` — [cargo-mutants .cargo/mutants.toml][cm-config]
- Performance: "anything you can do to make the `cargo build` and `cargo test` suite faster will have a multiplicative effect". Skip doctests ("every doctest example becomes a separate test binary") with `-- --all-targets`. Use `[profile.mutants] inherits = "test", debug = "none"` and a fast linker: mold gives about 20%, and wild cut one tree's time by more than half — [cargo-mutants book/src/performance.md][cm-perf]
- Skipping: use the attribute "so that the skip is visible in the code", path filters for whole untestable modules, and regex filters for classes such as `Debug` impls. `#[cfg_attr(test, mutants::skip)] // Returning false would cause a hang` is the documented style. `with_capacity` arguments are skipped by default — [cargo-mutants book/src/skip.md, attrs.md, skip_calls.md][cm-skip]
- Excluded automatically: `#[cfg(test)]` code, test functions and `unsafe` fns. Mutant genres include replacing a function body with a type-guessed value (`Ok(...)`, `None`, `vec![]`, `String::new()`, `true`/`false`, `0`/`1`), plus binary-operator swaps (`==`→`!=`, `&&`→`||`, `<`→`==`/`>`/`<=`) — [cargo-mutants book/src/mutants.md][cm-mutants]; [src/visit.rs L802-808][cm-visit]
- Outcomes are caught, missed, unviable or timeout. Missed mutants may be real gaps or equivalent code; the docs say to look first for surprising misses such as "replacing an important function with `Ok(())`" — [cargo-mutants book/src/using-results.md][cm-results]
- Coverage only says code ran. Mutation testing finds code whose behavior no test checks, for example a function that writes a file and is only tested on its return value — [cargo-mutants book/src/vs-coverage.md][cm-vscov]
- SQLite's equivalent of a skip annotation is the `OPTIMIZATION-IF-TRUE/FALSE` comment, needed because flipping a branch that exists only for speed (`strHash` `c!=0`) is still correct — [SQLite testing][sqlite-testing]

### Inferences
- **Mental CI job:** `cargo mutants --in-diff git.diff --in-place --profile mutants -- --all-targets`, and fail on missed mutants (the alpha kit gate). A `--timeout` multiple protects against the mentalvm search loops that mutants will inevitably make hang.
- **mentalvm hangs:** mutating loop bounds in search, minimization or POR will produce timeouts. Rather than skipping, enforce every bound in code with a hard cap that returns a "bound exceeded" outcome (TigerBeetle: "Put a limit on everything"). A mutant then fails fast instead of hanging.
- **Performance-only code** (state hashing caches, POR as a speed-up): this is the SQLite `strHash` case. Mutating POR to "no reduction" is behavior-preserving except for state counts. Make it detectable by asserting exact explored-state counts on tiny models (stateright-style golden counts), instead of skipping.
- **Equivalent mutants:** for `Display`/`Debug` impls, use a regex exclude in config. Elsewhere, require a one-line `// mutants::skip because …` reason. A tidy test can check that every `mutants::skip` has a comment on the line before (rust-analyzer's tidy style).
- **Periodic full run:** a weekly scheduled full `cargo mutants` (with `--shard` if needed) catches erosion that diff runs cannot see. It is optional for a one-person repo; turn it on when the suite is under about 30 s.

### Gaps
- No measurements of how long cargo-mutants takes on a workspace like Mental's. The time budget must come from Mental's own suite speed.

## What should be fuzzed vs property-tested in each crate?

### Takeaway
- **Property tests** (proptest, inside `cargo test`, deterministic thanks to committed regressions) suit structured invariants: round-trips, canonical forms, oracle agreement and algebraic laws.
- **Fuzzing** (cargo-fuzz, outside `cargo test`) suits every reader of untrusted bytes. Its oracle should be "no panic, plus a round-trip or idempotence check", not just "no crash".
- For a solo project: build the fuzz targets on every PR, run them briefly only when a reader changes, and turn every crash into a committed fixture.

### Cited findings
- toml_edit's fuzz target parses, prints and re-parses. It asserts the re-parse succeeds and that `doc.to_string()` is stable (print, parse, print gives the same text). Non-UTF-8 input is dropped with `Corpus::Reject` — [toml crates/toml_edit_fuzz/parse_document.rs][toml-fuzz]
- toml_edit property tests in `src/encode.rs`: for every printable string `"\\PC*"`, `Value::from(s).to_string()` must parse back, and so must `Key::new(s)`. Failures print the input and encoded value in fenced blocks — [toml crates/toml_edit/src/encode.rs L377-425][toml-encode]
- ruff has fuzz targets for parse idempotency, fix validity, formatter idempotency and formatter validity. The idempotency harness also detects "unsteady state", where results only settle on the third round-trip — [ruff fuzz/fuzz_targets/ruff_parse_idempotency.rs][ruff-fuzz-idem]; [ruff fuzz/README.md][ruff-fuzz]
- ruff fuzzing setup:
  - All targets share one corpus seeded from CPython sources and an optional dataset.
  - `cargo fuzz tmin` minimizes crashes.
  - CI builds the fuzz targets (`cargo fuzz build`) and runs the parser fuzzer only when fuzz-related code changes (change-detection job).

  [ruff fuzz/init-fuzzer.sh][ruff-fuzz-init]; [ruff ci.yaml L82-87, L648-680][ruff-ci]
- proptest persists failing seeds under `proptest-regressions/` and replays them first on later runs: "It is recommended to check these files in to your source control" — [proptest book/src/proptest/failure-persistence.md][pt-persist]
- The default is 256 cases, overridable with `PROPTEST_CASES`. For speed, set `opt-level = 3` for the proptest and `rand_chacha` packages in the test profile — [proptest proptest/src/test_runner/config.rs L26, L157-159][pt-config]; [proptest book tips-and-best-practices.md][pt-tips]
- `proptest-state-machine` checks a system under test against a `ReferenceStateMachine` (`init_state`, `transitions`, `apply`, `preconditions`). The generated transition sequence is itself shrunk — [proptest book/src/proptest/state-machine.md][pt-sm]
- wasmtime separates *generators* (raw fuzzer bytes interpreted "as 'DNA' or pre-determined choices through a decision tree", e.g. wasm-smith producing valid modules) from *oracles* (from "no assertions failed" up to comparing optimized and unoptimized runs). Both are kept fuzzer-agnostic in the `wasmtime-fuzzing` crate — [wasmtime docs/contributing-fuzzing.md L157-190][wt-fuzzdoc]
- SQLite's fuzz targets mutate SQL and database files together. Corrupt inputs must produce `SQLITE_CORRUPT` and never a crash — [SQLite testing][sqlite-testing]
- TigerBeetle's `exhaustigen.zig` enumerates every input in a small bounded space ("generate all permutations of abcd", asserting n! were produced) instead of sampling. It is based on matklad's "generate all the things" — [tigerbeetle src/testing/exhaustigen.zig][tb-exh]

### Inferences: fuzzing and property-test scope per crate

| Crate | Fuzz (cargo-fuzz) | Property test (proptest) | Exhaustive small-domain |
|---|---|---|---|
| `model` | JSONL reader: arbitrary bytes give `Ok` or a typed error, never a panic. On `Ok`, write then read gives the same model and write is idempotent (toml_edit pattern). Guard-expression parser, the same way. | `read(write(m)) == m` for generated models; `write` output is sorted and canonical; semantic ID parse/display round-trip; a structural check accepts every generated well-formed model, and rejects any model after one generated "seeded error" mutation (milestone 1) | guard algebra over ≤ 3–4 atoms: every guard set, compared with a truth table |
| `extract` | Python source to facts: arbitrary UTF-8 gives no panic. Every emitted anchor's quote occurs in the source, at its stated lines. Seed the corpus from `examples/` and a CPython checkout, like ruff. | Facts are invariant under formatting-only edits (adding blank lines or comments changes anchors but not fact IDs). Optional. | — |
| `mentalc` | LLM answer parser and MCP JSON-RPC line parser: arbitrary bytes give a typed error | the cache key is stable and sensitive (same inputs give the same key; any changed input gives a different key); the ledger round-trips | countdown failure sweep over the backend (previous section) |
| `mentalvm` | Run-file / schedule reader (YAML or JSON): no panic, round-trip. Optionally, a structured generator of small random models (wasm-smith style) whose oracle is the differential search check in the next section. | replay of any schedule the search produced reproduces the same trace; POR on and off give the same verdict set; the minimized witness is still a witness | all schedules of tiny models (2 actors × ≤ 3 steps) compared with the naive enumerator |

- **Fuzzing budget:** CI builds the fuzz targets on every PR (so they don't rot) and runs each target for about 60 s only when its reader's files change (ruff's change detection). Long runs happen locally on demand. Continuous or OSS-Fuzz fuzzing is bloat for now.
- **No dataset downloads in CI:** seed the fuzz corpus from the fixtures already committed.
- Commit `proptest-regressions/` and minimized fuzz crashes, which become `tests/data/err/fuzz-<hash>.*` fixtures. This is SQLite's `fuzzcheck` idea at zero infrastructure cost.

### Gaps
- Which Python parser `extract` will use (tree-sitter, ruff_python_parser or another) is not settled. A third-party parser's own crashes are out of scope, but the facts layer must survive its error recovery.

## How to test non-trivial algorithms (guard partition checks, model-checker search, POR) differentially against a simple reference implementation?

### Takeaway
For each clever algorithm, keep a dumb reference implementation of about 30 lines in the test module. Compare the two on exhaustively enumerated tiny inputs and on proptest-generated small inputs, the way wasmtime checks against wasmi and v8, and the way the proptest state machine compares against a reference model.

Check witnesses, not just verdicts: a reported overlap point must satisfy both guards, and a race schedule must replay into the violation. Pin exact state counts on tiny models, as stateright does, so that performance optimizations like POR, symmetry and caching cannot silently change what is explored.

### Cited findings
- wasmtime's `oracles::differential` runs the same generated module on `lhs_engine` and `rhs_engine` (`DiffEngine` implementations for wasmi, v8 and wasmtime with other settings). Results must match `assert_eq!(lhs, rhs)`, and errors must match through `assert_error_match` — [wasmtime crates/fuzzing/src/oracles.rs L460-604][wt-oracles]; [oracles/diff_wasmi.rs, diff_v8.rs][wt-oracles-dir]
- wasmtime warns that differential fuzzing only works where the semantics are fully specified. WASIp1 "is not suitable for differential fuzzing across engines" because "there is no great oracle" — [wasmtime docs/contributing-fuzzing.md L113-121][wt-fuzzdoc]
- stateright asserts exact state counts for BFS and DFS on the same model (`256 * 256`), and with and without symmetry reduction (9 versus 6). It also asserts both the specific discovery path and that another valid path is accepted (`assert_discovery("solvable", vec![Guess::IncreaseY; 27])`) — [stateright src/checker/bfs.rs L442-475][sr-bfs]; [dfs.rs L560-586][sr-dfs]
- proptest-state-machine compares a system under test with a reference state machine after every generated transition, and shrinks the transition sequence when they disagree — [proptest book state-machine.md][pt-sm]
- SQLite's SQL Logic Test runs 7.2 million queries and compares results with PostgreSQL, MySQL, SQL Server and Oracle. It also runs the suite with optimizations disabled and requires identical output, using the unoptimized engine as its own reference — [SQLite testing][sqlite-testing]
- TigerBeetle's simulator adds "checkers" on top of assertions, e.g. replicas' data files must be "byte-for-byte identical across caught-up nodes" — [tigerbeetle docs/internals/vopr.md][tb-vopr]
- TigerBeetle's exhaustive generator enumerates every choice sequence within bounds by incrementing the rightmost choice that is still below its bound, like an odometer — [tigerbeetle src/testing/exhaustigen.zig][tb-exh]

### Inferences: differential oracles for Mental
- **Guard algebra (spec 6.7: cover everything, no overlap).** The reference enumerates all 2^n assignments of the n atoms in the guards (n ≤ 10 in tests) and evaluates each guard. Production is whatever is clever (BDD, normalization, SAT).
  - Properties: `prod.verdict == ref.verdict`.
  - If production reports a gap, its witness assignment satisfies no guard.
  - If it reports an overlap, the witness satisfies at least two guards.
  - Test every guard set over ≤ 3 atoms exhaustively (an exhaustigen-style enumerator in Rust is about 30 lines), and random sets over more atoms with proptest.
- **Bounded-exhaustive search versus a naive enumerator.** The reference is a recursive DFS over `enabled`/`fire` (spec 8.2) with no POR, no state caching and no minimization. It returns the set of (outcome kind, final abstract state) reached within the bounds. Properties on tiny generated models:
  - The production search with POR and caching reaches the same set of verdict kinds.
  - Every violation it reports comes with a schedule that replays (spec 8.9: verdict `matched`) into the same violation.
  - The naive search never finds a violation the production search misses.
  - The production search explores at most as many states as the naive one.
- **POR soundness.** Run the same production search with POR off and on, and require identical verdict sets: SQLite's "optimizations off gives identical output" rule. Pin the exact state counts with POR on and off for 3–5 named tiny models in `tests.rs` (stateright-style). A change to the counts then needs a reviewed diff of an `expect!` value.
- **Minimizer.** The output is still a witness, and it is 1-minimal: removing any single choice makes the schedule infeasible or non-violating. The check is a short loop; no reference is needed.
- **Replay verdicts (spec 8.9).** Use a property: a schedule recorded on model M replays on M with every choice `matched`. Then apply a generated edit and check that each verdict is consistent with the edit (an edited-away choice is `infeasible`).
- **Step function (P semantics).** Hand-written scenario tests, one per rule in spec 8.2:
  - an unhandled message gives a model gap;
  - every write is checked against the lifecycle;
  - activations run until they wait;
  - IDs are derived from the creating step, and so are stable across two replays.

  Each is a tiny fixture model plus an `expect!` trace (one step per line). These correspond to TigerBeetle's `replica_test.zig` scenario tests.
- **Acceptance as differential.** The report-jobs build compared with the hand-written reference is the end-to-end oracle. F-S11 (the search finds the duplicate-claim race unprompted, then a fresh search after the fix is clean) becomes two named tests with fixed bounds.
- Keep each reference implementation inside the `tests.rs` of the module it checks, or in a `#[cfg(test)] mod reference;`. That way it is never shipped, never counted toward production coverage, and never mutated.

### Gaps
- How big tiny models can get before the naive enumerator becomes too slow for the speed budget is unknown until mentalvm exists. Pick bounds empirically so the suite stays under budget.
- There is no external oracle comparable to wasmi for Mental's semantics. The P language's checker is a candidate for spot-checking a few hand-translated models but is not researched here (another note covers P semantics).

## How to keep tests deterministic and within a speed budget?

### Takeaway
Determinism comes from four things:
- no wall clock, network or real LLM in tests;
- seeded randomness with the seed printed;
- committed proptest regressions;
- sorted output everywhere, since the stored model is sorted JSONL by design.

Speed comes from:
- tiny fixtures (rust-analyzer's no-libstd invariant);
- few heavy tests, gated locally but always run in CI;
- a fast suite, which multiplies into cheap mutation testing.

### Cited findings
- VOPR is "deterministic based on a _seed_ number and the Git commit". Time is simulated: "One minute of VOPR time is equivalent to days of real-world testing" — [tigerbeetle docs/internals/vopr.md][tb-vopr]
- proptest replays persisted failing seeds before generating new cases. It stores the seed rather than the value because "there is no way to determine whether any given value is actually within the domain of the strategy" — [proptest failure-persistence.md][pt-persist]
- rust-analyzer's fast-test invariant (fixtures carry their own minimal core library), its slow-test gate, and the `is_ci()` behavior of `ensure_file_contents` — [r-a architecture.md][ra-arch]; [r-a test-utils/src/lib.rs][ra-tu]
- cargo-mutants needs a tree "where `cargo test` passes reliably when copied to a temporary directory". Flaky or location-dependent tests break mutation testing — [cargo-mutants using-results.md][cm-results]
- ruff runs its suite under nextest in CI (`cargo nextest run --all-features --profile ci`) — [ruff ci.yaml L583][ruff-ci]
- Spec 4.9 says the model is "one record per line, sorted by ID", and spec 8.2 says run-time IDs "are derived from the step that created them, so replay and comparison stay stable" — `docs/specification.md`

### Inferences
- **Rules:**
  - Tests never read the clock: time-dependent code takes a `now` parameter.
  - Randomness comes only from proptest or from an explicit `seed: u64` that appears in the failure message.
  - Tests never touch the network or a real LLM. CI has no key, and the "fail rather than degrade" rule means a missing key makes the build fail, which tests assert.
  - Collections whose iteration order reaches output use `BTreeMap`/`BTreeSet` or are sorted. Enforce it with clippy `disallowed_types` for `std::collections::HashMap` in `model` and `mentalvm` output paths, or with a determinism test.
- **Determinism test:** build report-jobs twice in the same process with a scripted backend and once more in a fresh temp directory, and assert byte-identical `.mental/model/` output. Spec 11.5 lists stability as something to measure, so test it for the deterministic parts.
- **Speed budget (proposed, to tune):**
  - whole-workspace `cargo test` on a debug build under about 15 s, excluding fuzzing;
  - any single test under 1 s, enforced by nextest `slow-timeout` with `terminate-after` in a `ci` profile;
  - heavy CLI tests gated locally by an env var (rust-analyzer's `RUN_SLOW_TESTS`, auto-on in CI);
  - the acceptance test under 5 s.
- Opt-level 3 for proptest dependencies in the test profile. Use the `mutants` profile with `debug = "none"`.

### Gaps
- The numeric budgets above are proposals, not taken from any source. Calibrate them once the crates exist.

## Candidate testing guidelines (enforceable) and per-crate notes

### Takeaway
About twenty rules cover the alpha kit's exhaustive-testing principle. Each is enforced mechanically where possible: a tidy test inside `cargo test`, the CI coverage and mutants gates, clippy config, or a reviewer checklist item. Most cost nothing beyond discipline.

### Cited findings
- Rule sources are cited inline in the table below. All were introduced in the previous sections.

### Inferences: candidate guidelines

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| T1 | Unit tests live in `tests.rs` beside the module (`#[cfg(test)] mod tests;`). Public behavior lives in `<crate>/tests/`. | One obvious place to look; production files stay short | alpha kit; [r-a architecture][ra-arch] | tidy test: every `mod tests;` has a file, and no inline `mod tests {` blocks |
| T2 | A test is input plus expected output through one `check` helper per module, and tests one behavior | Readable, data-driven, cheap to add | [r-a testing.md][ra-testing]; [toml_writer `t()`][toml-writer] | reviewer |
| T3 | Fixtures are minimal: inline raw strings up to about 30 lines, otherwise a file pair in `tests/data/{ok,err}/` | Less to read, faster | [r-a style.md "Minimal Tests"][ra-style] | reviewer |
| T4 | No `#[should_panic]` and no `#[ignore]`. Assert `Err`/`None`; assert current wrong behavior with a `FIXME` | Panics are bugs; ignored tests rot | [r-a style.md][ra-style]; [r-a tidy.rs][ra-tidy] | tidy test |
| T5 | Snapshots use `expect!`/`expect_file!`, are updated only with `UPDATE_EXPECT=1` and reviewed in the diff. Snapshot human-readable renderings only, never `Debug` of internals. | Easy updates without churn | [r-a testing.md][ra-testing] | CI runs without the env var; reviewer |
| T6 | Goldens (the hand-written report-jobs reference model) are never written by tooling | The golden is the specification, not an observation | spec 11.4; this research | tidy test: no `expect_file!` points into `examples/` |
| T7 | No orphaned fixtures or snapshots | Dead expectations mislead | [ruff `--unreferenced reject`][ruff-ci] | tidy test lists `tests/data` and checks each file is consumed |
| T8 | The test harness checks global invariants on every fixture (IDs unique and sorted, references resolve, anchors' quotes found, output idempotent) | Every test checks more for free | [ruff test.rs][ruff-test]; [ruff parser fixtures][ruff-parser-fx] | shared `check` helper in `model` testkit |
| T9 | Every reader of outside input (JSONL model, guard expressions, Python source, LLM answers, MCP lines, run files) has a round-trip or idempotence property test and a cargo-fuzz target whose oracle is "no panic, plus round-trip" | Outside input is where crashes live | [SQLite testing][sqlite-testing]; [toml fuzz][toml-fuzz]; [ruff fuzz][ruff-fuzz] | CI: `cargo fuzz build` on every PR; 60 s per target when its reader changes; reviewer |
| T10 | Saved fuzz crashes and `proptest-regressions/` are committed and run by `cargo test` | Old bugs stay dead | [proptest persistence][pt-persist]; SQLite fuzzcheck | CI (git clean after tests); reviewer |
| T11 | Every failure path of the LLM backend is forced by a countdown sweep (fail call k for every k, in transient and persistent modes), followed by a consistency check | SQLite's OOM and I/O method at the one external seam | [SQLite testing][sqlite-testing]; [wasmtime oom.rs][wt-oom] | test helper; the coverage gate shows unforced paths |
| T12 | Fault injection uses seams the product already has (backend trait, pure core with an I/O shell). No `cfg(test)` branches in production logic and no mocking crates. | No production bloat | SQLite VFS/malloc config; this research | reviewer; tidy test bans `mockall` in Cargo.toml |
| T13 | Each limit (search bounds, depth, repair attempts, cache size) is tested at limit−1, limit and limit+1, and every loop has a hard cap | Off-by-one bugs live at limits; caps turn mutant hangs into failures | [SQLite testing][sqlite-testing]; [TIGER_STYLE][tb-style] | cargo-mutants (`<`↔`<=` mutants); reviewer |
| T14 | A bug fix starts with a failing test, in the same PR | A bug is not fixed until a test shows it | [SQLite testing][sqlite-testing] | reviewer; optional CI check that `fix:` commits touch tests |
| T15 | Invariants are asserted in code with `debug_assert!`, at pair points (before writing and after reading) | Assertions multiply what tests and fuzzing find | [TIGER_STYLE][tb-style]; SQLite's 6754 asserts | reviewer; tests run in debug |
| T16 | Each non-trivial algorithm has a test-only reference implementation and a differential test (guard algebra vs truth table, search with POR vs naive search, minimizer vs 1-minimality check). Verify witnesses, not only verdicts. | Clever code needs an independent oracle | [wasmtime differential][wt-oracles]; [proptest state machine][pt-sm] | reviewer; cargo-mutants |
| T17 | Exact explored-state counts are pinned in `expect!` for 3–5 tiny models, with POR on and off | Makes speed-only code observable to tests and mutants | [stateright bfs/dfs tests][sr-bfs] | test |
| T18 | Small domains are enumerated exhaustively, not sampled (all guard sets over ≤ 3 atoms, all schedules of tiny models) | "Exhaustive" literally, at low cost | [tigerbeetle exhaustigen][tb-exh] | reviewer |
| T19 | Tests are deterministic: no clock, network or real LLM; seeds printed; output order sorted | Flaky tests break mutants and trust | [VOPR][tb-vopr]; [cargo-mutants results][cm-results] | clippy `disallowed_types`/`disallowed_methods` (e.g. `SystemTime::now` outside the shell); determinism test |
| T20 | CI gate on the diff: 100% line and branch coverage (`cargo llvm-cov --branch`, pinned nightly) and zero missed mutants (`cargo mutants --in-diff --in-place --profile mutants`). Every skip is `#[mutants::skip]` or a config `exclude_re`, each with a reason. | Coverage shows code ran; mutants show behavior was checked | alpha kit; [cargo-mutants CI][cm-ci]; [cm config][cm-config] | CI; tidy test: every `mutants::skip` has a reason comment |
| T21 | Test what ships: the acceptance test also runs on `--release`, and the mentalvm tests (or at least the replay tests) also run on the WebAssembly target | Release and wasm builds differ (overflow checks, debug_asserts, target) | [SQLite as-delivered][sqlite-testing] | CI job |
| T22 | Speed budget: the workspace suite stays under about 15 s, no test over 1 s, heavy tests gated locally but run in CI | Mutation cost is multiplied by suite time | [cargo-mutants performance][cm-perf]; [r-a slow tests][ra-tu] | nextest `slow-timeout`/`terminate-after`; CI timing |

**Bloat to avoid for a one-person project:**
- insta plus `cargo insta review`, when expect-test does the job;
- mdtest-style Markdown test frameworks;
- coverage marks on every branch (use them sparingly);
- OSS-Fuzz or continuous fuzzing;
- a crash or power-loss simulator;
- turmoil/madsim, because there is no network and mentalvm is the simulator;
- mocking crates;
- MC/DC;
- an "ecosystem check" over many repos (ruff's), until stability across repositories becomes a goal; then a small corpus of Python repos diffed per PR is the natural next step;
- full weekly mutation runs, until the suite is fast.

### Inferences: per-crate notes
- **`model`** (the most exhaustively testable crate; pure, no I/O)
  - Reader and writer: proptest round-trip and canonical form, plus a cargo-fuzz target.
  - Semantic IDs: parse/display round-trip; ordering laws (sorting is total and consistent with `Ord`).
  - Guard expressions: parser fuzz, plus a differential partition check against a truth table, exhaustive for ≤ 3 atoms.
  - Structural checks: `tests/data/ok/` must pass and `tests/data/err/<seeded-error>/` must each fail with an `expect!` diagnostic. This is milestone 1 ("accepts the reference model and rejects seeded errors"); one fixture per error kind.
  - Ship a small `testkit` (behind a `testkit` feature or as a dev-dependency crate) with the fixture DSL and the invariant-checking `check` helper, so other crates reuse it. Keep it in this crate rather than adding a fifth crate unless it grows.
- **`extract`**
  - Directory-driven tests: `tests/data/<case>/input.py` → `expected.facts.jsonl`, compared with `expect_file!`.
  - Every case asserts that each anchor's quote occurs in the input at its lines (spec 6.7 anchor check).
  - Milestone 2 numbers become one named test: 6 entry points, 2 state variables, every branch listed.
  - Fuzz: arbitrary UTF-8 gives no panic, and the facts' anchors are valid.
  - Error-recovery fixtures with syntactically broken Python.
- **`mentalc`**
  - Pure functions per check (spec 6.7): each check gets ok and err fixture models.
  - The LLM pipeline is tested only through `Scripted` and `Countdown` backends. Recorded call ledgers double as fixtures.
  - Repair-loop limit tests at limit−1, limit and limit+1 failed repairs.
  - Cache: key stability (pin a golden key for a fixed input with `expect!`), invalidation on any input change, and recovery from a corrupt cache entry.
  - Missing credentials fail with a clear error (fail rather than degrade).
  - MCP: `handle(line) -> lines` transcript tests, plus one heavy test that spawns the binary.
  - CLI: a few heavy tests on a temp copy of a fixture tree (cargo-mutants' `copy_of_testdata`).
  - The report-jobs acceptance test lives here, in debug and release.
- **`mentalvm`** (no I/O; this crate is the deterministic simulator, so its tests target correctness of the search)
  - One scenario test per step rule (spec 8.2), each an `expect!` trace.
  - Strategy equivalences: a concrete run of a found schedule equals the search's trace, and replay of any produced schedule gives all `matched`.
  - Differential: naive search vs production search (POR, caching).
  - Pinned state counts with POR on and off.
  - Minimizer 1-minimality.
  - Every stuck outcome kind (spec 8.5: model gap, deadlock, illegal write, zero-time cycle, invariant violated) has a minimal fixture that produces it and only it.
  - Hard caps on every loop, with a "bound exceeded" outcome.
  - F-S11 as two acceptance tests: the race found before the fix, and clean after it within the same bounds.
  - Run the suite, or the replay subset, on the wasm32 target in CI, because this crate ships to WebAssembly.

### Gaps
- The exact coverage interaction with `debug_assert!` and the diff branch-coverage tooling (see the first section's gaps) must be settled before T15 and T20 can be finalized together.
- Whether `extract`'s facts format has a canonical writer (and therefore a round-trip property) depends on design decisions not yet made.

[sqlite-testing]: https://www.sqlite.org/testing.html
[sqlite-malloc]: https://github.com/sqlite/sqlite/blob/33fd659df2a4d2f07071ec61b329b4aa6a7377d5/src/test_malloc.c#L65-L86
[sqlite-fault]: https://github.com/sqlite/sqlite/blob/33fd659df2a4d2f07071ec61b329b4aa6a7377d5/src/fault.c
[sqlite-int]: https://github.com/sqlite/sqlite/blob/33fd659df2a4d2f07071ec61b329b4aa6a7377d5/src/sqliteInt.h#L492-L563
[llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[ra-testing]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/testing.md
[ra-arch]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#testing
[ra-style]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#minimal-tests
[ra-readme]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/README.md#ci
[ra-tidy]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/xtask/src/tidy.rs
[ra-tu]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/test-utils/src/lib.rs#L397-L510
[ra-parser-tests]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/parser/src/tests.rs#L55-L78
[ra-expr]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/parser/src/grammar/expressions.rs#L117-L126
[ra-inline]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/xtask/src/codegen/parser_inline_tests.rs
[ruff-bugbear]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/rules/flake8_bugbear/mod.rs
[ruff-test]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/test.rs#L244-L360
[ruff-parser-fx]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_parser/tests/fixtures.rs
[ruff-ci]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/.github/workflows/ci.yaml
[ruff-contrib]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/CONTRIBUTING.md#rule-testing-fixtures-and-snapshots
[ruff-fuzz]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/fuzz/README.md
[ruff-fuzz-idem]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/fuzz/fuzz_targets/ruff_parse_idempotency.rs
[ruff-fuzz-init]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/fuzz/init-fuzzer.sh
[cm-visit]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/src/visit.rs#L802-L808
[cm-indiff]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/in-diff.md
[cm-wf]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/examples/workflows/in-diff.yml
[cm-ci]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/ci.md
[cm-tests-wf]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/.github/workflows/tests.yml#L286-L330
[cm-config]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/.cargo/mutants.toml
[cm-perf]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/performance.md
[cm-skip]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/skip.md
[cm-mutants]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/mutants.md
[cm-results]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/using-results.md
[cm-vscov]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/vs-coverage.md
[cm-error]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/error-values.md
[cm-testdata]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/testdata/README.md
[cm-util]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/tests/util/mod.rs
[pt-persist]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/failure-persistence.md
[pt-config]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/test_runner/config.rs
[pt-tips]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/tips-and-best-practices.md
[pt-sm]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/state-machine.md
[toml-compl]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_edit/tests/encoder_compliance.rs
[toml-writer]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_writer/tests/string.rs
[toml-fuzz]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_edit_fuzz/parse_document.rs
[toml-encode]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_edit/src/encode.rs#L377-L425
[wt-oom]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oom.rs
[wt-fuzzdoc]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/docs/contributing-fuzzing.md
[wt-oracles]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oracles.rs#L460-L604
[wt-oracles-dir]: https://github.com/bytecodealliance/wasmtime/tree/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oracles
[tb-style]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/TIGER_STYLE.md
[tb-vopr]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/internals/vopr.md
[tb-exh]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/src/testing/exhaustigen.zig
[tb-marks]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/src/testing/marks.zig
[sr-bfs]: https://github.com/stateright-rs/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/bfs.rs#L442-L475
[sr-dfs]: https://github.com/stateright-rs/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/dfs.rs#L560-L586
[turmoil-readme]: https://github.com/tokio-rs/turmoil/blob/720d7bc038d58f3c9340ff62fae992da63195bb4/README.md
[turmoil-fs]: https://github.com/tokio-rs/turmoil/blob/720d7bc038d58f3c9340ff62fae992da63195bb4/crates/turmoil-fs/src/lib.rs
