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
