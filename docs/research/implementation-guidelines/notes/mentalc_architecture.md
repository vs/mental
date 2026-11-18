# mentalc architecture: phase pipeline, content-hash cache, diagnostics, CLI, tests

Studied from source (shallow clones, read only, nothing built or run), 2026-11-14:

- **rust-analyzer** (`114dcb8d6f50aca1ef14d67b831a7560362f8f69`): the Rust IDE engine; best-written architecture and style docs in the ecosystem, data-driven fixture tests.
- **Salsa** (`52b12a418909c9d982a842691c6eaacfc6d62936`, v0.28.5): the query-based incremental framework under rust-analyzer and ty; ~27.7k lines in `src/` + `components/`.
- **ruff** (`b140af2f48bddf33a1ae984174af3180236aaaaf`): Astral's Python linter (plus `ty`, a Salsa-based type checker, in the same repo); simple on-disk lint cache, `ExitStatus`, diagnostics renderers, `insta_cmd` CLI snapshots.
- **typst** (`84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03`) with **comemo** (`5944487f5e9d11949038662df4b307ec56f153b3`): a typesetting compiler; IO behind a `World` trait, in-memory constrained memoization (comemo is ~1.6k lines). typst itself is large (~154k lines), so it shows clean architecture, not small size.
- **jaq** (`c6cc7d3721ccfb212f153006c4fc90c56567540c`): a jq clone; the compact counterexample (core ~6.3k lines, whole workspace ~14.7k), a lex, parse, compile, run pipeline and one-line data-driven tests.
- **ninja** (`1a8fee23135e57cda12d47aedb19f795353fa13c`): build tool; read only for its rule for when to rebuild: inputs newer, or the command hash changed, plus `restat`, which skips rebuilding what comes after.

uv was not studied; ruff covers the Astral patterns relevant here (see Gaps).

URL prefixes used below (all pinned to the commits above):
RA = `https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69`,
SALSA = `https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936`,
RUFF = `https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf`,
TYPST = `https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03`,
COMEMO = `https://github.com/typst/comemo/blob/5944487f5e9d11949038662df4b307ec56f153b3`,
JAQ = `https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c`,
NINJA = `https://github.com/ninja-build/ninja/blob/1a8fee23135e57cda12d47aedb19f795353fa13c`.

## 1. How should the phase pipeline be expressed: pure typed functions, side effects only at the edges?

### Takeaway

All the clean codebases converge on the same pattern: a core that never does IO and cannot fail, which returns `(value, diagnostics)`. Around it is a thin outer layer that does IO and holds the only `Result`s that matter. The pipeline is plain sequential function calls in one driver function, with no pass-manager framework. For `mentalc` the one real complication is the LLM. Each LLM phase can stay pure if it is split into "plan tasks" (pure) and "apply answers" (pure), with the driver doing cache lookups and backend calls between them.

### Cited Findings

