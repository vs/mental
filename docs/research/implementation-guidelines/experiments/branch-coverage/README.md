# Branch coverage of assertions

The experiment behind [the coverage answer in the testing playbook](../../../../guidelines/testing.md#branch-coverage-of-assertions-settled). The question: does `cargo llvm-cov --branch` count the panic side of an assertion, or the error side of `?`, as code no test reached? If it did, asserting invariants and requiring 100% diff coverage would conflict.

`src/lib.rs` has one function for each form. The single test in `src/tests.rs` runs each function on its happy path only, so no assertion fails and no `?` returns an error.

## Run

From this folder:

```sh
CARGO_TARGET_DIR="$(mktemp -d)" cargo +nightly-2026-11-10 llvm-cov --branch --cobertura --output-path coverage.xml
```

CI uses the same toolchain and cargo-llvm-cov 0.9.1. diff-cover reads `coverage.xml`.

## Result (2026-11-19)

| Form | Lines with 0 hits | Branches |
|---|---|---|
| `assert!`, `debug_assert!`, `assert_eq!`, one line or several | none; the message lines are not counted as lines | none |
| `.expect("…")` | none | none |
| `?` after other code on its line that ran (`let n: u32 = s.parse()?;`) | none | none |
| `?` that starts the first code region of its line, as after rustfmt splits a method chain (`.sum::<Result<u16, _>>()?;`) | that line | none |
| `if` / `else` on one line, only one side taken | none | 1 of 2 taken, which fails diff coverage |

Assertions are free: neither side of one shows up in the report. The error side of `?` shows up only when the `?` starts the first code region of its line.
