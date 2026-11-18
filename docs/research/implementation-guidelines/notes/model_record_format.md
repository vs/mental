# Record format, semantic IDs and fact store: guidelines for Mental's `model` crate

Evidence is the source code of shallow clones (`git clone --depth 1`, read only, nothing built or run), taken on 2026-11-14:

| Repo | Commit | What was read |
|---|---|---|
| sourcegraph/scip | `be1d9f769a02a11be3b3ec5485231e2531a78a1c` | `scip.proto`, Rust and Go symbol parser, formatter, lint, sort, tests |
| kythe/kythe | `69141f022689a611e8a4a1d9b08a3783a2e8a9ed` | `storage.proto` (VName, Entry), `kytheuri`, `schema`, `graphstore`, `entryset`, verifier test data |
| facebookincubator/Glean | `bdc9a9235e0af30434f02c0b6515247576444ea8` | schema `.angle` files, `rts/factset.cpp`, `rts/validate.cpp`, schema docs |
| rust-lang/rust-analyzer | `114dcb8d6f50aca1ef14d67b831a7560362f8f69` | `lib/la-arena`, `lib/text-size`, `crates/vfs` (this clone was already in the shared scratch folder; a separate clone of `rust-analyzer/text-size` at `f5048daa` was made but the in-tree copy in `lib/text-size` is the one cited) |
| rust-lang/cargo | `382ecb8707b82592bb7176ba62b07590c29dee35` | Cargo.lock encoder, registry index (a JSON Lines format), `cargo-util-schemas`, `cargo-platform` cfg expressions |
| EmbarkStudios/cfg-expr | `7e43423c5e32b24f9da0ff65cf3cad5d07bfada9` | `src/expr.rs`, parser tests |
| sybila/biodivine-lib-bdd | `f2c39b81630d4fd1e3e51245c0dfebf11855811e` | size, dependencies, expression parser, fuzz tests |
| cfallin/boolean_expression | `795b89567e05f54907b89453bdd481d0b01f0c93` | `Expr`, BDD, tests |

URL prefixes used below (each link points at the cloned commit):
SCIP = `https://github.com/sourcegraph/scip/blob/be1d9f7...`, KY = `https://github.com/kythe/kythe/blob/69141f0...`, GL = `https://github.com/facebookincubator/Glean/blob/bdc9a92...`, RA = `https://github.com/rust-lang/rust-analyzer/blob/114dcb8...`, CG = `https://github.com/rust-lang/cargo/blob/382ecb8...`, CE = `https://github.com/EmbarkStudios/cfg-expr/blob/7e43423...`, BD = `https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b8...`, BE = `https://github.com/cfallin/boolean_expression/blob/795b895...`. Full hashes are in the table above.

One line per product:

