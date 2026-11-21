# Mental

Mental compiles a repository into a model of the system that you can run: play a scenario and get a trace, zoom in and out between levels, search for races, and change something to see what changes. A coding agent shares the same view.

## Principles

- **Simplest sufficient architecture**: As simple as possible, as complex as the problem requires.
- **Fail fast, fail helpfully**: Detect errors as early as possible. Every error message describes the problem and tells the user how to proceed.
- **Tests are as important as the core**: Test design and writing get the same effort and intensity as the logic. Tests trace back to acceptance checks. Risky logic (a branch, a parser, money, security, data writes, a bug fix) gets a test that fails when it breaks: one good test, not coverage. A trivial change (a rename, a constant, wiring) may go without one, with the reason recorded.
- **Exhaustive testing**: Tests take every branch both ways: a change's lines and branches are all covered, and a branch no test can reach is deleted or replaced by an assertion. Every failure path is forced in a test by injecting the failure (a failed allocation, read or write, a timeout, a crash between two writes). Everything that reads outside input has a fuzz or property-based test, and every limit is tested at, just below and just above its value. A bug fix starts with a test that fails without it. Invariants are asserted in the code, and the tests run on the build that ships.
- **One design language**: Every visual the project makes (a diagram, a chart, an image, the UI) is built from the design language in docs/alpha/design-language.md: read it before making or changing one. A visual may step outside it for a stated reason; it may not drift from it.
- **Docs move with the code**: A change to public behavior updates the docs in the same change.
- **Build and hosting are decided up front**: Build and hosting are decided up front and runnable in CI.
- **Validate at the edges, trust the inside**: External input is parsed into typed data once, at the boundary. Inside, code doesn't re-check what its own modules return; each remaining cast or runtime check says which boundary or invariant it serves.
- **Strict types**: TypeScript `strict`, Python type-checked in strict mode.
- **Logic in the middle, side effects at the edges**: Business logic is kept pure and in the middle; side effects live at the edges.
- **No dead code**: No commented-out code, no speculative options.
- **Code over comments**: A comment stays only for what the code can't say: a license header, a rule set by a dependency, platform or protocol we can't change, a public API's contract, an issue link for a constraint, or a `shortcut: <the limit>, <when to upgrade>` note on a deliberate shortcut. Anything surprising in our own code is fixed by renaming, extracting or retyping it until it reads plainly, not explained in prose.
- **Generated files come from their source**: A generated file changes only by changing its source and regenerating it, never by hand.
- **Contracts are the source of truth**: Interfaces are defined once (OpenAPI/JSON Schema/protobuf). Clients and checks are generated from them.
- **One vocabulary**: A glossary, and consistent names across code, docs, and UI.
- **One command each to set up, test, and run**: One command each to set up, test, and run, per component, on a fresh machine.
- **Reproducible**: Pinned dependencies and lockfiles. For ML: seeds, data versions, recorded experiment configs.
- **Every external call has a timeout and a failure path**: Every external call has a timeout and a failure path. Retries only on safe-to-repeat operations.
- **No secrets in the repo**: Secrets come from the environment or a secret store.
- **Every new dependency has a recorded reason**: Every new dependency has a recorded reason and is pinned.
- **Clean, readable history**: Each commit is one logical change, holds only the files changed for it, passes checks on its own, and has a message that explains why.
- **Evidence before "done"**: No claim of success without running the check that shows it.
- **Never weaken a check to pass it**: No skipped tests, loosened lint, or catch-all exception handlers, and no mocking the thing under test. A type error from an outdated dependency is fixed by upgrading it, not by removing or downgrading code. If a check is wrong in one place, add a row to CONTRIBUTING.md's Exceptions table that names the path, the rule and why, then make the exception there.
- **Stay in scope**: No drive-by refactors of code the task doesn't touch.
- **Subtract before you add**: In the code a task touches, first remove what's dead there: unused code, redundant checks, stubs. A new internal API replaces the old one in the same change: every caller migrated, the old one deleted, no compatibility layer.
- **Smallest complete change**: Take the first option that fully works: skip what nobody asked for, then reuse what the codebase has, then the standard library or platform, then an installed dependency, then one line a reader gets at a glance, and only then new code. No new dependency for a few lines, and no abstraction, wrapper, option, config or "for later" code nobody asked for; keep values in the form the platform already gives you, and keep the structure the codebase already has. Never cut validation at trust boundaries, error handling that prevents data loss, security, accessibility, or anything the user asked for.
- **Scope before writing**: Before changing code, list every place the change must reach (callers, tests, fixtures, config, exports) and what it could break for users (data lost or exposed, callers that stop working), then finish every part of it. Features nobody asked for aren't scope.
- **Fix bugs at the root**: Before a bug fix, find every caller of the function it touches and fix the root cause once, in the shared code, not in one caller. Code that is moved or merged keeps its error handling and validation.
- **Lean structure**: Near-copies that must change together are merged into one. A function is split by job, never by line count, and never into helpers that only make it shorter. Between options of equal size, the one that is correct on edge cases wins.
- **Never block on the human**: Don't stop to ask about reversible work. When a requirement is ambiguous, make the reasonable choice, write it into the docs, and say what you chose. Ask first only when the choice sets product direction or can't be undone: rewriting the main branch, deploys, deleting data, messages to people.
- **Definition of done**: Tests written and passing, types and lint clean, docs updated, committed per the history principle, reviewed before it reaches main.

