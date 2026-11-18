# Contributing

## Working with alpha

- `/alpha:install` adopted this repo on `alpha/adopt`; run again, it carries out the next step of `docs/alpha/adoption.md` while one is left.
- `/alpha:update [request]` keeps it current: with no request it brings alpha's files to the installed kit; with one it adds or changes a component or a setup answer, or settles a product area (`/alpha:update performance`).
- `/alpha:do <task>` works each task by a playbook; a feature goes through research, a spec and a plan the user approves before any code.
- `/alpha:uninstall` switches alpha's hooks off and `/alpha:install` switches them back on; `/alpha:purge` removes alpha in one commit.

## One-time setup

```sh
git config core.hooksPath .claude/alpha/githooks
```

Then set up each component:

- `model`: `cd model && cargo fetch`
- `extract`: `cd extract && cargo fetch`
- `mentalc`: `cd mentalc && cargo fetch`

## Components

Layout: monorepo.

| Name | Path | Kind | Stack | Traits |
|---|---|---|---|---|
| model | model | library | rust | nontrivial_logic |
| extract | extract | library | rust | nontrivial_logic |
| mentalc | mentalc | cli | rust | network_calls, nontrivial_logic |

Components communicate with each other; keep their contract explicit.

### Tool choices

- rust: rustfmt, clippy, cargo-llvm-cov, cargo-machete. Commands: `cargo fetch`, `cargo test`, `cargo clippy --all-targets --no-deps -- -D warnings`, `cargo fmt`, `cargo fmt --check`, `cargo check --all-targets`, `cargo machete`. Its components form one workspace at the repository root (`Cargo.toml`, `clippy.toml`, `rust-toolchain.toml`, one lockfile); each command covers the component it runs in

## Checks

```sh
mise install
mise run check
```

`mise install` installs the pinned tools; `mise run check` runs every component's checks and the repository checks below, the same as CI.

The principles in `docs/alpha/principles.toml` name them in `checked_by` as `ci:<name>`: a name in the table is that check, and `ci:tree` is when every CI job ends by checking that the checks left the tree unchanged; `ci:diff-coverage` is when a pull request's job fails when its tests cover less of the changed lines than `testing.diff_coverage`.

| Check | What it checks | Runs |
|---|---|---|
| actions | GitHub Actions workflows (jactionlint) | pull request, at Stop |
| secrets | Secrets in the working tree (betterleaks) | pull request |
| vulns | Known vulnerabilities in locked dependencies (osv-scanner) | pull request, weekly |
| typos | Spelling in code and docs (typos) | pull request, at Stop |
| markdown | Markdown style (rumdl) | pull request, at Stop |
| links | Relative links in docs resolve (lychee) | pull request, at Stop |
| links-online | Links to other sites still work (lychee) | weekly |
| agent-config | CLAUDE.md and .claude/ configuration (agnix) | pull request, at Stop |
| review-dates | Suppressions past their review date | pull request, weekly |
| toolchain | Pinned tools that have a newer release | weekly |

## Commit convention

Conventional commits: `type(scope): description`.

- Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `build`, `chore`, `perf`, `ci`, `revert`
- Scopes: `model`, `extract`, `mentalc`, `mentalvm`, `spec`, `research`, `plans`, `guidelines`, `examples`, `resolution`, `glossary`, `findings`, `report-jobs`, or another area the commit touches
- The description is at most 50 characters, imperative, with no trailing period; it may start lowercase, e.g. with the symbol it changes.
- Body lines wrap at 72 characters.

## Testing

| Component | Levels |
|---|---|
| model | unit, contract |
| extract | unit, contract |
| mentalc | unit, integration, end-to-end, contract |

