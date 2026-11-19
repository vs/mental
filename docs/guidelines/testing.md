# Testing playbook

**Status: Accepted by the owner on 2026-11-14.** Distilled from [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md), with the testing sections of the five other [research notes](../research/implementation-guidelines/notes/). Crate-specific tests are in each crate's file: [model](model.md), [extract](extract.md), [mentalc](mentalc.md), [mentalvm](mentalvm.md).

The goal is the alpha kit's exhaustive-testing principle at the cost a one-person project can carry: every branch taken both ways, every failure path forced, every reader fuzzed or property-tested, every limit tested at its edges, every mutant on a changed line caught, and a suite fast enough that mutation testing stays cheap.

**One settled choice:** snapshots use **insta** (inline snapshots, `insta_cmd` for the CLI). The testing note preferred expect-test; the owner chose insta, and expect-test is never added beside it.

## Test layers

| Layer | Where | What | Budget |
|---|---|---|---|
| Unit | `src/<module>/tests.rs`, declared as `#[cfg(test)] mod tests;` | one behavior per test, through the module's `check` helper | < 10 ms each |
| Crate API | `<crate>/tests/*.rs` | public behavior; directory walks over `tests/data/{ok,err}/` | < 100 ms each |
| Property and differential | `tests.rs`, or `tests/props.rs` for public properties | round trips, canonical forms, oracle agreement | 256 cases by default |
| CLI and MCP | `mentalc/tests/cli.rs`, `mentalc/tests/mcp.rs` | a few heavy tests: the real binary on a temp copy of a fixture tree | gated locally, always in CI |
| Acceptance | `mentalc/tests/report_jobs.rs`, `extract/tests/`, `mentalvm/tests/report_jobs.rs` | report-jobs end to end against the hand-written reference | < 5 s; also in release |
| Tidy | `model/tests/tidy.rs` | source-level checks of these guidelines | < 1 s |
| Fuzz | `fuzz/`, its own workspace | readers of outside input only | outside `cargo test` |

## The `check` helper

Every module with tests has one helper; every test is one call to it plus one expected output.

```rust
// src/guard/tests.rs
#[track_caller]
fn check(fixture: &str) -> String {
    let model = testkit::model(fixture);    // fixture DSL: one string, many records
    testkit::assert_invariants(&model);     // global invariants, free for every test
    render(&partition(&model))              // the one call into the API under test
}

#[test]
fn gap_reports_lowest_witness() {
    insta::assert_snapshot!(check(r#"
behavior b
  case a when pred:x & pred:y
  case b when !pred:x
"#), @"gap: pred:x=true pred:y=false");
}
```