`docs/alpha/principles.toml` is the source of truth, with each principle's parameters, rationale and overrides.

## Working rules

- Before you change code, docs or config, read the parts of docs/alpha/specification.md and CONTRIBUTING.md that bear on the change and the principle's parameters in docs/alpha/principles.toml. Follow them.
- A decision updates the docs, as the decision skill says: what the product does goes in docs/alpha/specification.md and needs the user's yes; how the work is done goes in CONTRIBUTING.md, with the reason in the commit; a changed principle is an override in the answers, which the user agrees to first.
- When a requirement is ambiguous, make the reasonable choice, record it with the decision skill and say what you chose; don't stop to ask about reversible work. Ask first only about product direction or what can't be undone (rewriting main, deploys, deleting data, messages to people).
- Read a file in full before you edit it, and every file a wide change touches; search snippets aren't enough.
- Look up a dependency's API in its installed source or types instead of guessing it.
- Remove dead code and replaced internal APIs without asking, but ask before removing code or behavior that looks intentional.
- Commit only the files you changed for the work, staged by path. Other changes in the tree may be the user's or another session's, so don't stage, stash or reset them.
- Subagents work under the same principles, docs and checks.
- Run `/alpha:uninstall` or `/alpha:purge` only when the user asks for it.
- Before you report work as done, re-read every changed file top to bottom as a stranger would.
- Run the principles-reviewer agent once, before your work reaches main: the merge guard asks for it, and it reviews only what changed since its last review.
- When you add a code generator, add a `generate` task to `mise.toml` and to the `ci` task's `depends`; CI fails when checks change tracked files.
- Never disable a lint rule to pass; an exception names one path and one rule, and a row in CONTRIBUTING.md's Exceptions table names that path, the rule and the reason.
- A security suppression (osv-scanner, betterleaks) needs a reason and a review date: in the `osv-scanner.toml` next to the lockfile it applies to, `ignoreUntil` and `reason`; in `.betterleaksignore`, a `# <reason> (review YYYY-MM-DD)` line directly above the entry.

## Communication

- Answer a question before making edits or running commands for it.
- When responding to feedback or a review, say first whether you agree, then what you changed.
- Be concise and direct: plain technical prose, no filler, no emojis. Define jargon you can't avoid.
- Explain a non-trivial problem or design as the problem, a concrete example or short trace, then the solution and why it's needed.
- When you report finished work, end with one or two lines: what you skipped or didn't check, and any risk the user must know.

## Stack rules

### rust (model, extract, mentalc)