- rust-analyzer: "core parts of rust-analyzer (`ide`/`hir`) don't interact with the outside world and thus can't fail. Only parts touching LSP are allowed to do IO." Analyses "compute `(T, Vec<Error>)` rather than `Result<T, Error>`" — [RA architecture.md L451-457](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L451)
- rust-analyzer: "parsing never fails, the parser produces `(T, Vec<Error>)` rather than `Result<T, Error>`" — [RA architecture.md L88](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L88)
- rust-analyzer: "The analyzer keeps all this input data in memory and never does any IO"; the input is `(PathBuf, String)` pairs plus a `CrateGraph` (the "ground state"); everything else is derived state — [RA architecture.md L25-33](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L25)
- rust-analyzer: `base-db` "doesn't know about file system and file paths. Files are represented with opaque `FileId`" — [RA architecture.md L150-151](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L150)
- rust-analyzer: build-system particularities "are *not* the part of the ground state ... `base-db` knows nothing about cargo" — [RA architecture.md L144-148](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L144)
- rust-analyzer: "the server is stateless, a-la HTTP ... the second request should include enough info to re-create the context from scratch" — [RA architecture.md L241-245](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L241)
- rust-analyzer: "`syntax` crate is completely independent from the rest of rust-analyzer. It knows nothing about salsa or LSP"; "syntax tree is a value type" — [RA architecture.md L118-126](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L118)
- typst: every external access (sources, files, fonts, *the current date*) goes through `trait World` (`library`, `book`, `main`, `source`, `file`, `font`, `today`); the compiler crates never touch the file system or clock directly — [TYPST crates/typst-library/src/lib.rs L63-101](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/lib.rs#L63)
- typst: `pub fn compile<T>(world: &dyn World) -> Warned<SourceResult<T>>`: errors in a `Result`, warnings collected in a `Sink` and returned beside it — [TYPST crates/typst/src/lib.rs L75-83](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst/src/lib.rs#L75)
- typst: `compile_impl` is a straight sequence of plain calls (fetch main source, then `typst_eval::eval(...)`, then a bounded relayout loop "until all introspections stabilize ... within five attempts"); there is no phase registry — [TYPST crates/typst/src/lib.rs L100-140](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst/src/lib.rs#L100)
- jaq: the public pipeline is spelled out in the crate doc as explicit steps: `Loader::new(defs)` → `loader.load(&arena, program)` (lex and parse) → `Compiler::default().with_funs(funs).compile(modules)` → `filter.id.run((ctx, input))`. `jaq-core` is `#![no_std]`, `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]` — [JAQ jaq-core/src/lib.rs L1-50](https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/src/lib.rs#L1)
- rust-analyzer style, "Functions Over Objects": "Avoid creating 'doer' objects ... `do_thing(arg1, arg2)` GOOD, `ThingDoer::new(arg1, arg2).do()` BAD"; an internal `Ctx` struct is fine as an implementation detail — [RA style.md L350-404](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L350)
- rust-analyzer style: "Express function preconditions in types and force the caller to provide them"; "do not hide control flow inside functions, push it to the caller" (push ifs up) — [RA style.md L179-258](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L179)
- rust-analyzer style: avoid many optional or boolean parameters and use a `Config` struct; "Do not implement `Default` for the `Config` struct ... Do not store `Config` as a part of the `state`, pass it explicitly" — [RA style.md L406-445](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L406)
- rust-analyzer: "the event loop accepts an `enum` of possible events (closed)" rather than callbacks, so "It's easy to see all the things that trigger rust-analyzer processing" — [RA architecture.md L470-472](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L470)
- ruff: `lib.rs` exposes `pub fn run(args: Args) -> Result<ExitStatus>`, and `run_command` is one `match command { ... }` dispatching to one function per subcommand. `main.rs` only parses args, calls `run`, and maps errors to an exit code — [RUFF crates/ruff/src/lib.rs L128-212](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/lib.rs#L128), [RUFF crates/ruff/src/main.rs](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/main.rs)

### Inferences

- **The pipeline is one function, not a framework.** `mentalc::build` should read like the spec table in 6.2: twelve `let x = phase(&a, &b);` lines, each phase a free function in its own module with a typed output struct. A `trait Phase` or a dynamic pass registry gives nothing when there are exactly twelve fixed phases in a fixed order, and it hides the data flow that the spec's Reads/Writes columns make explicit. (Based on typst `compile_impl`, jaq's explicit chain, and RA "functions over objects".)
- **Each phase takes only what its "Reads" column names** (borrowed typed outputs of earlier phases) and returns `(Output, Vec<Diagnostic>)`. No phase receives a god-context holding everything. An internal `Ctx` inside a phase is fine (RA L364-383).
- **One `World`-like edge for IO.** The non-LLM phases should read the repository through an in-memory snapshot (`RepoSnapshot { files: BTreeMap<RelPath, Bytes>, ... }`, loaded once, as RA's ground state) or through a small `trait Repo`. Then a phase never calls `std::fs`, `std::env` or the clock, and tests feed it a fixture string. typst putting `today()` in `World` is the precedent for treating the clock as input too (the call ledger's timestamps belong to the driver).
- **LLM phases as resumable pure steps.** The compile phase (and vocabulary, group names, mechanisms, business level, repairs) can be written as two pure functions: `plan(inputs) -> Vec<Task>` and `apply(inputs, answers: &Answers) -> (Output, Vec<Diagnostic>)`. The driver sits between them: for each task it computes the key, looks in the cache, and on a miss asks the backend. Bottom-up compilation along the call graph means a unit's task depends on its callees' summaries, so `plan` is iterated level by level: plan the leaves, apply, plan the next level. Everything except the driver loop stays a pure, snapshot-testable function, which fits "no async runtime in v1".
- **Stateless commands, as in RA's "stateless a-la HTTP".** With the cache keyed by content, `mentalc task next` can rerun the pipeline from scratch, which is cheap because the non-LLM phases are fast on small repos, until the first cache miss, and print that task. `task submit` checks the answer, stores it in the cache, then behaves like `task next`. The only build state on disk is the cache, plus repair-attempt counts, which can also be keyed by task hash. This avoids a separate mutable "build session" file that could drift. (The LLM loop itself is covered by the separate study; this is only the architectural consequence.)
- The repair loop (phase 12 sends repairs back) stays in the driver: `check` is pure and returns diagnostics, each optionally carrying a `RepairTask`. The driver bounds the number of rounds.

### Gaps

- None of the studied projects has an external oracle like an LLM inside its pipeline. The plan/apply split is my derivation from the pure-core pattern, not something copied from a codebase.

## 2. What is the minimal incremental cache that meets spec 6.8? Salsa vs a hand-written hash map on disk?

### Takeaway

Use a hand-written, content-addressed answer store on disk: about 100 to 200 lines, one file per validated LLM answer under `.mental/cache/`, keyed by a stable cryptographic hash of the task's canonical bytes. Salsa solves a different problem: fine-grained, in-memory, revision-based recomputation inside a long-running process. Its persistence is a prototype feature, and its size (~27.7k lines, 15+ dependencies, proc macros, unwinding-based cancellation) is out of proportion for a batch CLI whose costly step is a few dozen LLM calls. Content addressing also gives "early cutoff" (Salsa's backdating, ninja's `restat`) for free.

### Cited Findings

- Salsa's model: inputs plus pure memoized "query" functions in a database, recomputed when inputs change; "Salsa assumes that `your_program` is a purely deterministic function of its inputs, or else this whole setup makes no sense" — [SALSA book/src/overview.md](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/book/src/overview.md), [SALSA book/src/how_salsa_works.md](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/book/src/how_salsa_works.md)
- Salsa structs "are just integers": a generated `#[salsa::input]` struct is a newtyped `salsa::Id`, and data lives in the database — [SALSA book/src/overview.md](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/book/src/overview.md)
- Salsa size and weight: 27,719 lines of Rust in `src/` and `components/`; required dependencies include `boxcar`, `crossbeam-queue`, `crossbeam-utils`, `hashbrown`, `hashlink`, `indexmap`, `intrusive-collections`, `parking_lot`, `portable-atomic`, `rustc-hash`, `smallvec`, `thin-vec`, `tracing`; default features pull `rayon`, `inventory`, and the proc-macro crate — [SALSA Cargo.toml L20-71](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/Cargo.toml#L20) (line count measured locally with `wc -l`)
- Salsa persistence is opt-in (`persistence = ["dep:serde", "dep:erased-serde", ...]`), and the changelog calls it an "Initial persistent caching prototype (#967)" followed by "Persistent caching fixes (#982)" — [SALSA Cargo.toml L62-67](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/Cargo.toml#L62), [SALSA CHANGELOG.md L297-306](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/CHANGELOG.md#L297)
- Salsa durability exists to skip revalidation work in a long-running process: "Typically, one assigns low durabilities to inputs that the user is frequently editing" — [SALSA src/durability.rs L1-20](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/src/durability.rs#L1)
- Salsa backdating (early cutoff): when a recomputed value equals the old memo, `changed_at` is moved back so dependents are not re-executed. The code carries a caveat: "We've seen issues where queries weren't re-validated when backdating provisional values in ty. This is more of a bandaid ..." — [SALSA src/function/backdate.rs L15-59](https://github.com/salsa-rs/salsa/blob/52b12a418909c9d982a842691c6eaacfc6d62936/src/function/backdate.rs#L15)
- rust-analyzer cancellation relies on Salsa panicking with a special value: "rust-analyzer requires unwinding" — [RA architecture.md L377-388](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L377)
- rust-analyzer's incrementality invariant is about an interactive editor: "typing inside a function's body never invalidates global derived data" — [RA architecture.md L173-175](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L173)
- In the same Astral repo, the long-running type checker `ty` uses Salsa (`ty_python_semantic/Cargo.toml` depends on `salsa`), while the batch linter `ruff` does not: it has a separate small file cache in `crates/ruff/src/cache.rs` — [RUFF crates/ty_python_semantic/Cargo.toml L46](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ty_python_semantic/Cargo.toml#L46), [RUFF crates/ruff/src/cache.rs](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs)
- ruff cache design: one cache file per package at `cache_dir/VERSION/<hash(package_root, settings)>`. Per-file entries hold a `u64` key plus small result data (`linted`, `formatted`). The per-file key is **mtime + permissions, not content** (`FileCacheKey { file_last_modified, file_permissions_mode }`). The tool version and settings are folded into the directory path, so changing them makes a fresh cache — [RUFF crates/ruff/src/cache.rs L30-56, L94-98, L337-371](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L30)
- ruff caches only clean results: `let linted = diagnostics.is_empty() && use_fixes; cache.set_linted(...)`. A test asserts that a file with a diagnostic is not added — [RUFF crates/ruff/src/diagnostics.rs L365-366](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/diagnostics.rs#L365), [RUFF crates/ruff/src/cache.rs L691](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L691)
- ruff writes atomically: "Write the cache to a temporary file first and then rename it for an 'atomic' write. Protects against data loss if the process is killed during the write and races between different ruff processes" — [RUFF crates/ruff/src/cache.rs L157-199](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L157)
- ruff treats an unreadable or corrupt cache as empty and warns (`warn_user!("Failed parse cache file ...")`), and evicts entries not seen for 30 days (`MAX_LAST_SEEN = 720h`) — [RUFF crates/ruff/src/cache.rs L100-133, L202-247](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L100)
- ruff's `CacheKey` trait exists apart from `Hash` because "Cache keys must be deterministic where hash keys do not have this constraint" and "Ideally, cache keys are portable"; it also warns that "the [`Hash`] implementation may not be portable". The hasher is seahash (64-bit, non-cryptographic) — [RUFF crates/ruff_cache/src/cache_key.rs L1-80](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_cache/src/cache_key.rs#L1)
- ruff tests the cache's invalidation rules directly: `cache_invalidated_on_file_modified_time`, `cache_invalidated_on_permission_change`, `cache_removes_stale_files_on_persist`, `file_changes_invalidate_file_cache` — [RUFF crates/ruff/src/cache.rs L640-1010](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L640)
- comemo (typst) is in-memory only: no serde and no file-system code in `src/`. typst-cli calls `comemo::evict(10)` in `watch` mode after each recompile, so a one-shot `typst compile` gets no reuse across processes — [COMEMO src/memoize.rs L100](https://github.com/typst/comemo/blob/5944487f5e9d11949038662df4b307ec56f153b3/src/memoize.rs#L100), [TYPST crates/typst-cli/src/watch.rs L83](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-cli/src/watch.rs#L83)
- comemo's idea is "constrained memoization": a memoized function's result is reused as long as the specific parts of `Tracked` arguments it actually read are unchanged — [COMEMO README.md](https://github.com/typst/comemo/blob/5944487f5e9d11949038662df4b307ec56f153b3/README.md)
- ninja marks an output dirty when it is older than its newest input *or* when the recorded `command_hash` differs from the current command. `restat` rules let unchanged outputs stop rebuilding further down. `-d explain` records a human-readable reason for every rebuild ("output %s older than most recent input %s") — [NINJA src/graph.cc L225-262](https://github.com/ninja-build/ninja/blob/1a8fee23135e57cda12d47aedb19f795353fa13c/src/graph.cc#L225), [NINJA src/build_log.h L54-78](https://github.com/ninja-build/ninja/blob/1a8fee23135e57cda12d47aedb19f795353fa13c/src/build_log.h#L54)

### Inferences

- **Why Salsa is wrong for Mental v1:**
  1. Each `mentalc` command is a short-lived process. With the `agent` backend, every `task next` and `task submit` is a *new* process, so an in-memory memo table such as Salsa's or comemo's is thrown away between calls. All the value is in persistence across processes, which in Salsa is a prototype.
  2. The costly operations are LLM calls (15 for a first build of report-jobs, 3 after a one-line fix, spec 6.4). The non-LLM phases on a 5-file repo take milliseconds and can simply rerun. Salsa's fine-grained dependency tracking optimizes the cheap part.
  3. LLM answers are not deterministic, which breaks Salsa's core assumption. Mental needs the reverse: the stored answer *is* the fact, and reuse is how a build stays reproducible.
  4. Salsa brings ~27.7k lines, more than a dozen transitive dependencies, proc macros, interned-ID indirection, and unwinding-based cancellation. That conflicts with "compact code, no bloat, a human can understand it".
  5. Even Astral, which maintains Salsa-based ty, kept ruff's batch CLI on a simple file cache.
- **What the minimal cache is:**
  - `key = blake3(canonical_json({cache_format, role, prompt.template, prompt.version, prompt.hash, answer_schema_hash, inputs}))`. Inputs are code slices *by content*, callee summaries *by content*, and the glossary slice *by content*. The LLM identity is deliberately excluded (spec 6.6).
  - `value =` the validated answer JSON, at `.mental/cache/<k[0..2]>/<k>.json`, written by temp file and rename (ruff L157-199).
  - API is roughly `fn get(&self, key: &TaskKey) -> Option<Answer>` and `fn put(&self, key: &TaskKey, answer: &Answer) -> io::Result<()>`.
  - No invalidation logic is needed: a changed input produces a different key, and the old entry is simply never asked for.
- **Early cutoff comes for free.** Callee summaries enter a caller's key by content. So when a recompiled callee yields an identical summary, every caller's key is unchanged and nothing above it recompiles. This is what Salsa's backdating and ninja's `restat` buy, without revision counters.
- **Store only answers that passed checks**, as ruff caches only clean results. A cached answer that later fails a *re-run* check (for example, the anchor quote moved) is a repair trigger, not a silent reuse.
- **Do not copy ruff's leniency on loss.** ruff can drop a corrupt cache or evict after 30 days because a ruff cache entry is cheap to recompute and deterministic. A Mental cache entry is a paid, non-deterministic LLM judgment, and the model is "a function of ... the LLM answers" (spec 4.9). Rules for Mental: no automatic time-based eviction (garbage collection only by an explicit command or by `build` pruning keys unreachable from the current model). A corrupt entry is reported, then treated as a miss. In a no-LLM mode (`mentalc check`, CI replay) a miss is a hard error, never a silent recompile. This also follows the owner's "fail rather than degrade" rule.
- **Use a cryptographic hash over canonical bytes, not `std::hash::Hash`.** ruff itself warns that `Hash` is not portable. Mental's keys also appear in the call ledger (`input_hash`) and may be shared between machines later (spec 12), so they must be stable across platforms and Rust versions. One `canonical_json` function, with sorted keys and no floats, plus a property test.
- **Explain recompiles like ninja `-d explain`.** `build --json` should list, for each task that missed, which input hash changed. That makes spec 6.8's promise testable ("a one-line fix recompiles exactly these 3 units").
- Extract results could later be cached per file by content hash, so that large repos do not re-parse on each `task next`. This is not needed for v1 (an open question for the `extract` crate).

### Gaps

- Salsa's `persistence` feature was not read in depth (no docs page found in `book/src`); its maturity is judged only from the CHANGELOG wording "prototype".
- No published measurements of Salsa overhead for batch pipelines were found; the size and dependency comparison is the evidence.
- buck2 (content-digest keyed actions, DICE) was not studied; ninja was enough to show the command-hash idea.

## 3. How do these projects design diagnostics and CLI exit codes?

### Takeaway

A diagnostic is a value: a stable id or code, a severity, a short message, a primary location, and optional secondary notes or hints. It is rendered by separate renderers (human, JSON, JSON lines, SARIF and so on) from a serializable *view* struct, so the internal type stays free to change. Exit codes use three levels: 0 clean, 1 "ran fine but found problems", 2 "the tool itself failed". Tool errors go to stderr as a chain of causes, and a broken pipe exits quietly.

### Cited Findings

- ruff `Diagnostic::new(id: DiagnosticId, severity: Severity, message)`: "The identifier should be something that uniquely identifies the *type* of diagnostic ... usable as a reference point for humans"; the primary message is "a single terse description". Inner fields include `annotations` (spans), `subs` (sub-diagnostics), `fix`, `documentation_url`, `secondary_code` — [RUFF crates/ruff_db/src/diagnostic/mod.rs L29-80, L499-512](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/mod.rs#L29)
- ruff `Severity { Hint, Info, Warning, Error, Fatal }` — [RUFF crates/ruff_db/src/diagnostic/mod.rs L1280-1286](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/mod.rs#L1280)
- ruff has one renderer per format in `diagnostic/render/`: `azure, concise, full, github, gitlab, grouped, json, json_lines, junit, pylint, rdjson, sarif`, each with insta snapshots — [RUFF crates/ruff_db/src/diagnostic/render/](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/render)
- ruff JSON is a separate serializable view, `JsonDiagnostic { cell, code, end_location, filename, fix, location, message, name, noqa_row, severity, url }`, with 1-based `{row, column}` locations built from the internal `Diagnostic` by `diagnostic_to_json` — [RUFF crates/ruff_db/src/diagnostic/render/json.rs L47-230](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_db/src/diagnostic/render/json.rs#L47)
- rust-analyzer serialization invariant: "serializable types impose significant backwards compatibility constraints. If a type is serializable, then it is a part of some IPC boundary ... the types in `ide`, `base_db` and below are not serializable by design"; and "`rust-analyzer` is the only crate that knows about LSP and JSON serialization ... create a serializable counterpart ... and manually convert" — [RA architecture.md L505-517, L232-234](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L505)
- typst `SourceDiagnostic { severity, span, message, trace, hints }` with `Severity { Error, Warning }`; hints without a span render as `hint:` lines, and hints with a span annotate secondary code — [TYPST crates/typst-library/src/diag.rs L427-454](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/crates/typst-library/src/diag.rs#L427)
- ruff `ExitStatus { Success => 0, Failure => 1 /* ran, found lint errors */, Error => 2 /* linting failed */ }` — [RUFF crates/ruff/src/lib.rs L37-56](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/lib.rs#L37)
- ruff `main.rs::report_error`: exits 0 on `BrokenPipe` anywhere in the error chain (borrowed from ripgrep); otherwise prints `ruff failed` then `Cause: ...` for each link of the `anyhow` chain to stderr with `writeln!` (not `eprintln!`, "to avoid panicking when the stderr pipe is broken"), and exits 2 — [RUFF crates/ruff/src/main.rs](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/main.rs)
- jaq maps an error enum to exit codes with `impl Termination for Error`: 1 for false/null result, 141 for broken pipe, 2 IO, 3 report, 4 no output, 5 parse or compile — [JAQ jaq/src/main.rs L290-305](https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq/src/main.rs#L290)
- ruff installs a panic hook that asks the user to file an issue, before running the command — [RUFF crates/ruff/src/lib.rs L128-160](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/lib.rs#L128)
- rust-analyzer error trivia: "Use `anyhow::Result` rather than just `Result`"; "Do not end error and context messages with `.`" — [RA style.md L853-864](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L853)

### Inferences

- **Mental's diagnostic is a model record, not a CLI artifact.** `diagnostics.jsonl` is part of `.mental/` (spec 4.9) and is read by `mentalvm` and `mentaldbg`. So the `Diagnostic` type belongs in the `model` crate and is deliberately `Serialize`: it *is* an IPC boundary in RA's sense. Phase-internal types (syntax trees, call-graph indices, plan structs) must not derive `Serialize`.
- Suggested shape:
  - `Diagnostic { id: DiagId, code: Code, severity: Severity, message: String, anchor: Option<Anchor>, records: Vec<RecordId>, kind: Finding | Gap | Assumption | CheckFailed, repair: Option<RepairRef> }`.
  - `Code` is a stable kebab-case enum value per check (`check/coverage`, `check/guard-overlap`, `check/stranded-state`, …), as ruff's `DiagnosticId`.
  - `Anchor` is spec 4.4's `file, lines, quote`, rather than byte spans, because Mental anchors must survive re-reading by an LLM.
  - Use two or three severities (`error`, `warning`, `info`), as typst's minimal enum. ruff's five are more than needed.
- **One human renderer and one JSON renderer, nothing else in v1.** ruff's 12 output formats are bloat for Mental. `--json` emits the model's own `Diagnostic` JSON (or JSON lines), so there is no second schema to maintain.
- **Exit codes for `mentalc`** (copying ruff's three levels):
  - 0: success. For `build`, a complete model was written and no error-severity diagnostics. For `task next`, a task or "done" was printed.
  - 1: the command ran but found problems (`check` with error diagnostics; `build` that stopped after repairs ran out, spec 6.7).
  - 2: `mentalc` itself failed (IO, config, backend unreachable, cache miss in offline mode, bad CLI usage; clap's own usage error is also 2).
  - Broken pipe exits quietly.
  - Exit codes are part of the CLI contract and are snapshot-tested.
- `main.rs` stays as small as ruff's: parse args, call `mentalc::run(args) -> anyhow::Result<ExitStatus>`, report the error chain. A custom panic hook is optional and not v1.

### Gaps

- None of the studied tools has a "repair task" concept attached to a diagnostic. The `repair` field is Mental-specific and is covered by the LLM-loop study.

## 4. Which style rules best keep code compact and readable, and which can clippy enforce?

### Takeaway

rust-analyzer's `style.md` is the best single source, and most of its rules aim at small surface area and local reasoning. About a third can be enforced mechanically: workspace clippy and rustc lints in `Cargo.toml`, `clippy.toml` `disallowed-methods`, and a `#[test]` tidy check in the RA style. The rest need a reviewer checklist. The key mechanical lever for "side effects only at the edges" is `disallowed_methods` on `std::fs`, `std::process`, `std::env` and the clock, allowed only in the IO modules.

### Cited Findings

- RA workspace lints: rust `unreachable_pub = "warn"`, `unused_extern_crates`, `unused_lifetimes`, `elided_lifetimes_in_paths`; clippy groups `correctness`/`perf` deny, `complexity`/`style`/`suspicious` warn, `restriction` allow; individually `dbg_macro`, `todo`, `print_stdout`, `print_stderr`, `str_to_string`, `rc_buffer` warn ("CI raises these to deny"); allowed as subjective or noisy: `single_match`, `too_many_arguments`, `type_complexity`, `new_ret_no_self`, and others — [RA Cargo.toml L174-237](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/Cargo.toml#L174)
- RA: "if some lints annoy you, allow them in the Cargo.toml [workspace.lints.clippy] section" — [RA style.md L102-104](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L102)
- RA dependency policy: "be very conservative with usage of crates.io dependencies. Don't use small 'helper' crates (exception: `itertools` and `either`) ... read Cargo.lock and see if some *transitive* dependencies do not make sense" — [RA style.md L55-62](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L55)
- RA scale of changes: three kinds of change (internal; new `pub` API; new dependency between components). "adding an innocent-looking `pub use` is a very simple way to break encapsulation" — [RA style.md L16-52](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L16)
- RA getters and setters: "If a field can have any value without breaking invariants, make the field public ... Never provide setters"; getters return borrowed data — [RA style.md L267-298](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L267)
- RA: "Prefer Separate Functions Over Parameters": if a `bool` or `Option` parameter is always passed as a literal, split the function — [RA style.md L466-497](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L466)
- RA: "Avoid creating single-use helper functions ... A block serves just as well"; nested helpers go at the end of the enclosing function; "Introduce helper variables freely" — [RA style.md L1031-1116](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L1031)
- RA: "Avoid making a lot of code type parametric, *especially* on the boundaries between crates"; "Avoid `AsRef` polymorphism" — [RA style.md L591-625](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L591)
- RA: "Assert liberally" (with `never!`), "Do not use `#[should_panic]` tests", "Do not `#[ignore]` tests ... assert the wrong behavior and add a fixme" — [RA style.md L163-265](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L163)
- RA enforces some style rules as ordinary tests: `xtask/src/tidy.rs` has `check_test_attrs` (fails on `#[should_panic]` without a FIXME), `check_trailing_ws`, `check_cargo_toml`, `check_licenses`; "all code invariants are tested by `#[test]` tests. There's no additional checks in CI, formatting and tidy tests are run with `cargo test`" — [RA xtask/src/tidy.rs L83-220](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/xtask/src/tidy.rs#L83), [RA architecture.md L441-442](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L441)
- RA documentation style: comments are full sentences; markdown is one sentence per line — [RA style.md L1139-1166](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L1139)
- jaq crate-level attributes `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]` — [JAQ jaq-core/src/lib.rs](https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/src/lib.rs)
- RA avoids bootstrapping and keeps codegen simple: "we use syn and manual string parsing" — [RA architecture.md L372-375](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L372)

### Inferences

- **Mechanically enforceable for mentalc:**
  - `[workspace.lints.rust]`: `unreachable_pub`, `unsafe_code = "forbid"`, `unused_extern_crates`, `elided_lifetimes_in_paths`.
  - `[workspace.lints.clippy]`: groups as RA, plus `dbg_macro`, `todo`, `print_stdout`, `print_stderr`, `str_to_string`, `fn_params_excessive_bools`, `large_enum_variant`, `needless_pass_by_value`, `disallowed_methods`, `disallowed_types`. In CI `-D warnings` turns them into errors, as RA does.
  - `clippy.toml`: `too-many-arguments-threshold = 5` (forces Config structs, per RA style L406), plus a `disallowed-methods` list (`std::fs::*`, `std::process::Command::new`, `std::env::var`, `std::time::SystemTime::now`, `std::thread::spawn`).
  - The IO modules (`io.rs`/`repo.rs`, `cache.rs`, `backend/`, `cli/`) get a scoped `#[allow(clippy::disallowed_methods)]` (or `print_stdout` for `cli/render.rs`). Any other use fails CI, which makes the pure-core boundary machine-checked. (`disallowed_methods` and `print_stdout` are real clippy lints. The per-module allow pattern is standard Rust attribute scoping. I did not see this exact configuration in the studied repos.)
- **Enforceable by a tidy `#[test]`** (RA pattern):
  - No `#[should_panic]`, no `#[ignore]`.
  - No `Serialize` derive outside `model` and `cli` (grep for `derive(...Serialize`).
  - Crate dependency directions: `mentalc` and `mentalvm` never depend on each other; `model` depends on no workspace crate (parse each `Cargo.toml`, as RA's `check_cargo_toml`).
  - A dependency allowlist (diff `Cargo.lock` package names against a checked-in list, so a new transitive crate needs an explicit edit). That is RA's "read Cargo.lock" advice turned into a test.
- **Reviewer checklist (not mechanical):** no doer objects; split functions that take literal bool or Option arguments; no `Default` on Config structs; public fields unless there is an invariant; no setters; no single-use helper functions; no generic parameters across crate boundaries; comments as sentences.
- Bloat flags in the sources themselves: RA's `stdx::never!`/`always!` soft assertions and per-request `catch_unwind` exist because RA is a long-running server that must not die. A batch compiler should just use `assert!`/`debug_assert!` and fail. ruff's jemalloc/mimalloc global allocators, colored output, and arg-file expansion (`ruff_command_line::expand_args`) are not v1 needs.

### Gaps

- I did not verify which clippy lints have false positives on this toolchain; thresholds (for example, `too-many-arguments-threshold = 5`) are suggestions to tune once code exists.

## 5. Which testing strategies fit a compiler whose acceptance target is examples/report-jobs?

### Takeaway

Use four layers.

1. Data-driven unit tests: one `check(fixture, expect)` helper per phase, many one-line tests (RA, jaq).
2. Multi-file fixture strings with inline annotations for expected diagnostics (RA fixture DSL, typst and ty annotations).
3. CLI snapshot tests through `insta_cmd` with path filters (ruff).
4. A golden end-to-end build of report-jobs replayed offline from a committed answer cache, with the four acceptance findings asserted.

Content addressing makes layer 4 cheap: the cache *is* the record/replay fixture, so CI needs no LLM.

### Cited Findings

- RA: "tests are data driven and do not test the API. Tests which directly call various API functions are a liability"; a single `#[track_caller] fn check(input: &str, expect: Expect)` is "the single place that actually exercises a particular API", followed by "a hundred more tests that don't care about the specific API at all" — [RA architecture.md L416-436](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L416)
- RA tests at three boundaries (LSP: "heavy", few, gated by `RUN_SLOW_TESTS`; `ide`: "the middle, and most important"; `hir`) — "in a statically typed language, it's hard to make an error in the protocol itself if messages are themselves typed" — [RA architecture.md L390-407](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L390)
- RA: "tests do not depend on any kind of external resources, they are perfectly reproducible"; tests use no libstd ("All required library code must be a part of the tests. This ensures fast test execution") — [RA architecture.md L412-444](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/architecture.md#L412)
- RA fixture DSL: one string describes many files with `//- /main.rs crate:a deps:b cfg:... env:...` headers — [RA crates/test-utils/src/fixture.rs L1-63](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/test-utils/src/fixture.rs#L1); markers `$0` (position), `$0...$0` (range), and `// ^^^ label` annotations parsed by `extract_annotations` — [RA testing.md](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/testing.md), [RA crates/test-utils/src/lib.rs L233](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/test-utils/src/lib.rs#L233)
- RA "Minimal Tests": cut fixtures to the minimum and use unindented raw strings — [RA style.md L108-149](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L108); `cov_mark::hit!/check!` links a code branch to its one canonical test — [RA style.md L151-161](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/docs/book/src/contributing/style.md#L151)
- jaq: tiny helpers `give(input, filter, output)`, `gives`, `fail`, and a `yields!(name, filter, output)` macro produce one test per line, e.g. `yields!(cartesian_arith, "[(1,2) * (3,4)]", [3, 4, 6, 8]);` — [JAQ jaq-core/tests/common/mod.rs](https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/tests/common/mod.rs), [JAQ jaq-core/tests/tests.rs L1-25](https://github.com/01mf02/jaq/blob/c6cc7d3721ccfb212f153006c4fc90c56567540c/jaq-core/tests/tests.rs#L1)
- typst test suite: `.typ` files with inline annotations `// Error: 4-19 cannot add array and integer` (also `Warning`, `Hint`, `Trace`) that refer to the next non-annotation line; references are refreshed with `--update` — [TYPST tests/README.md L128-241](https://github.com/typst/typst/blob/84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03/tests/README.md#L128)
- ty mdtest: "Any Markdown file can be a test suite"; fenced code blocks are files; assertions are comments (`# revealed:`, `# error:`, `# snapshot`) matched against produced diagnostics — [RUFF crates/ty_test/README.md](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ty_test/README.md)
- ruff rule tests: `#[test_case(Rule::UnusedImport, Path::new("F401_0.py"))]` over fixture files in `resources/test/fixtures/<linter>/`, with insta snapshots per case — [RUFF crates/ruff_linter/src/rules/pyflakes/mod.rs L36-40](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff_linter/src/rules/pyflakes/mod.rs#L36)
- ruff CLI tests: a `CliTest` fixture builds a canonicalized temp dir, writes files, and binds insta filters that replace the temp path with `[TMP]/`, the crate root with `CRATE_ROOT/`, Windows separators, panic locations, and the version with `[VERSION]`. `assert_cmd_snapshot!` records `success`, `exit_code`, stdout and stderr inline — [RUFF crates/ruff/tests/cli/main.rs](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/tests/cli/main.rs), [RUFF crates/ruff/tests/cli/lint.rs L10-44](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/tests/cli/lint.rs#L10)
- ruff CLI tests pin flags (`check --no-cache --output-format concise`) so snapshots do not depend on cache state or default formats — [RUFF crates/ruff/tests/cli/lint.rs L15](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/tests/cli/lint.rs#L15)
- ruff tests cache invalidation as unit tests beside the code (`cache_adds_file_on_lint`, `cache_invalidated_on_file_modified_time`, …) — [RUFF crates/ruff/src/cache.rs L531-1010](https://github.com/astral-sh/ruff/blob/b140af2f48bddf33a1ae984174af3180236aaaaf/crates/ruff/src/cache.rs#L531)

### Inferences

- **Layer 1: per-phase `check` helpers.** Each phase module's `tests.rs` (decided layout) has one `#[track_caller] fn check(fixture: &str, expect: ...)` that parses a fixture, runs *only* that phase on typed inputs, and renders the output as stable text, compared with an inline `insta::assert_snapshot!(..., @"...")`. Inline insta snapshots give expect-test's ergonomics, so `expect-test` is not added as a second snapshot tool. Tests do not call phase internals.
- **Layer 2: a Mental fixture DSL, kept tiny.**
  - One string with `//- path/to/file.py` headers for a miniature repository (RA format), plus optional `//- answers/<role>/<unit>.json` blocks holding canned LLM answers.
  - A `FakeBackend` returns those answers by unit ID and fails loudly on any unexpected task. This keeps tests hermetic, as RA requires.
  - Expected diagnostics are written inline next to the code they anchor, typst/ty style (`# mental: stranded-state`), and a check-phase test asserts the produced diagnostics equal the annotations.
  - Fixtures should be minimal (RA "Minimal Tests").
  - A Markdown suite (ty mdtest) would be elegant for spec-like check examples but adds a parser and runner. Flag it as optional later, not v1.
- **Layer 3: CLI snapshots** with `insta_cmd`, using a `CliTest` like ruff's, with filters for the temp path and the mentalc version. One snapshot per command and mode (`init`, `build`, `check`, `diff`, `task next`/`submit`, each with and without `--json`), recording exit codes. Keep them few: RA's point is that typed messages rarely break at the protocol layer.
- **Layer 4: golden report-jobs.** Commit `examples/report-jobs` plus a recorded answer cache (the content-addressed files from one real build).
  - The e2e test runs `mentalc build --offline` (any cache miss is exit 2, never an LLM call). It snapshots `.mental/model/*.jsonl` and `diagnostics.jsonl` (sorted JSONL makes diffs readable), and asserts the four acceptance findings of spec 6.9 by `Code` and anchor.
  - A second e2e test applies the one-line fix and asserts that the "recompile explanation" lists exactly the expected units (F-S10's 3 calls). Spec 6.8 becomes a regression test.
  - When prompts change, the recorded cache misses by design. Re-recording is an explicit, reviewed step that needs an LLM, and that run fails clearly without a key, per the owner's rule.
- **Property tests (proptest), where they pay:**
  - `canonical_json` is invariant under map key order.
  - The cache key changes when any input byte changes and stays the same when nothing changes.
  - JSONL write then read returns the same records, with sorted output.
  - Pure phases are deterministic: run twice, same bytes.
- **Cache unit tests** mirror ruff's: put/get round trip, a key differs on a prompt version bump, a corrupt file is reported and treated as a miss, an atomic write leaves no partial file.
- `cov_mark` is optional. It helps find "the canonical test for each bit of code", but cargo-mutants plus 100% branch coverage (already decided) covers most of that value. Flag it as possible bloat.

### Gaps

- None of the studied projects tests a pipeline against a recorded LLM; the record/replay design is derived, not observed.
- I did not check `insta_cmd` maintenance status or version compatibility with the chosen `insta`.

## 6. Which enforceable rules should govern mentalc's architecture? (derived from questions 1–5)

### Takeaway

About fifteen rules capture what the studied codebases do well and keep `mentalc` small. Each is listed with its rationale, its source, and how it is enforced. The most important are: pure phases with IO behind one edge (enforced by `disallowed_methods`), one sequential `build` function, a hand-written content-addressed answer cache instead of Salsa, diagnostics as model records, and data-driven fixture tests plus an offline golden build of report-jobs.

### Cited Findings

- Each rule below cites its sources, which are the findings in sections 1–5 (RA architecture.md and style.md, ruff `cache.rs`/`lib.rs`/`main.rs`/`tests/cli`, typst `World`/`compile`, jaq tests, Salsa size and persistence status, ninja `graph.cc`).

### Inferences

Candidate guidelines (rule, rationale, source, enforcement):

1. **Phases are pure functions.** Every phase is `pub(crate) fn <phase>(inputs: &…) -> (Output, Vec<Diagnostic>)` with no IO, clock, environment or process access.
   - *Rationale:* it makes each phase testable alone from a fixture, and keeps results reproducible.
   - *Source:* RA architecture.md L451-457, L88; typst `World` lib.rs L63-101.
   - *Enforcement:* clippy `disallowed_methods`/`disallowed_types` (`std::fs`, `std::process::Command`, `std::env::var`, `SystemTime::now`, `std::thread::spawn`) denied crate-wide, allowed only in `repo.rs`, `cache.rs`, `backend/`, `cli/`; CI `-D warnings`.
2. **The pipeline is one function.** `build()` calls the twelve phases in spec order as plain function calls with typed values. There is no `Phase` trait, registry or plugin system.
   - *Rationale:* the data flow is visible in about 40 lines and matches spec 6.2.
   - *Source:* typst `compile_impl` lib.rs L100-140; jaq lib.rs doc chain; RA "functions over objects" style.md L350.
   - *Enforcement:* reviewer; a test that asserts the phase order recorded in `build --json` output.
3. **LLM phases split into `plan` and `apply`.** Both are pure; only the driver touches the cache and backend.
   - *Rationale:* LLM steps stay snapshot-testable with canned answers; there is no async code; it fits the pull-based `agent` backend.
   - *Source:* derived from rule 1 plus RA's "stateless a-la HTTP" (architecture.md L241).
   - *Enforcement:* rule 1's lint (the backend lives in `backend/` only); reviewer.
4. **Commands are stateless; the cache is the build state.** `task next`/`task submit` recompute from the repository plus `.mental/cache/`, so there is no mutable session file.
   - *Rationale:* nothing can drift, and crash recovery is trivial.
   - *Source:* RA architecture.md L241-245.
   - *Enforcement:* an e2e test that kills or restarts between `task submit` calls and gets an identical model.
5. **Incrementality is a hand-written, content-addressed answer store, not Salsa.**
   - *Rationale:* processes are short-lived, the cost lives in LLM calls, LLM answers are not deterministic, and Salsa is ~27.7k lines with persistence still a prototype.
   - *Source:* SALSA Cargo.toml L20-71 and CHANGELOG L297-306; ruff `cache.rs` vs ty's Salsa; comemo in-memory only.
   - *Enforcement:* dependency allowlist test (no `salsa`/`comemo`); reviewer.
6. **The cache key is a cryptographic hash of canonical task bytes:** `{cache_format, role, prompt template, version and hash, answer-schema hash, inputs by content}`, never `std::hash::Hash`, never the LLM identity.
   - *Rationale:* stable across platforms and versions; usable in the call ledger; changing the LLM invalidates nothing (spec 6.6).
   - *Source:* ruff `cache_key.rs` L60-73 (Hash not portable); spec 6.6 and 6.8.
   - *Enforcement:* proptest (key order invariance, sensitivity to every field); a unit test that a bumped prompt version changes the key.
7. **Callee summaries enter keys by content.**
   - *Rationale:* early cutoff for free: an unchanged recompiled summary stops recompilation further up (the effect of Salsa's backdating and ninja's `restat`).
   - *Source:* SALSA backdate.rs L15-59; NINJA graph.cc L225-262.
   - *Enforcement:* e2e test "one-line fix → exactly the expected units missed".
8. **The cache stores only checked answers, writes atomically, never evicts silently, and in offline or `check` mode a miss is fatal.**
   - *Rationale:* an entry is a paid, non-deterministic judgment that defines the model; follows "fail rather than degrade".
   - *Source:* ruff caches only clean results (diagnostics.rs L365) and writes by temp file and rename (cache.rs L157-199); *contrast* ruff's 30-day eviction and silent reset (cache.rs L100-133, L202-247), deliberately not copied.
   - *Enforcement:* cache unit tests (corrupt file → reported miss; offline miss → exit 2); a code path that fails partway through leaves no partial file.
9. **Every recompile is explained.** `build --json` lists each cache miss with the input hash that changed.
   - *Rationale:* spec 6.8 becomes observable and testable; it makes debugging cost surprises easy.
   - *Source:* ninja `explanations_.Record` (graph.cc L243-250).
   - *Enforcement:* CLI snapshot test of the explanation output.
10. **Only the record format is serializable.** `Serialize`/`Deserialize` derive only in the `model` crate (records, anchors, diagnostics, ledger lines) and in `mentalc::cli` view structs; phase-internal types never derive them.
    - *Rationale:* serializable types are compatibility promises; keep the promise surface small.
    - *Source:* RA architecture.md L505-517, L232-234; ruff `JsonDiagnostic` view (json.rs L205-230).
    - *Enforcement:* tidy `#[test]` that greps for `Serialize` derives outside allowed paths.
11. **Crate boundaries are fixed and tested.** `model` depends on no workspace crate; `extract` on `model`; `mentalc` on `model` and `extract`; `mentalvm` on `model`; `mentalc` and `mentalvm` never depend on each other. Adding a workspace dependency edge or a `pub use` re-export is an "API change" that needs explicit review.
    - *Rationale:* spec 3 tool rules; RA's three kinds of change.
    - *Source:* RA style.md L16-52; spec 3.
    - *Enforcement:* tidy test parsing each `Cargo.toml` (RA `check_cargo_toml` pattern); `unreachable_pub` lint.
12. **Diagnostics are model records with stable codes.** `Diagnostic { code, severity: error|warning|info, message, anchor: Option<Anchor>, records, kind, repair }`, rendered by exactly two renderers (human, JSON).
    - *Rationale:* `mentalvm`/`mentaldbg` read `diagnostics.jsonl`; stable codes let tests and agents match findings; ruff's 12 formats are bloat here.
    - *Source:* ruff `Diagnostic::new` (mod.rs L29-80) and `render/`; typst `SourceDiagnostic` (diag.rs L427-454).
    - *Enforcement:* `Code` is an enum (exhaustive match in the renderer); snapshot tests per code.
13. **Three-level exit codes and a thin `main`.** 0 ok, 1 ran but found errors or repairs ran out, 2 mentalc failed; quiet exit on broken pipe; the error chain goes to stderr; `main.rs` only parses, calls `mentalc::run`, and reports.
    - *Rationale:* scripts and agents branch on exit codes; the library stays testable without a subprocess.
    - *Source:* ruff lib.rs L37-56, main.rs `report_error`; jaq main.rs L290-305.
    - *Enforcement:* `insta_cmd` snapshots record `exit_code` for each command.
14. **Errors split by layer.** Phases never return `Result` (problems are diagnostics); IO, cache and backend functions return typed `thiserror` errors; the binary layer adds `anyhow` context. Context messages do not end with a period.
    - *Rationale:* problems in the analyzed repository are data, not failures.
    - *Source:* RA architecture.md L451-457, style.md L853-864; typst `Warned<SourceResult<T>>`.
    - *Enforcement:* reviewer; rule 1's lint keeps IO errors out of phases structurally.
15. **Tests are data-driven through one `check` helper per phase,** with minimal fixtures in a multi-file string DSL (`//- path`), inline insta snapshots, and no tests that call internals. No `#[should_panic]`, no `#[ignore]`.
    - *Rationale:* refactoring the API touches one helper, not a hundred tests.
    - *Source:* RA architecture.md L416-439, style.md L108-177; jaq `yields!`; RA tidy `check_test_attrs`.
    - *Enforcement:* tidy `#[test]` banning `should_panic`/`ignore`; reviewer for "one check helper per phase".
16. **Tests are hermetic.** Unit and integration tests use a `FakeBackend` with fixture answers that fails on an unexpected task. The golden report-jobs build replays a committed answer cache with `--offline`. Re-recording is an explicit step that needs an LLM key and fails without one.
    - *Rationale:* reproducible CI, no LLM cost, acceptance facts (spec 6.9) guarded on every commit.
    - *Source:* RA architecture.md L444; ruff CLI tests pin `--no-cache` and the output format (lint.rs L15).
    - *Enforcement:* CI runs without network or API key; an e2e test asserts the four acceptance findings by code and anchor.
17. **Dependencies are conservative.** No small helper crates; every new crate, transitive ones included, is a deliberate decision.
    - *Rationale:* compile time, fewer things to break, a one-person project.
    - *Source:* RA style.md L55-62.
    - *Enforcement:* a test comparing `Cargo.lock` package names to a checked-in allowlist (or `cargo-deny` bans); reviewer.
18. **Lints in `Cargo.toml`, with tests enforcing the rest.**
    - *Configuration:*
      - `[workspace.lints]` as RA (L174-237) plus `unsafe_code = "forbid"`, `fn_params_excessive_bools`, `print_stdout`/`print_stderr` (allowed only in `cli/`).
      - `clippy.toml` `too-many-arguments-threshold = 5`.
      - All style invariants that can be grepped are `#[test]` tidy checks run by `cargo test`.
    - *Rationale:* one command (`cargo test` + `cargo clippy -D warnings`) checks everything, with no separate scripts.
    - *Source:* RA architecture.md L441-442, Cargo.toml L174-237, xtask/src/tidy.rs.
    - *Enforcement:* CI.
19. **Reviewer checklist from RA's style.md.** No doer objects; split functions taking literal bool or Option; Config structs without `Default`, passed explicitly; public fields unless invariant; no setters; no single-use helpers; no generics across crate boundaries; full-sentence comments.
    - *Rationale:* compact code and local reasoning.
    - *Source:* RA style.md L267-498, L591-625, L1031-1166.
    - *Enforcement:* reviewer (PR template checklist).

Bloat to avoid explicitly in v1:

- Salsa or comemo.
- A pass-manager or `Phase` trait.
- `expect-test` beside insta.
- Output formats beyond human and JSON.
- RA-style `never!` soft assertions and `catch_unwind` (those are for long-running servers).
- Custom global allocators.
- Panic hooks.
- Argument-file expansion.
- A Markdown test runner (optional later).
- `cov_mark` (optional; mutants plus coverage already cover it).
- Time-based cache eviction.

### Gaps

- The exact module tree of `mentalc` (for example, `cli/`, `build.rs`, `phases/*.rs`, `cache.rs`, `backend/`, `check/*.rs`, `repo.rs`) is a proposal derived from these rules; it has not been validated against real code yet.
- Whether rerunning the non-LLM phases on every `task next` stays fast enough on repositories larger than report-jobs is unmeasured. If not, per-file caching of extract results by content hash is the next step, still without Salsa.
