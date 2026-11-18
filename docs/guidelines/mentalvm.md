# `mentalvm`: guidelines

**Status: Accepted by the owner on 2026-11-14.** Distilled from [mentalvm_model_checking.md](../research/implementation-guidelines/notes/mentalvm_model_checking.md), with parts of [testing_strategy.md](../research/implementation-guidelines/notes/testing_strategy.md). The cross-cutting rules in [README.md](README.md) and [testing.md](testing.md) apply too.

## One job

`mentalvm` interprets the model (spec 12): `initial`, `enabled` and `fire` define every step, and four functions use them: a concrete or symbolic `run`, bounded-exhaustive `explore`, `replay` with verdicts, and the witness minimizer. Its outcomes are model gaps, deadlocks, illegal writes, zero-time cycles, violated invariants and normal ends.

**It never:** reads source code; depends on `mentalc` (models arrive as files); does I/O, reads the clock or entropy, or spawns threads in its core; hides a reduction inside the step function.

The core is a library that compiles to `wasm32-unknown-unknown` for the viewer. The binary is a thin CLI that owns files, the seed and the clock.

## Module layout

```
mentalvm/
  src/
    lib.rs          the public API below
    config.rs       Config: instances, state, locks, channels, activations, timers, path condition, now
    choice.rs       Choice, Schedule
    step.rs         initial, enabled, fire
    step/           one file per step-kind group (deliver.rs, write.rs, lock.rs, ...) as size requires
    outcome.rs      Outcome, Step
    run.rs          the run loop and the Chooser trait
    explore.rs      bounded DFS, visited set, preemption counting
    replay.rs       replay with per-choice verdicts
    minimize.rs     the witness minimizer
    runfile.rs      run files (JSON): binding, bounds, schedule, result
    main.rs         thin CLI: reads files, picks and prints a seed           [edge: fs, stdio, clock, entropy]
  tests/            corpus/{bug,correct}/<name>/, determinism.rs, replay.rs, report_jobs.rs
```

Proposed public API (all pure):

```rust
pub fn initial(m: &Model, b: &Binding) -> Config;
pub fn enabled(m: &Model, c: &Config) -> Vec<Choice>;          // sorted, deduplicated
pub fn fire(m: &Model, c: &Config, ch: &Choice) -> Fired;      // never panics on model input
pub enum Fired { Next(Config, Step), End(Outcome, Step) }
pub trait Chooser { fn choose(&mut self, c: &Config, enabled: &[Choice]) -> Option<Choice>; }
pub fn run(m: &Model, b: &Binding, ch: &mut impl Chooser, bounds: &Bounds) -> Run;
pub fn explore(m: &Model, b: &Binding, bounds: &Bounds) -> Report;
pub fn replay(m: &Model, b: &Binding, s: &Schedule) -> Replay;
pub fn minimize(m: &Model, b: &Binding, witness: &Run) -> Run;
```

## Dependency allowlist

| Kind | Crates |
|---|---|
| workspace | `model` |
| normal | `serde`, `serde_json` (run files, reports) |
| optional, feature `cli` (on by default) | `clap` (derive), `anyhow` |
| dev | `insta`, `proptest`; `model` with the `testkit` feature |

The binary declares `required-features = ["cli"]`, and the WebAssembly build uses `--no-default-features`, so the core never compiles the CLI's dependencies. No `rand`, no hashing crate, no persistent-collection crate (`im`, `rpds`).

## Rules

### State and step

#### V1. Configurations are values
- **Rule:** the core exposes `initial`, `enabled` and `fire` as pure functions over `&Model` and `&Config`; `fire` returns a new `Config` and one `Step`, or an `Outcome`. No public signature takes `&mut Config`. The trace is output, never part of the configuration.
- **Why:** snapshots, stepping backwards, DFS backtracking and "equal however reached" (spec 12.1) come free; undo logs would roughly double the step code and its tests.
- **Source:** [Stateright `Model` trait, `lib.rs` L158-L179][sr-model]; PEx's copy-and-restore cost ([`StepState.java`][pex-stepstate]).
- **Enforced by:** reviewer; insta snapshot of the public API (rustdoc JSON or a hand-kept list).