- **rust:cargo-for-dependencies**: Declare each dependency once, in `[workspace.dependencies]` of the root Cargo.toml, and name it in a member with `foo.workspace = true`; add and remove them with cargo commands where cargo supports it, never by copying versions into a member.
- **rust:typed-boundaries**: Parse external input (arguments, environment, files, JSON) into typed structs at the boundary (clap and serde derives); don't pass `serde_json::Value` or raw strings inward.
- **rust:errors-with-context**: A library returns its own error enum; a binary adds context to each error it passes up, saying what failed and how to fix it. `expect()` only for an invariant, with a message naming it.
- **rust:tests-beside-code**: Put a module's unit tests in a `tests.rs` file beside it (`#[cfg(test)] mod tests;`), not inline, and behavior through the public API in `tests/`.

## Commands

### model (rust, `model/`)

Code: `model/**/*.rs`. Tests: `model/tests/**/*.rs`, `model/**/tests.rs`. Not checked: `model/**/.*/**`, `model/**/target/**`.

| Step | Command |
|---|---|
| setup | `cargo fetch` |
| test | `cargo test` |
| lint | `cargo clippy --all-targets --no-deps -- -D warnings` |
| format | `cargo fmt` |
| format_check | `cargo fmt --check` |
| typecheck | `cargo check --all-targets` |
| structure | `cargo machete` |
| run | add when the entry point exists |

### extract (rust, `extract/`)

Code: `extract/**/*.rs`. Tests: `extract/tests/**/*.rs`, `extract/**/tests.rs`. Not checked: `extract/**/.*/**`, `extract/**/target/**`.

| Step | Command |
|---|---|
| setup | `cargo fetch` |
| test | `cargo test` |
| lint | `cargo clippy --all-targets --no-deps -- -D warnings` |
| format | `cargo fmt` |
| format_check | `cargo fmt --check` |
| typecheck | `cargo check --all-targets` |
| structure | `cargo machete` |
| run | add when the entry point exists |

### mentalc (rust, `mentalc/`)

Code: `mentalc/**/*.rs`. Tests: `mentalc/tests/**/*.rs`, `mentalc/**/tests.rs`. Not checked: `mentalc/**/.*/**`, `mentalc/**/target/**`.

| Step | Command |
|---|---|
| setup | `cargo fetch` |
| test | `cargo test` |
| lint | `cargo clippy --all-targets --no-deps -- -D warnings` |
| format | `cargo fmt` |
| format_check | `cargo fmt --check` |
| typecheck | `cargo check --all-targets` |
| structure | `cargo machete` |
| run | `cd ../examples/report-jobs && uv sync --locked && cargo run -p mental-mentalc -- build` |

Not code: `examples/`, `docs/research/*/experiments/`, `extract/tests/fixtures/`, `extract/tests/data/`: examples, fixtures and reference data. No format, lint, test or docs rule and no principles review applies there; the secrets scan and the repository's text checks (spelling, Markdown, links) still do.

Run every check: `mise run check` (CI runs the same checks).

The hooks leave the full test suites to one checkpoint: before your work reaches main (a push to main, or merging a pull request), run `python3 .claude/alpha/hooks/stop_checks.py tests`. It runs the suite of each component you changed and remembers a pass until those files change. While you work, run the tests your change touches.

## Definition of done

1. Tests written and passing.
2. Types and lint clean.
3. Docs updated.
4. Committed per the commit convention.
5. Reviewed before it reaches main.

## Commits

Follow the commit skill (`.claude/skills/commit/SKILL.md`, which the user runs as /commit). Convention: conventional commits, scopes: model, extract, mentalc, mentalvm, spec, research, plans, guidelines, examples, resolution, glossary, findings, report-jobs, or another area it touches, enforced by the commit-msg hook. Work on a short-lived branch and open a pull request into main; don't commit to main directly.

## Docs

- [Principles](docs/alpha/principles.toml)
- [Specification](docs/alpha/specification.md): what the product does; `/alpha:update <area>` settles each open part, and each full-track task gets a folder in `docs/alpha/features/`, `docs/alpha/bug-fixes/`, `docs/alpha/refactorings/` or `docs/alpha/perf/`
- [Contributing](CONTRIBUTING.md): how the project is built, checked, released and kept secure
- [Dependencies](docs/alpha/dependencies.md)
- [Glossary](docs/alpha/glossary.md)
- [Changelog](CHANGELOG.md)
- [Adoption plan](docs/alpha/adoption.md): the steps that bring this repo's code up to the principles; `/alpha:install` resumes it
