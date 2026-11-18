# `model`: guidelines

**Status: Accepted by the owner on 2026-11-14.** Distilled from [model_record_format.md](../research/implementation-guidelines/notes/model_record_format.md), with parts of [mentalc_architecture.md](../research/implementation-guidelines/notes/mentalc_architecture.md) and [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md). The cross-cutting rules in [README.md](README.md) and [testing.md](testing.md) apply too.

## One job

`model` defines the record format and decides whether a set of records is well formed. It holds:

- the record types, one struct per kind (spec 7.4);
- semantic IDs (`Id`, `Kind`, the slug alphabet);
- anchors;
- guard expressions, their evaluator and the partition check;
- reading and writing canonical sorted JSONL;
- the structural, reference and anchor checks;
- the `Diagnostic` record that every tool reports with.

**It never:** touches the file system, the clock or the network (callers pass strings in and get strings out); knows about Python, packs, LLMs, tasks or repairs; runs a model. It depends on no workspace crate.

## Module layout

```text
model/
  src/
    lib.rs          the public surface and FORMAT_VERSION
    id.rs           Id, Kind, the slug alphabet, slugify        (+ id/tests.rs)
    record.rs       the Record trait (fn id(&self) -> &Id) and the Model struct
    record/         build.rs  structure.rs  state.rs  behavior.rs  above.rs (term, level, ...)
    anchor.rs
    guard.rs        Expr, parse, Display, the Logic trait, eval, the partition check
    jsonl.rs        read(&str) and write(&[T]) -> String
    check.rs        check(&Model, &Sources) -> Vec<Diagnostic>
    check/          refs.rs  sets.rs  anchors.rs  guards.rs
    diagnostic.rs   Diagnostic, Code, Severity
    testkit.rs      behind the `testkit` feature: the fixture DSL and the invariant check
  tests/            public-API tests; data/ok/ and data/err/<code>/ fixtures; tidy.rs
```

Each module's unit tests sit in a `tests.rs` beside it (`id/tests.rs`, `guard/tests.rs`, and so on).

## Dependency allowlist

| Kind | Crates |
|---|---|
| normal | `serde` (with `derive`), `serde_json`, `blake3` |
| dev | `insta`, `proptest` |

`thiserror` is **not** on the list, although the README's error rule ([G9](README.md#g9-errors-by-layer)) names it for libraries. `model` has two small error types (`IdError`, `ParseError`); they implement `Display` and `std::error::Error` by hand, in about 20 lines. The owner approved this exception on 2026-11-14.

## Rules

### Identity

#### M1. IDs enter only through the parser

- **Rule:** `Id` is a newtype with a private `String`; it is built only by `Id::parse` or `Id::child(&self, Kind, &Slug)`, and its `Deserialize` calls `Id::parse`. No crate formats an ID with `format!`.
- **Why:** an invalid ID can then enter neither through serde nor through ad-hoc string building.
- **Source:** [text-size `Deserialize for TextRange`][ts-serde]; [cargo-platform `Deserialize` via `FromStr`][cg-cfg].
- **Enforced by:** unit test (a bad ID fails with the same message through `parse` and through `serde_json`); reviewer.

#### M2. One canonical text form

- **Rule:** every accepted ID and guard re-displays to the same bytes: `parse(s)?.to_string() == s`. Input that is not canonical is rejected, never normalized.
- **Why:** one text form per identity makes sorting, diffs and text tools reliable.
- **Source:** [SCIP `lintSymbolString` / `nonCanonicalSymbolError`][scip-lint].
- **Enforced by:** property test over arbitrary strings: accepted implies identical re-display.

#### M3. One slug alphabet, no escaping

- **Rule:** slugs match `[a-z0-9]+(-[a-z0-9]+)*`, and a single function defines that alphabet for `slugify`, the parser and the checker. `slugify(code_name)` is the only path from a code name to a slug.
- **Why:** Mental builds slugs itself, so escaping is pure bloat; SCIP's Rust parser, Rust formatter and Go binding each accept a different language because they use different character predicates.
- **Source:** [SCIP Rust `symbol.rs` L296-L324][scip-sym] versus [SCIP Go `identifier.go`][scip-go-id].
- **Enforced by:** `good`/`bad` unit table; property test that `slugify(x)` always parses.