Unit tests sit in a `tests.rs` beside the module they test (`#[cfg(test)] mod tests;`), and behavior through the public API in `<crate>/tests/*.rs`, one file per level or area ([T1](docs/guidelines/testing.md#t1-where-tests-live)); Cargo finds no test in a deeper folder of `tests/`.

A test that covers an acceptance check names its requirement id (e.g. `R1`) in its name, so each check in docs/alpha/specification.md traces to a test.

### Exhaustive testing

This project is tested exhaustively, on the model of [SQLite's testing](https://www.sqlite.org/testing.html). The reviewer holds every change to it, and a pull request must cover every line and branch it changes.

- **Every branch, both ways.** Delete a branch no test can reach, or replace it with an assertion.
- **Every failure, forced.** Inject the failure in a test: a failed allocation, read or write, a timeout, a crash between two writes.
- **Outside input is fuzzed.** Whatever parses input from outside gets a fuzz or property-based test.
- **Limits on both sides.** Test each limit at, just below and just above its value.
- **A bug fix starts with its test.** Write the test that fails without the fix first.
- **Invariants are asserted** in the code, not only in tests.
- **Test what ships.** Run the tests on the build users get.

## Documentation

alpha's docs (the specification, the principles, the dependency log) live in `docs/alpha/`. There is no user-facing documentation.

| When these change | Update |
|---|---|
| `model/src/lib.rs`, `model/src/id.rs`, `model/src/record.rs`, `model/src/record/**`, `model/src/anchor.rs`, `model/src/guard.rs`, `model/src/diagnostic.rs` | `docs/specification.md`, `docs/glossary.md` |
| `extract/src/lib.rs`, `extract/src/build.rs`, `extract/src/build/**` | `docs/specification.md`, `docs/glossary.md` |
| `extract/src/python/resolve.rs`, `extract/src/pack.rs`, `extract/src/pack/**` | `docs/resolution.md`, `docs/specification.md` |
| `mentalc/src/cli.rs`, `mentalc/src/mcp.rs`, `mentalc/src/config.rs` | `docs/specification.md`, `docs/glossary.md` |
| `model/Cargo.toml` | `docs/guidelines/model.md` |
| `extract/Cargo.toml` | `docs/guidelines/extract.md` |
| `mentalc/Cargo.toml` | `docs/guidelines/mentalc.md` |
| `mentalc/src/build.rs`, `mentalc/src/repo.rs`, `extract/src/python/sites.rs` | `docs/specification.md`, `docs/glossary.md` |

The Stop hook blocks a change to these paths that leaves the docs untouched.

## Branching

Feature branches: short-lived branches merged into main through pull requests, each leaving the checks green. Before opening a pull request, rebase the branch on main into small, ordered commits and push it with --force-with-lease; the hooks block a plain --force and any rewrite of main.

Claude runs the principles reviewer and each changed component's full test suite before its work reaches main: a push to main, or merging a pull request. People can push by hand.

## Build, CI and hosting

### CI

GitHub Actions runs `.github/workflows/ci.yml` on every push to main and every pull request: one job per component, running its checks in order.

| Component | Checks |
|---|---|
| model | `cargo fetch`, `cargo fmt --check`, `cargo clippy --all-targets --no-deps -- -D warnings`, `cargo check --all-targets`, `cargo machete`, `cargo test` |
| extract | `cargo fetch`, `cargo fmt --check`, `cargo clippy --all-targets --no-deps -- -D warnings`, `cargo check --all-targets`, `cargo machete`, `cargo test` |
| mentalc | `cargo fetch`, `cargo fmt --check`, `cargo clippy --all-targets --no-deps -- -D warnings`, `cargo check --all-targets`, `cargo machete`, `cargo test` |

### Artifacts and hosting

| Component | Artifact | Hosting |
|---|---|---|
| model | none | - |
| extract | none | - |
| mentalc | binary | - |

### Environments

None: nothing is deployed.

Hosting is decided and documented here; alpha creates no cloud resources and no deploy jobs.

## Releases

Semantic versioning (MAJOR.MINOR.PATCH): a breaking change bumps MAJOR, a new feature MINOR, a fix PATCH.

One version for the whole repository.

Changes are recorded in [CHANGELOG.md](CHANGELOG.md) under Unreleased as they land, and moved under the version at release.

### Cutting a release

Tag vX.Y.Z on main after updating CHANGELOG.md; the mentalc binary is built with cargo build --release. Publishing binaries is set up when distribution is decided.

## Security

### Secrets

Nothing is deployed.

Locally, secrets live in `.env` files, which git ignores; never commit them. Claude can't edit them, and the Stop hook blocks a change that adds a secret to the repository.

### Protected paths

Claude can't edit these. Lockfiles change only through the package manager; change the others by hand.

- `**/.env`
- `**/.env.*`
- `**/*.pem`
- `**/*.key`
- `Cargo.lock`

## Exceptions

A check skipped or loosened in one place, or a baseline loosened, needs a row here naming the path and the check, with the reason; the checks look for it.

| Path | Check | Reason |
|---|---|---|
| `mentalc/src/main.rs` | `clippy::disallowed_methods` | the CLI edge: stdio and the exit code ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `mentalc/src/main.rs` | `clippy::print_stderr` | prints the error chain ([G12](docs/guidelines/README.md#g12-exit-codes-and-a-thin-main)) |
| `mentalc/src/main.rs` | `clippy::exit` | sets the exit code 0, 1 or 2 ([G12](docs/guidelines/README.md#g12-exit-codes-and-a-thin-main)) |
| `mentalc/src/cli.rs` | `clippy::disallowed_methods` | the CLI edge: renders to stdout ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `mentalc/src/cli.rs` | `clippy::print_stdout` | human and JSON output ([G12](docs/guidelines/README.md#g12-exit-codes-and-a-thin-main)) |
| `mentalc/src/repo.rs` | `clippy::disallowed_methods` | the file-system edge ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `mentalc/src/cache.rs` | `clippy::disallowed_methods` | writes the answer cache under `.mental/` ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `mentalc/src/ledger.rs` | `clippy::disallowed_methods` | appends the call ledger under `.mental/` ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `mentalc/src/backend/api.rs` | `clippy::disallowed_methods` | the network edge of the `api` backend ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `**/tests/**` | `clippy::disallowed_methods` | tests make temp directories and read fixture files ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `**/tests.rs` | `clippy::disallowed_methods` | unit tests make temp directories and read fixture files ([G4](docs/guidelines/README.md#g4-io-and-ambient-inputs-only-at-named-edges)) |
| `docs/research/*/experiments/results/` | `typos` | recorded output of tools and LLMs (hashes, raw answers, prompts as sent); editing it would falsify the record, so `_typos.toml` excludes it |
| `docs/research/*/experiments/results/` | `markdown` | the same recorded output, prompts as sent included; `.rumdl.toml` excludes it |
| `docs/research/*/experiments/results/` | `links` | the same recorded output; the prompts as sent hold links relative to the projects they were written for; `lychee.toml` excludes it |
| `**/*.md` | `markdown` (MD013) | headings are not length-checked, so their anchors and the links to them stay stable, and code blocks may run to 120 columns for layouts, JSON examples and formulas (owner choice, 2026-11-18) |