The helper is the only place that knows the API. When the API changes, one helper changes, not a hundred tests. The inline snapshot sits in the test, not in the helper, because insta ties each inline snapshot to the place its macro is written. (The test DSL may abbreviate IDs; stored models never do, see [M15](model.md#m15-only-full-ids-are-stored).)

## Rules

### T1. Where tests live

- **Rule:** unit tests live in a `tests.rs` beside their module (`#[cfg(test)] mod tests;`), never in an inline `mod tests { ... }` block; public behavior is tested in `<crate>/tests/`.
- **Why:** one obvious place to look, and production files stay short.
- **Source:** alpha rule `tests-beside-code`; [Codex `*_tests.rs` beside modules][codex-src].
- **Enforced by:** tidy test `test_modules_are_files`.

### T2. One check helper per module

- **Rule:** a test is one call to the module's single `#[track_caller] fn check` and one expected output (usually an inline snapshot in the test); tests do not call internals, and each test checks one behavior and is named after it.
- **Why:** readable, data-driven and cheap to add; refactoring the API touches one function.
- **Source:** [rust-analyzer architecture.md L416-L436][ra-arch]; [toml_writer `t()`][toml-writer]; [jaq `yields!`][jaq-tests].
- **Enforced by:** reviewer.

### T3. Minimal fixtures with meaningful folders

- **Rule:** fixtures are minimal: inline unindented raw strings up to about 30 lines, otherwise a file pair in `tests/data/ok/` (must produce no diagnostics) or `tests/data/err/` (must produce at least one). Multi-file fixtures are one string with `//- path` headers, parsed by `model::testkit`. Every fixture file is used by some test.
- **Why:** less to read and faster to run; the folder states the expectation.
- **Source:** [rust-analyzer "Minimal Tests"][ra-style-min]; [rust-analyzer `parse_ok`/`parse_err`][ra-parser-tests]; [ruff `--unreferenced reject`][ruff-ci].
- **Enforced by:** reviewer; tidy test `no_orphan_fixtures`.

### T4. No should_panic, no ignore

- **Rule:** no `#[should_panic]` and no `#[ignore]`: assert the `Err` or `None`; for a known wrong behavior, assert the current behavior and add a `FIXME` comment.
- **Why:** a panic is a bug, and an ignored test rots (Shuttle's replay tests have been ignored since replay broke).
- **Source:** [rust-analyzer style.md][ra-style-min]; [rust-analyzer `tidy.rs`][ra-tidy].
- **Enforced by:** tidy test `no_should_panic_or_ignore`.

### T5. Snapshots with insta, reviewed and never orphaned

- **Rule:** snapshot only human-readable renderings (sorted JSONL, rendered diagnostics, traces one step per line), never the `Debug` output of internal types. Prefer inline snapshots (`@"..."`); use `.snap` files only for large outputs; CLI tests use `insta_cmd` with filters for temp paths and versions. CI runs with `INSTA_UPDATE=no` and rejects changed or unreferenced snapshots.
- **Why:** snapshots stay stable under refactors, changes are reviewed in the diff, and dead expectations can't mislead.
- **Source:** [ruff CI `cargo insta test --unreferenced reject`][ruff-ci]; [ruff `CliTest` filters][ruff-cli].
- **Enforced by:** CI step `INSTA_UPDATE=no cargo insta test --check --unreferenced=reject`; reviewer.

### T6. The golden is never a snapshot

- **Rule:** the hand-written reference model at `examples/report-jobs/model/` is a specification: tests compare against it with a plain helper that prints the first differing lines, and no snapshot tool, `--update` flag or test ever writes into `examples/`. It grows by milestone: each milestone's plan writes by hand, before its code, the reference records of the kinds that milestone first produces (milestone 1: the build unit, the pins and every site), and every later comparison covers all kinds written so far.
- **Why:** a golden that tooling can rewrite stops being a specification and becomes an observation. Writing each part with the milestone that fixes its fields (spec 7.4) keeps the reference from waiting on kinds that have no fields yet.
- **Source:** spec 14.2; the testing note's snapshot policy.
- **Enforced by:** tidy test `no_snapshots_into_examples`; CI fails if `examples/` changes during the test run.

### T7. The harness checks invariants on every fixture

- **Rule:** every `check` helper also runs the shared invariant check from `model::testkit`: IDs unique and sorted, references resolve, every anchor's quote found at its lines, write after read is byte-identical.
- **Why:** every test checks more for free, and individual tests stay short.
- **Source:** [ruff fix-convergence and syntax checks in `test.rs`][ruff-test]; [ruff parser `validate_ast` on every fixture][ruff-parser-fx].
- **Enforced by:** the shared helper; reviewer.

### T8. Property tests are reproducible

- **Rule:** randomized tests use proptest only; `proptest-regressions/` files are committed; no other source of randomness appears in tests; a seed used anywhere is printed in the failure message. Every reader and writer has a round-trip property (`read(write(x)) == x`, and `write(read(b)) == b` on canonical bytes).
- **Why:** a failure seen once can be replayed forever; biodivine fixes its seeds, boolean_expression doesn't and can't reproduce failures.
- **Source:** [proptest failure persistence][pt-persist]; [biodivine fixed seeds][bd-fuzz] versus [boolean_expression `thread_rng`][be-bdd].
- **Enforced by:** CI step that fails when the tree is dirty after tests (an untracked regression file); reviewer.

### T9. Enumerate small domains

- **Rule:** where the input space is small, enumerate it instead of sampling it: every guard set over at most 3 predicates, every schedule of 2 activations by at most 3 steps.
- **Why:** "exhaustive" literally, at low cost, with no flakiness.
- **Source:** [TigerBeetle `exhaustigen.zig`][tb-exh].
- **Enforced by:** reviewer.

### T10. Clever code has a dumb oracle

- **Rule:** each non-trivial algorithm (the guard partition check, the search, the minimizer) has a reference implementation of about 30 lines inside its module's `tests.rs` and a differential test against it on enumerated and generated inputs. Tests verify witnesses, not only verdicts: a reported gap satisfies no guard, a reported race replays into the violation.
- **Why:** an independent oracle catches what hand-picked cases miss; keeping it in `tests.rs` keeps it out of the shipped code, the coverage report and the mutants.
- **Source:** [wasmtime differential oracles][wt-oracles]; [proptest state machines][pt-sm]; [Stateright pinned counts and discoveries][sr-bfs].
- **Enforced by:** reviewer; mutation testing.

### T11. Fuzz only readers of outside input

- **Rule:** cargo-fuzz targets exist for the readers of outside input and nothing else: the JSONL model reader and the guard parser (`model`), Python source to facts (`extract`), the LLM answer parser and the MCP line parser (`mentalc`), the run-file reader (`mentalvm`). Each target's oracle is "no panic, plus a round trip or idempotence check". CI builds every target on each PR and runs a target for about 60 s when its reader changes; each crash is minimized and committed as `tests/data/err/fuzz-<hash>.*`, which `cargo test` then runs.
- **Why:** outside input is where crashes live; building targets keeps them from rotting; saved crashes keep old bugs dead at no infrastructure cost.
- **Source:** [toml_edit round-trip fuzzer][toml-fuzz]; [ruff fuzz setup and CI change detection][ruff-fuzz]; [SQLite `fuzzcheck`][sqlite-testing].
- **Enforced by:** CI step (`cargo fuzz build`, and a timed run on change); reviewer.

### T12. Inject faults through seams that already exist

- **Rule:** inject LLM failures through the `Backend` trait: a scripted or replay backend, and a `Countdown<B>` that fails call k once (transient) or every call from k on (persistent), swept over every k up to a dry run's call count, with a consistency check after each run. File I/O faults use real temp directories (missing file, read-only directory, truncated cache entry). No `Fs` trait, no `cfg(test)` branch in production logic, no mocking crate.
- **Why:** SQLite's countdown method without adding production complexity; `mentalvm` needs none of this, because its faults (drop, duplicate, timeout) are `env` choices the search enumerates.
- **Source:** [SQLite testing][sqlite-testing]; [SQLite `test_malloc.c` countdown][sqlite-malloc]; [wasmtime `OomTest`][wt-oom].
- **Enforced by:** the sweep helper; diff coverage shows unforced error paths; tidy test `banned_crates` (no `mockall`).

### T13. Test every limit at its edges

- **Rule:** every limit (search bounds, depth, repair cap, guard-check cap, retry count) is tested just below, at and just above its value, and every loop has a hard cap that ends with an error or a `bound-exceeded` outcome.
- **Why:** off-by-one bugs live at limits; caps turn hanging mutants into fast failures.
- **Source:** [SQLite boundary tests][sqlite-testing]; [TigerBeetle "put a limit on everything"][tb-style]; [cargo-mutants operator swaps][cm-visit].
- **Enforced by:** cargo-mutants (`<` and `<=` swaps); reviewer.

### T14. A fix starts with a failing test

- **Rule:** a bug fix adds, in the same change, a test that fails without the fix.
- **Why:** a bug is not fixed until a test shows it.
- **Source:** [SQLite regression rule][sqlite-testing].
- **Enforced by:** reviewer.

### T15. Invariants are asserted in the code

- **Rule:** internal invariants are `assert!` or `debug_assert!`, placed in pairs where data crosses a boundary (before writing and after reading); there are no soft assertions (`never!`), no `catch_unwind` and no panic hook. A branch no test can reach is deleted or replaced by an assertion.
- **Why:** assertions multiply what tests and fuzzing find; soft assertions exist for long-running servers, not batch tools.
- **Source:** [TigerBeetle pair assertions][tb-style]; [SQLite `ALWAYS`/`NEVER`][sqlite-int]; rust-analyzer's `never!` (rejected, see [architecture note](../research/implementation-guidelines/notes/mentalc_architecture.md)).
- **Enforced by:** reviewer; tests run in debug.

### T16. Mutation testing on the diff

- **Rule:** CI runs `cargo mutants --in-diff` on the PR diff, in place, with a `mutants` profile (`debug = "none"`) and a timeout multiplier; any missed mutant fails. A mutant that can't be killed on purpose is skipped with `#[mutants::skip]` and a reason comment above it, or with an `exclude_re` entry that has a reason, and either way has a row in the Exceptions table of [CONTRIBUTING.md](../../CONTRIBUTING.md#exceptions); speed-only code is made observable (pinned state counts) instead of skipped.
- **Why:** coverage shows code ran; mutants show behavior was checked.
- **Source:** [cargo-mutants in-diff][cm-indiff], [CI][cm-ci], [config][cm-config], [performance][cm-perf]; alpha rule `mutants-skips`.
- **Enforced by:** CI step (alpha `ci:mutants`); tidy test `mutants_skip_has_reason`.

### T17. Full coverage of every change

- **Rule:** every changed line and branch is covered, measured by `cargo llvm-cov --branch` on the pinned nightly; test code (`tests/`, `tests.rs`) is outside the report.
- **Why:** a branch no test takes is a branch nobody knows works.
- **Source:** [cargo-llvm-cov README][llvm-cov]; alpha `exhaustive-testing`.
- **Enforced by:** CI step (alpha `ci:diff-coverage`). What the report counts is [measured below](#branch-coverage-of-assertions-settled).

### T18. Deterministic and hermetic

- **Rule:** tests never read the clock, the network or a real LLM; time-dependent code takes `now` as a parameter; each crate has a test that runs its main output twice (and once in a fresh temp directory) and requires byte-identical bytes. CI has no API key and no network, with one exception: the job that makes the report-jobs environment with `uv` before the acceptance tests, which then read it as files. Resolver tests use tiny environments checked in as fixtures.
- **Why:** a flaky test breaks mutation testing and trust.
- **Source:** [TigerBeetle VOPR][tb-vopr]; [cargo-mutants on flaky trees][cm-results].
- **Enforced by:** clippy `disallowed-methods` ([G4](README.md#g4-io-and-ambient-inputs-only-at-named-edges), [G8](README.md#g8-seeds-and-clocks-are-parameters)); determinism tests; CI environment.

### T19. A speed budget

- **Rule:** the workspace `cargo test` on a debug build stays under about 15 s (fuzzing excluded), no single test takes over 1 s, the acceptance tests stay under 5 s, and heavy tests run locally only when `RUN_SLOW_TESTS` is set (always in CI). Every dependency, proptest and ty's crates included, builds with `opt-level = 3` in the dev profile (`[profile.dev.package."*"]`, which the test profile inherits); the workspace's own crates stay unoptimized, with their assertions on.
- **Why:** mutation cost is the number of mutants times the suite time. In the [step 0 spike](../plans/milestone-1-build-model.md#9-the-ty-spike), ty answered every call of report-jobs in 9.6 s on a plain debug build and in 0.9 s with optimized dependencies; the cost is a clean build of `extract` in 138 s instead of 45 s, once per cache, while a rebuild after a change stays at about 1 s.
- **Source:** [cargo-mutants performance][cm-perf]; [rust-analyzer slow-test gate][ra-tu]; [proptest tips][pt-tips].
- **Enforced by:** CI step with per-test timeouts (nextest `ci` profile with `slow-timeout` and `terminate-after`); numbers to calibrate once the crates exist.

### T20. Tidy tests run inside cargo test

- **Rule:** every source-level rule that a string scan can check is a `#[test]` in `model/tests/tidy.rs`, which walks the whole workspace from `CARGO_MANIFEST_DIR/..` with plain string matching (no extra dependency). The list is below.
- **Why:** if `cargo test` passes locally, CI is green; there are no separate scripts.
- **Source:** [rust-analyzer `xtask/src/tidy.rs`][ra-tidy]; [rust-analyzer CI note][ra-readme].
- **Enforced by:** `cargo test`.

### T21. Test what ships

- **Rule:** the report-jobs acceptance tests also run on a `--release` build, and `mentalvm`'s replay and determinism tests also run on the WebAssembly target.
- **Why:** release and wasm builds differ (overflow checks, `debug_assert!`, the target).
- **Source:** [SQLite as-delivered testing][sqlite-testing].
- **Enforced by:** CI jobs.

## Tidy tests

| Test | Checks | Rule |
|---|---|---|
| `crate_graph` | workspace dependency edges; only `extract` uses `ruff_*` and `ty_*` | [G1](README.md#g1-fixed-crate-graph), [X2](extract.md#x2-no-ruff-or-ty-type-escapes) |
| `no_foreign_reexports` | no `pub use` of another crate's items | [G2](README.md#g2-small-public-surface) |
| `traits_have_two_impls` | every trait has two non-test impls, `Backend` excepted | [G3](README.md#g3-functions-over-frameworks) |
| `edges_are_listed` | `#[expect(clippy::disallowed_methods ...)]` only in listed edge files | [G4](README.md#g4-io-and-ambient-inputs-only-at-named-edges) |
| `serde_derives_in_allowed_modules` | `Serialize`/`Deserialize` derives only in allowed modules | [G14](README.md#g14-serializable-types-are-contracts) |
| `dependencies_justified` | a comment above each `[workspace.dependencies]` entry; member deps within the crate's allowlist | [G15](README.md#g15-every-dependency-justified-once) |
| `lockfile_allowlist` | `Cargo.lock` package names equal the checked-in `deps.allow` | [G15](README.md#g15-every-dependency-justified-once) |
| `banned_crates` | none of the banned crates in `Cargo.lock`, except packages reachable only through ruff's and ty's crates; none at all under `model` or `mentalvm` | [G16](README.md#g16-banned-weight) |
| `no_async` | no `async fn` or `.await` | [G16](README.md#g16-banned-weight) |
| `file_length` | non-test source files at most 400 lines | [G17](README.md#g17-size-limits) |
| `no_non_exhaustive` | no `#[non_exhaustive]` | [G19](README.md#g19-no-non_exhaustive-inside-the-workspace) |
| `test_modules_are_files` | every `mod tests;` has its file; no inline `mod tests {` | [T1](#t1-where-tests-live) |
| `no_should_panic_or_ignore` | neither attribute anywhere | [T4](#t4-no-should_panic-no-ignore) |
| `no_orphan_fixtures` | every file under `tests/data/` is referenced | [T3](#t3-minimal-fixtures-with-meaningful-folders) |
| `no_snapshots_into_examples` | no snapshot macro or path points into `examples/` | [T6](#t6-the-golden-is-never-a-snapshot) |
| `mutants_skip_has_reason` | a comment line above every `mutants::skip` | [T16](#t16-mutation-testing-on-the-diff) |
| `model_records_deny_unknown_fields` | every `Deserialize` struct in `model` has the attribute | [M9](model.md#m9-unknown-data-is-an-error) |
| `model_no_json_value` | no `serde_json::Value` in `model/src` | [M9](model.md#m9-unknown-data-is-an-error) |
| `extract_no_library_names` | no library names in `extract/src` outside tests | [X10](extract.md#x10-no-library-names-in-rust) |
| `extract_ruff_ty_pinned` | every `ruff_*` and `ty_*` dependency names the same pinned commit | [X1](extract.md#x1-ruffs-parser-and-tys-crates-pinned-to-one-commit) |
| `mentalvm_no_interior_mutability` | no `RefCell` or `Cell` in `mentalvm/src` | [V2](mentalvm.md#v2-cheap-ordered-clones) |

The alpha kit's generated `tests/setup.rs` already checks that `unsafe_code = "forbid"` and clippy's pedantic group stay on.

## Branch coverage of assertions (settled)

Measured on 2026-11-19 with `cargo llvm-cov --branch` 0.9.1 on nightly-2026-11-10, the setup CI uses ([experiment](../research/implementation-guidelines/experiments/branch-coverage/README.md)):

- `assert!`, `debug_assert!`, `assert_eq!` and `.expect()` add no line and no branch to the report, on one line or several. T15 and T17 do not conflict: assert freely, and no `debug_assertions` switch or `#[coverage(off)]` helper is needed.
- The error side of `?` counts only when it starts the first code region of its line. That happens on the last line of a method chain that rustfmt split, such as `.sum::<Result<u16, _>>()?;`: the `.sum(…)` call belongs to the region that starts on the chain's first line, so the error side of the `?` is the line's own region. That line then shows 0 hits and fails diff coverage until a test forces the error. That is the rule working: a `?` is a failure path, and the [exhaustive-testing principle](../alpha/principles.toml) wants every failure path forced by a test. Anywhere else, as in `let n: u32 = s.parse()?;`, coverage cannot see the error side, so the reviewer checks that a test forces it.
- A `?` whose error cannot happen is not a failure path. It is an invariant, written as `.expect("<the invariant>")`, not as a `?` with a test that cannot exist.

The alpha kit's coverage step already turns `--branch` output into diff coverage through Cobertura, so no separate diff tool is needed.

[codex-src]: https://github.com/openai/codex/tree/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/src
[ra-arch]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L416
[ra-style-min]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#minimal-tests
[ra-parser-tests]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/parser/src/tests.rs#L55-L78
[ra-tidy]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/xtask/src/tidy.rs
[ra-readme]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/README.md#ci
[ra-tu]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/test-utils/src/lib.rs#L397-L510
[toml-writer]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_writer/tests/string.rs
[toml-fuzz]: https://github.com/toml-rs/toml/blob/563cb1a5d133e39d1008d4d4148ef35d3bba379f/crates/toml_edit_fuzz/parse_document.rs
[jaq-tests]: https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/tests/common/mod.rs
[ruff-ci]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/.github/workflows/ci.yaml
[ruff-cli]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/tests/cli/main.rs
[ruff-test]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/test.rs#L244-L360
[ruff-parser-fx]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_parser/tests/fixtures.rs
[ruff-fuzz]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/fuzz/README.md
[pt-persist]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/failure-persistence.md
[pt-sm]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/state-machine.md
[pt-tips]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/book/src/proptest/tips-and-best-practices.md
[bd-fuzz]: https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_test_bdd/_test_bdd_logic_fuzzing.rs#L1-L18
[be-bdd]: https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/bdd.rs#L755-L800
[tb-exh]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/src/testing/exhaustigen.zig
[tb-style]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/TIGER_STYLE.md
[tb-vopr]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/internals/vopr.md
[wt-oracles]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oracles.rs#L460-L604
[wt-oom]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oom.rs
[sr-bfs]: https://github.com/stateright-rs/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/bfs.rs#L442-L475
[sqlite-testing]: https://www.sqlite.org/testing.html
[sqlite-malloc]: https://github.com/sqlite/sqlite/blob/33fd659df2a4d2f07071ec61b329b4aa6a7377d5/src/test_malloc.c#L65-L86
[sqlite-int]: https://github.com/sqlite/sqlite/blob/33fd659df2a4d2f07071ec61b329b4aa6a7377d5/src/sqliteInt.h#L492-L563
[cm-visit]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/src/visit.rs#L802-L808
[cm-indiff]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/in-diff.md
[cm-ci]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/ci.md
[cm-config]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/.cargo/mutants.toml
[cm-perf]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/performance.md
[cm-results]: https://github.com/sourcefrog/cargo-mutants/blob/6c720ae82a1784b84cfa06066f7b0d59ed1798d0/book/src/using-results.md
[llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
