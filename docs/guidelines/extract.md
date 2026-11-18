# `extract`: guidelines

**Status: Accepted by the owner on 2026-11-14.** Distilled from [extract_static_analysis.md](../research/implementation-guidelines/notes/extract_static_analysis.md), with parts of [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md). The cross-cutting rules in [README.md](README.md) and [testing.md](testing.md) apply too.

## One job

`extract` turns build files and Python source files into facts, in `model` types: the build model (stage 1 of the [specification](../specification.md#5-stage-1-the-build-model)), then reference sites with their outcomes, the call graph, obligations (branch points), effects, state variables and their write sites, and entry points through pack slots (stage 2). It lists every reference site itself and asks a type checker, ty, what each one points to.

**It never:** calls an LLM; names an abstraction; guesses dynamic behavior (a guess belongs to Link and is marked `heuristic` there); touches the file system in its own code or runs a process; lets a ruff or ty type escape its API; names a library in Rust code; resolves a reference by rules of its own.

`mentalc` reads the repository at its I/O edge, then hands `extract` the file contents as strings. The type checker is the one exception: ty reads the roots and the environment from disk itself, read-only, inside `python/resolve.rs` ([X5](#x5-pure-stages-one-module-each)). This tightens rule 5 of the research note, which let `discover` do I/O itself, so that our own code has a single I/O edge, as the [architecture note](../research/implementation-guidelines/notes/mentalc_architecture.md) asks.

## Module layout

```text
extract/
  src/
    lib.rs          pub fn build_model(inputs) and pub fn extract(files, build model, packs) -> (Facts, Vec<Diagnostic>)
    build.rs        build files as strings -> the build model (build/ holds one reader per format)
    pack.rs         pack rows and the access-path grammar (pack/ for submodules if needed)
    python.rs       the per-language entry point
    python/
      parse.rs      ruff parse_unchecked -> parsed file + syntax diagnostics   (per file)
      sites.rs      parsed file -> reference sites                              (per file)
      resolve.rs    sites + the build unit's roots, version and environment -> outcomes, by ty   (the only cross-file stage)
      flow.rs       one function -> obligations with labeled edges              (per function)
      facts.rs      Resolved + function walks + packs -> calls, effects, writes, entry points
  tests/            fixtures/<category>/<case>.py, fixtures/report_jobs.py, public-API tests
```

A second language later is a sibling of `python/` that emits the same `model` facts. There is no language-neutral layer.

## Dependency allowlist

| Kind | Crates |
|---|---|
| workspace | `model` |
| normal | `ruff_python_parser`, `ruff_python_ast`, `ruff_text_size`, `ruff_source_file` and ty's crates (`ty_python_semantic`, `ty_project` and what they need), all from one pinned commit of the ruff repository and bumped together; `serde`, `serde_json` (pack rows); `toml` (`pyproject.toml`, `uv.lock`, pack files); `thiserror` |
| dev | `insta` (with `glob`, `json`), `proptest`; `model` with the `testkit` feature |

The ruff crates set the minimum Rust version (1.97 at 0.0.22), so the workspace toolchain follows ruff's.

## Rules

### Parser and boundary

#### X1. ruff's parser and ty's crates, pinned to one commit

- **Rule:** parse with ruff's parser and resolve references with ty's crates, all from one pinned commit of the ruff repository. Do not use tree-sitter, stack-graphs, SCIP indexers or a language server, and do not write resolution rules of our own.
- **Why:** a typed, CPython-shaped AST makes a missing case a compile error; ty works on the same syntax tree, in process, and resolved 88.5% of calls on five projects, agreeing with Pyrefly on 99.7% of the sites both answer ([type checkers as libraries](../research/type-checkers-as-libraries/report.md)). The alternatives are untyped, archived, or need another program installed. What these crates pull in (salsa, rayon, rand; 242 crates at ty 0.0.105) is accepted in this crate only, the one exception to [G16](README.md#g16-banned-weight).
- **Source:** [ruff versioning policy][ruff-versioning]; [ruff parser `lib.rs`][ruff-parser]; [stack-graphs README][sg-readme].
- **Enforced by:** tidy tests `extract_ruff_ty_pinned` and `banned_crates`.

#### X2. No ruff or ty type escapes

- **Rule:** public functions take strings and return `model` types; no ruff or ty type appears in a public signature, and no other crate depends on a ruff or ty crate.
- **Why:** ruff's AST changes shape within months (`ExprCompare`, `ExprCall`), and the churn must stay inside one crate.
- **Source:** [ruff versioning policy][ruff-versioning]; the note's measured churn between 0.16.0 and 0.17.5.
- **Enforced by:** rustc `unreachable_pub`; tidy test `crate_graph` (only `extract` lists `ruff_*`).

#### X3. Bump ruff on purpose

- **Rule:** a ruff bump is one change that touches only `extract` and `Cargo.lock`, with every snapshot unchanged or each change reviewed.
- **Why:** the snapshot suite is the safety net that makes an unstable dependency affordable.
- **Source:** the note's churn measurement.
- **Enforced by:** CI check on the PR diff (a ruff change in `Cargo.lock` plus files outside `extract/` fails); reviewer.

#### X4. Always parse_unchecked

- **Rule:** use `parse_unchecked` only; every syntax error becomes an anchored diagnostic, and extraction continues on the recovered tree.
- **Why:** one broken file must not hide the facts of the others; a missing *input* still fails the build.
- **Source:** [ruff `parse_unchecked`, `lib.rs` L289][ruff-parser].
- **Enforced by:** clippy `disallowed-methods` (`ruff_python_parser::parse_module`); property test on every prefix of every fixture ([X21](#x21-broken-input-never-panics)).

### Pipeline

#### X5. Pure stages, one module each

- **Rule:** each stage in the layout is one module with a pure function and its own `tests.rs`; `extract` has no I/O edge of its own. The only reads are ty's, of the roots and the environment, read-only, inside `python/resolve.rs`.
- **Why:** each stage can be snapshotted on its own; PyCG's five shared mutable managers are the counterexample.
- **Source:** [ruff `build_cfg(&[Stmt])`][ruff-cfg]; [PyCG `pycg.py` L161][pycg-main].
- **Enforced by:** clippy `disallowed-methods` with no `#[expect]` allowed in `extract` (tidy test `edges_are_listed`).

#### X6. Per-file stages see one file

- **Rule:** `parse`, `sites` and `flow` take one file or one function; only `resolve` reads across files, and it is one call into the type checker per build unit.
- **Why:** testable alone, cacheable per file later, and no silent `max_iter`.
- **Source:** [stack-graphs README][sg-readme]; [PyCG fixpoint][pycg-main], made explicit.
- **Enforced by:** function signatures (`fn index(file: &ParsedFile) -> ModuleIndex`); reviewer.

#### X7. Lookups return outcomes, never Option

- **Rule:** every lookup returns `Resolved(id) | External(qualified_name) | Unresolved(Reason)`, where `Reason` is the closed enum of the [specification](../specification.md#63-outcomes) (`missing_module`, `no_type`, `not_found`, `ambiguous`, `dynamic`, `unreachable`), and each `Unresolved` becomes a fact. The checker's `Unknown`, `Any` and `Never` are `Unresolved`.
- **Why:** what the extractor can't see is recorded, never dropped; PyCG silently returns on calls it can't resolve.
- **Source:** [ruff `ReadResult`][ruff-readresult]; [PyCG `visit_Call`][pycg-call] (counterexample).
- **Enforced by:** reviewer; the conservation oracle ([X8](#x8-conservation)).

#### X8. Conservation

- **Rule:** every call expression, decorator and branch form in the AST appears in the output exactly once, resolved or not.
- **Why:** this makes "complete" checkable; spec P6 makes extractors own completeness.
- **Source:** the note's candidate rule 8 (no precedent in the studied tools).
- **Enforced by:** oracle test: a separate ~30-line visitor counts calls, decorators and branch statements per file and compares with the emitted facts on every fixture and on report-jobs.

#### X9. Extract never guesses

- **Rule:** no heuristic edges, no name-similarity matching, no "most likely" type. A site's target is what the type checker answers; anything it does not answer is `unresolved`, and `extract` adds no inference of its own on top.
- **Why:** spec P6 and 10: guesses belong to Link, marked `heuristic`.
- **Source:** [ruff `check_type` initializer typing][ruff-typing]; [CodeQL literal-only `getUrlPattern`][codeql-concepts-url].
- **Enforced by:** extract's fact types in `model` have no `heuristic` field; fixtures in the `assignments` category; reviewer rejects flow-sensitive inference.

### Packs and effects

#### X10. No library names in Rust

- **Rule:** framework and library knowledge lives only in pack data files; `extract/src` names no library.
- **Why:** packs are declarative data (spec 6.6); ruff's FastAPI check written in Rust is the counterexample.
- **Source:** [CodeQL models-as-data][codeql-mad]; [ruff `fastapi/rules/mod.rs`][ruff-fastapi].
- **Enforced by:** tidy test `extract_no_library_names` (fails on `fastapi|celery|sqlalchemy|boto|click|redis|requests|kombu|pydantic` outside tests).

#### X11. One small, closed access-path grammar

- **Rule:** pack rows use the tokens `Member[a,b]`, `ReturnValue`, `Argument[i,name:]`, `Parameter[i]`, `Instance`, `Subclass`, plus Mental's `Decorates` and `WithArg[i=literal]`, and the row kinds `type`, `effect`, `slot`, `raises`, `config`. An unknown token or kind is a pack load error.
- **Why:** CodeQL's format is proven; only slots and string-dispatched clients need additions.
- **Source:** [CodeQL `ApiGraphModels.qll`][codeql-mad]; [CodeQL `Asyncpg.model.yml`][codeql-asyncpg].
- **Enforced by:** parser unit tests; round-trip property test (print then parse is identity).

#### X12. Effect kinds are generic

- **Rule:** effect kinds are a closed enum in `model` (`db.read`, `db.write`, `tx.end`, `send:<channel>`, `http`, `file.read`, `file.write`, `clock`, `env`), never library names.
- **Why:** `mentalc` reasons about a database write, not about SQLAlchemy.
- **Source:** [CodeQL concepts][codeql-concepts].
- **Enforced by:** serde rejects unknown values ([M9](model.md#m9-unknown-data-is-an-error)).

### Control flow

#### X13. Obligation kinds come from an exhaustive match

- **Rule:** `flow` classifies every `Stmt` and `Expr` variant explicitly, with no wildcard arm.
- **Why:** a new ruff node kind must fail the build, not vanish; ruff's own half-finished `build_cfg` shows how empty arms hide gaps.
- **Source:** [ruff `cfg/graph.rs` L203-L256][ruff-cfg-arms].
- **Enforced by:** clippy `wildcard_enum_match_arm` denied at the top of `flow.rs` (`#![deny(...)]`).

#### X14. Exception edges follow CodeQL's economy

- **Rule:** exception edges come from explicit `raise`, from calls inside a `try` (one candidate per handler whose type resolves), and from pack `raises` rows; there are no per-call edges outside a `try`. Edge labels are `true`, `false`, `exception(type)`, `exhausted`, `normal`.
- **Why:** keeps obligations small and still finds every handler the code wrote.
- **Source:** [CodeQL `flow.py` edge kinds L41][codeql-flow] and [`in_try` guard L1283-L1350][codeql-flow-try].
- **Enforced by:** one fixture per rule in the `control_flow` and `exceptions` categories; obligation snapshots ([X20](#x20-obligations-render-as-text)).

### Output

#### X15. Every fact carries an anchor

- **Rule:** every fact has a non-optional anchor: file, byte range, and 1-based line and column for display; the range slices to non-empty source text.
- **Why:** spec P7; ruff gives every node a `TextRange`.
- **Source:** [ruff AST `generated.rs`][ruff-ast].
- **Enforced by:** non-optional field in `model`; harness invariant on every fixture ([T7](testing.md#t7-the-harness-checks-invariants-on-every-fixture)).

#### X16. Output is deterministic

- **Rule:** facts are sorted by `(file, offset, kind)`; extracting report-jobs twice gives byte-identical JSON.
- **Why:** facts feed cache keys, so any jitter would cause spurious recompiles.
- **Source:** [ruff lint set with `iter_over_hash_type`][ruff-lints].
- **Enforced by:** clippy `disallowed-types` ([G6](README.md#g6-ordered-collections-only)); determinism test.

#### X17. Out of scope for version one

- **Rule:** none of our own: no concrete syntax tree or trivia, no incremental reparsing, no language server, no query engine, no language-neutral abstraction layer, no taint tracking. ty's query engine is used through its crates ([X1](#x1-ruffs-parser-and-tys-crates-pinned-to-one-commit)), never rebuilt or extended.
- **Why:** stack-graphs (archived) and ty (236k lines) show where these lead.
- **Source:** [stack-graphs README][sg-readme]; [ty `Cargo.toml`][ty-cargo].
- **Enforced by:** tidy test `banned_crates`; reviewer.

### Tests (beyond [testing.md](testing.md))

#### X18. Fixtures carry inline expectations

- **Rule:** one `.py` file per case; multi-file cases use `# --- path: x.py ---` fragments; expectations are `# $ tag=value` comments on the line they describe (or `# ^ tag=value` under a column); known gaps are `MISSING:` and `SPURIOUS:` markers. The harness stays under ~200 lines, discovers files with `insta::glob!`, fails on any unmatched expectation or unexpected fact for a tag in use, and fails on a fixed `MISSING:` so the marker gets removed.
- **Why:** expectations sit next to the code they describe and known gaps stay visible.
- **Source:** [CodeQL inline expectations][codeql-inline]; [stack-graphs test syntax][sg-test].
- **Enforced by:** the harness; reviewer requires a fixture for each new fact kind.

#### X19. Fixtures by category

- **Rule:** micro fixtures live in `tests/fixtures/<category>/<case>.py`, one concern each: imports, assignments, decorators, classes, with, control_flow, exceptions, effects (per pack), config, dynamic (must yield `unresolved`), plus collisions (two same-named handlers, [M5](model.md#m5-names-say-what-never-when-or-where)).
- **Why:** PyCG's categories cover the ground a call-graph extractor gets wrong.
- **Source:** [PyCG micro-benchmark][pycg-bench].
- **Enforced by:** reviewer.

#### X20. Obligations render as text

- **Rule:** per function, obligations are snapshotted as compact text, one branch point per line (`L22 if ... -> [true: raise HTTPException(400)] [false: ...]`), not as JSON.
- **Why:** reads well in review, as ruff's Mermaid CFG snapshots do.
- **Source:** [ruff CFG snapshot tests][ruff-cfg-tests].
- **Enforced by:** inline insta snapshots.

#### X21. Broken input never panics

- **Rule:** every prefix of every fixture file extracts without a panic and yields at least one diagnostic whenever the parser reports a syntax error; error-recovery fixtures with broken Python exist on purpose.
- **Why:** real repositories contain broken files.
- **Source:** [tree-sitter error corpus][ts-errors]; the note's candidate rule 4.
- **Enforced by:** property test; the cargo-fuzz target "Python source to facts" ([T11](testing.md#t11-fuzz-only-readers-of-outside-input)).

#### X22. report-jobs is the acceptance test

- **Rule:** one `insta::assert_json_snapshot!` of the full output for `examples/report-jobs/src`, plus explicit counts: 5 files, 6 entry points (2 routes, 2 Celery tasks, 1 schedule, 1 command), 2 state variables, every branch listed. An annotated copy (`tests/fixtures/report_jobs.py`, fragment headers) carries inline expectations, and a drift test strips the annotations and compares the result with the real example files.
- **Why:** spec 6, F-S10 and milestone 2.
- **Source:** spec 6 and 14.2; [ruff whole-output snapshots][ruff-parser-fx].
- **Enforced by:** public-API test; reviewer approves snapshot diffs. The call-graph edge count is asserted under the definition in spec 6; the trial's 41 is re-measured under it, not copied.

[ruff-versioning]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/docs/versioning.md
[ruff-parser]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_parser/src/lib.rs#L289
[ruff-ast]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_ast/src/generated.rs#L9465
[ruff-cfg]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_semantic/src/cfg/graph.rs
[ruff-cfg-arms]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_semantic/src/cfg/graph.rs#L203
[ruff-cfg-tests]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_semantic/src/cfg/mod.rs
[ruff-readresult]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_semantic/src/model.rs#L3131
[ruff-typing]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_semantic/src/analyze/typing.rs#L631
[ruff-fastapi]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/rules/fastapi/rules/mod.rs#L14
[ruff-lints]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/Cargo.toml#L270
[ruff-parser-fx]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_python_parser/tests/fixtures.rs
[ty-cargo]: https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ty_python_semantic/Cargo.toml
[sg-readme]: https://github.com/github/stack-graphs/blob/fcb7705d5b38ae13b3665a9b2c882e5a97243d44/README.md
[sg-test]: https://github.com/github/stack-graphs/blob/fcb7705d5b38ae13b3665a9b2c882e5a97243d44/tree-sitter-stack-graphs/src/test.rs#L14
[pycg-main]: https://github.com/vitsalis/PyCG/blob/8d5dc40837803beef1d8d379fbf2cdad6cd94641/pycg/pycg.py#L161
[pycg-call]: https://github.com/vitsalis/PyCG/blob/8d5dc40837803beef1d8d379fbf2cdad6cd94641/pycg/processing/cgprocessor.py#L126
[pycg-bench]: https://github.com/vitsalis/PyCG/tree/8d5dc40837803beef1d8d379fbf2cdad6cd94641/micro-benchmark/snippets
[codeql-mad]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/ql/lib/semmle/python/frameworks/data/internal/ApiGraphModels.qll
[codeql-asyncpg]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/ql/lib/semmle/python/frameworks/Asyncpg.model.yml
[codeql-concepts]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/ql/lib/semmle/python/Concepts.qll#L85
[codeql-concepts-url]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/ql/lib/semmle/python/Concepts.qll#L966
[codeql-flow]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/extractor/semmle/python/passes/flow.py#L41
[codeql-flow-try]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/python/extractor/semmle/python/passes/flow.py#L1283
[codeql-inline]: https://github.com/github/codeql/blob/20ed6480fa1d8c44c99368be99dda541082dc968/shared/util/codeql/util/test/InlineExpectationsTest.qll
[ts-errors]: https://github.com/tree-sitter/tree-sitter-python/blob/26855eabccb19c6abf499fbc5b8dc7cc9ab8bc64/test/corpus/errors.txt