#### V2. Cheap, ordered clones
- **Rule:** `Config` derives `Clone, Eq, Ord, Hash, Serialize, Deserialize` and holds only `Vec`, `BTreeMap`, `BTreeSet`, `Arc<T>` and plain values; per-instance and per-activation parts sit behind `Arc` and change through `Arc::make_mut`. No `RefCell` or `Cell`.
- **Why:** clone-on-step stays cheap and iteration order deterministic; `Arc` works on wasm32.
- **Source:** [Stateright `Vec<Arc<A::State>>`, `model_state.rs` L15-L23][sr-state] and [copy-on-write delivery, `model.rs` L402-L450][sr-cow].
- **Enforced by:** clippy `disallowed-types` ([G6](README.md#g6-ordered-collections-only)); tidy test `mentalvm_no_interior_mutability`; property test that a serde round trip of a generated `Config` is identity.

#### V3. The step function prunes nothing
- **Rule:** every legal choice appears in `enabled`, and every enabled choice fires. Reductions (visited set, preemption bound, partial-order reduction, symmetry) live only in strategies, and each can be switched off.
- **Why:** hidden pruning changes semantics silently; Stateright's no-op pruning carries its own caveat.
- **Source:** [Stateright `model.rs` L418-L440][sr-noop]; [PEx create-first rule][pex-create].
- **Enforced by:** property test: for every generated configuration, every element of `enabled` yields `Fired::Next` or `Fired::End`; reviewer.

#### V4. Outcomes are data
- **Rule:** every way a run ends is a value of `enum Outcome` returned by `fire` and `run`; `panic!`, `unwrap` and `expect` are reserved for internal invariants, each `expect` naming its invariant.
- **Why:** outcomes are reported, minimized and replayed; unlike Shuttle and P, Mental runs a model, not real code that throws.
- **Source:** [P unhandled-event assert, `StateMachine.cs` L1043-L1070][p-unhandled]; [Shuttle panic-based replay failures][sh-replay].
- **Enforced by:** clippy `panic` warned at the crate root (`#![warn(clippy::panic)]`), `unwrap_used` workspace-wide; one corpus model per outcome kind ([V20](#v20-a-corpus-of-bug-and-correct-models)).

### Choices, schedules and replay

#### V5. Choices are named by semantic IDs and carry values
- **Rule:** all nondeterminism is one `Choice` enum (`Sched`, `Env`, `Data`, `Pick`) named by semantic IDs and carrying its value; a schedule is `Vec<Choice>`, never action indices and never "regenerate from a seed". Bool and integer choices are `Data` choices.
- **Why:** replay on a changed model must match choices by name, stored values survive model edits, and one enum means one DFS stack (P keeps three parallel ones).
- **Source:** [Stateright index paths, `path.rs` L21-L90][sr-path]; [Shuttle seed-derived random steps][sh-sched]; [P value-carrying `ScheduleStep`][p-step].
- **Enforced by:** the changed-model replay test ([V22](#v22-replay-round-trips-gate-ci)); reviewer.

#### V6. enabled is sorted
- **Rule:** `enabled` returns choices sorted by `Choice: Ord` and deduplicated.
- **Why:** bit-for-bit reproducibility; Stateright had to add sorting after unordered sets broke path rebuilding.
- **Source:** [Stateright `model.rs` L342-L386][sr-sort].
- **Enforced by:** `debug_assert!` that the result is sorted; determinism test ([V23](#v23-determinism-and-differential-checks)).

#### V7. One chooser, one run loop
- **Rule:** strategies that make choices one at a time implement a single `Chooser` trait with one method, driving one `run` loop; `explore` is a separate function that owns its DFS stack.
- **Why:** one choice point and one stack; P's split into operation, bool and integer choices forced three coordinated stacks.
- **Source:** [Shuttle `Scheduler`, `scheduler/mod.rs` L75-L100][sh-sched]; [P `DFSStrategy.cs`][p-dfs].
- **Enforced by:** reviewer; tidy test `traits_have_two_impls` (replay and binding choosers).

#### V8. Replay gives a verdict per choice
- **Rule:** `replay` reports `matched`, `infeasible` (the named choice is not in `enabled`) or `diverged` (the kind or position no longer lines up, or the run ends before or after the schedule) for each choice, and accepts an optional strategy that continues after the first mismatch.
- **Why:** spec 12.3; P's three named "not reproducible" cases map onto these verdicts, and its suffix strategy is "replay, then search on".
- **Source:** [P `ReplayStrategy.cs` L93-L107][p-replay].
- **Enforced by:** one unit test per verdict in `replay/tests.rs`; snapshot of the report-jobs post-fix replay.

### Search

#### V9. Bounded DFS with an exact visited set
- **Rule:** bounded-exhaustive search is a single-threaded, stateful DFS over cloned configurations with an exact visited set (`BTreeSet` of full keys). The visited key includes the remaining budgets (duplicates, preemptions, depth). No fingerprints in version one; if they ever come, they hash a canonical serialization with blake3 and the exact mode stays as a test oracle.
- **Why:** sound by construction, deterministic, no collision analysis; budgets in the key avoid pruning a state first reached with less budget left.
- **Source:** [PEx `StateCachingMode` Exact][pex-caching]; [Stateright `dfs.rs`][sr-dfs] (depth outside the key); [TLC collision reporting][tlc-fp].
- **Enforced by:** pinned state counts ([V24](#v24-properties-and-pinned-counts)); a test model where pruning without budgets would miss a violation.

#### V10. Preemptions are counted as loom counts them
- **Rule:** a preemption is a `Sched` choice of another activation while the current one can still step; the first choice tried is always the non-preempting one.
- **Why:** matches the spec's bounds language ("1 preemption") and finds witnesses with the fewest preemptions first.
- **Source:** [loom `execution.rs` L163-L200][loom-exec].
- **Enforced by:** unit tests on the counter; the report-jobs witness shows exactly one preemption.

#### V11. Zero-time cycles come from the current path
- **Rule:** a configuration (with `now`) revisited on the current DFS path is a zero-time cycle; one revisited off the current path is pruned.
- **Why:** one mechanism gives both pruning and cycle detection.
- **Source:** [PEx `ExplicitSearchScheduler.java` L249-L290][pex-cycle].
- **Enforced by:** corpus model `zero-time-cycle`.

#### V12. Every loop has a hard cap
- **Rule:** search, minimization and replay loops each enforce a bound in code and return a `bound-exceeded` outcome when it is hit, never loop on.
- **Why:** a mutated loop bound then fails fast instead of hanging the mutation run ("put a limit on everything").
- **Source:** [TigerBeetle style][tb-style]; [loom branch cap][loom-path].
- **Enforced by:** limit tests ([T13](testing.md#t13-test-every-limit-at-its-edges)); cargo-mutants timeouts.

#### V13. Sampling is seeded and optional
- **Rule:** sampled search, if built, is a `Chooser` of under ~50 lines over a hand-written SplitMix64 or PCG seeded from the run file; the core never reads entropy, and the CLI prints any seed it picks.
- **Why:** reproducibility; Stateright carries a FIXME about `StdRng` changing between versions.
- **Source:** [Stateright `simulation.rs` L46-L48][sr-rng]; [Shuttle `Pcg64Mcg`][sh-pct].
- **Enforced by:** clippy `disallowed-methods` (G4); test that the same seed gives a byte-identical report.

#### V14. The minimizer deletes one choice at a time
- **Rule:** the minimizer repeats until nothing changes: try the schedule with choice i removed, re-check it by replay with skipped choices filled by the default non-preempting choice, and keep it if the outcome kind and node match. It returns the best schedule found within its cap. Candidates are ranked by fewest unforced choices, then preemptions, then `env` faults, then `Choice` order.
- **Why:** no studied checker ships a shrinker; proptest's delete-element phase is the simplest proven loop.
- **Source:** [proptest `VecValueTree::simplify`, `collection.rs` L591-L650][pt-vec]; [proptest shrink loop, `runner.rs` L769-L880][pt-runner].
- **Enforced by:** property test (output still fails, is no longer than input, and is a fixpoint); snapshot of the report-jobs minimal witness.

#### V15. No partial-order reduction in version one
- **Rule:** search has no partial-order reduction unless report-jobs bounds prove too slow. If it is added, it is sleep sets over static footprints and ships with a differential test: the same set of `(outcome kind, node)` with it on and off, on every corpus model and on generated tiny models, with both state counts pinned.
- **Why:** "activations run until they wait" plus the visited set already merge most reorderings; loom's DPOR is coupled to its runtime, and no studied project tests a reduction against full search. Accepted and written into spec 12.3.
- **Source:** [loom DPOR, `execution.rs` L134-L160][loom-dpor]; [Stateright 2pc counts with and without symmetry][sr-2pc].
- **Enforced by:** reviewer; the differential test, if a reduction is added.

### Scope and build

#### V16. The core builds for WebAssembly
- **Rule:** the library builds for `wasm32-unknown-unknown` with no file system, clock, thread or environment access; bounds replace time limits; the CLI's dependencies are behind the `cli` feature.
- **Why:** spec 3 and 8: the viewer runs models in the browser. Stateright, Shuttle and loom all read environment variables or wall clocks in their checkers.
- **Source:** [Shuttle `seed_from_env`][sh-pct]; [loom `LOOM_MAX_*`][loom-lib].
- **Enforced by:** CI job `cargo build -p mentalvm --lib --no-default-features --target wasm32-unknown-unknown`; clippy `disallowed-methods` (G4).

#### V17. Run files are JSON
- **Rule:** run files and reports are JSON through `deny_unknown_fields` types; a run file round-trips byte for byte.
- **Why:** the no-YAML decision ([G13](README.md#g13-json-out-toml-in-no-yaml)); spec 12.6 now shows a JSON run file.
- **Source:** owner decision; spec 12.6.
- **Enforced by:** property test of the round trip; cargo-fuzz target for the run-file reader ([T11](testing.md#t11-fuzz-only-readers-of-outside-input)).

#### V18. Version one scope
- **Rule:** version one has concrete `run`, bounded-exhaustive `explore`, `replay` and the minimizer. No threads, no PCT, no symmetry reduction, no fingerprint modes, no web explorer, no checkpoint files.
- **Why:** these are exactly where the studied checkers grew (P's strategies alone are about 3,770 lines).
- **Source:** [Stateright `checker.rs`][sr-checker]; [Shuttle schedulers][sh-schedulers]; [P strategies][p-strategies].
- **Enforced by:** reviewer; dependency allowlist (tidy test `dependencies_justified`).

### Tests (beyond [testing.md](testing.md))

#### V19. One scenario test per step rule
- **Rule:** each rule of spec 12.1 (an unhandled message is a model gap, every write is checked against the lifecycle, an activation runs until it waits, run-time IDs are derived from the creating step) has a tiny fixture model and an inline snapshot of its trace, one step per line.
- **Why:** hand-written scenarios cover what random search reaches slowly.
- **Source:** [TigerBeetle VOPR scenario tests][tb-vopr].
- **Enforced by:** unit tests in `step/tests.rs`.

#### V20. A corpus of bug and correct models
- **Rule:** `tests/corpus/{bug,correct}/<name>/` holds a model, its bounds and an insta snapshot of outcome, minimal witness and counts (configurations, transitions, maximum depth). Every outcome kind has at least one bug model, correct models must report "none within bounds", and every bug fixed in `mentalvm` adds a corpus case.
- **Why:** known-bug corpora with golden counts are how P, TLC and Stateright test themselves.
- **Source:** [P regression tests][p-regression]; [TLC `TLC_STATS` assertions][tlc-stats].
- **Enforced by:** CI; reviewer checklist item "bug fix adds a corpus case".

#### V21. Witnesses are checked by replay
- **Rule:** an `assert_witness(model, schedule, outcome)` helper replays a witness and accepts it if it really produces the outcome; tests use it alongside snapshots.
- **Why:** a legitimate change in search order shows up as a reviewed snapshot diff, not as a false correctness failure.
- **Source:** [Stateright `assert_discovery`, `checker.rs` L534-L560][sr-discovery].
- **Enforced by:** the helper.

#### V22. Replay round trips gate CI
- **Rule:** for every corpus witness, explore, serialize the run file, parse it, replay it: all choices `matched`, same outcome. report-jobs after the CAS fix replays the stored race schedule to `diverged` at `worker[1]/n:claim`, and a fresh `explore` with the same bounds is clean; a deletion test yields `infeasible`.
- **Why:** spec 12.3 and F-S11; Shuttle's round-trip tests are `#[ignore]`d today because replay broke unnoticed.
- **Source:** [Shuttle `tests/basic/replay.rs` L32-L51][sh-replay-tests].
- **Enforced by:** CI; tidy test `no_should_panic_or_ignore`.

#### V23. Determinism and differential checks
- **Rule:** every strategy runs twice in-process on the corpus with byte-identical serialized output; a naive recursive enumerator over `enabled` and `fire` (no visited set, no minimization), kept in a `tests.rs`, reaches the same set of `(outcome kind, node)` as `explore` on tiny generated models; every sampled run's outcome lies in the exhaustive outcome set.
- **Why:** every runtime checker studied ships a nondeterminism detector because it bites; clever search needs a dumb oracle.
- **Source:** [Shuttle nondeterminism checker][sh-nondet]; [wasmtime differential oracles][wt-oracles].
- **Enforced by:** `tests/determinism.rs`, CI-gating; property tests.

#### V24. Properties and pinned counts
- **Rule:** property tests over generated small models check that `fire` never panics, every element of `enabled` fires, and `run` with a replay chooser reproduces any schedule `explore` produced. Exact explored-state counts are pinned in inline snapshots for 3 to 5 named tiny models.
- **Why:** pinned counts make speed-only changes visible to tests and to mutants.
- **Source:** [Stateright pinned `unique_state_count`][sr-bfs]; [Shuttle DFS iteration counts][sh-dfs-tests].
- **Enforced by:** property tests; inline snapshots.

#### V25. The core is tested on the target that ships
- **Rule:** the replay and determinism tests also run on the WebAssembly target.
- **Why:** release and wasm builds differ (overflow checks, `debug_assert!`, target); fingerprints or ordering that differ between native and wasm would break the viewer.
- **Source:** [SQLite as-delivered testing][sqlite-testing].
- **Enforced by:** CI job (wasmtime or wasm-bindgen-test runner; choose in the first spike).

[sr-model]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/lib.rs#L158-L179
[sr-state]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model_state.rs#L15-L23
[sr-cow]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L402-L450
[sr-noop]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L418-L440
[sr-sort]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L342-L386
[sr-path]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/path.rs#L21-L90
[sr-dfs]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/dfs.rs#L31
[sr-rng]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/simulation.rs#L46-L48
[sr-discovery]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs#L534-L560
[sr-checker]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs
[sr-2pc]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/examples/2pc.rs#L183-L198
[sr-bfs]: https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/bfs.rs#L450
[sh-sched]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/scheduler/mod.rs#L75-L100
[sh-replay]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/replay.rs
[sh-replay-tests]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/replay.rs#L32-L51
[sh-pct]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/pct.rs#L50
[sh-nondet]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/uncontrolled_nondeterminism.rs
[sh-schedulers]: https://github.com/awslabs/shuttle/tree/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src
[sh-dfs-tests]: https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/dfs.rs#L30-L279
[p-unhandled]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/Runtime/StateMachines/StateMachine.cs#L1043-L1070
[p-step]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Traces/ScheduleStep.cs
[p-dfs]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/Exhaustive/DFSStrategy.cs
[p-replay]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/Special/ReplayStrategy.cs#L93-L107
[p-strategies]: https://github.com/p-org/P/tree/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies
[p-regression]: https://github.com/p-org/P/tree/260da928f67569ca7a8c0039a02d0bad7cde6096/Tst/RegressionTests
[pex-stepstate]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/StepState.java
[pex-create]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java#L552-L568
[pex-cycle]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java#L249-L290
[pex-caching]: https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/StateCachingMode.java
[loom-exec]: https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/execution.rs#L163-L200
[loom-dpor]: https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/execution.rs#L134-L160
[loom-path]: https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs
[loom-lib]: https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/lib.rs#L271-L291
[tlc-fp]: https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/ModelChecker.java#L962-L972
[tlc-stats]: https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/test/tlc2/tool/EvalExceptionLivenessTest.java#L50-L51
[pt-vec]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/collection.rs#L591-L650
[pt-runner]: https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/test_runner/runner.rs#L769-L880
[tb-style]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/TIGER_STYLE.md
[tb-vopr]: https://github.com/tigerbeetle/tigerbeetle/blob/a2a3c7064519e08233abe26c54a43fca2f9ad9bc/docs/internals/vopr.md
[wt-oracles]: https://github.com/bytecodealliance/wasmtime/blob/c52bc6d8fc7ebf2ba3f7e39ab3203a3c13c1d272/crates/fuzzing/src/oracles.rs#L460-L604
[sqlite-testing]: https://www.sqlite.org/testing.html