- **SCIP**: a protobuf index format whose symbols are one string with a small formal grammar (scheme, package, then descriptors with suffix characters), with Rust and Go parsers, a formatter and a `lint` that rejects symbols that aren't in canonical form.
- **Kythe**: a graph of nodes and edges stored as flat `Entry` tuples (source VName, edge kind, target VName, fact name, fact value); a VName is five plain strings with strict evolution rules.
- **Glean**: a typed fact database with its own schema language (Angle); a fact's identity is its predicate plus its key (stored facts are deduplicated), and derived predicates are either stored or computed on demand.
- **rust-analyzer small crates**: `la-arena` (typed `u32` indices into a `Vec`), `text-size` (a `u32` newtype that checks its invariants in `Deserialize`), `vfs` `PathInterner` (an `IndexSet` that maps paths to `u32` IDs).
- **cargo**: Cargo.lock (TOML, hand-written output with an exact layout, a format `version` marker) and the registry index (JSON Lines, a schema version `v` on each line, lines it can't read are skipped).
- **cargo-platform / cfg-expr**: small Rust parsers and evaluators for `all/any/not` boolean expressions; cfg-expr stores them in postfix form and evaluates over a `Logic` trait that includes three-valued (Kleene) logic.
- **biodivine-lib-bdd / boolean_expression**: general BDD libraries. Both test their BDDs against brute-force truth tables, which is the useful part for Mental.

---

## Q1. How do these projects make identifiers stable, readable and parseable? What grammar, escaping and validation do they use, and how is it tested?

### Takeaway

Every mature format gives the identifier one canonical text form, a written grammar, and one parse and one format function, and treats "parse then format returns the input" as the definition of a valid ID (SCIP's `lint` enforces this literally). Escaping exists only because their names are arbitrary user text. Mental builds slugs itself from code names, so it can use a closed ASCII alphabet and skip escaping entirely. The biggest real-world bug class is the parser and formatter disagreeing on which characters are allowed.

### Cited Findings

- **SCIP's grammar is written out in EBNF in the schema file.** `<symbol> ::= <scheme> ' ' <package> ' ' (<descriptor>)+ | 'local ' <local-id>`. Each descriptor kind is a name plus one suffix character: namespace `/`, type `#`, term `.`, meta `:`, macro `!`, method `name(disambiguator).`, type parameter `[name]`, parameter `(name)`. — [SCIP scip.proto L153-L180](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/scip.proto#L153-L180)
- **SCIP's identity is the ancestry path of names, not positions.** "The list of descriptors for a symbol should together form a fully qualified name … it will include one descriptor for every node in the AST (along the ancestry path)". File-local entities get a separate `local <id>` form. — [SCIP scip.proto L182-L189](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/scip.proto#L182-L189)
- **SCIP needs two escaping schemes because its names are arbitrary.** Package fields escape a space by doubling it. Descriptor names outside `[_+\-$A-Za-z0-9]` are wrapped in backticks, with a backtick doubled inside. `.` is a placeholder for an empty package field. — [SCIP scip.proto L159-L178](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/scip.proto#L159-L178); [SCIP Rust escape_name L150-L159](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L150-L159)
- **SCIP's parser and formatter disagree on character classes.** In the Rust binding:
  - the formatter escapes anything that isn't ASCII alphanumeric (`is_ascii_alphanumeric`);
  - the parser accepts unescaped Unicode letters (`c.is_alphabetic() || c.is_digit(10)`);
  - `is_simple_identifier` uses Unicode `is_alphanumeric`.
  So an unescaped `Α#` parses but formats back as `` `Α`# ``: the round trip isn't canonical. The Go binding is ASCII-only and its tests require `nonSimpλeIdentifier.` to be rejected, so the two bindings accept different languages. — [SCIP Rust symbol.rs L32-L34, L150-L159, L296-L324](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L296-L324); [SCIP Go identifier.go L3-L5](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/identifier.go#L3-L5); [SCIP Go symbol_test.go L142](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_test.go#L142)
- **SCIP's validator defines canonical form by round trip.** `lintSymbolString` parses the symbol, formats it again with `VerboseSymbolFormatter`, and reports `nonCanonicalSymbolError{symbol, formatted, context}` when the two differ. — [SCIP cmd/scip/lint.go L249-L264](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/cmd/scip/lint.go#L249-L264)
- **SCIP's Rust error type is small.** `SymbolError` is a `#[non_exhaustive]` enum: `InvalidIndex`, `InvalidIdentifier(String)`, `MissingDescriptor`, `MissingCharacter(String)`, `EndOfSymbol(String)`, `InvalidLocalSymbol(String)`. One variant builds its message with `format!("{:?}", self)` of the whole parser state, so error text is unstable. — [SCIP Rust symbol.rs L7-L16, L372-L374](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L7-L16)
- **SCIP's Go parser was rewritten for performance.** It is a zero-allocation parser (571 lines, with `stringWriter` and `descriptorsWriter` slot writers) that can also validate without allocating (`ValidateSymbolUTF8`). It has a `parseCtx` enum for error context, with a TODO to enable an exhaustiveness linter. — [SCIP Go symbol_parser.go L10-L60, L318-L361](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_parser.go#L318-L361); [SCIP Go symbol.go L72](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol.go#L72)
- **SCIP's identifier tests use three patterns.**
  1. Table-driven parse tests that compare against expected structs.
  2. A table of malformed strings that must fail without panicking (`require.NotPanics`): empty input, a missing descriptor, a trailing string, an unknown suffix `@`, an unterminated backtick or bracket, `local 🧠`.
  3. Round-trip tests: `format(parse(s)) == s` for Java, Rust, macro and escape cases.

  No fuzz test or property test for symbol parsing was found; `rapid` property tests exist only for sorting and binary search. — [SCIP Go symbol_test.go L10-L155](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_test.go#L130-L155); [SCIP Go symbol_formatter_test.go L31-L50](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_formatter_test.go#L31-L50); [SCIP Rust symbol.rs tests L440-L670](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L640-L654)
- **Kythe's VName follows explicit design rules.** It has five fields: `signature`, `corpus`, `root`, `path`, `language`. "All fields must be strings, not messages"; "No field may ever be removed"; "New fields are always added to the end"; "we want as few fields as possible in a vname … find a minimal basis. Be conservative." — [KY storage.proto L23-L36](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L23-L36)
- **Kythe deliberately leaves time and revision out of names.** "time is not a good axis for a name -- a name should say 'what' something is, not 'when'". Timestamps and revisions are stored as facts instead. Paths must be normalized, with no `""`, `.` or `..` elements and no leading `/`. — [KY storage.proto L56-L70, L85-L90](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L85-L90)
- **Kythe defines fact names with a closed grammar.** `name = "/" | 1*path; path = "/" word; word = 1*{LETTER|DIGIT|PUNCT}`, with `PUNCT = [-.@#$%&_+:()]`. The schema's fact names are constants such as `/kythe/node/kind` and `/kythe/loc/start`. — [KY storage.proto L110-L117](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L110-L117); [KY schema/facts/facts.go](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/schema/facts/facts.go)
- **Kythe's URI form has a canonical rendering.** The string has a fixed query order and is %-escaped outside an unreserved set; `/` is left unescaped in paths. "The resulting string is in canonical ordering … parsing this string will always give back the same URI". Equality is defined as `u.String() == v.String()`. The tests include `TestRoundTripURI`, `TestRoundTripVName` and `TestUnicode`. — [KY kytheuri/uri.go L71-L80](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/kytheuri/uri.go#L71-L80); [KY kytheuri/escape.go L34-L47](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/kytheuri/escape.go#L34-L47); [KY kytheuri/uri_test.go L166-L256](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/kytheuri/uri_test.go#L166)
- **Glean's fact IDs are database-local numbers, and identity is the key.** Fact IDs are "a 64-bit integer that uniquely identifies the fact in a particular database", which the guide likens to "memory addresses". `FactSet::define` inserts by `(predicate, key)`:
  - a duplicate key with an equal value returns the existing ID (deduplication);
  - a duplicate key with a different value returns `Id::invalid()` (an error).
  — [GL docs/angle/advanced.md L33](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/angle/advanced.md); [GL rts/factset.cpp L278-L293](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/factset.cpp#L278-L293)
- **Cargo disambiguates only when needed.** In a lockfile dependency reference, "version and source are only included when necessary to disambiguate": the version is added only if names collide, and the source only if name and version collide. This was done "to reduce merge conflicts". — [CG cargo-util-schemas/src/lockfile.rs L210-L212](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-util-schemas/src/lockfile.rs#L210-L212); [CG src/resolver/encode.rs module docs](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/resolver/encode.rs)
- **cargo-platform tests canonical form in both directions.** Its test helpers are `good(s, expected)`, `good_canon(s, canon, expected)` (whitespace is normalized to one canonical string) and `bad(s, exact_error_message)`. Each runs both through `FromStr` and through serde (`assert_tokens` / `assert_de_tokens_error`), so text parsing and deserialization are proven identical. — [CG crates/cargo-platform/tests/test_cfg.rs L49-L96](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/tests/test_cfg.rs#L49-L96)

### Inferences

- **What Mental should copy:**
  - a grammar written in the spec and in a doc comment on `Id`;
  - one `parse` and one `Display`;
  - a validity rule of "parses and re-displays to the same bytes" (SCIP lint), enforced by rejecting input that isn't canonical, never by quietly normalizing it;
  - one character-class function used by the slugifier, the parser and the validator. This avoids SCIP's Rust/Go divergence.
- **What Mental should avoid (bloat for version one):**
  - backtick and double-space escaping, the `.` placeholder, package/version components, `SymbolFormatOptions`, and a zero-allocation parser. Mental's slugifier controls the alphabet: if every slug is `[a-z0-9]+(-[a-z0-9]+)*`, there is nothing to escape, and `/` and `:` can never appear inside a slug.
  - Kythe-style %-escaping, for the same reason.
- **Mental's grammar is a strict subset of SCIP's idea:**
  - SCIP's descriptor suffix plays the role of Mental's `kind-prefix:`;
  - SCIP's `local` form plays the role of Mental's behavior-local node IDs (`…/n:claim`).

  Using a prefix (`beh:`) instead of a suffix character makes IDs readable and lets the parser branch on the kind before reading the slug.
- **Use Glean's identity rule as Mental's duplicate rule.** Two records with the same ID and byte-identical canonical lines are deduplicated; two with the same ID and different content are an error naming both sources. This turns P9 ("identity comes from the code") into a checkable property.
- **Use cargo's "disambiguate only when needed" for unavoidable collisions,** such as two handlers with the same name in different files. Add a qualifying code name (the module, file stem or class) only when a collision exists, never a counter. This keeps most IDs short and stable when unrelated code changes.

### Gaps

- SCIP's actual canonicalization behavior for non-ASCII names wasn't run, by constraint. The asymmetry is inferred from reading the code: parser and formatter use different predicates. The Rust test `parses_non_ascii` only tests the escaped form `` `Α`# ``.
- No fuzz harness for symbol parsing was found in SCIP. That doesn't prove none exists elsewhere (for example in OSS-Fuzz configurations outside the repo).

---

## Q2. How do they keep the record schema flat, versioned and evolvable? How do they handle unknown fields, schema versions and deterministic ordering?

### Takeaway

Two families exist:

- **Shared formats with many writers and readers** (protobuf in SCIP and Kythe, Glean databases, the cargo registry index) evolve by adding optional fields, never reusing field numbers, and ignoring unknown data.
- **Derived artifacts owned by one tool** (Cargo.lock) use a single format version marker and an exact, hand-controlled output layout so that diffs are minimal.

Mental's model is the second kind ("derived, never edited, never merged"). It should take Cargo.lock's discipline (a version marker, a deterministic layout, a hard error on a version mismatch, no catch-all bag) and none of the forward-compatibility machinery.

### Cited Findings

- **Kythe's storage is maximally flat.** Everything is an `Entry {source VName, edge_kind, target VName, fact_name, fact_value bytes}`. Edge kind and target are "either both empty, or both nonempty". A node is just the set of its facts, so a new fact needs no schema change. — [KY storage.proto L101-L119](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto#L101-L119); [KY schema/schema.go Node.ToEntries](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/schema/schema.go)
- **Kythe defines a canonical entry order and a dedup step.**
  - VName order: corpus, language, path, root, signature, compared lexicographically.
  - Entry sets "are automatically deduplicated" and a `Canonicalize()` call restores canonical order after `Add`.
  - Stream dedup is hash-based and documented as "quick-and-dirty … it will not be perfect".

  — [KY vnameutil/order.go L27-L40](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/vnameutil/order.go#L27-L40); [KY storage/entryset/entryset.go L17-L60, L251](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/storage/entryset/entryset.go#L251); [KY platform/delimited/dedup/dedup.go](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/platform/delimited/dedup/dedup.go)
- **SCIP evolves with protobuf rules.** It reserves removed field numbers ("Reserved field numbers … to prevent accidental reuse, which would break wire compatibility"), marks deprecated enum values and fields with `[deprecated = true]` while keeping them, and has a `ProtocolVersion` enum whose zero value is `UnspecifiedProtocolVersion`. — [SCIP scip.proto L43-L57, L214, L245-L248, L693-L709](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/scip.proto#L245-L248)
- **SCIP canonicalizes and sorts in a library layer.** `CanonicalizeDocument` and `CanonicalizeSymbols` sort and deduplicate, and `FindSymbolBinarySearch` requires a canonicalized document. — [SCIP Go canonicalize.go L11-L93](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/canonicalize.go); [SCIP Go sort.go L11-L37](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/sort.go#L25)
- **Glean allows only compatible schema changes.**
  - Compatible changes are: adding or removing a field with a *defaultable* type, adding or removing a sum-type alternative, adding or removing a predicate, and switching between list and set.
  - Changing a field's type is incompatible and is handled by bumping the whole schema version (`schema graphql.1` → `graphql.2`).
  - Every non-predicate type has a fixed default (`nat` 0, `string` "", `[T]` [], `maybe T` nothing, an enum's first alternative).
  — [GL docs/schema/changing.md](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/schema/changing.md)
- **Glean's design advice covers the same trade-offs as Mental's records.**
  - *Nested arrays versus separate facts:* arrays are more compact and faster to read, while separate facts let you generate data independently and support incremental changes.
  - *Key-value ("functional") predicates* store the value once and make duplicate keys with different values an error.
  - *Arrays versus sets:* if order isn't meaningful, forcing an array makes the data "non-deterministic which leads to spurious differences in things like test outputs".
  — [GL docs/schema/design.md](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/schema/design.md); [GL docs/angle/advanced.md L66-L79](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/angle/advanced.md)
- **Glean separates stored facts from derived ones.** "Stored" derived predicates are computed and written into the database. "On demand" derived predicates are computed at query time and never stored, and the schema design advice "do[es]n't apply" to them. — [GL docs/derived.md L10-L27, L84-L86](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/website/docs/derived.md)
- **Glean's schemas are versioned per namespace and keep deprecated types.** Examples: `schema src.1 { … }`, `predicate FileDigest : File -> string`, and `# DEPRECATED type ByteRange`. — [GL schema/source/src.angle L3-L40](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/schema/source/src.angle)
- **Cargo.lock's format policy.**
  - New format versions are added as `ResolveVersion` variants, but the default is flipped only after a long wait.
  - A `version` marker at the top lets old Cargo "give a formal error if they see a lock file from a super-future Cargo".
  - The file starts with an `@generated` comment.
  - The generic `[metadata]` string map, an early forward-compatibility attempt, "is nowadays sort of deemed a bad idea".
  — [CG src/resolver/encode.rs L1-L110](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/resolver/encode.rs)
- **Cargo.lock output is written by hand for exact layout.** `serialize_resolve` writes the header, `version = N`, then each `[[package]]` through a dedicated emitter, then trims trailing blank lines for V2 and later. Packages are sorted (`ids.sort()`, `deps.sort()`, `unused.sort()`). The file is rewritten only if the content differs (`are_equal_lockfiles` compares lines first). — [CG src/ops/lockfile.rs L138-L243](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/ops/lockfile.rs#L138-L243); [CG src/resolver/encode.rs L505-L612](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/resolver/encode.rs#L511)
- **The cargo registry index is a JSON Lines file with a schema version on each line.** `v: Option<u32>` defaults to 1. "Entries with unknown versions are ignored". The reader skips any line it can't parse ("ignoring everything that we ourselves don't understand") and classifies lines as `Unsupported`, `Invalid`, `Yanked` or `Candidate`. `INDEX_V_MAX = 2`. No `deny_unknown_fields` was found in `cargo-util-schemas/src`. — [CG cargo-util-schemas/src/index.rs L65-L87](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-util-schemas/src/index.rs#L65-L87); [CG src/sources/registry/index/mod.rs L52, L577-L600, L754-L770](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/sources/registry/index/mod.rs#L577-L600)

### Inferences

- **Typed records, not entity-attribute-value tuples.** Kythe's generic entry is the most evolvable layout, but every reader must re-validate and the compiler can't help. Mental already has a fixed set of record kinds that the interpreter must understand ("every kind must have a meaning in the step function"). One serde struct per kind gives compile-time checks for free. Copy Kythe's *spirit*: a minimal identity, everything else as plain fields, and time and revision never in the name.
- **Reject unknown fields** (`#[serde(deny_unknown_fields)]`). Unlike the cargo index or protobuf, Mental's model has exactly one writer (the same `mentalc` version) and is always rebuildable. Skipping unknown data would hide compiler/model version skew, which the user's "fail rather than degrade" rule forbids. This deliberately departs from SCIP, Kythe and the cargo index; the reason is the single writer, not taste.
- **Mental's version strategy:**
  - one format version for the whole model, not per record (per-line `v` is cargo-index bloat for a single-writer artifact);
  - a mismatch is a hard error telling the user to rebuild;
  - no migration code in version one, because the model is derived.
- **The version marker's location is an open choice** (see Gaps): a tiny `.mental/model/format.json`, or a first header line in each JSONL file.
- **Copy Cargo.lock's output discipline:**
  - lines sorted bytewise by ID;
  - field order fixed by struct declaration order (serde_json keeps it);
  - absent optional fields omitted, never written as `null`; empty sets omitted, never `[]`;
  - a final newline;
  - files rewritten only when the bytes change, so mtimes and downstream caches stay still.
- **No catch-all bag.** Cargo regrets `[metadata]`. A `serde_json::Value` or `BTreeMap<String,String>` "extra" field on records would be exactly that.
- **Apply Glean's array-versus-set rule to each field:**
  - Ordered lists (meaningful order): `steps`, `transitions`, a behavior's `cases` (if order is display-only, sort them too).
  - Sets, stored as sorted, deduplicated `Vec`s and validated as such: `checks`, `tags`, `decides`, `written_by`, `evidence`.
- **Map Glean's stored/on-demand split onto Mental's checks:** computed facts (for example "written_by", which can be derived from behaviors' write nodes) should be either stored *and* checked against a recomputation, or not stored at all. Duplicated derived data that is never checked drifts.
- **Keep lines small (Kythe's flat facts, Glean's separate facts).** Anchors already live in `anchors.jsonl` (spec 4.4). The same reasoning argues for keeping a behavior's full graph (`lod: full`) out of the summary line when it gets big; this is a version-two question.

### Gaps

- None of the studied Rust codebases writes JSONL with a written determinism contract. Cargo.lock (TOML) is the closest. The cargo index is JSONL but append-ordered by publish time, not sorted.
- Where to put the format version marker isn't settled by any source; it is a design choice for Mental.

---

## Q3. How are references between records validated (dangling IDs), and how are validation errors reported?

### Takeaway

Validators in these projects are whole-set passes:

- they check that every reference resolves, and resolves to a record of the *expected type* (Glean's "fact type mismatch", SCIP's "missing symbol");
- they collect every error, not just the first (SCIP's `errorSet`, deduplicated and sorted);
- they report each error with the offending ID and context.

Mental's kind prefix in every ID makes the type check cheap. Sorting records makes lookup a binary search.

### Cited Findings

- **Glean checks reference types during validation.** `validate` walks facts in ID order and checks, for every referenced ID:
  - it is in range ("id {} out of range, expecting id of type {}");
  - its predicate matches the expected one ("fact type mismatch. Expected {} to be of type {} … but got {}").

  It also checks that enumeration is in order ("enumeration out of order"), that each fact re-typechecks to the same bytes ("invalid fact"), and that `idByKey(type, key) == id` (key uniqueness). — [GL rts/validate.cpp L50-L89, L105-L140](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/validate.cpp#L66-L89)
- **Glean's `define` contract is documented on the interface.** It returns "an existing id if the fact is already in the set, a new id if it's a new fact and Id::INVALID if the set contains a fact with the same type and key but a different value". The `max_ref` argument is only an optimization hint ("Not setting it … is always safe"). Reference integrity is therefore checked by `validate`, not guaranteed when facts are defined. — [GL rts/define.h L25-L35](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/define.h#L25-L35)
- **Kythe checks entry shape.** `ValidEntry` has four rules:
  - an entry must have a source and a fact name;
  - an edge must have a target;
  - a node fact must not have a target ("node fact entry has extraneous target").

  It returns the first error. — [KY services/graphstore/graphstore.go L134-L147](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/services/graphstore/graphstore.go#L134-L147)
- **SCIP's lint builds a symbol table and reports one typed error per broken rule.** Examples: `missingSymbolForOccurrenceError`, `missingSymbolInRelationshipError`, `bothLocalAndExternalSymbolError`, `forwardDefIsDefinitionError`, `invalidOccurrenceRangeError`, `emptyStringError`, `nonCanonicalSymbolError`, and warnings for duplicates. Errors go into an `errorSet`; `Unique()` deduplicates them by message and sorts them by message for stable output. — [SCIP cmd/scip/lint.go L41-L245, L288-L435](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/cmd/scip/lint.go#L414-L435)
- **cargo-platform and cfg-expr report parse errors with structure.**
  - cargo-platform: `ParseError {kind, orig}`, where `kind` is a `#[non_exhaustive]` enum (`UnexpectedToken{expected, found}`, `IncompleteExpr`, `UnterminatedExpression(rest)`, …). `Display` renders "failed to parse `{orig}` as a cfg expression: {kind}".
  - cfg-expr adds `span: Range<usize>` and a `reason` enum, and its tests compare the whole error struct, span included.
  — [CG crates/cargo-platform/src/error.rs](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/error.rs); [CE src/error.rs L5-L13](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/error.rs#L5-L13); [CE tests/parser.rs err! macro](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/tests/parser.rs)
- **cargo-platform rejects trailing input.** `FromStr for CfgExpr` fails with `UnterminatedExpression(rest)` if anything follows a complete expression, so `foo(a)` doesn't silently parse as `foo`. — [CG crates/cargo-platform/src/cfg.rs L178-L191](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/cfg.rs#L178-L191)
- **SCIP looks symbols up by binary search over a sorted, canonical list.** A `rapid` property test checks that every inserted symbol is found and that absent symbols aren't. — [SCIP Go sort.go L25](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/sort.go#L25); [SCIP Go sort_test.go L129-L160](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/sort_test.go#L129-L160)

### Inferences

- **Mental's reference check, as a pure function `check(&Model) -> Vec<Diagnostic>`.** It should report:
  1. *syntax*: every ID-valued field parses;
  2. *dangling*: the referenced record exists;
  3. *wrong kind*: a field declared to hold a port reference points at a record whose kind prefix isn't `port`, as in Glean's typed refs;
  4. *orphan*: every ID's parent prefix (the text before the last `/`) exists as a record, except roots;
  5. *duplicate ID*, following the Glean rule from Q1;
  6. *unsorted or duplicate set fields*.
- **Never stop at the first error** (Kythe's `ValidEntry` does). Sort and deduplicate diagnostics (SCIP `Unique()`), but sort by `(record_id, code, field)`, not by message text, so a wording change doesn't reorder output.
- **A diagnostic is a small struct** with a `code` (a closed enum, serialized as a stable kebab-case string), `id`, `field` (a JSON path such as `cases[1].when`), `message` and an optional `related` ID. These lines are what goes into `diagnostics.jsonl` (spec 4.9).
- **Parse errors carry the original string and a byte span** (cfg-expr). The spec's guard strings and IDs are short, so span plus original is enough for good messages without a source-map library.
- **Mental's "anchor check" is the same pattern on a second file:** every anchor ID refers to an existing record, and every record whose kind requires an anchor has one whose status isn't `broken`.

### Gaps

- None of the sources validates *relative* references like Mental's `pred:range-too-large` inside a guard, or `.../case:skip` in spec 4.7. SCIP's `local` symbols are document-scoped but not resolved relative to a parent. Mental needs its own written rule:
  - *option one:* resolve against the nearest ancestor entity that has a child with that kind and slug;
  - *option two:* store only full IDs and allow abbreviations only in human-facing display.

  The second is simpler and makes the reference check purely textual. This is a design decision, not a finding.

---

## Q4. What is the minimal Rust API shape for such a crate (types, ID newtypes, arenas, interning) that stays simple?

### Takeaway

The Rust building blocks that last are tiny:

- a newtype whose constructor enforces the invariant;
- `Deserialize` implemented *through* that constructor, so invalid data can't enter through serde;
- derived `Ord` so sorting is free.

Arenas and interners (`la-arena`, `PathInterner`) pay off for millions of short-lived items in an IDE. For a model of a few thousand records loaded once, `Vec<Record>` sorted by `Id` with binary search is simpler, and interning would only add dependencies.

### Cited Findings

- **`text-size` keeps invariants behind a private field.** `TextSize` is `#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct TextSize { pub(crate) raw: u32 }`: no public constructor that bypasses checks, and `TryFrom<usize>` for fallible conversion. — [RA lib/text-size/src/size.rs L15-L26](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/src/size.rs#L23-L26)
- **`text-size` validates in `Deserialize`.** `Deserialize for TextRange` reads `(start, end)` and returns `de::Error::custom("invalid range: {start}..{end}")` if `start > end`. A test asserts that exact error with `serde_test::assert_de_tokens_error`. — [RA lib/text-size/src/serde_impls.rs L34-L44](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/src/serde_impls.rs); [RA lib/text-size/tests/serde.rs L40-L50](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/tests/serde.rs#L40-L50)
- **`text-size` checks auto traits at compile time.** `assert_impl_all!(TextSize: Send, Sync, Unpin, …, Copy, Debug, Default, Hash, Ord)`, using the `static_assertions` crate. — [RA lib/text-size/tests/auto_traits.rs](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/tests/auto_traits.rs)
- **`la-arena`'s typed index.**
  - `Idx<T>` is `{ raw: RawIdx(u32), _ty: PhantomData<fn() -> T> }`.
  - `Clone`, `Copy`, `Eq`, `Ord` and `Hash` are implemented *by hand*, so they don't require `T: Clone` and so on (a derive would add those bounds).
  - `Arena<T>` is just `{ data: Vec<T> }`.
  - The crate is 524 lines in `lib.rs` plus 304 in `map.rs`, including `IdxRange` and `ArenaMap`.
  — [RA lib/la-arena/src/lib.rs L59-L105, L251-L256](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/la-arena/src/lib.rs#L59-L105)
- **`vfs` interns paths with an `IndexSet`.** `PathInterner` wraps an `IndexSet<VfsPath, BuildHasherDefault<FxHasher>>`, so it pulls in the `indexmap` and `rustc-hash` crates. It maps paths to `FileId(u32)` and never frees: "the assumption is total size of paths we ever look at is not too big". `FileId::from_raw` asserts `raw <= 0x7fff_ffff`. — [RA crates/vfs/src/path_interner.rs L1-L43](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/vfs/src/path_interner.rs); [RA crates/vfs/src/lib.rs L62-L76](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/crates/vfs/src/lib.rs#L62-L76)
- **cargo-platform's model is plain enums with derived traits.** `CfgExpr` derives `Eq, PartialEq, Hash, Ord, PartialOrd, Clone, Debug`, with variants `Not(Box)`, `All(Vec)`, `Any(Vec)`, `Value(Cfg)`, `True`, `False`. It implements `FromStr`, `Display`, `Serialize` (via `Display`) and `Deserialize` (via `FromStr`). The whole crate is 469 + 67 + 183 lines. — [CG crates/cargo-platform/src/cfg.rs L8-L16, L132-L150, L178-L230](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/cfg.rs#L8-L16)
- **cfg-expr keeps the original text and stores the expression in postfix form.** `Expression { expr: SmallVec<[ExprNode; 5]>, original: String }`. Its docs warn that `PartialEq` is "syntactical", not semantic. — [CE src/expr.rs L559-L564, L698-L705](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/expr.rs#L559-L564)
- **SCIP's Rust symbol API is free functions over protobuf-generated structs** (`parse_symbol`, `format_symbol`, `is_local_symbol`). The generated types carry `special_fields: SpecialFields`, which leaks into every constructor and test. — [SCIP Rust symbol.rs L161-L213](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/rust/src/symbol.rs#L171-L213)

### Inferences

Proposed minimal shape for `model` (to judge against the "compact, no bloat" goal):

- **`Id`**
  - `pub struct Id(String)` with a private field;
  - `Id::parse(&str) -> Result<Id, IdError>`, `Display`, `AsRef<str>`, and derived `Ord, Eq, Hash, Clone, Debug`;
  - accessors `parent() -> Option<&str>` (or `Option<Id>`), `kind() -> Kind` and `slug() -> &str`;
  - `Serialize` writes the string; `Deserialize` calls `Id::parse` (the text-size pattern), so `serde_json::from_str::<Entity>` can never produce a bad ID;
  - a builder `Id::child(&self, Kind, &Slug)` so `extract` never formats IDs by hand.

  The `Ord` of `Id` must equal byte order of the string, because the JSONL files are sorted that way. Derive `Ord` on the `String` and add a test that checks it.
- **`Kind`**: a closed `enum` of kind prefixes (`port`, `beh`, `store`, `var`, `pred`, `case`, `n`, `in`, `class`, …), with `as_str` and `FromStr`. Every ID-valued field documents its expected `Kind`, enforced by the checker.
- **`Slug`** (optional): a newtype for the validated slug, plus a `slugify(code_name) -> Result<Slug, …>` that is the only path from code names to slugs.
- **Records**: one `#[derive(Serialize, Deserialize)] #[serde(deny_unknown_fields)]` struct per kind, plus a `Model` holding one `Vec<T>` per kind, sorted by ID. Lookup is `binary_search_by(|r| r.id.cmp(id))` (SCIP's canonical-then-binary-search).
- **No arenas, interning, `SmallVec`, `smol_str` or `indexmap`.** They need dependencies outside the allowed set (serde, serde_json, blake3) and solve performance problems that a few-thousand-record model doesn't have. If profiling ever shows ID string cost, a 30-line `Idx<T>` copied from la-arena's pattern (manual trait impls, `PhantomData<fn() -> T>`) is the upgrade path, inside the crate and with no new dependency.
- **A trait for "record with ID"** (`fn id(&self) -> &Id`) lets one generic checker and one generic JSONL reader/writer serve every kind.
- **No `#[non_exhaustive]`** on error enums inside the workspace. SCIP and cargo-platform use it because they are published crates with external users. In an internal workspace it only forces wildcard arms, which weakens exhaustiveness checks.
- **Content hashes stay outside the identity:** `blake3` over the canonical line bytes is computed on demand, or stored beside the record as a cache key (spec 4.3), never as part of `Id`.

### Gaps

- No studied project uses exactly "string ID, sorted Vec, binary search" in Rust. The recommendation combines SCIP's Go approach (canonical sorted list plus binary search) with text-size's newtype pattern.
- Scale numbers for Mental models (records per repo) are not measured yet. The claim "a few thousand" comes from the spec's report-jobs example, not from data.

---

## Q5. What testing strategies do they use for the format layer, and which fit Mental?

### Takeaway

The highest-value, lowest-cost tests seen are:

1. table-driven `good` / `good_canon` / `bad` cases that assert exact error text and go through both `FromStr` and serde (cargo-platform);
2. round-trip properties, `format(parse(s)) == s` and `parse(format(x)) == x` (SCIP, Kythe);
3. a brute-force oracle for anything clever: BDDs are tested against truth tables (boolean_expression, biodivine);
4. golden snapshots of whole outputs (SCIP's reprolang snapshots, Kythe's verifier fixtures).

Randomized tests must use fixed or persisted seeds: biodivine does, boolean_expression doesn't.

### Cited Findings

- **cargo-platform's exact-error tests.** `bad::<CfgExpr>("all(a", "failed to parse `all(a` as a cfg expression: expected `)`, but cfg expression ended")`, about 20 such cases. Each `bad` case also checks `assert_de_tokens_error`, proving that serde reports the same message. `good_canon(" foo  ", "foo", …)` pins the canonical rendering. — [CG crates/cargo-platform/tests/test_cfg.rs L49-L135, L199-L237](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/tests/test_cfg.rs#L199-L237)
- **cfg-expr's whole-struct error tests.** The tests use `err!(text => Reason @ span)` and compare a whole `ParseError {original, span, reason}` with `similar_asserts::assert_eq!`. — [CE tests/parser.rs L27-L40](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/tests/parser.rs#L27-L40)
- **SCIP's round-trip and malformed-input tests.** The round-trip tests are `TestSymbolFormatterRoundTrip` (Go) and `parses_java_symbol`, which asserts `input == format_symbol(parse_symbol(input))` (Rust). The malformed-input table runs under `require.NotPanics`. — [SCIP Go symbol_formatter_test.go L31-L50](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_formatter_test.go#L31-L50); [SCIP Go symbol_test.go L136-L155](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/symbol_test.go#L136-L155)
- **SCIP's property tests (Go `rapid`) and snapshot tests.**
  - Property test: `TestFindSymbolBinarySearch` draws up to 10 random symbols, canonicalizes them, and checks that each is found and that a random other string is found exactly when present.
  - Snapshot tests: the `reprolang` test language has `testdata/snapshots/input` and `…/output` directories.
  — [SCIP Go sort_test.go L129-L160](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/bindings/go/scip/sort_test.go#L129-L160); [SCIP reprolang/testdata/snapshots](https://github.com/sourcegraph/scip/tree/be1d9f769a02a11be3b3ec5485231e2531a78a1c/reprolang/testdata/snapshots)
- **Kythe writes test expectations as comments in the fixture source.** Assertions like `//- @Pinky defines/binding Pinky` and `//- Pinky.node/kind variable` sit next to the code they describe, and the verifier checks the emitted facts against them. — [KY go/indexer/testdata/structinit.go](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/indexer/testdata/structinit.go)
- **Kythe tests URI round trips.** `TestRoundTripURI` and `TestRoundTripVName` check string → URI → string and VName → URI → VName. — [KY kytheuri/uri_test.go L166-L253](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/go/util/kytheuri/uri_test.go#L166)
- **boolean_expression tests its BDD against a truth table, but with an unseeded RNG.** For random expressions over 6 variables, it checks for all 2^6 assignments that `Expr::evaluate == BDD::evaluate`, and that `sat_one` is consistent. The RNG is seeded from `rand::thread_rng()`, so failures can't be reproduced. — [BE src/bdd.rs L755-L800](https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/bdd.rs#L755-L800)
- **biodivine fuzzes against a truth table with fixed seeds.** It builds random operation trees, then "exhaustively check[s] whether all valuations actually match … To get predictable test cases, we use a predefined set of randomness seeds", usually "up to 12 variables". — [BD src/_test_bdd/_test_bdd_logic_fuzzing.rs L1-L18, L48-L49, L152](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_test_bdd/_test_bdd_logic_fuzzing.rs#L1-L18)
- **text-size tests serde at the token level.** It uses `serde_test::assert_tokens` and `assert_de_tokens_error`, and compile-time trait checks via `static_assertions`. — [RA lib/text-size/tests/serde.rs](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/tests/serde.rs)

### Inferences

Test plan for the `model` crate, mapped to the decided tools (insta, proptest, cargo-mutants, 100% diff coverage):

- **Unit tests in `id/tests.rs` and `guard/tests.rs`:** cargo-platform-style helpers `good(s)`, `good_canon(s, canon)` (only if any normalization is accepted; preferably none), and `bad(s, "exact message")`. Each helper also runs through `serde_json::from_str::<Id>(&format!("\"{s}\""))`, so serde can't diverge from `FromStr`.
- **Round-trip properties (proptest):**
  - For a generator of *valid* IDs and guards: `parse(display(x)) == x`.
  - For *arbitrary* strings: `parse(s).map(|x| x.to_string()) == Ok(s)` or `parse(s)` is an error. Nothing accepted may re-display differently, which is SCIP's canonical rule as a property and catches the SCIP Rust asymmetry class of bug.
  - For records: `read(write(model)) == model`, and `write(read(bytes)) == bytes` on golden files.
  - Proptest regressions are persisted (`proptest-regressions/` committed). This is the biodivine lesson; boolean_expression's unseeded test is the anti-pattern.
- **Oracle tests for the guard checker:** a brute-force evaluator over all 2^n assignments (n ≤ 10 in tests) is the specification; the production checker must agree with it on proptest-generated guard sets. Copy the BDD libraries' oracle structure without copying the BDDs.
- **insta snapshots:**
  1. canonical JSONL output for a small handcrafted model (golden "format freeze": any byte change in the format is a reviewed snapshot change, like SCIP's reprolang snapshots);
  2. `diagnostics.jsonl` for a set of broken fixture models, one fixture per diagnostic code.
- **A completeness test:** iterate over every `DiagnosticCode` variant (a `const ALL: &[DiagnosticCode]` plus an exhaustive `match`, so a new variant won't compile until it is listed) and assert each has at least one fixture that triggers it. This gives the "outstanding tests" goal a mechanical check and makes cargo-mutants effective on the checker.
- **Kythe-style inline fixture assertions are valuable for the `extract` crate** (source fixtures annotated with the records they must produce), not for `model`.
- **Fuzzing (cargo-fuzz)** is probably redundant in version one, given proptest on arbitrary strings for `Id::parse` and `Expr::parse`. Neither SCIP nor cargo-platform has a fuzz target for its parser. Add one only if a parser grows past the tiny grammars proposed here.

### Gaps

- Mutation testing (cargo-mutants) wasn't observed in any studied repo, so there is no evidence here about its yield on format code.
- SCIP's reprolang snapshot runner code wasn't read in detail. The directory layout is cited; the update workflow isn't.

---

## Q6. Guard algebra: how do small crates parse and evaluate boolean expressions, and what is the most compact way to check that 1-30 guards over named predicates cover everything and never overlap?

### Takeaway

Parse guards into a tiny enum `Pred | Not | And(Vec) | Or(Vec)` (cargo-platform's shape) with a canonical `Display`, and evaluate them over a `Logic` trait so the same code serves two-valued and three-valued (Kleene) evaluation (cfg-expr).

For partition and overlap, enumerate truth assignments over only the predicates that the behavior's guards mention. Cap this at **n ≤ 16 distinct predicates** (65,536 assignments). Above the cap, either fail with a diagnostic or switch to a Kleene case-split search that prunes, within the same API. BDDs and SAT are bloat for version one: the BDD crates are 2,400 to 10,600 lines with dependencies that Mental's dependency policy forbids.

### Cited Findings

- **cargo-platform's evaluator is recursive and short.** `CfgExpr::matches`, using `iter().all/any`, is about 10 lines. Its parser is a hand-written recursive descent over `all(…)`, `any(…)`, `not(…)` and atoms, about 100 lines with the tokenizer. — [CG crates/cargo-platform/src/cfg.rs L166-L175, L251-L290](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/src/cfg.rs#L166-L175)
- **cfg-expr's evaluator.**
  - It evaluates postfix nodes with a value stack.
  - It is generic over `trait Logic { top, bottom, and, or, not }`, with implementations for `bool` and for `Option<bool>` as Kleene three-valued logic: `None` means unknown, false absorbs `and`, true absorbs `or`.
  - `all()` of nothing is `top()`, `any()` of nothing is `bottom()`.
  — [CE src/expr.rs L630-L680, L735-L826](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/expr.rs#L735-L826)
- **boolean_expression has a silent default.** `Expr::evaluate` treats a variable missing from the assignment as `false` (`*vals.get(t).unwrap_or(&false)`). The crate is 2,370 lines (BDD 1,083, cubes, simplify, IDD) and depends on `smallvec` and `itertools`. — [BE src/expr.rs L129-L137](https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/expr.rs#L129-L137); [BE Cargo.toml](https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/Cargo.toml)
- **biodivine-lib-bdd is large.**
  - It is about 10,570 lines over 43 source files and depends on `fxhash`, `rand`, `num-bigint`, `num-traits` and `num-rational`.
  - Variables are `u16`, capped at `u16::MAX - 1`.
  - It offers `is_true`, `is_false`, `and_not`, `sat_witness` and `exact_cardinality` (a `BigUint`).
  - Its expression parser supports `! & | ^ => <=>` and `?:`, with precedence `<=>`, `=>`, `|`, `&`, `^`.
  — [BD Cargo.toml L22-L28](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/Cargo.toml#L22-L28); [BD src/_impl_bdd/_impl_util.rs L138-L147, L271](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_impl_bdd/_impl_util.rs#L138-L147); [BD src/boolean_expression/_impl_parser.rs L1-L27](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/boolean_expression/_impl_parser.rs#L1-L27); [BD src/_impl_bdd_variable_set.rs L11-L14](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_impl_bdd_variable_set.rs#L11-L14)
- **Both BDD crates use exhaustive truth-table enumeration as their test oracle**: 6 variables in boolean_expression, up to 12 in biodivine. — [BE src/bdd.rs L755-L775](https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/bdd.rs#L755-L775); [BD _test_bdd_logic_fuzzing.rs L1-L18](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_test_bdd/_test_bdd_logic_fuzzing.rs#L1-L18)

### Inferences

- **Grammar for Mental guards.** The spec's examples use `!`, `&` and atoms `pred:<slug>` (spec 4.7). Proposed:

  ```text
  expr := or
  or   := and ('|' and)*
  and  := not ('&' not)*
  not  := '!' not | atom
  atom := ID | '(' expr ')'
  ```

  with canonical `Display`: single spaces around `&` and `|`, minimal parentheses, and `!` binding tighter. Leave out `=>`, `<=>`, `^` and `?:` (biodivine's extras are bloat here). Represent it as `enum Expr { Pred(Id), Not(Box<Expr>), And(Vec<Expr>), Or(Vec<Expr>) }`, plus `True` / `False` only if a real guard needs them (an `otherwise` case can be represented as the complement and need not be stored as an expression).
- **Unknown predicate names are a link-time error, never "false".** boolean_expression's `unwrap_or(false)` is the anti-pattern, because it would turn a typo into a silently unreachable case.
- **Recommended algorithm (most compact): plain enumeration.**
  - Collect the distinct predicates in a behavior's guards (n of them) and map each to a bit index.
  - For each `mask in 0..(1u32 << n)`, evaluate every guard, about 15 lines.
  - A mask where 0 guards are true is a *gap*; a mask where 2 or more are true is an *overlap*.
  - Report the first (lowest) such mask as a **witness assignment** naming the predicates, for example "gap when status-is-pending=false, upload-ok=true". This is deterministic and makes error messages concrete.
  - Cost is 2^n × (number of guards) × (guard size). At n = 16, 30 guards and about 5 nodes each, that is about 10^7 node evaluations, which is milliseconds. This estimate isn't measured.
- **The cap (n ≤ 16)** follows the "fail rather than degrade" rule: above it, emit a `guard-check-too-large` diagnostic instead of skipping the check.
- **If real behaviors exceed 16 predicates, use a Kleene case split** (DPLL without clause learning, about 40 lines) behind the same function signature:
  - evaluate all guards under a *partial* assignment with `Option<bool>` (cfg-expr's `Logic`);
  - if two guards are `Some(true)`, there is an overlap; if all are `Some(false)`, there is a gap; if exactly one is `Some(true)` and the rest are `Some(false)`, this branch is covered;
  - otherwise branch on an unassigned predicate that appears in an undecided guard.

  Guards that are short conjunctions (the spec's examples) prune after a few levels, so 30 predicates is fine in practice. The worst case stays exponential.
- **Keep plain enumeration as the proptest oracle** for the case split (n ≤ 10), as the BDD crates do with truth tables.
- **Why not BDDs or SAT:** both exceed the allowed dependencies (serde, serde_json, blake3). Writing either by hand is hundreds to thousands of lines to solve an n ≤ 30 problem that enumeration or a 40-line split handles. Revisit only if guards start to span hundreds of predicates.
- **Reuse three-valued logic for ordered reads.** The same `Option<bool>` evaluation directly implements spec 4.7's "Ordered reads" rule: before the node that `decides` a predicate, that predicate is `None` (unknown), and a guard that evaluates to `None` can't be taken yet. One `Logic` trait serves both the checker and the interpreter. This is an argument for putting the expression type and evaluator in `model`, not in `mentalvm`.

### Gaps

- **Predicates aren't always independent.** For example, `status-is-pending` and `status-is-done` over the same enum `state_var` can't both be true. Plain enumeration would report a spurious overlap for that impossible assignment, and a spurious gap when every value is listed but "none true" is impossible. No studied crate addresses domain constraints. Mental needs a rule, such as declaring predicate groups over one enum variable as "exactly one" or "at most one" and skipping infeasible masks. That is a design decision to make before implementing.
- The cost estimate for enumeration is arithmetic, not a benchmark.

---

## Q7. Candidate guidelines for the `model` crate (enforceable rules, rationale, source, enforcement)

### Takeaway

About two dozen rules cover identity, format, validation, API and tests. Most can be enforced by tests the crate needs anyway: round-trip properties, exact-error tables, snapshot goldens and the diagnostic-fixture completeness test. A few need clippy configuration (`disallowed_types`) or a CI dependency allowlist. The rest are reviewer rules, and each states the bloat it prevents.

### Cited Findings

- Each guideline below cites its source; the evidence is in Q1-Q6 above. Sources: SCIP [lint.go](https://github.com/sourcegraph/scip/blob/be1d9f769a02a11be3b3ec5485231e2531a78a1c/cmd/scip/lint.go), Kythe [storage.proto](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/storage.proto), Glean [factset.cpp](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/factset.cpp) and [validate.cpp](https://github.com/facebookincubator/Glean/blob/bdc9a9235e0af30434f02c0b6515247576444ea8/glean/rts/validate.cpp), text-size [serde_impls.rs](https://github.com/rust-lang/rust-analyzer/blob/114dcb8d6f50aca1ef14d67b831a7560362f8f69/lib/text-size/src/serde_impls.rs), cargo [encode.rs](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/resolver/encode.rs), [lockfile.rs](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/src/ops/lockfile.rs) and [test_cfg.rs](https://github.com/rust-lang/cargo/blob/382ecb8707b82592bb7176ba62b07590c29dee35/crates/cargo-platform/tests/test_cfg.rs), cfg-expr [expr.rs](https://github.com/EmbarkStudios/cfg-expr/blob/7e43423c5e32b24f9da0ff65cf3cad5d07bfada9/src/expr.rs), biodivine [fuzzing tests](https://github.com/sybila/biodivine-lib-bdd/blob/f2c39b81630d4fd1e3e51245c0dfebf11855811e/src/_test_bdd/_test_bdd_logic_fuzzing.rs), boolean_expression [expr.rs](https://github.com/cfallin/boolean_expression/blob/795b89567e05f54907b89453bdd481d0b01f0c93/src/expr.rs).

### Inferences

#### Identity

1. **IDs are only built by `Id::parse` or `Id::child`; the inner `String` is private, and `Deserialize` calls `Id::parse`.**
   - *Why:* invalid IDs can't enter through serde or ad-hoc `format!`.
   - *Source:* text-size `Deserialize for TextRange`; cargo-platform `Deserialize` via `FromStr`.
   - *Enforce:* a test that deserializing a bad ID fails with the same message as `parse`; reviewer check for `format!` building IDs outside `model` (a CI grep is possible).
2. **Every accepted ID is canonical: `Id::parse(s)?.to_string() == s`. The parser rejects input that isn't canonical instead of normalizing it.**
   - *Why:* one text form per identity, so sorting, diffs and text tools are reliable.
   - *Source:* SCIP `lintSymbolString` / `nonCanonicalSymbolError`; Kythe "canonical ordering" `String()`.
   - *Enforce:* a proptest over arbitrary strings: accepted implies identical re-display.
3. **One function defines the slug alphabet, and the slugifier, the parser and the validator all call it.**
   - *Why:* SCIP's Rust parser (Unicode letters), its formatter (ASCII) and its Go binding (ASCII) accept different languages.
   - *Source:* SCIP `symbol.rs` L32, L150, L305 versus Go `identifier.go`.
   - *Enforce:* reviewer; a proptest that `slugify(x)` always parses.
4. **Slugs are ASCII `[a-z0-9]+(-[a-z0-9]+)*`, with no escaping syntax in version one.**
   - *Why:* Mental creates slugs from code names, so escaping (SCIP backticks and doubled spaces, Kythe %-encoding) is pure bloat; `/` and `:` can't occur in a slug, so splitting is trivial.
   - *Source:* the contrast with SCIP `scip.proto` L159-L178 and Kythe `escape.go`.
   - *Enforce:* `good`/`bad` unit table; proptest.
5. **The kind prefix is a closed `Kind` enum, and every reference field declares the `Kind` it expects.**
   - *Why:* reference type checks become string-prefix checks.
   - *Source:* Glean `validate.cpp` "fact type mismatch"; SCIP descriptor suffixes.
   - *Enforce:* checker test per reference field (wrong-kind fixture).
6. **IDs contain no positions, counters, line numbers, hashes, versions or timestamps; a slug made only of digits is rejected.**
   - *Why:* stability across rebuilds (P9). A name says "what", not "when".
   - *Source:* Kythe VName comment L85-L90; spec 4.3.
   - *Enforce:* parser rule plus test; reviewer for `extract` code that feeds `child`.
7. **Disambiguate only on collision, using another code name, never an index.**
   - *Why:* short IDs that don't change when unrelated code changes.
   - *Source:* cargo lockfile package IDs (`lockfile.rs` L210-L212).
   - *Enforce:* reviewer; an `extract` test with two same-named handlers.
8. **The same ID with identical canonical content is deduplicated; the same ID with different content is a `duplicate-id` diagnostic.**
   - *Why:* identity is the key, and conflicting facts mean a bug.
   - *Source:* Glean `FactSet::define` L278-L293.
   - *Enforce:* checker fixture.

#### Format

9. **One JSONL file per kind; one record per line; lines sorted by ID in byte order; struct field order is declaration order; `None` and empty sets are omitted; the file ends in `\n`; nothing else is in the file.**
   - *Why:* byte-deterministic output, diffable by text tools and LLMs (spec 4.9).
   - *Source:* Cargo.lock hand-written emitter and sorts (`lockfile.rs` L138-L219, `encode.rs` L511); Glean's sets-versus-arrays advice.
   - *Enforce:* an insta golden of a small model; a proptest that `write(read(b)) == b`; a test that `Id`'s `Ord` equals `str` byte order.
10. **Set-like fields (`checks`, `tags`, `decides`, `written_by`, `evidence`) are sorted, deduplicated `Vec`s; ordered fields are used only where order has meaning.**
    - *Why:* no spurious diffs.
    - *Source:* Glean `design.md` ("non-deterministic … spurious differences").
    - *Enforce:* a checker diagnostic `unsorted-set`; the writer sorts too.
11. **No `HashMap` or `HashSet` in serialized types; use `BTreeMap` or a sorted `Vec`.**
    - *Why:* iteration order leaks into output.
    - *Source:* Kythe/SCIP canonicalize-before-emit; cargo `BTreeMap` in `cargo-util-schemas`.
    - *Enforce:* clippy `disallowed_types` in `clippy.toml` for the `model` crate.
12. **All record structs use `#[serde(deny_unknown_fields)]`; unknown kinds and unknown fields are hard errors.**
    - *Why:* the single writer is versioned with the compiler, so skipping data would hide skew ("fail rather than degrade"). This deliberately departs from the cargo index and protobuf, which have many writers.
    - *Source:* the contrast with cargo `index/mod.rs` L577-L600.
    - *Enforce:* a test that an extra field is rejected; a CI grep that every `Deserialize` struct in `model` carries the attribute (or a reviewer).
13. **One model-wide format version; a mismatch is a hard error saying "rebuild"; no migration code and no per-record `v`.**
    - *Why:* the model is derived and always rebuildable; multi-version support is Cargo.lock-scale machinery Mental doesn't need.
    - *Source:* Cargo.lock `version` marker ("formal error"); cargo index `v` (rejected as bloat).
    - *Enforce:* a test reading a future version fails with that message.
14. **No catch-all fields (`serde_json::Value`, `extra`/`metadata` maps) on records.**
    - *Why:* cargo calls its `[metadata]` map "deemed a bad idea", and such a field defeats typing and `deny_unknown_fields`.
    - *Source:* cargo `encode.rs` module docs.
    - *Enforce:* clippy `disallowed_types` for `serde_json::Value` in the `model` crate's record modules; reviewer.
15. **Anchors, content hashes and display names never feed identity: anchors live in `anchors.jsonl`, and hashes are blake3 over canonical line bytes, stored beside the record.**
    - *Why:* code moves shouldn't change IDs, and caches shouldn't masquerade as identity.
    - *Source:* spec 4.3/4.4; Kythe keeps locations as facts on anchor nodes, not in VNames; Glean fact IDs are "like memory addresses".
    - *Enforce:* reviewer; a test that moving an anchor changes only `anchors.jsonl` bytes.
16. **The writer rewrites a file only if its bytes changed.**
    - *Why:* stable mtimes and cheap incremental builds.
    - *Source:* cargo `are_equal_lockfiles`.
    - *Enforce:* unit test. Optional for version one.

#### Validation

17. **Checking is a pure `fn check(&Model) -> Vec<Diagnostic>` that never stops early. Diagnostics are sorted by `(id, code, field)` and deduplicated.**
    - *Why:* the user sees every problem at once, and output is stable.
    - *Source:* SCIP `errorSet.Unique()` (sorted); Glean validate collects (with a limit). Kythe's first-error `ValidEntry` is the anti-pattern.
    - *Enforce:* snapshot of `diagnostics.jsonl` for a fixture with many errors.
18. **The diagnostic codes form a closed enum with stable kebab-case names; each diagnostic carries `code`, `id`, `field` (a JSON path), `message` and an optional `related` ID.**
    - *Why:* agents and CI can filter by code, and humans get context.
    - *Source:* SCIP's typed lint errors; cfg-expr's `ParseError {original, span, reason}`.
    - *Enforce:* a completeness test that every code has a fixture triggering it (an exhaustive `match` over `ALL`).
19. **The reference check covers: parses, exists, expected kind, parent exists, no duplicate.**
    - *Why:* dangling and mistyped references are the main structural failure.
    - *Source:* Glean `validate.cpp` L66-L89; SCIP `missingSymbolForOccurrenceError`.
    - *Enforce:* one fixture per sub-rule.
20. **Store only fully qualified IDs in the model; relative forms such as `pred:x` or `.../case:y` exist only in human-facing display.**
    - *Why:* the reference check stays purely textual, and there is no scope-resolution rule to get wrong.
    - *Source:* inferred (no studied project resolves parent-relative references; SCIP's `local` is file-scoped).
    - *Enforce:* the guard parser accepts only full IDs (test); reviewer. This is a spec decision.
21. **Parse errors (IDs, guards) carry the original input, a byte span and a reason enum; tests compare exact messages.**
    - *Source:* cfg-expr `error.rs` / `tests/parser.rs`; cargo-platform `bad()`.
    - *Enforce:* `bad(s, msg)` tables; thiserror `Display`.

#### API and dependencies

22. **Public surface:** `Id`, `Kind`, one struct per record kind, `Model { kind: Vec<T> sorted }`, `read`/`write` per file, `check`, `Expr` with `parse`/`Display`/`eval<L: Logic>`. No arenas, interners or index maps in version one.
    - *Why:* a few thousand records don't need IDE-scale structures, and those need extra dependencies.
    - *Source:* la-arena / PathInterner (rejected as bloat; the upgrade path is a 30-line `Idx<T>`); SCIP sorted list plus binary search.
    - *Enforce:* reviewer; a CI dependency allowlist (`cargo tree -p model --edges normal` must list only serde, serde_json, blake3).
23. **No `#[non_exhaustive]` on workspace-internal enums.**
    - *Why:* it exists for published crates (SCIP `SymbolError`, cargo-platform `ParseErrorKind`). Internally it forces `_` arms and hides missing cases.
    - *Enforce:* clippy `wildcard_enum_match_arm` (restriction lint) in `model`; reviewer.

#### Guards

24. **The guard grammar is `!`, `&`, `|`, parentheses and full predicate IDs; canonical `Display`; an unknown predicate is a link error, never false.**
    - *Source:* cargo-platform `CfgExpr`; the boolean_expression `unwrap_or(false)` anti-pattern.
    - *Enforce:* `good`/`bad` tables; round-trip proptest; checker fixture `unknown-predicate`.
25. **The partition check enumerates the 2^n assignments over the predicates mentioned (n ≤ 16, otherwise a `guard-check-too-large` diagnostic) and reports the lowest witness assignment for each gap or overlap.**
    - *Why:* about 15 lines, deterministic, with concrete messages. BDD/SAT are bloat and forbidden by the dependency policy.
    - *Source:* the truth-table oracles in boolean_expression and biodivine; biodivine's size and dependencies.
    - *Enforce:* unit tests with known witnesses; insta snapshot of messages.
26. **Evaluation is generic over a `Logic` trait implemented for `bool` and `Option<bool>` (Kleene), so the interpreter's "not decided yet" uses the same code.**
    - *Source:* cfg-expr `Logic`.
    - *Enforce:* proptest that `Option<bool>` evaluation with every predicate `Some(_)` equals `bool` evaluation.

#### Tests

27. **Every parser has a table of `good`, `good_canon` and `bad(s, exact_msg)` cases, each also run through `serde_json`.**
    - *Source:* cargo-platform `test_cfg.rs` L49-L96.
    - *Enforce:* reviewer; 100% diff branch coverage makes missing error branches visible.
28. **Every randomized test is reproducible: proptest with committed `proptest-regressions/`, and no `thread_rng` seeds.**
    - *Source:* biodivine fixed seeds versus the boolean_expression `thread_rng` seed (`bdd.rs` L796).
    - *Enforce:* clippy `disallowed_methods` for `rand::thread_rng` in tests; CI fails if regression files are untracked.
29. **Clever algorithms have a brute-force oracle test:** the guard checker against plain enumeration (and a case split, if added, against enumeration), with n ≤ 10.
    - *Source:* BDD crates testing against truth tables.
    - *Enforce:* proptest.
30. **The format is frozen by an insta golden of a small hand-built model covering every record kind.**
    - *Why:* any byte change to the format becomes a reviewed snapshot diff.
    - *Source:* SCIP reprolang snapshots; Cargo.lock compatibility tests (`lockfile_compat`, referenced in the `encode.rs` docs).
    - *Enforce:* insta in CI with `--check` (`INSTA_UPDATE=no`).

#### Explicitly flagged as bloat for version one

- SCIP: the zero-allocation Go-style parser, `SymbolFormatOptions`, backtick and space escaping, package/version components.
- Kythe: the 5-field VName with `VNameMask`, URI encoding, and generic entity-attribute-value `Entry` storage.
- Glean: a schema language, code generation, a compatibility checker, and stored-versus-on-demand derivation machinery.
- la-arena: `IdxRange` and `ArenaMap`; `PathInterner` and its `indexmap`/`rustc-hash` dependencies.
- `SmallVec` and `smol_str`.
- BDD and SAT libraries.
- Multi-version readers (`ResolveVersion`), a per-line `v`, and skipping unknown lines.
- A `[metadata]`-style extension map.
- `#[non_exhaustive]`.
- `static_assertions` (auto-trait checks don't matter for this crate).

### Gaps

- Guideline 20 (fully qualified IDs only) and the domain-constraint rule for dependent predicates (see Q6 Gaps) are spec decisions that no source settles.
- The clippy lints named (`disallowed_types`, `disallowed_methods`, `wildcard_enum_match_arm`) exist in clippy, but this research didn't verify configuring them per crate in this workspace. They are configured through `clippy.toml` or `#![deny]` at the crate root.
- No studied project measured how many records or predicates a model like Mental's holds. The thresholds (n ≤ 16, "a few thousand records") should be checked against the first real build of report-jobs.
