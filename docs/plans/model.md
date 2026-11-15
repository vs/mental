# Implementation plan: the `model` crate

**Status:** accepted, 2026-11-14 (decisions D1 to D9 approved by the owner and written into the specification). **Milestone:** 1, "the record format" ([specification 11.4](../specification.md#114-milestones)).
**Rules it follows:** [guidelines](../guidelines/README.md), especially [model](../guidelines/model.md) and [testing](../guidelines/testing.md). **Evidence:** [implementation research](../research/implementation-guidelines/report.md).

## 1. Goal

`model` is the record format that every other crate reads and writes. It turns sorted JSONL text into typed records and back, and says whether a model is **well-formed**.

**Done when** (milestone 1, made precise):
1. The hand-written reference model of report-jobs loads and checks with zero diagnostics.
2. Every diagnostic code has a seeded-error fixture that produces exactly that code.
3. `write(load(files)) == files`, byte for byte, for the reference model and for generated models.
4. Gates on the crate: 100% line and branch coverage, zero missed mutants, fuzz targets build.

**One line that separates `model` from `mentalc`:** `model` checks that a model is *well-formed* (it parses, references resolve, anchors match, guards parse and partition, machines are consistent). `mentalc` checks that it is *right* (obligation coverage, composition, atomicity, stranded states). Checks of the second kind need facts from `extract` or judgment about the code; checks of the first kind need only the model and the source text.

## 2. Scope

| In | Out (and where it goes) |
|---|---|
| Semantic IDs: grammar, parse, display, child builder | Content hashes and the compile cache (`mentalc`, milestone 3; `blake3` is added then) |
| The nine core record kinds (spec 4.5 to 4.7), summary-level behaviors | Full-detail behavior graphs (`lod: full`), added when zoom needs them |
| Anchors and the anchor check against source text | Records above the core: `term`, `level`, `assumption`, `variation_point` (milestone 6) |
| Diagnostics: the record, codes, ordering | Run files and traces (`mentalvm`; they move here only when a second crate reads them) |
| Loading and writing a model as a map of file name → text | File-system access (callers do it; `model` stays pure and builds for WebAssembly) |
| Guard expressions: parse, display, evaluate in two- and three-valued logic, partition check | Any command-line tool (`mentalc check` arrives in milestone 3) |
| Lifecycle machine consistency | Semantic checks of section 6.7 other than guard algebra |

## 3. Decisions

The owner accepted every recommendation on 2026-11-14. They are written into the specification ([4.2](../specification.md#42-fields-every-record-has), [4.3](../specification.md#43-semantic-ids), [4.7](../specification.md#47-behavior-records), [4.9](../specification.md#49-storage)) and the glossary.

| # | Question | Decision | Why |
|---|---|---|---|
| D1 | Short references (`pred:x`, `n:load`, `.../case:y`) or full IDs in storage? | **Full IDs only.** Short forms exist only in what people read (viewer, terminal) | The reference check becomes purely textual; no scope rule to get wrong. The trial model's short forms are exactly where it was ambiguous |
| D2 | How are predicates that cannot both be true (`status-is-pending`, `status-is-failed`) handled by the guard check? | **A predicate over an enum state variable may say `equals: <value>`.** The partition check then treats that variable as one dimension over its domain values, not as independent true/false bits | Without it the check reports gaps and overlaps that cannot happen. It also gives the interpreter the meaning of the predicate for free |
| D3 | Where does the format version live? | **`.mental/model/format.json`** = `{"format":1}`. A mismatch is a hard error that says "rebuild". No migration code ([M10](../guidelines/model.md#m10-one-format-version-no-migrations)) | One place, one number, and every JSONL file stays pure records; the model is always rebuildable |
| D4 | File names | **One file per kind, named after it:** `entities`, `ports`, `wires`, `instances`, `state_vars`, `machines`, `predicates`, `characteristics`, `behaviors` (`.jsonl`) | Spec 4.9 mixes `stores.jsonl` (state variables and machines) and `inputs.jsonl` (characteristics); one kind per file makes "wrong file" a simple check |
| D5 | Store the `kind` field (spec 4.2)? | **No.** The kind is the ID's last prefix and the file it sits in | A stored copy of derivable data can only drift |
| D6 | Instance IDs contain a replica number (`reports/worker[0]`), which looks like the counter P9 forbids | **Allow `[n]` only on instances.** The number comes from configuration (`replicas: 2`), not from code order, so it is stable across rebuilds | The spec already writes `worker[0]`; the rule needs one stated exception |
| D7 | Two records with the same ID | **Always a `duplicate-id` diagnostic**, even if identical. Guideline [M6](../guidelines/model.md#m6-identity-is-the-key) now says the same; it used to follow Glean and merge identical lines | One writer; a duplicate is a writer bug. Glean merges identical facts because it has many writers |
| D9 | `thiserror` in `model`? | **No:** two small error types with hand-written `Display` (about 20 lines), keeping the allowlist at `serde`, `serde_json`, `blake3` ([model.md](../guidelines/model.md#dependency-allowlist)) | The general rule ([G9](../guidelines/README.md#g9-errors-by-layer)) names `thiserror` for libraries; this crate is small enough to make an exception |
| D8 | The ID pattern of every kind | Adopt the table below | IDs must be fixed before the reference model is written |

**ID table (D8),** now in [spec 4.3](../specification.md#43-semantic-ids):

| Kind | Pattern | Example |
|---|---|---|
| entity | `<parent>/<slug>`, or `<slug>` at the root | `reports/worker`, `reports/db/store:report` (a store is an entity with prefix `store:`) |
| actor (entity of type Actor) | `<system>/actor:<slug>` | `reports/actor:operator` |
| port | `<entity>/port:<slug>` | `reports/api/port:post-reports` |
| wire | `<from-port>/wire:<to-entity-slug>-<to-port-slug>` | `reports/api/port:enqueue-generate/wire:broker-tasks` |
| instance | `<entity>[<n>]` | `reports/worker[1]` |
| state_var | `<store>/var:<slug>` | `reports/db/store:report/var:status` |
| machine | `<state-var>/machine:lifecycle` | `reports/db/store:report/var:status/machine:lifecycle` |
| predicate | `<entity>/pred:<slug>` | `reports/api/pred:range-too-large` |
| characteristic, class | `<port>/in:<slug>`, `<characteristic>/class:<slug>` | `reports/api/port:post-reports/in:range/class:too-large` |
| behavior, case, node | `<entity>/beh:<slug>`, `<behavior>/case:<slug>`, `<behavior>/n:<slug>` | `reports/worker/beh:generate-report/n:claim` |

Cases, classes and nodes are **embedded**: they live inside their record's line but have IDs of their own, because other records reference them (a machine's `by` names a node).

## 4. Layout and size

The layout follows [model.md](../guidelines/model.md#module-layout).

```
model/
├── Cargo.toml          deps: serde, serde_json (blake3 is allowed, added with its first use) · dev: insta, proptest
├── src/
│   ├── lib.rs          the public surface and FORMAT_VERSION                       ~30 lines
│   ├── id.rs           Id, Kind, Slug, the slug alphabet, slugify, parse/display    ~180
│   ├── record.rs       the Record trait, Set<T>, common fields, the Model struct    ~200
│   ├── record/         structure.rs (entity, port, wire, instance)                 ~150
│   │                   state.rs (state_var, machine)                               ~80
│   │                   behavior.rs (predicate, characteristic, behavior)           ~150
│   ├── anchor.rs       the Anchor record                                           ~40
│   ├── jsonl.rs        read(&str) and write(&[T]) -> String; Model::load and write ~150
│   ├── diagnostic.rs   Diagnostic, Code, Severity                                  ~120
│   ├── check.rs        check(&Model, &Sources) -> Vec<Diagnostic>                  ~40
│   ├── check/          refs.rs  sets.rs  anchors.rs  guards.rs  machines.rs        ~300
│   ├── guard.rs        Expr, parser, Display, Logic, eval, the partition check     ~250
│   └── testkit.rs      behind the `testkit` feature: fixture DSL, invariant check  ~120
│   (each module has its tests.rs beside it)
├── tests/              public-API tests, tidy.rs; data/ok/ and data/err/<code>/ fixtures
└── fuzz/               cargo-fuzz targets: JSONL reader, guard parser (own workspace)
```

`record/above.rs` (term, level, assumption, variation point) arrives with milestone 6, not now.

**Budget:** about 1,400 lines of non-test code; review anything past 1,800. No file over 400 lines, no function over 60.

## 5. Public API

```rust
// id.rs
pub struct Slug(String);                     // [a-z0-9]+(-[a-z0-9]+)*, one alphabet function
pub fn slugify(code_name: &str) -> Result<Slug, IdError>;  // the only path from a code name to a slug
pub struct Id(String);                       // private field; only parse and child build one
impl Id {
    pub fn parse(s: &str) -> Result<Id, IdError>;  // rejects non-canonical input, never normalizes
    pub fn child(&self, kind: Kind, slug: &Slug) -> Id;
    pub fn kind(&self) -> Kind;
    pub fn slug(&self) -> &str;
    pub fn parent(&self) -> Option<Id>;
}
pub enum Kind { Entity, Store, Actor, Port, Wire, Instance, Var, Machine, Pred, In, Class, Beh, Case, Node }
// Serialize writes the string; Deserialize calls Id::parse. Ord is the byte order of the string.

// record.rs, jsonl.rs
pub struct Model { pub entities: Vec<Entity>, pub ports: Vec<Port>, /* … one Vec per kind, */ pub anchors: Vec<Anchor> }
impl Model {
    pub fn load(files: &BTreeMap<String, String>) -> Result<Model, Vec<Diagnostic>>; // all line errors, not the first
    pub fn write(&self) -> BTreeMap<String, String>;                                 // file name → canonical bytes
    pub fn get(&self, id: &Id) -> Option<RecordRef<'_>>;                             // binary search
}

// check.rs: Sources is the source text the caller already read, by path
pub fn check(model: &Model, sources: &Sources) -> Vec<Diagnostic>;

// guard
pub enum Expr { Pred(Id), Not(Box<Expr>), And(Vec<Expr>), Or(Vec<Expr>) }
pub trait Logic: Copy { /* and, or, not, from_bool */ }   // for bool and Option<bool>
pub fn partition(guards: &[(Id, Option<&Expr>)], dims: &Dimensions) -> Vec<Diagnostic>; // None = `otherwise`
```

`otherwise` is not an expression: it is a case with no guard, meaning the complement of the others. A guard of `true` (seen in the trial) is a single-case behavior with no guard.

## 6. The format

- **Files:** `format.json` plus the nine `<kind>s.jsonl` files (D3, D4). Anchors in `anchors.jsonl`, diagnostics in `diagnostics.jsonl`, both beside `model/` as spec 4.9 says.
- **Each line:** one record; fields in struct order; absent optionals and empty sets omitted, never `null` or `[]`; no `kind` field (D5).
- **Each file:** lines sorted by ID in byte order, no duplicates, ending in `\n`, nothing else.
- **Strict reading:** `deny_unknown_fields` on every struct; unknown kinds, unknown fields and a wrong format version are errors. No catch-all field, no `serde_json::Value`, no `HashMap` in any serialized type.
- **Sets** (`checks`, `tags`, `decides`, `evidence`) are `Set<T>`, which refuses unsorted or repeated input when loaded. **Ordered lists** (`steps`, `transitions`, `cases`) keep their order because it means something.
- **`written_by`** on a state variable can be derived from behaviors' write nodes, so it is either checked against that derivation or not stored. Decision: not stored in version one ([spec 4.6](../specification.md#46-state-records)).

## 7. Checks and diagnostic codes

A closed `Code` enum, written as stable kebab-case. The `Diagnostic` record is as [M13](../guidelines/model.md#m13-diagnostic-codes-are-closed-and-complete) defines it: `code`, `id`, `field` (a JSON path such as `cases[1].when`), `message`, `related` IDs and an optional anchor, with kind and severity fixed per code. Output is sorted by `(id, code, field)` and deduplicated. Checks never stop at the first error.

| Code | Raised by | Means |
|---|---|---|
| `bad-line` | load | a line is not valid JSON for its kind (unknown or missing field, wrong type) |
| `unsorted-set` | load | a set field is not sorted or repeats a value |
| `bad-id` | load | an ID does not parse or is not canonical |
| `format-version` | load | `format.json` names a format this build does not read |
| `unsorted` | load | lines out of byte order |
| `duplicate-id` | load | an ID of a record, case or class appears twice, across all files (D7) |
| `node-mismatch` | check | one node appears in two cases of its behavior with different fields |
| `wrong-file` | load | a record's ID kind does not belong in the file it is in |
| `orphan` | check | the parent part of an ID is not a record |
| `dangling-ref` | check | a reference names no record |
| `wrong-kind-ref` | check | a reference names a record of the wrong kind |
| `bad-guard` | check | a guard does not parse |
| `unknown-predicate` | check | a guard names a predicate that does not exist (never read as false) |
| `guard-gap`, `guard-overlap` | check | the cases of a behavior, or the classes of a characteristic, miss or double-cover an assignment; the message names the lowest such assignment |
| `guard-check-too-large` | check | more than 16 dimensions; the check refuses rather than skips |
| `base-class` | check | a characteristic has no class tagged `base`, or more than one |
| `machine-value` | check | an initial, `from` or `to` value is outside the variable's domain |
| `machine-writer` | check | a transition's `by` is not a write node of that variable |
| `anchor-missing` | check | a record that needs an anchor has none |
| `anchor-dangling` | check | an anchor names no record |
| `anchor-broken` | check | the quote is not found in the file |
| `anchor-stale` | check | the stored status (`exact`, `moved`) differs from what the source shows |

## 8. The reference model

`examples/report-jobs/model/` is written by hand and is the golden file for the whole project: `extract` and `mentalc` are later judged against it. It is never rewritten by snapshot tooling.

It starts from the trial model in `docs/local/archive/report-jobs-simulation/compiled/` (176 lines in 8 files, plus 53 anchors), ported to the decided format:
- IDs follow the table in D8, and every reference is a full ID (D1);
- fields the spec does not define are removed (`gen_key`, `lazy`, `instantiated_from`, `note`, `deps`, `unknowns`, …). A field that turns out to be needed goes into the specification first, then into the record struct;
- `stores.jsonl` splits into `state_vars.jsonl` and `machines.jsonl`; `inputs.jsonl` becomes `characteristics.jsonl`;
- predicates over `status` get `equals` (D2);
- the facts the spec states are counted by a test: 6 entry points (2 routes, 2 tasks, 1 schedule, 1 command) and 2 state variables (F-S10).

## 9. Tests

| What | How | Where |
|---|---|---|
| ID grammar | `good`, `good_canon` and `bad(input, exact message)` tables, each also run through serde; a property test that any accepted string displays back unchanged and that `Ord` equals byte order; a property test that every string `child` builds parses | `id/tests.rs` |
| `Set<T>` | unsorted and repeated input rejected with exact messages | `set/tests.rs` |
| Records and format | an insta snapshot of a small hand-built model with every kind and every optional field, which freezes the format; a test per strictness rule (extra field, `null`, `[]`, wrong version) | `record/tests.rs`, `model/tests.rs` |
| Round-trip | property tests over generated models: `write(load(write(m))) == write(m)`; write is idempotent; output is sorted | `tests/roundtrip.rs` |
| Guards | parser tables; display round-trip; `Option<bool>` evaluation with every predicate known equals `bool` evaluation; the partition check against a truth-table oracle, exhaustively over every guard set on up to 3 predicates and by property test up to 10; one test each at 15, 16 and 17 dimensions | `guard/*/tests.rs` |
| Checks | one fixture under `tests/data/err/<code>/` per code, each producing exactly that code; a completeness test that matches over every `Code` and fails if one has no fixture | `tests/seeded.rs` |
| Reference model | loads with no diagnostics; its counts match F-S10; seeded errors made by changing one line of it (rename a referenced ID, break a quote, add an overlapping case) each give the expected code | `tests/reference.rs` |
| Fuzzing | the loader and the guard parser: arbitrary bytes give a diagnostic, never a panic; whatever loads writes back to the same bytes | `fuzz/` |

Every test is one behavior through a module-local `check(input, expected)` helper. No `#[should_panic]`, no `#[ignore]`. Proptest regression files are committed.

## 10. Steps

Each step is one pull request that leaves every gate green, with its tests in the same change.

| # | Step | Ends with |
|---|---|---|
| 0 | alpha generates the workspace and the `model` component; add the crate's lint settings (`disallowed_types` for `HashMap`, `HashSet` and `serde_json::Value`; `wildcard_enum_match_arm`) and its dependency allowlist test | an empty crate passing every gate |
| 1 | ~~Settle D1 to D9; write them into the specification (4.2, 4.3, 4.7, 4.9) and the glossary~~ **Done 2026-11-14** | the docs say exactly what the code will do |
| 2 | `id`, `Kind`, slug alphabet | ID tables and properties green |
| 3 | `diagnostic`: `Diagnostic`, `Severity`, and `Code` with the codes of `id` | the completeness test. `Code` grows step by step: each later step adds its codes together with their fixtures, so the test is green after every step |
| 4 | `record`: `Set<T>`, common fields, the nine kinds, `anchor` | the format snapshot |
| 5 | `jsonl`: load, write, `format.json`, load-time codes; `testkit` fixture DSL | round-trip properties |
| 6 | `guard`: expressions, parser, evaluation | parser tables, logic properties |
| 7 | `check`: references, embedded IDs, repeated nodes, guard references, machines | one fixture per code so far |
| 8 | `check/anchors` | anchor fixtures |
| 9 | The reference model | `tests/reference.rs` green: milestone 1's acceptance |
| 10 | `partition` with dimensions (D2) | oracle tests; the reference model's behaviors and characteristics partition |
| 11 | Fuzz targets; record real sizes (records, dimensions per behavior) in findings | thresholds checked against data |

## 11. Risks and spikes

- **Assertions and branch coverage.** If `cargo llvm-cov --branch` counts the panic side of `assert!`/`debug_assert!` as an untaken branch, a 100% branch gate punishes invariant checks. A one-hour spike in step 0 settles it. If it does count them, invariants become typed errors where the code can recover, and are otherwise kept as assertions excluded from the branch count by a documented rule.
- **Branch coverage depends on alpha's update** (pinned nightly, diff-cover reading Cobertura branch data). If that lands without branch data, the gate is lines only until fixed, and the reviewer checks branches.
- **The reference model is the largest single piece of work.** Hand-porting 176 lines under new rules will surface format questions. Each one goes into the specification first, so the code never invents format.
- **Thresholds are estimates** (16 dimensions, a few thousand records). The trial model's largest behavior uses 5 predicates. Step 11 checks the estimates against real counts.
