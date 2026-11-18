# Mental implementation guidelines

**Status: Accepted by the owner on 2026-11-14,** together with every proposed specification change, which is now applied to the specification. These are the rules the code of each Rust crate in Mental must follow. They are distilled from six research notes in [`docs/research/implementation-guidelines/notes/`](../research/implementation-guidelines/notes/), each of which read the source of mature projects (rust-analyzer, ruff, SQLite, Stateright, CodeQL, cargo and others) and proposed rules with evidence. Duplicates across notes are merged, weak rules are dropped, and every rule says how it is checked, so the set can later become alpha-kit configuration (principles, clippy settings, CI steps, tidy tests) and be applied to the code.

The owner's goals decide every trade-off: **compact code, no bloat, an architecture simple enough for one human to understand, and outstanding tests.**

## How to read a rule

Each rule is one short block:

- **ID and title.** The prefix names the file: `G` this README, `T` [testing](testing.md), `M` [model](model.md), `X` [extract](extract.md), `C` [mentalc](mentalc.md), `V` [mentalvm](mentalvm.md). IDs are stable; a dropped rule leaves a gap.
- **Rule:** what the code must do, stated so that someone can check it.
- **Why:** the reason, in one sentence.
- **Source:** the studied project and file the rule comes from, linked at the commit the note read. "Counterexample" marks a project that shows what goes wrong without the rule.
- **Enforced by:** one or more of: **clippy** (a named lint or `clippy.toml` setting, failing CI with `-D warnings`), **tidy test** (a `#[test]` in `model/tests/tidy.rs` that reads source, run by `cargo test`), **CI check** (a step in the workflow), **unit or property test**, or **reviewer**.

A rule that only a reviewer can check is kept only when it prevents real bloat or a real bug class.

## Files

| File | Covers |
|---|---|
| [README.md](README.md) | this index; cross-cutting rules G1-G21; mapping to alpha |
| [testing.md](testing.md) | the testing playbook: layers, the `check` helper, fixtures, snapshots, properties, fuzzing, fault injection, mutants, coverage, determinism, speed, tidy tests (T1-T21) |
| [model.md](model.md) | record types, semantic IDs, canonical JSONL, guards, checks, diagnostics (M1-M25) |
| [extract.md](extract.md) | build files and Python to facts: parser, stages, packs, control flow, fixtures (X1-X22) |
| [mentalc.md](mentalc.md) | the compiler: pipeline, LLM loop, backends, cache, ledger, MCP, CLI (C1-C25) |
| [mentalvm.md](mentalvm.md) | the interpreter: step, choices, replay, search, minimizer, WebAssembly (V1-V25) |

## The workspace at a glance

One Cargo workspace at the repository root, with flat top-level crate folders:

```text
model/      library   record format, IDs, guards, JSONL, checks, Diagnostic     deps: serde, serde_json, blake3
extract/    library   build files, Python -> build model, facts; no LLM   deps: model, ruff, ty, ...
mentalc/    binary    pipeline, LLM tasks and backends, cache, ledger, checks, MCP; packs/ and prompts/ embedded
mentalvm/   library + thin binary   step, run, explore, replay; core does no I/O and compiles to WebAssembly

model <- extract <- mentalc
model <- mentalvm                 mentalc and mentalvm never depend on each other, only on files
```

Already decided by the owner: no async runtime in version one; clap derive; thiserror in libraries and context in binaries; JSON or JSONL for tool output, TOML for human-written input, no YAML; unit tests in a `tests.rs` beside each module and public-API tests in `tests/`; insta for snapshots; proptest with committed regressions; cargo-mutants on the PR diff; 100% diff line and branch coverage; the hand-written reference model at `examples/report-jobs/model/` as the golden file.

Also decided by the owner on 2026-11-14, when these guidelines were accepted:

- `extract` does no I/O of its own: `mentalc` reads the repository, then hands `extract` strings; ty alone reads the roots and the environment itself ([X5](extract.md#x5-pure-stages-one-module-each));
- `model` uses no `thiserror`: its two error types have hand-written `Display`, so its allowlist stays `serde`, `serde_json`, `blake3` (an exception to the "thiserror in libraries" rule);
- `blake3` is the only hash in the workspace;
- `mentalc` adds `anyhow` (error context in the binary) and `tempfile` (atomic writes of the cache and ledger) to its allowlist;
- `mentalvm`'s `clap` and `anyhow` sit behind a default `cli` feature, so its core builds for WebAssembly without them;
- CI adds nextest, `cargo insta test --check` and `cargo fuzz build`;
- any duplicate ID in a model is an error, even when the two lines are identical ([M6](model.md#m6-identity-is-the-key));
- the format rules of the [specification](../specification.md#7-the-model): full IDs only, `equals` on enum predicates, one format version, one file per kind, no stored kind.

**I/O edges.** Only these files may touch the file system, processes, network, environment, clock or stdio ([G4](#g4-io-and-ambient-inputs-only-at-named-edges)):

| Crate | Edge files |
|---|---|
| `model` | none |
| `extract` | none of its own (`mentalc` hands it file contents; ty reads the roots and the environment, read-only) |
| `mentalc` | `main.rs`, `cli.rs`, `repo.rs`, `cache.rs`, `ledger.rs`, `backend/api.rs` |
| `mentalvm` | `main.rs` |

Test code (`tests/`, `tests.rs`) may also opt out, for temp directories and fixture files.

## Cross-cutting rules

### Architecture

#### G1. Fixed crate graph

- **Rule:** the workspace has exactly the edges `model <- extract <- mentalc` and `model <- mentalvm`; `model` depends on no workspace crate, and `mentalc` and `mentalvm` exchange only files in the model format.
- **Why:** each tool can be built, tested and replaced alone (spec 3).
- **Source:** [rust-analyzer style.md, "scale of changes"][ra-style-scale]; spec 3.
- **Enforced by:** tidy test `crate_graph`, parsing each member's `Cargo.toml`.

#### G2. Small public surface

- **Rule:** items are private or `pub(crate)` unless another crate uses them; no crate re-exports another crate's items with `pub use`; a new `pub` item or workspace edge is reviewed as an API change.
- **Why:** "adding an innocent-looking `pub use` is a very simple way to break encapsulation".
- **Source:** [rust-analyzer style.md L16-L52][ra-style-scale].
- **Enforced by:** rustc lint `unreachable_pub`; tidy test `no_foreign_reexports`; reviewer.

#### G3. Functions over frameworks

- **Rule:** write the flow as plain function calls over typed data: no doer objects (`ThingDoer::new(..).do()`), no pass managers, registries or plugin systems, and no trait with a single implementation; the LLM `Backend` trait is the one allowed exception.
- **Why:** the data flow stays visible in one place, and one human can follow it.
- **Source:** [rust-analyzer "Functions Over Objects", style.md L350-L404][ra-style-fn]; [typst `compile_impl`][typst-compile].
- **Enforced by:** tidy test `traits_have_two_impls`; reviewer.

### Side effects

#### G4. I/O and ambient inputs only at named edges

- **Rule:** only the edge files listed above call the file system, processes, network, environment, clock, threads or stdio; everywhere else these calls are clippy errors, and an edge file opts out with `#[expect(clippy::disallowed_methods, reason = "...")]`. Each edge file, and test code, has a row in the Exceptions table of [CONTRIBUTING.md](../../CONTRIBUTING.md#exceptions) naming the path, the lint and why; a new edge is a new row there and in the list above.
- **Why:** pure code is testable from a string fixture and reproducible, and the boundary is machine-checked rather than remembered.
- **Source:** [rust-analyzer architecture.md L451-L457][ra-arch-core]; [typst `World` trait][typst-world].
- **Enforced by:** clippy `disallowed-methods` in `clippy.toml`; tidy test `edges_are_listed`.

#### G5. Input problems are data

- **Rule:** analyses return `(T, Vec<Diagnostic>)` and keep going after a problem in their input; `Result` is reserved for failures of the tool itself (I/O, config, backend, a cache miss offline).
- **Why:** the user sees every problem at once, and the core cannot fail.
- **Source:** [rust-analyzer architecture.md L88 and L451-L457][ra-arch-core]; [typst `Warned<SourceResult<T>>`][typst-compile-fn].
- **Enforced by:** reviewer; G4 keeps I/O errors out of the core structurally.

### Determinism

#### G6. Ordered collections only

- **Rule:** no `HashMap` or `HashSet` anywhere; use `BTreeMap`, `BTreeSet` or a sorted `Vec`, and sort every list before it reaches output.
- **Why:** hash iteration order leaks into files, IDs and schedules; Stateright had to add sorting after it broke path rebuilding.
- **Source:** [Stateright `actor/model.rs` L342-L386][sr-sort]; [ruff lint `iter_over_hash_type`][ruff-lints].
- **Enforced by:** clippy `disallowed-types`; determinism tests ([T18](testing.md#t18-deterministic-and-hermetic)).

#### G7. Stable hashes

- **Rule:** every hash that is stored, printed or compared across processes is blake3 over canonical bytes (sorted keys, no floats), never `std::hash::Hash`, `DefaultHasher` or ahash.
- **Why:** `Hash` output is not portable across platforms and Rust versions, and ahash says its output differs by machine.
- **Source:** [ruff `cache_key.rs`][ruff-cachekey]; [Stateright "stable" ahash fingerprint, `lib.rs` L344-L390][sr-fp].
- **Enforced by:** clippy `disallowed-types` (`DefaultHasher`); tidy test `lockfile_allowlist` (blake3 is the only hashing crate); property tests in [C12](mentalc.md#c12-the-cache-key-is-content-never-the-llm).

#### G8. Seeds and clocks are parameters

- **Rule:** code that needs time or randomness takes `now` or a `seed` as a parameter; only an edge reads the clock or entropy, and a command that picks a seed prints it.
- **Why:** runs and tests reproduce bit for bit.
- **Source:** [Stateright `StdRng` FIXME][sr-rng]; [Shuttle seeded `Pcg64Mcg`][sh-pct]; [typst `World::today`][typst-world].
- **Enforced by:** clippy `disallowed-methods` (`SystemTime::now`, `Instant::now`); reviewer.

### Errors, diagnostics and exit codes

#### G9. Errors by layer

- **Rule:** a library returns its own error enum (thiserror; `model` writes its two by hand, see [model.md](model.md#dependency-allowlist)); a binary adds anyhow context to each error saying what failed and how to fix it; messages start lowercase and do not end with a period; `unwrap` appears only in tests and `expect` only for an invariant it names.
- **Why:** errors read as a chain of causes and tell the user what to do next.
- **Source:** [rust-analyzer style.md L853-L864][ra-style-err]; [ruff `main.rs` error chain][ruff-main]; alpha rule `errors-with-context`.
- **Enforced by:** clippy `unwrap_used` (alpha); reviewer.

#### G10. One diagnostic type, two renderers

- **Rule:** every problem any tool reports is a `model::Diagnostic` with a stable `Code`; it is rendered either as human text or as the model's own JSON lines, and in no other format.
- **Why:** `mentalvm` and `mentaldbg` read `diagnostics.jsonl`; ruff's twelve output formats are bloat here.
- **Source:** [ruff `Diagnostic` and `render/`][ruff-diag]; [typst `SourceDiagnostic`][typst-diag].
- **Enforced by:** exhaustive `match` on `Code` in the renderer; reviewer.

#### G11. Fail rather than degrade

- **Rule:** a missing input or key, an unknown field or format version, a cache miss in offline or `check` mode, and exhausted repairs end the command with an error that says how to fix it; no code path writes a partial model or falls back silently.
- **Why:** a degraded model is later read as complete (spec P9).
- **Source:** [Specula fail-open stop gate][specula-gate] (counterexample); [ruff's silent cache reset][ruff-cache-evict] (deliberately not copied).
- **Enforced by:** one test per failure path ([T12](testing.md#t12-inject-faults-through-seams-that-already-exist)); reviewer.

#### G12. Exit codes and a thin main

- **Rule:** binaries exit 0 on success, 1 when the command ran and found error-level problems (failed checks, repairs exhausted, a violation found by search), and 2 when the tool itself failed (I/O, config, backend, offline cache miss, bad usage); a broken pipe exits quietly. `main.rs` only parses arguments, calls `run`, and writes the error chain to stderr.
- **Why:** scripts and agents branch on exit codes, and the library stays testable without a subprocess.
- **Source:** [ruff `ExitStatus`, `lib.rs` L37-L56][ruff-exit]; [ruff `main.rs`][ruff-main].
- **Enforced by:** `insta_cmd` snapshots record every exit code; clippy `print_stdout`, `print_stderr` and `exit`, with `#[expect]` only in the CLI edge files.

### Formats

#### G13. JSON out, TOML in, no YAML

- **Rule:** tool output is JSON or JSONL; human-written input (config, packs) is TOML; every command has a `--json` form; no YAML crate is used.
- **Why:** one format per direction, and the spec's text model is JSONL (spec 7.5).
- **Source:** owner decision; spec 7.5.
- **Enforced by:** tidy test `banned_crates`.

#### G14. Serializable types are contracts

- **Rule:** `Serialize` and `Deserialize` are derived only on file formats and boundary types: model records, run files, the ledger, cache entries, packs, config, answer types, and CLI and MCP view structs. Phase-internal types never derive them.
- **Why:** a serializable type is a compatibility promise; keep the promise surface small.
- **Source:** [rust-analyzer architecture.md L505-L517][ra-arch-serde]; [ruff `JsonDiagnostic` view][ruff-json].
- **Enforced by:** tidy test `serde_derives_in_allowed_modules`.

### Dependencies and size

#### G15. Every dependency justified once

- **Rule:** each dependency appears once in `[workspace.dependencies]` with a one-line comment above it saying why; members name it as `foo.workspace = true`; each crate uses only its allowlist (in its guideline file); a new crate, transitive ones included, is a reviewed change.
- **Why:** compile time, fewer things to break, and a codebase one person can own.
- **Source:** [rust-analyzer dependency policy, style.md L55-L62][ra-style-deps]; alpha rule `cargo-for-dependencies`.
- **Enforced by:** tidy tests `dependencies_justified` and `lockfile_allowlist` (Cargo.lock package names against a checked-in `deps.allow`); `cargo machete` (alpha).

#### G16. Banned weight

- **Rule:** no `async fn` and none of: `tokio` or any async runtime, `salsa`, `comemo`, `rmcp`, `tree-sitter*`, `ruff_python_semantic`, `jsonschema`, `reqwest`, `mockall`, `expect-test`, `serde_yaml`, `rand`, `ahash`. **One exception:** what the pinned commit of ruff's and ty's crates pulls into `extract` ([X1](extract.md#x1-ruffs-parser-and-tys-crates-pinned-to-one-commit)) is accepted there, salsa, rayon and rand included (242 crates in all at ty 0.0.105). The list holds for every other dependency path, and `model` and `mentalvm` stay free of those crates.
- **Why:** each listed crate solves a problem Mental doesn't have at a cost it can't afford (rmcp: 61 crates and mandatory tokio for 3 tools; Salsa: 27.7k lines; jsonschema: 64 crates). ty's tree is the price of resolving references with a real type checker instead of rules of our own (spec 6.2), and it is paid in one crate.
- **Source:** [rmcp `Cargo.toml`][rmcp-cargo]; [Salsa `Cargo.toml`][salsa-cargo]; [rust-analyzer dependency policy][ra-style-deps]; owner decision of 2026-11-18.
- **Enforced by:** tidy tests `banned_crates` (packages in `Cargo.lock` reachable only through ruff's and ty's crates are skipped; the dependency trees of `model` and `mentalvm` must hold none of the list) and `no_async`.

#### G17. Size limits

- **Rule:** functions at most ~60 lines, non-test source files at most ~400 lines, at most 5 parameters; split a function rather than pass a literal `bool` or `Option` to select behavior.
- **Why:** small units are what one person can hold in their head.
- **Source:** [rust-analyzer "Prefer Separate Functions Over Parameters", style.md L466-L497][ra-style-params]; [rust-analyzer config structs, style.md L406-L445][ra-style-config].
- **Enforced by:** clippy `too_many_lines` (threshold 60), `too_many_arguments` (threshold 5), `fn_params_excessive_bools`; tidy test `file_length`; reviewer for literal arguments.

#### G18. One lint configuration

- **Rule:** lints are set once in the workspace `Cargo.toml` and `clippy.toml` (the alpha baseline plus the settings below); crate-specific lints are inner attributes at a crate or module root; a suppression is `#[expect(lint, reason = "...")]`, never `#[allow]`, and has a row in the Exceptions table of [CONTRIBUTING.md](../../CONTRIBUTING.md#exceptions) naming the path, the lint and why.
- **Why:** one command (`cargo clippy -- -D warnings` plus `cargo test`) checks everything, and a stale suppression fails.
- **Source:** [rust-analyzer workspace lints, `Cargo.toml` L174-L237][ra-lints]; alpha's Rust recipe.
- **Enforced by:** CI `cargo clippy --all-targets -- -D warnings`; alpha's generated `tests/setup.rs`.

#### G19. No non_exhaustive inside the workspace

- **Rule:** no `#[non_exhaustive]` on any type.
- **Why:** it exists for published crates with outside users; inside one workspace it forces `_` arms and hides missing cases.
- **Source:** [SCIP `SymbolError`][scip-symerr] and cargo-platform's error enum (published crates).
- **Enforced by:** tidy test `no_non_exhaustive`.

### Style

#### G20. Plain data and concrete signatures

- **Rule:** a field is public unless an invariant protects it; there are no setters, and getters return borrows; settings travel in a `Config` struct passed explicitly, never stored in long-lived state and without `Default`; functions across crate boundaries take concrete types, not generic parameters or `AsRef` polymorphism.
- **Why:** less code, local reasoning, and no monomorphization cost at crate edges.
- **Source:** [rust-analyzer getters and setters, style.md L267-L298][ra-style-getters]; [config, L406-L445][ra-style-config]; [generics, L591-L625][ra-style-generics].
- **Enforced by:** reviewer.

#### G21. Preconditions in types

- **Rule:** express preconditions as types (`Id`, `Slug`, a sorted `Model`) and parse outside input into them once, at the boundary; push branching up to the caller instead of hiding it in callees; inner code does not re-validate.
- **Why:** the compiler carries the proof, and each check exists exactly once.
- **Source:** [rust-analyzer style.md L179-L258][ra-style-pre]; alpha principle `validate-at-edges` and rule `typed-boundaries`.
- **Enforced by:** reviewer.

## Configuration sketch

What G4, G6-G8, G12 and G17-G18 add to the alpha Rust template. Lint names and `clippy.toml` keys are real; the exact lists must be verified on the pinned toolchain when the workspace is created.

```toml
# Cargo.toml (workspace), in addition to alpha's [workspace.lints]
[workspace.lints.rust]
unreachable_pub = "warn"              # G2
elided_lifetimes_in_paths = "warn"

[workspace.lints.clippy]
print_stdout = "warn"                 # G12: printing only in CLI edge files
print_stderr = "warn"
exit = "warn"                         # G12: main returns an ExitCode
str_to_string = "warn"
```

```toml
# clippy.toml, in addition to alpha's allow-unwrap-in-tests and allow-dbg-in-tests
too-many-lines-threshold = 60         # G17
too-many-arguments-threshold = 5      # G17
disallowed-types = [
  { path = "std::collections::HashMap", reason = "G6: use BTreeMap" },
  { path = "std::collections::HashSet", reason = "G6: use BTreeSet" },
  { path = "std::collections::hash_map::DefaultHasher", reason = "G7: use blake3 over canonical bytes" },
]
disallowed-methods = [
  # G4: I/O only in edge files (list every std::fs free function the code base could use)
  { path = "std::fs::read", reason = "G4" }, { path = "std::fs::read_to_string", reason = "G4" },
  { path = "std::fs::write", reason = "G4" }, { path = "std::fs::read_dir", reason = "G4" },
  { path = "std::fs::create_dir_all", reason = "G4" }, { path = "std::fs::rename", reason = "G4" },
  { path = "std::fs::File::open", reason = "G4" }, { path = "std::fs::File::create", reason = "G4" },
  { path = "std::process::Command::new", reason = "G4" }, { path = "std::net::TcpStream::connect", reason = "G4" },
  { path = "std::env::var", reason = "G4" }, { path = "std::env::args", reason = "G4" },
  { path = "std::io::stdin", reason = "G4" }, { path = "std::io::stdout", reason = "G4" },
  { path = "std::thread::spawn", reason = "G4" },
  # G8: time is a parameter
  { path = "std::time::SystemTime::now", reason = "G8" }, { path = "std::time::Instant::now", reason = "G8" },
  # X4
  { path = "ruff_python_parser::parse_module", reason = "X4: use parse_unchecked" },
]
```

Crate-level additions, as inner attributes: `#![deny(clippy::wildcard_enum_match_arm)]` at the top of `model/src/lib.rs` and `extract/src/python/flow.rs` (M4, X13), and `#![warn(clippy::panic)]` in `mentalvm/src/lib.rs` (V4).

## Mapping to alpha

How each rule would enter the alpha kit. **Goes to** is the main home:

- `principle:<id>`: a project principle in [`docs/alpha/principles.toml`](../alpha/principles.toml), where the eight below are registered for alpha's reviewer (`(alpha)` marks one the kit already has);
- `clippy:<setting>`: a lint in the workspace `Cargo.toml` or a key in `clippy.toml`;
- `tidy:<test>`: a test in `model/tests/tidy.rs` ([T20](testing.md#t20-tidy-tests-run-inside-cargo-test));
- `ci:<step>`: a step in the CI workflow (`ci:diff-coverage` and `ci:mutants` are alpha's existing gates);
- `reviewer`: checked only in review.

**Also** lists the ordinary tests written with the code and any secondary check.

### Project principles

Ten principles carry the rules that are statements of intent; the rest stay as rules in these files. Two of the ten exist in the alpha kit already (`pure-core`, `exhaustive-testing`). A few rules also point to alpha core principles that every project has (`validate-at-edges`, `tests-first-class`); those need no new entry.

| Principle id | Title | Carries |
|---|---|---|
| `fixed-crate-graph` | Four crates, one direction | G1, G2 |
| `pure-core` (alpha) | Logic in the middle, side effects at the edges | G4, G5, C2, C3, X5, V1, V16 |
| `deterministic-output` | Same inputs, same bytes | G6, G7, G8, M7, X16, V6, T18 |
| `fail-rather-than-degrade` | A partial result is an error | G11, M9, M10, C11, C13 |
| `no-bloat` | Every line and crate earns its place | G3, G15, G16, G17, M20, X17, V18 |
| `one-canonical-form` | One text form per identity | M1, M2, M3, M7 |
| `extractors-never-guess` | Unknown is recorded, never guessed or dropped | X7, X8, X9 |
| `one-judge-for-every-backend` | Every LLM answer passes the same checks | C4, C9, C11 |
| `semantics-stay-in-the-step` | Reductions live in strategies, never in the step | V3, V5 |
| `exhaustive-testing` (alpha) | Exhaustive testing | T1-T21 |

### Rules

| Rule | Goes to | Also |
|---|---|---|
| G1 Fixed crate graph | tidy:crate_graph | principle:fixed-crate-graph |
| G2 Small public surface | clippy:unreachable_pub | tidy:no_foreign_reexports; reviewer |
| G3 Functions over frameworks | tidy:traits_have_two_impls | principle:no-bloat; reviewer |
| G4 I/O only at named edges | clippy:disallowed-methods | tidy:edges_are_listed; principle:pure-core |
| G5 Input problems are data | principle:pure-core | reviewer |
| G6 Ordered collections only | clippy:disallowed-types | determinism tests |
| G7 Stable hashes | clippy:disallowed-types | tidy:lockfile_allowlist; property tests |
| G8 Seeds and clocks are parameters | clippy:disallowed-methods | principle:deterministic-output |
| G9 Errors by layer | clippy:unwrap_used (alpha) | alpha rule errors-with-context; reviewer |
| G10 One diagnostic type, two renderers | reviewer | exhaustive match on `Code` |
| G11 Fail rather than degrade | principle:fail-rather-than-degrade | one test per failure path |
| G12 Exit codes and a thin main | clippy:print_stdout, print_stderr, exit | insta_cmd exit-code snapshots |
| G13 JSON out, TOML in, no YAML | tidy:banned_crates | reviewer |
| G14 Serializable types are contracts | tidy:serde_derives_in_allowed_modules | |
| G15 Every dependency justified once | tidy:dependencies_justified, lockfile_allowlist | ci: cargo machete (alpha) |
| G16 Banned weight | tidy:banned_crates, no_async | |
| G17 Size limits | clippy:too-many-lines-threshold, too-many-arguments-threshold, fn_params_excessive_bools | tidy:file_length |
| G18 One lint configuration | ci: cargo clippy -D warnings | alpha tests/setup.rs |
| G19 No non_exhaustive | tidy:no_non_exhaustive | |
| G20 Plain data and concrete signatures | reviewer | |
| G21 Preconditions in types | principle:validate-at-edges (alpha) | reviewer |
| T1 Where tests live | tidy:test_modules_are_files | alpha rule tests-beside-code |
| T2 One check helper per module | reviewer | principle:tests-first-class (alpha) |
| T3 Minimal fixtures | tidy:no_orphan_fixtures | reviewer |
| T4 No should_panic, no ignore | tidy:no_should_panic_or_ignore | |
| T5 Snapshots with insta | ci: INSTA_UPDATE=no cargo insta test --check --unreferenced=reject | reviewer |
| T6 The golden is never a snapshot | tidy:no_snapshots_into_examples | ci: tree unchanged after tests |
| T7 Harness checks invariants | reviewer | `model::testkit` helper |
| T8 Reproducible property tests | ci: tree clean after tests | reviewer |
| T9 Enumerate small domains | reviewer | |
| T10 Dumb oracles | reviewer | ci:mutants |
| T11 Fuzz only readers | ci: cargo fuzz build; timed run on change | reviewer |
| T12 Faults through existing seams | reviewer | tidy:banned_crates; ci:diff-coverage |
| T13 Limits at their edges | ci:mutants | reviewer |
| T14 A fix starts with a failing test | reviewer | principle:exhaustive-testing (alpha) |
| T15 Invariants asserted in code | reviewer | open spike |
| T16 Mutation testing on the diff | ci:mutants (alpha) | tidy:mutants_skip_has_reason; alpha rule mutants-skips |
| T17 Full coverage of every change | ci:diff-coverage (alpha) | |
| T18 Deterministic and hermetic | clippy:disallowed-methods | determinism tests; ci: no key, no network |
| T19 A speed budget | ci: nextest ci profile with timeouts | |
| T20 Tidy tests in cargo test | tidy (the file itself) | |
| T21 Test what ships | ci: release and wasm test jobs | |
| M1 IDs enter only through the parser | principle:one-canonical-form | unit test; reviewer |
| M2 One canonical text form | principle:one-canonical-form | property test |
| M3 One slug alphabet | principle:one-canonical-form | unit table; property test |
| M4 Closed kinds | reviewer | wrong-kind fixtures; clippy:wildcard_enum_match_arm (crate attribute) |
| M5 Names say what | reviewer | parser test; extract collision fixture |
| M6 Identity is the key | reviewer | checker fixture |
| M7 Canonical JSONL | principle:deterministic-output | golden; round-trip properties |
| M8 Sets are sorted vectors | reviewer | `unsorted-set` diagnostic |
| M9 Unknown data is an error | tidy:model_records_deny_unknown_fields, model_no_json_value | unit test |
| M10 One format version | principle:fail-rather-than-degrade | unit test |
| M11 Identity apart from anchors and hashes | reviewer | unit test |
| M12 One pure pass, never stops early | principle:pure-core | snapshot |
| M13 Closed, complete diagnostic codes | reviewer | completeness test |
| M14 Complete reference check | reviewer | one fixture per sub-rule |
| M15 Only full IDs are stored | reviewer | guard-parser test; spec change 4 |
| M16 Parse errors carry input, span, reason | reviewer | `bad()` tables |
| M17 A tiny guard grammar | reviewer | tables; round-trip property |
| M18 Partition by enumeration with a cap | reviewer | witness tests; oracle |
| M19 One evaluator for two and three values | reviewer | property test |
| M20 Small public surface | principle:no-bloat | tidy:dependencies_justified |
| M21 Parser tables through serde | reviewer | ci:diff-coverage |
| M22 Format frozen by a golden | ci: insta check | |
| M23 Brute-force guard oracle | reviewer | ci:mutants |
| M24 Reference model fixture suite | reviewer | completeness test |
| M25 A testkit, not a fifth crate | reviewer | |
| X1 ruff's parser and ty's crates, pinned | tidy:extract_ruff_ty_pinned, banned_crates | |
| X2 No ruff type escapes | tidy:crate_graph | clippy:unreachable_pub |
| X3 Bump ruff on purpose | ci: ruff bump touches only extract | reviewer |
| X4 Always parse_unchecked | clippy:disallowed-methods | property test |
| X5 Pure stages | clippy:disallowed-methods | tidy:edges_are_listed; principle:pure-core |
| X6 Per-file stages | reviewer | signatures |
| X7 Outcomes, never Option | principle:extractors-never-guess | reviewer |
| X8 Conservation | principle:extractors-never-guess | oracle test |
| X9 Extract never guesses | principle:extractors-never-guess | fixtures; reviewer |
| X10 No library names in Rust | tidy:extract_no_library_names | |
| X11 Closed access-path grammar | reviewer | parser tests; round-trip property |
| X12 Generic effect kinds | reviewer | serde rejects unknown values |
| X13 Exhaustive obligation match | clippy:wildcard_enum_match_arm (module attribute) | |
| X14 Exception-edge economy | reviewer | fixtures; snapshots |
| X15 Every fact anchored | reviewer | harness invariant |
| X16 Deterministic output | principle:deterministic-output | determinism test |
| X17 Out of scope for v1 | principle:no-bloat | tidy:banned_crates |
| X18 Inline expectations | reviewer | the harness |
| X19 Fixtures by category | reviewer | |
| X20 Obligations as text | reviewer | inline snapshots |
| X21 Broken input never panics | reviewer | property test; fuzz target |
| X22 report-jobs acceptance | reviewer | acceptance test |
| C1 One pipeline function | reviewer | phase-order test |
| C2 Pure phases | clippy:disallowed-methods | principle:pure-core |
| C3 Plan and apply | principle:pure-core | reviewer |
| C4 One judge for every backend | principle:one-judge-for-every-backend | protocol property test |
| C5 Synchronous backend | tidy:banned_crates, no_async | spec change 1 |
| C6 Small blocking API client | reviewer | request snapshot; parsing and retry tests |
| C7 Answer types are schema and validator | reviewer | schema subset test; property test |
| C8 Prompts are versioned data | reviewer | prompts.lock unit test |
| C9 Checks re-run on every hit | principle:one-judge-for-every-backend | unit test |
| C10 A repair names every failure | reviewer | repair snapshots |
| C11 Repairs capped, fail closed | principle:fail-rather-than-degrade | limit tests; ci:mutants |
| C12 Content cache key | reviewer | property tests; golden key |
| C13 Cache keeps paid answers safe | principle:fail-rather-than-degrade | cache unit tests |
| C14 Every recompile explained | reviewer | CLI snapshot |
| C15 Append-only ledger | reviewer | crash property test |
| C16 Hand-written dual-era MCP | tidy:banned_crates | transcript tests; release smoke check |
| C17 Fixed tool list | reviewer | tools/list snapshot |
| C18 Typed CLI, two renderers | clippy:print_stdout, print_stderr | CLI snapshots |
| C19 Phase check helpers | reviewer | |
| C20 No live model in tests | ci: no API key, no network | ci:diff-coverage |
| C21 Countdown sweep | reviewer | sweep helper |
| C22 Few, filtered CLI snapshots | reviewer | |
| C23 Pull protocol at three layers | reviewer | ci:mutants |
| C24 Golden report-jobs build | ci: debug and release acceptance jobs | |
| C25 Golden tasks | ci: insta check | |
| V1 Configurations are values | principle:pure-core | public-API snapshot |
| V2 Cheap, ordered clones | clippy:disallowed-types | tidy:mentalvm_no_interior_mutability; property test |
| V3 The step prunes nothing | principle:semantics-stay-in-the-step | property test |
| V4 Outcomes are data | clippy:panic (crate attribute), unwrap_used | corpus |
| V5 Named choices with values | principle:semantics-stay-in-the-step | changed-model replay test |
| V6 enabled is sorted | principle:deterministic-output | debug_assert; determinism test |
| V7 One chooser, one run loop | reviewer | tidy:traits_have_two_impls |
| V8 Replay verdicts | reviewer | one test per verdict |
| V9 Bounded DFS, exact visited set | reviewer | pinned counts; spec change 3 |
| V10 Preemptions counted as loom does | reviewer | unit tests |
| V11 Zero-time cycles | reviewer | corpus model |
| V12 Every loop has a hard cap | reviewer | limit tests; ci:mutants |
| V13 Seeded, optional sampling | clippy:disallowed-methods | same-seed test |
| V14 Delete-one minimizer | reviewer | property test; snapshot |
| V15 No POR in v1 | reviewer | spec change 3 |
| V16 Core builds for WebAssembly | ci: wasm32 build | clippy:disallowed-methods; principle:pure-core |
| V17 Run files are JSON | reviewer | round-trip property; fuzz target |
| V18 Version one scope | principle:no-bloat | tidy:dependencies_justified |
| V19 One scenario per step rule | reviewer | |
| V20 Corpus of bug and correct models | ci: corpus tests | reviewer |
| V21 Witnesses checked by replay | reviewer | `assert_witness` |
| V22 Replay round trips gate CI | tidy:no_should_panic_or_ignore | CI |
| V23 Determinism and differential checks | ci: determinism tests | property tests |
| V24 Properties and pinned counts | reviewer | inline snapshots |
| V25 Tested on the shipped target | ci: wasm test job | |

[ra-style-scale]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L16
[ra-style-deps]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L55
[ra-style-pre]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L179
[ra-style-getters]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L267
[ra-style-fn]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L350
[ra-style-config]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L406
[ra-style-params]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L466
[ra-style-generics]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L591
[ra-style-err]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L853
[ra-arch-core]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L451
[ra-arch-serde]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L505
[ra-lints]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/Cargo.toml#L174
[typst-compile]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst/src/lib.rs#L100
[typst-compile-fn]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst/src/lib.rs#L75
[typst-world]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/lib.rs#L63
[typst-diag]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/diag.rs#L427
[ruff-lints]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/Cargo.toml#L270
[ruff-cachekey]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_cache/src/cache_key.rs#L1
[ruff-cache-evict]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L100
[ruff-main]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/main.rs
[ruff-exit]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/lib.rs#L37
[ruff-diag]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/mod.rs#L29
[ruff-json]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/render/json.rs#L47
[sr-sort]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L342-L386
[sr-fp]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/lib.rs#L344-L390
[sr-rng]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/simulation.rs#L46-L48
[sh-pct]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/pct.rs#L50
[specula-gate]: https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/stop_gate.py
[rmcp-cargo]: https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/Cargo.toml
[salsa-cargo]: https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/Cargo.toml#L20
[scip-symerr]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L7-L16
