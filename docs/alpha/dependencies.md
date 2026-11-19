# Dependencies

## Policy

- Adding: every new dependency gets a row in the log below, with the reason. The Stop hook blocks a lockfile change without one; a release that only bumps the project's own version in uv.lock or Cargo.lock needs none.
- Pinning: lockfiles pin every version. Only the package manager changes them; Claude can't edit them by hand.
- Upgrading: Upgrades are manual: check for updates weekly.

## Log

Log every dependency added, with the reason.

| Dependency | Component | Why | Added |
|---|---|---|---|
| rustfmt | model | Formatting (a toolchain component) | 2026-11-18 |
| clippy | model | Linting, with the pedantic group (a toolchain component) | 2026-11-18 |
| cargo-llvm-cov | model | Coverage of the lines and branches a pull request changes, run on the dated nightly the kit pins (a kit release moves the date, and `alpha update` brings it in) | 2026-11-18 |
| cargo-machete | model | Unused dependencies | 2026-11-18 |
| rustfmt | extract | Formatting (a toolchain component) | 2026-11-18 |
| clippy | extract | Linting, with the pedantic group (a toolchain component) | 2026-11-18 |
| cargo-llvm-cov | extract | Coverage of the lines and branches a pull request changes, run on the dated nightly the kit pins (a kit release moves the date, and `alpha update` brings it in) | 2026-11-18 |
| cargo-machete | extract | Unused dependencies | 2026-11-18 |
| rustfmt | mentalc | Formatting (a toolchain component) | 2026-11-18 |
| clippy | mentalc | Linting, with the pedantic group (a toolchain component) | 2026-11-18 |
| cargo-llvm-cov | mentalc | Coverage of the lines and branches a pull request changes, run on the dated nightly the kit pins (a kit release moves the date, and `alpha update` brings it in) | 2026-11-18 |
| cargo-machete | mentalc | Unused dependencies | 2026-11-18 |
| serde | model | Derives the JSON form of the model's records and diagnostics ([model allowlist](../guidelines/model.md#dependency-allowlist)) | 2026-11-19 |
| serde_json | model | Reads and writes the model's canonical JSONL files | 2026-11-19 |
| proptest | model (dev) | Property tests of the ID parser, slugs and the JSONL round trip ([T8](../guidelines/testing.md#t8-property-tests-are-reproducible)); without default features, so no process is forked per case | 2026-11-19 |
| insta | model (dev) | Snapshots that freeze the JSONL format ([M22](../guidelines/model.md#m22-the-format-is-frozen-by-a-golden)); the owner chose insta for snapshots; without its default terminal colors | 2026-11-19 |
| serde | extract | Derives how `pyproject.toml` and `uv.lock` read into typed structs at the boundary ([extract allowlist](../guidelines/extract.md#dependency-allowlist)) | 2026-11-19 |
| toml | extract | Reads `pyproject.toml` and `uv.lock`, with the byte spans that anchor each record; without its default writer | 2026-11-19 |
| proptest | extract (dev) | Property tests that damaged build files never panic the build and that every record it writes is anchored ([T8](../guidelines/testing.md#t8-property-tests-are-reproducible)) | 2026-11-19 |
| ruff_python_parser | extract | Parses each Python file into ruff's typed, CPython-shaped syntax tree, recovering from syntax errors ([X1](../guidelines/extract.md#x1-ruffs-parser-and-tys-crates-pinned-to-one-commit), [X4](../guidelines/extract.md#x4-always-parse_unchecked)); from git at ruff commit `4228f4c`, the commit of ty 0.0.105 | 2026-11-19 |
| ruff_python_ast | extract | The syntax tree and its visitor, which the site walk runs on; same commit | 2026-11-19 |
| ruff_text_size | extract | The byte ranges of syntax nodes, which anchor each site; same commit | 2026-11-19 |