#### M4. Kinds are closed and every reference declares one

- **Rule:** the kind prefix is a closed `Kind` enum; every ID-valued field documents the `Kind` it must hold, and the checker enforces it.
- **Why:** a reference type check becomes a prefix check, as in Glean's typed references.
- **Source:** [Glean `validate.cpp` "fact type mismatch"][gl-validate].
- **Enforced by:** one wrong-kind fixture per reference field; clippy `wildcard_enum_match_arm` denied at the crate root, so every `match` on `Kind` or `Code` stays exhaustive.

#### M5. Names say what, never when or where

- **Rule:** IDs contain no positions, counters, line numbers, hashes, versions or timestamps, and a slug made only of digits is rejected. When two code names collide, add another code name (module, file stem or class) to the colliding IDs only, never an index. The one exception is reference sites (spec 6.1, owner decision of 2026-11-18). Sites are grouped by scope, kind and the slug of the name with any trailing segments made only of digits removed, so `s.commit()` and `s.commit_0()` fall in one group; every site in a group of two or more gets its order in the group, counted from 0, appended to its own slug.
- **Why:** IDs stay stable across rebuilds (P8), and unrelated code changes don't rename things.
- **Source:** [Kythe `storage.proto` L85-L90][ky-vname]; [cargo lockfile "only when necessary to disambiguate"][cg-lockfile-ref].
- **Enforced by:** parser rule and unit test; an `extract` fixture with two same-named handlers ([X19](extract.md#x19-fixtures-by-category)); a site fixture with three identical calls in one function, and one with `s.commit()` twice and `s.commit_0()` once.

#### M6. Identity is the key

- **Rule:** two records, cases or classes with the same ID are a `duplicate-id` diagnostic naming both, even when their lines are byte-identical. A node repeated across the cases of one behavior is not a duplicate, but every appearance must carry the same fields (`node-mismatch`).
- **Why:** Mental has one writer, so any duplicate is a writer bug; conflicting facts under one identity mean a bug upstream. Glean merges identical facts because it has many writers, and the owner chose the stricter rule on 2026-11-14.
- **Source:** [Glean `FactSet::define` L278-L293][gl-factset], as the counterexample for a single-writer format.
- **Enforced by:** checker fixture.

### Format

#### M7. Canonical JSONL

- **Rule:** one file per kind, one record per line, lines sorted by ID in byte order, fields in struct declaration order, `None` and empty sets omitted, a final `\n`, nothing else in the file. `write` is a pure function returning a `String`; the caller rewrites a file only when its bytes changed. `Id`'s `Ord` equals byte order of its string.
- **Why:** the model is byte-deterministic and diffable by text tools and LLMs (spec 7.5).
- **Source:** [Cargo.lock emitter and sorts, `lockfile.rs` L138-L243][cg-lockfile].
- **Enforced by:** insta golden of a small model ([M22](#m22-the-format-is-frozen-by-a-golden)); property tests `read(write(m)) == m` and `write(read(b)) == b`; unit test that `Id` order equals `str` order.

#### M8. Sets are sorted vectors

- **Rule:** set-like fields (`checks`, `tags`, `decides`, `evidence`) are sorted, deduplicated `Vec`s; ordered `Vec`s are used only where order has meaning (`steps`, `transitions`).
- **Why:** forced order on an unordered set makes output non-deterministic and diffs spurious.
- **Source:** [Glean schema design advice][gl-design].
- **Enforced by:** checker diagnostic `unsorted-set`; the writer sorts too.

#### M9. Unknown data is an error

- **Rule:** every record struct carries `#[serde(deny_unknown_fields)]`, an unknown kind is an error, and no record has a catch-all field (`serde_json::Value`, an `extra` or `metadata` map).
- **Why:** the model has one writer, versioned with the compiler, so skipping unknown data would hide version skew ("fail rather than degrade"); Cargo regrets its `[metadata]` map. This departs on purpose from the cargo index and protobuf, which have many writers.
- **Source:** contrast with [cargo index `mod.rs` L577-L600][cg-index]; [cargo `encode.rs` module docs][cg-encode].
- **Enforced by:** tidy tests `model_records_deny_unknown_fields` and `model_no_json_value`; unit test that an extra field is rejected.

#### M10. One format version, no migrations

- **Rule:** the whole model carries one format version; a mismatch is a hard error telling the user to rebuild. There is no per-record version and no migration code.
- **Why:** the model is derived and always rebuildable; multi-version readers are machinery Mental doesn't need.
- **Source:** [Cargo.lock `version` marker, `encode.rs` L1-L110][cg-encode].
- **Enforced by:** unit test that reading a future version fails with the "rebuild" message.

#### M11. Identity stays apart from anchors and hashes

- **Rule:** anchors live in `anchors.jsonl`; content hashes are blake3 over canonical line bytes, stored beside a record, never inside its `Id`; display names never feed IDs.
- **Why:** moving code must not change IDs, and a cache key must not pose as an identity.
- **Source:** spec 7.2 and 7.3; [Kythe `storage.proto` L56-L70][ky-names].
- **Enforced by:** unit test that moving an anchor changes only the bytes of `anchors.jsonl`; reviewer.

### Checks

#### M12. Checking is one pure pass that never stops early

- **Rule:** `check(&Model, &Sources) -> Vec<Diagnostic>` reports every problem, sorted by `(id, code, field)` and deduplicated; `Sources` is the file text the caller already read.
- **Why:** the user sees every problem at once, and the order doesn't change when message wording does.
- **Source:** [SCIP `errorSet.Unique()`][scip-lint-set]; Kythe's first-error `ValidEntry` is the counterexample.
- **Enforced by:** snapshot of the diagnostics for a fixture with many errors.

#### M13. Diagnostic codes are closed and complete

- **Rule:** `Code` is a closed enum serialized as a stable kebab-case string; each code fixes its kind (finding, gap, assumption, check failed) and severity (`error`, `warning`, `info`) by an exhaustive `match`. A `Diagnostic` carries `code`, `id`, `field` (a JSON path such as `cases[1].when`), `message`, `related` IDs and an optional anchor. Repair tasks are not part of it; `mentalc` maps codes to repairs.
- **Why:** agents, tests and CI filter by code; the record stays free of LLM concepts.
- **Source:** [ruff `Diagnostic::new`][ruff-diag]; [typst `SourceDiagnostic`][typst-diag]; [cfg-expr `ParseError`][ce-error].
- **Enforced by:** completeness test: `const ALL: &[Code]` with an exhaustive `match`, and each code has a fixture in `tests/data/err/<code>/`.

#### M14. The reference check is complete

- **Rule:** for every ID-valued field the check reports, as separate codes: does not parse, does not exist, wrong kind, parent missing, duplicate.
- **Why:** dangling and mistyped references are the main structural failure.
- **Source:** [Glean `validate.cpp` L66-L89][gl-validate]; [SCIP lint errors][scip-lint-set].
- **Enforced by:** one fixture per sub-rule.

#### M15. Only full IDs are stored

- **Rule:** stored records hold fully qualified IDs everywhere, guards and node references included; short forms such as `pred:x` or `.../case:y` exist only in human-facing display.
- **Why:** the reference check stays textual, with no scope-resolution rule to get wrong. Accepted and written into spec 7.2.
- **Source:** inferred; no studied project resolves parent-relative references (SCIP's `local` is file-scoped).
- **Enforced by:** the guard parser accepts only full IDs (unit test).

#### M16. Parse errors carry input, span and reason

- **Rule:** parse errors for IDs and guards hold the original input, a byte span and a reason enum; tests compare whole messages.
- **Why:** short inputs need no source map to give a precise message.
- **Source:** [cfg-expr `error.rs` and `tests/parser.rs`][ce-error]; [cargo-platform `bad()`][cg-test-cfg].
- **Enforced by:** `bad(s, msg)` tables ([M21](#m21-parser-tables-run-through-serde-too)).

### Guards

#### M17. A tiny guard grammar

- **Rule:** guards use `!`, `&`, `|`, parentheses and full predicate IDs, parsed into `enum Expr { Pred(Id), Not(Box<Expr>), And(Vec<Expr>), Or(Vec<Expr>) }` with a canonical `Display` (single spaces around `&` and `|`, minimal parentheses). A predicate that does not exist is a diagnostic, never `false`.
- **Why:** the spec's guards need nothing more; boolean_expression's `unwrap_or(false)` turns a typo into a silently dead case.
- **Source:** [cargo-platform `CfgExpr`][cg-cfg-expr]; [boolean_expression `expr.rs` L129-L137][be-expr].
- **Enforced by:** `good`/`bad` tables; round-trip property test; fixture `unknown-predicate`.

#### M18. Partition by enumeration with a cap

- **Rule:** the partition check enumerates every assignment of the n dimensions a behavior's guards mention (a plain predicate is one yes-or-no dimension; a state variable tested by predicates with `equals` is one dimension over its domain values, so impossible combinations are never enumerated), reports the lowest gap or overlap as a witness assignment, and above n = 16 emits `guard-check-too-large` instead of skipping. No BDD or SAT. This is the guard-algebra check of spec 9.1.
- **Why:** about 15 lines, deterministic, with concrete messages; the BDD crates are 2,400 to 10,600 lines with forbidden dependencies.
- **Source:** truth-table oracles in [boolean_expression `bdd.rs` L755-L800][be-bdd] and [biodivine fuzz tests][bd-fuzz].
- **Enforced by:** unit tests with known witnesses; insta snapshot of the messages; oracle test ([M23](#m23-the-guard-checker-has-a-brute-force-oracle)).

#### M19. One evaluator for two and three values

- **Rule:** `Expr::eval` is generic over a `Logic` trait implemented for `bool` and for `Option<bool>` (Kleene logic), so `mentalvm`'s "not decided yet" uses the same code.
- **Why:** during a run a predicate has no value until the run decides it (φ, the predicate values assumed so far, in spec 12.1), so the interpreter evaluates guards in three values; one evaluator serves the checker and the interpreter.
- **Source:** [cfg-expr `Logic` trait][ce-expr].
- **Enforced by:** property test: with every predicate `Some(_)`, `Option<bool>` evaluation equals `bool` evaluation.

### API

#### M20. A small public surface without IDE machinery

- **Rule:** the public surface is `Id`, `Kind`, `Slug`, one struct per record kind, `Model` (one `Vec<T>` per kind, sorted by ID, looked up by binary search), `read`/`write`, `check`, `Expr`, `Diagnostic`. No arenas, interners, `SmallVec`, `smol_str` or index maps in version one.
- **Why:** a few thousand records need none of it, and each would add a dependency; a 30-line typed index in la-arena's style is the upgrade path if profiling ever asks.
- **Source:** [la-arena][ra-arena] and `PathInterner` (rejected); [SCIP sorted list and binary search][scip-sort].
- **Enforced by:** dependency allowlist (tidy test `dependencies_justified`); reviewer.

### Tests (beyond [testing.md](testing.md))

#### M21. Parser tables run through serde too

- **Rule:** every parser (`Id`, `Slug`, `Expr`) has a table of `good(s)`, `bad(s, exact_message)` and, only if some normalization is ever accepted, `good_canon(s, canon)`; each case also runs through `serde_json`.
- **Why:** proves text parsing and deserialization accept the same language with the same messages.
- **Source:** [cargo-platform `test_cfg.rs` L49-L96][cg-test-cfg].
- **Enforced by:** reviewer; diff branch coverage shows missing error branches.

#### M22. The format is frozen by a golden

- **Rule:** an insta snapshot of the JSONL written for a small hand-built model that covers every record kind; any byte change to the format is a reviewed snapshot diff. The hand-written report-jobs reference model, which grows by milestone, is checked by `check` and never rewritten ([T6](testing.md#t6-the-golden-is-never-a-snapshot)).
- **Why:** format changes become visible decisions.
- **Source:** [SCIP reprolang snapshots][scip-repro]; Cargo.lock compatibility tests.
- **Enforced by:** CI with `INSTA_UPDATE=no`.

#### M23. The guard checker has a brute-force oracle

- **Rule:** a test-only evaluator over all 2^n assignments is the specification: every guard set over at most 3 predicates is checked exhaustively, larger sets (n ≤ 10) by proptest, and every reported witness must really be a gap (no guard true) or an overlap (two or more true).
- **Why:** clever code needs an independent oracle, and witnesses must be checked, not just verdicts.
- **Source:** [boolean_expression `bdd.rs` L755-L800][be-bdd]; [biodivine fixed-seed fuzzing][bd-fuzz].
- **Enforced by:** property test; mutation testing.

#### M24. The reference model is a fixture suite

- **Rule:** the hand-written report-jobs reference model passes `check` with no diagnostics, and `tests/data/err/<code>/` holds one seeded-error model per diagnostic code, each producing exactly that code.
- **Why:** the reference model is what a build is compared against ([C24](mentalc.md#c24-the-golden-report-jobs-build)), so it must itself pass every check, and every diagnostic code needs a model that shows it firing.
- **Source:** [rust-analyzer `parse_ok`/`parse_err`][ra-parser-tests].
- **Enforced by:** public-API test in `tests/`; completeness test of [M13](#m13-diagnostic-codes-are-closed-and-complete).

#### M25. A testkit, not a fifth crate

- **Rule:** the fixture DSL (a tiny model written as one string) and the invariant check that every harness runs (IDs unique and sorted, references resolve, write idempotent) live in `model::testkit` behind a `testkit` feature that only dev-dependencies enable.
- **Why:** `extract`, `mentalc` and `mentalvm` reuse one harness without a new crate.
- **Source:** [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md), per-crate notes; [ruff harness invariants][ruff-test].
- **Enforced by:** reviewer.

Fuzz targets for the JSONL reader and the guard parser follow [T11](testing.md#t11-fuzz-only-readers-of-outside-input).

## Open questions for the owner

- **Where the format version lives.** Either a tiny `.mental/model/format.json` or a header line in each JSONL file. The first keeps every JSONL file pure records, as M7 requires; it is the recommendation.
- **Dependent predicates.** Predicates over one enum variable (`status-is-pending`, `status-is-done`) can't both be true, so plain enumeration reports spurious overlaps and gaps. The check needs a rule, such as declaring such predicates an "exactly one" group and skipping infeasible assignments. No studied source settles it.
- **Thresholds.** n ≤ 16 and "a few thousand records" are estimates; check them against the first real build of report-jobs.

[ts-serde]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/src/serde_impls.rs
[cg-cfg]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/cfg.rs#L178-L230
[cg-cfg-expr]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/cfg.rs#L8-L16
[scip-lint]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/cmd/scip/lint.go#L249-L264
[scip-lint-set]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/cmd/scip/lint.go#L414-L435
[scip-sym]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L296-L324
[scip-go-id]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/identifier.go#L3-L5
[scip-sort]: https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/sort.go#L25
[scip-repro]: https://github.com/sourcegraph/scip/tree/be1d9f769a02a11be3b3ec5485231e2531a78a1c/reprolang/testdata/snapshots
[gl-validate]: https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/validate.cpp#L66-L89
[gl-factset]: https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/factset.cpp#L278-L293
[gl-design]: https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/schema/design.md
[ky-vname]: https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L85-L90
[ky-names]: https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L56-L70
[cg-lockfile-ref]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-util-schemas/src/lockfile.rs#L210-L212
[cg-lockfile]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/ops/lockfile.rs#L138-L243
[cg-encode]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/resolver/encode.rs
[cg-index]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/sources/registry/index/mod.rs#L577-L600
[cg-test-cfg]: https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/tests/test_cfg.rs#L49-L96
[ce-error]: https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/error.rs#L5-L13
[ce-expr]: https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/expr.rs#L735-L826
[be-expr]: https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/expr.rs#L129-L137
[be-bdd]: https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/bdd.rs#L755-L800
[bd-fuzz]: https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_test_bdd/_test_bdd_logic_fuzzing.rs#L1-L18
[ra-arena]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/la-arena/src/lib.rs#L59-L105
[ra-parser-tests]: https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/parser/src/tests.rs#L55-L78
[ruff-diag]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/mod.rs#L29-L80
[ruff-test]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/test.rs#L244-L360
[typst-diag]: https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/diag.rs#L427-L454
