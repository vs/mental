# How model checkers implement step, scheduler, search, reduction, hashing and replay: guidelines for `mentalvm`

Scope: source-code reading of six projects, shallow-cloned on 2026-11-14. Nothing was built or run. Every claim cites a file at a pinned commit. Mental context comes from `docs/specification.md` §2 (P1-P4) and §8, and from `docs/research/model-interpreter-semantics/report.md`.

One line per product:
- **Stateright** (Rust, commit `ab8c8be`, 2025-07-27): explicit-state checker over a `Model` trait (`init_states`/`actions`/`next_state`). Has BFS, DFS, simulation and on-demand checkers, an actor layer with lossy and duplicating networks, symmetry by representative, and 64-bit fingerprints. Witnesses are action-index paths.
- **Shuttle** (Rust, `72bdbdb`, 2026-11-12): randomized concurrency tester. Re-runs real code under a `Scheduler` trait (random, PCT, DFS, replay). A schedule is a seed plus task IDs, encoded as a compact string. No state hashing, no POR, no shrinker.
- **loom** (Rust, `9d549cd`, 2026-11-08): stateless exhaustive explorer. Re-runs real code over a recorded `Path` of branch points and uses DPOR with vector clocks plus an optional preemption bound.
- **P** (C# checker plus the Java `PEx` explicit checker, `260da92`, 2026-08-03): has a strategy interface (random, PCT, DFS, replay with a suffix strategy), and an unhandled event is an error. PEx adds state caching with selectable hash modes, including an "Exact" mode, and detects cycles.
- **TLC / tlatools** (Java, `d8fb231`, 2026-11-06): BFS over 64-bit Rabin fingerprints. The trace file holds only (predecessor pointer, fingerprint) pairs; traces are rebuilt by re-generating successors. Supports symmetry and VIEW, and reports collision probability.
- **proptest** (Rust, `70cf37d`, 2026-10-10; shrinking only): a `ValueTree` with `simplify`/`complicate`; vectors shrink by deleting elements and then shrinking each element.

URL bases used below:
- SR = `https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/`
- SH = `https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/`
- LO = `https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/`
- P = `https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/`
- TL = `https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/`
- PT = `https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/`

## Q1. What is the smallest clean Rust API for enabled/fire plus pluggable strategies, where all nondeterminism goes through one recorded choice point?

### Takeaway
Stateright's three-method `Model` trait is already Mental's `enabled`/`fire` pair. Shuttle's one-method-per-decision `Scheduler` trait is the right shape for strategies. Mental should merge them: a pure `fire(&Model, &Config, &Choice)`, and one `Chooser` that picks from the sorted `enabled` list. A choice should be a semantic-ID enum, not an index or a seed, because replay on a changed model has to match choices by name. P's replay strategy shows exactly which mismatch cases a replay verdict must name.

### Cited Findings
**Step function shape**
- Stateright's whole model interface is `init_states(&self) -> Vec<State>`, `actions(&self, &State, &mut Vec<Action>)` and `next_state(&self, &State, Action) -> Option<State>`. Every checker (BFS, DFS, simulation, on-demand, explorer UI) is written against these alone. — [SR src/lib.rs L158-L179](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/lib.rs#L158-L179)
- Stateright's actor layer turns every source of nondeterminism into one `ActorModelAction` enum: `Deliver{src,dst,msg}`, `Drop(Envelope)`, `Timeout(Id,Timer)`, `Crash(Id)`, `Recover(Id)`, `SelectRandom{actor,key,random}`. Actors never call an RNG; `Command::ChooseRandom(key, Vec<Random>)` publishes the options, and the checker turns them into actions. This maps one-to-one onto Mental's `sched`/`env`/`data`/`pick`. — [SR src/actor/model.rs L46-L66](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L46-L66); [SR src/actor.rs L162-L172](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor.rs#L162-L172)
- `next_state` returns `Option<State>`, and the actor model returns `None` for "no-op" deliveries to prune them. The code itself warns: "Strictly speaking, this state should be updated regardless … only safe if invariants do not relate to the existence of envelopes on the network". The pruning is a hidden reduction inside the step function. — [SR src/actor/model.rs L418-L440](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L418-L440)

**Strategy interfaces**
- Shuttle's `Scheduler` trait has three methods:
  - `new_execution() -> Option<Schedule>`
  - `next_task(runnable: &[&Task], current: Option<TaskId>, is_yielding) -> Option<TaskId>`
  - `next_u64() -> u64`

  DFS, PCT, random, round-robin, replay and the nondeterminism checker all implement it. Returning `None` ends the execution. — [SH shuttle-engine/src/scheduler/mod.rs L75-L100](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/scheduler/mod.rs#L75-L100)
- P's `ISchedulingStrategy` splits the choice point into three methods: `GetNextOperation`, `GetNextBooleanChoice` and `GetNextIntegerChoice`, plus iteration control (`PrepareForNextIteration`, `HasReachedMaxSchedulingSteps`, `IsFair`). — [P Src/PChecker/CheckerCore/SystematicTesting/Strategies/ISchedulingStrategy.cs](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/ISchedulingStrategy.cs)
- Because of that split, P's exhaustive DFS keeps three parallel stacks (`ScheduleStack`, `BoolNondetStack`, `IntNondetStack`) with two indices. Backtracking has to coordinate them. — [P …/Strategies/Exhaustive/DFSStrategy.cs L27-L49, L209-L230](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/Exhaustive/DFSStrategy.cs)
- Stateright's simulation strategy takes a `Chooser` trait. It is built with `new_state(seed)` and has `choose_initial_state` and `choose_action(&mut state, &M::State, &[Action]) -> usize`, so random search is just "pick an index from the action list". — [SR src/checker/simulation.rs L22-L39](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/simulation.rs#L22-L39)

**Schedule representation**
- Shuttle's schedule is `Schedule { seed: u64, steps: Vec<ScheduleStep> }` with `enum ScheduleStep { Task(TaskId), Random }`. Random values are not stored; replay regenerates them from the seed. — [SH shuttle-engine/src/scheduler/mod.rs L10-L28](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/scheduler/mod.rs)
- The schedule is serialized as a varint-and-bitvector string: "[task id bitwidth] [number of schedule steps] [seed] [step]*". — [SH shuttle-engine/src/scheduler/serialization.rs](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/scheduler/serialization.rs)
- Stateright records witnesses as `Vec<usize>` action indices from an init-state index (`type Job<State> = (State, Vec<usize>, EventuallyBits, NonZeroUsize)`), and rebuilds the path by re-running `actions`/`next_state` (`Path::from_action_indices`). — [SR src/checker/dfs.rs L31](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/dfs.rs#L31); [SR src/checker/path.rs L21-L90](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/path.rs#L21-L90)
- Stateright also offers `Path::from_actions(model, init, &[Action])`, which matches each action by equality against `next_steps`. That is a by-value, not by-index, replay. — [SR src/checker/path.rs L92-L119](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/path.rs#L92-L119)
- P records `ScheduleStep { Type: SchedulingChoice | NondeterministicChoice, ScheduledOperationId, BooleanChoice?, IntegerChoice? }`. Values are stored explicitly, not derived from a seed. — [P …/SystematicTesting/Traces/ScheduleStep.cs](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Traces/ScheduleStep.cs)
- loom records a `Path` of typed branch entries (`Schedule`, `Load`, `Spurious`). Each entry keeps the exploration state of every alternative (thread states `Disabled/Skip/Yield/Pending/Active/Visited`), so the record doubles as the DFS work-list. It is serde-serializable under a `checkpoint` feature. — [LO src/rt/path.rs L1-L90](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs)

**Replay**
- Shuttle's `ReplayScheduler::next_task` panics with "schedule ended early" or "scheduled task is not runnable, expected to run {next:?}, but choices were {runnable:?}" unless `allow_incomplete` is set. — [SH shuttle-schedulers/src/replay.rs L82, L122](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/replay.rs)
- Shuttle's replay can also take a `target_clock`. It then skips schedule steps that are not causally before the failure, which is a causal slice of the witness. — [SH shuttle-schedulers/src/replay.rs L64-L115](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/replay.rs#L64-L115)
- P's `ReplayStrategy` names its failure modes:
  - "Trace is not reproducible: execution is longer than trace."
  - "… next step is not a scheduling choice."
  - "… cannot detect id '…'."

  — [P …/Strategies/Special/ReplayStrategy.cs L93-L107](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/Special/ReplayStrategy.cs#L93-L107)
- P's replay takes an optional `SuffixStrategy`. On the first mismatch it sets `IsReplaying = false` and hands control to that strategy, so it replays a prefix and then keeps exploring. — [P …/Strategies/Special/ReplayStrategy.cs](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies/Special/ReplayStrategy.cs)

**P semantics for unhandled events**
- When no state in the stack handles an event, P raises `UnhandledEventException` and then `Assert(isHalting, "{0} received event '{1}' that cannot be handled.")`. The only exception is `PHalt`, which halts gracefully. — [P Src/PChecker/CheckerCore/Runtime/StateMachines/StateMachine.cs L1043-L1070, L833-L866](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/Runtime/StateMachines/StateMachine.cs#L1043-L1070)

**Reductions that live in the enabled list**
- PEx's choice generator returns only the machine that creates another machine whenever one exists ("prioritize create machine events"). Otherwise it returns every machine whose send-buffer head targets a running machine. This is a built-in reduction in `enabled`. — [P Src/PEx/…/explicit/ExplicitSearchScheduler.java L552-L568](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java#L552-L568)

### Inferences
- **Proposed core API.** All of it is pure, does no I/O and needs no threads, so it compiles to `wasm32-unknown-unknown`:
  ```rust
  pub fn initial(m: &Model, b: &Binding) -> Config;
  pub fn enabled(m: &Model, c: &Config) -> Vec<Choice>;            // sorted, deduplicated
  pub fn fire(m: &Model, c: &Config, ch: &Choice) -> Fired;        // pure; never panics on model input
  pub enum Fired { Next(Config, Step), End(Outcome, Step) }        // the outcome is data, not a panic
  #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  pub enum Choice { Sched(ActivationId), Env { at: WireId, act: EnvAct },
                    Data { at: PredId, value: ClassId }, Pick { at: WireId, target: InstanceId } }
  pub trait Chooser { fn choose(&mut self, c: &Config, enabled: &[Choice]) -> Option<Choice>; }
  pub fn run(m: &Model, b: &Binding, ch: &mut impl Chooser, bounds: &Bounds) -> Run; // Run = schedule + outcome
  pub fn explore(m: &Model, b: &Binding, bounds: &Bounds) -> Report;                 // owns its own frontier
  pub fn replay(m: &Model, b: &Binding, s: &Schedule) -> Replay;                     // per-choice verdicts
  ```
  This is Stateright's trait as free functions over data, with Shuttle's chooser shape. Bool and int choices fold into `Choice::Data`, which avoids P's three parallel DFS stacks.
- **One `Choice` enum.** One enum, one `Vec<Choice>` schedule and one DFS stack is the smallest design that still names every kind of nondeterminism. Shuttle's `Random` step that stores no value does not suit Mental, because a `data` value must survive a model change. Store values, as P does.
- **Index-based witnesses break under model change.** Stateright's `Vec<usize>` witnesses only replay against the exact same model; its own panic text blames HashMap order. Mental's replay on a changed model needs choices named by semantic IDs, matched by equality as in `Path::from_actions`.
- **Replay verdicts follow P's messages.** P's three messages map onto Mental's verdicts:

  | P failure | Mental verdict |
  |---|---|
  | "cannot detect id" (the choice's ID is not in `enabled`) | `infeasible` |
  | Type mismatch, or the trace runs out | `diverged` |
  | Choice found and applied | `matched` |

  P's suffix strategy is the right shape for "replay, then search on" after a fix.
- **Outcomes are data.** Model gap, illegal write, deadlock and the rest should come back from `fire`/`run`, never as panics. Shuttle and P detect failures through panics and exceptions because they run real code; Mental runs a model and has no such constraint.
- **Keep reductions out of `fire` and `enabled`.** Stateright's no-op pruning and PEx's create-first rule show that semantics and reduction get mixed up easily. Mental should keep `enabled` complete and do all pruning in strategies, where a test can switch it off.

### Gaps
- None of the projects carries a path condition φ. The symbolic strategy (forking on unknown predicates) has no direct precedent in these codebases; it is only "a `Data` choice with both values enabled".

## Q2. Persistent/immutable configurations vs clone-on-step vs undo logs: trade-offs for clarity, speed and WebAssembly

### Takeaway
None of the projects uses undo logs. The checkers that own their state (Stateright, TLC, PEx) clone on step, and Stateright makes cloning cheap by keeping each actor's state behind an `Arc` and using `Cow`. The testers that run real code (Shuttle, loom, the P C# checker) cannot snapshot, so they re-execute from the start on every path. For `mentalvm`, a value-typed `Config` that is cloned on step, with `Arc` around per-instance parts, is the simplest choice and fully WebAssembly-safe.

### Cited Findings
- Stateright's actor system state is `ActorModelState { actor_states: Vec<Arc<A::State>>, network, timers_set, random_choices, crashed: Vec<bool>, history, actor_storages }`. Cloning it copies the vector of `Arc`s, not the actors. — [SR src/actor/model_state.rs L15-L23](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model_state.rs#L15-L23)
- On delivery, Stateright runs the actor handler on a `Cow::Borrowed` actor state. It clones the system state (`last_sys_state.clone()`) and replaces only that actor's `Arc` if the handler produced `Cow::Owned`. — [SR src/actor/model.rs L402-L450](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L402-L450)
- PEx keeps mutable machines. For stateful backtracking it snapshots with `StepState.copyState()` (deep copy of per-machine local state) and restores with `setTo(...)`. When no snapshot exists, it resets to zero and re-executes the prefix. — [P Src/PEx/…/explicit/StepState.java](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/StepState.java); [P …/ExplicitSearchScheduler.java L520-L545](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java)
- Shuttle's DFS is stateless:
  - `levels: Vec<(TaskId, bool /*was last*/)>`, and each execution re-runs the test from scratch.
  - The scheduler keeps the previous choice at each level while deeper levels still have options, and advances the deepest level that has more.
  - The whole scheduler is 106 lines.

  — [SH shuttle-schedulers/src/dfs.rs L16, L69-L100](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/dfs.rs)
- loom is stateless as well. `Path::step()` resets `pos = 0` and advances the last branch with an unexplored alternative; each iteration re-runs the closure and follows the recorded branches. — [LO src/rt/path.rs L393](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs#L393)
- loom fails if re-execution leaves the recorded path, using `expect("Reached unexpected exploration state. Is the model fully deterministic?")`. Stateless search depends on the program being deterministic. — [LO src/rt/path.rs](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs)
- TLC keeps a mutable state class (`TLCStateMut`) but generates a fresh successor per transition. Its trace file stores no states at all, only `(predecessorLoc, fingerprint)` pairs of 12 bytes. — [TL src/tlc2/tool/TLCTrace.java L77-L81](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/TLCTrace.java#L77-L81)

### Inferences
- **Clone on step.** Mental's spec already says "two configurations are equal however they were reached" (§8.1). `fire(&C) -> C` with `#[derive(Clone, Eq, Hash, Ord)]` on `Config` makes snapshots, stepping backwards (keep the old value) and DFS backtracking (pop the stack) free of extra code.
- **No undo logs.** An undo log needs inverse operations for every step kind (deliver, write, lock, spawn, duplicate…), which roughly doubles the step code and its tests. That cuts against "compact code".
- **Make clones cheap the way Stateright does.** Put `Arc<Instance>` or `Arc<ActivationState>` inside vectors or `BTreeMap`s keyed by ID, and use `Arc::make_mut` in `fire`. `Arc` works on wasm32 (single-threaded, atomics compile to plain ops).
- **Prefer stateful DFS over stateless re-execution.** Mental can snapshot, so it should not re-execute like loom and Shuttle: re-execution costs O(depth) per leaf and rules out a visited set. Shuttle's 106-line `levels` DFS is still a useful reference for the backtracking bookkeeping.
- **No persistent-collection crate in v1.** `im` or `rpds` would be a new dependency. Add one only if profiling shows clone cost dominates; report-jobs bounds are tiny.

### Gaps
- No source here measures clone-on-step against undo logs for actor-style models. The choice above rests on simplicity, not on benchmarks.

## Q3. How to fingerprint states deterministically and make runs reproducible bit-for-bit

### Takeaway
Every project has been bitten by unstable iteration order or unstable hashing:
- Stateright sorts its action list for that reason and hand-rolls order-independent hashing for hash sets.
- Stateright's "stable" hasher is ahash with fixed keys, which ahash's own README says differs across machines.
- PEx combines hashes with XOR.
- TLC alone uses a fingerprint with a fixed specification (Rabin, irreducible polynomial) and reports the collision probability.

For Mental:
- `BTreeMap`/`Vec` only inside `Config`.
- `enabled` returns a sorted list.
- A portable seeded RNG (a PCG variant, as Shuttle uses).
- An exact visited set in v1, with a fixed-spec 64- or 128-bit fingerprint only when memory requires it.

### Cited Findings
**Fingerprints**
- Stateright: `type Fingerprint = NonZeroU64`, computed by `fingerprint<T: Hash>(value)` with `stable::hasher()`. That is ahash `RandomState::with_seeds(KEY1, KEY2, 0, 0)`, and the source carries `// TODO: how to get these?` on the last two keys. — [SR src/lib.rs L344-L390](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/lib.rs#L344-L390)
- ahash's README: "Because it does not have a fixed standard, different computers or computers on different versions of the code will observe different hash values." Stateright's fingerprints are therefore not portable across machines, versions, or native vs wasm builds. — ahash 0.8.12 README L22 (read from the local cargo registry; upstream [github.com/tkaitchuck/aHash](https://github.com/tkaitchuck/aHash))
- Stateright's `HashableHashSet::hash` hashes each element with the stable hasher into a buffer, then `sort_unstable()` and feeds the sorted u64s. This is order-independent hashing of an unordered set. — [SR src/util.rs L137-L158](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/util.rs#L137-L158)
- PEx `ComputeHash.getHashCode` combines element hashes with `hashValue ^ val.hashCode()` from a constant seed. XOR is order-insensitive, and equal elements cancel each other (x ^ x = 0). — [P Src/PEx/…/values/ComputeHash.java L24-L52](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/values/ComputeHash.java#L24-L52)
- PEx lets the user pick the state key: `StateCachingMode { None, HashCode, SipHash24, Murmur3_128, Sha256, Exact }`. "Exact" uses the full state string (`ComputeHash.getExactString`), which gives a collision-free reference mode. — [P …/explicit/StateCachingMode.java](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/StateCachingMode.java); [P …/ExplicitSearchScheduler.java L296-L310](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java#L296-L310)
- TLC fingerprints with `FP64` (a Rabin fingerprint over a fixed irreducible polynomial, `IrredPoly`). The state fingerprint extends it variable by variable: `fp = minVals[i].fingerPrint(fp)`. With VIEW, it fingerprints the value of the view expression instead, so only part of the state is abstracted for deduplication. — [TL src/tlc2/util/FP64.java](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/util/FP64.java); [TL src/tlc2/tool/TLCStateMut.java L171-L272](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/TLCStateMut.java#L171-L272)
- TLC reports the probability of a fingerprint collision at the end of every run. It first computes an "optimistic" probability and computes the actual one (`checkFPs()`, based on the observed fingerprint distribution) only when the optimistic one exceeds 1e-10. — [TL src/tlc2/tool/ModelChecker.java L962-L972](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/ModelChecker.java#L962-L972)

**Symmetry**
- TLC symmetry: the fingerprint is taken of the lexicographically smallest permutation of the state, computed by trying all permutations (O(|perms|·|vars|); the comment notes the constructive orbit problem is NP-hard). — [TL src/tlc2/tool/TLCStateMut.java L174-L247](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/TLCStateMut.java#L174-L247)
- Stateright symmetry: a `Representative::representative(&self) -> Self` trait, with `RewritePlan::from_values_to_sort` to sort and reindex symmetric actors. Only the representative is fingerprinted. — [SR src/checker/representative.rs](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/representative.rs); [SR examples/2pc.rs L149-L160](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/examples/2pc.rs#L149-L160)

**Determinism of order**
- Stateright's actor model sorts the action list before returning it: "Action indices based `Path` construction relies on the consistent ordering of the returned actions. Some iterators like `HashableHashSet` do not guarantee a stable order. For simplicity, we sort actions here." — [SR src/actor/model.rs L342-L386](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/actor/model.rs#L342-L386)
- Stateright's path reconstruction panics with a diagnosis naming the usual causes: "iterating over a `HashMap` or `HashableHashMap` does not always happen in the same order … the file system, a thread local `RefCell`, or a source of randomness". — [SR src/checker/path.rs L28-L75](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/path.rs#L28-L75)

**RNG**
- Stateright's `UniformChooserState` uses `StdRng` under the comment `// FIXME: use a reproducible rng, one that will not change over versions.` — [SR src/checker/simulation.rs L46-L48](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/simulation.rs#L46-L48)
- Shuttle's PCT and random schedulers use `rand_pcg::Pcg64Mcg::seed_from_u64(seed)`, a named algorithm. The seed can be overridden by the `SHUTTLE_RANDOM_SEED` environment variable (`seed_from_env`). — [SH shuttle-schedulers/src/pct.rs L50](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/pct.rs#L50)
- Shuttle's DFS fixes its data seed (`const DFS_RANDOM_SEED: u64 = 0x12345678`), so every DFS execution sees the same "random" values. Its doc says enabling random data "makes the DFS search incomplete". — [SH shuttle-schedulers/src/dfs.rs L6, L22-L28](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/dfs.rs)

**Run-time nondeterminism checks**
- Shuttle's `UncontrolledNondeterminismCheckScheduler` wraps any scheduler and replays each schedule once. It checks "that the schedule is still valid and that the set of runnable tasks is the same at each step". — [SH shuttle-schedulers/src/uncontrolled_nondeterminism.rs L1-L30](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/uncontrolled_nondeterminism.rs)

**Visited set and cycles**
- PEx uses the visited set to detect cycles. If the same state key reappears within the same schedule, it reports "Cycle detected: Infinite loop found due to revisiting a state multiple times in the same schedule". If it was seen in another schedule, it prunes ("done with this schedule"). — [P …/ExplicitSearchScheduler.java L249-L290](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java#L249-L290)

### Inferences
- **Determinism by construction.** Three rules:
  - No `HashMap`/`HashSet` in the `mentalvm` core. Enforce with clippy `disallowed-types` in `clippy.toml`.
  - `enabled` returns `Vec<Choice>` sorted by `Choice: Ord`.
  - Every collection inside `Config` is a `BTreeMap`, `BTreeSet` or `Vec` indexed by IDs derived from the creating step (spec §8.2).

  Then no sorting tricks like `HashableHashSet` are needed, and `#[derive(Hash)]` is order-stable.
- **Visited set in v1: exact, not fingerprinted.** Use a `BTreeSet<Config>`, or a `HashSet<Config>` with a fixed-key hasher, since lookup order never leaks. This is PEx's `Exact` mode and is sound by construction. The report-jobs bounds give a tiny state space.
- **If fingerprints are needed later:**
  - Fingerprint a canonical serialization of `Config` with a hash whose output is specified, for example SipHash-1-3 or 2-4 with fixed keys, FNV-1a or xxh3. Never ahash, never `std::collections::hash_map::DefaultHasher` (its algorithm is not guaranteed across Rust releases; this comes from std documentation, not read here).
  - Keep 128 bits, or 64 bits plus TLC-style collision reporting.
  - Keep `Exact` as a test mode and assert that both modes give identical results on the corpus.
- **Bounds must be part of the visited key.** With bounds (dup ≤ 1, preemptions ≤ 1, depth ≤ 40), a configuration reached with budget left is not equivalent to the same configuration reached with the budget spent. Either include the remaining budgets in the visited key, or store, per configuration, the best (largest) remaining budget seen and revisit when a better one arrives. Stateright's DFS keeps depth in the job tuple, not in the state, and still deduplicates by state fingerprint (`dfs.rs` L31, L329), so with `target_max_depth` it can prune a state first reached deep. Inference from code reading; not tested.
- **Zero-time cycles fall out of the visited set.** "Same configuration (excluding the trace) revisited on the current path with `now` unchanged" is PEx's cycle check and Mental's zero-time-cycle outcome. Keep the set of configurations on the current DFS path; a search with a visited set has that path on its stack already.
- **RNG.** Use one named, version-pinned algorithm for sampled search (`rand_pcg::Pcg64Mcg` as in Shuttle, or a hand-written SplitMix64 of about 10 lines, which avoids the `rand` dependency). Put the seed in the run file. Never take it from entropy in the core; the CLI may take it from entropy and must print it.
- **Avoid XOR combining.** PEx's XOR combining loses structure (cancellation, order-insensitivity); avoid it.
- **Symmetry is v2.** Mental's "worker[0]/worker[1]" replicas are symmetric, so Stateright's `representative` approach (sort replicas by state, reindex) is the simple form. Treat it as a later optimisation with a differential test (see Q6).

### Gaps
- Did not read Rust `std` docs here to cite the instability of `DefaultHasher`. That claim comes from prior knowledge and should be verified.
- No project measures how often fingerprint collisions happen at Mental's scale; TLC's bound formula is the only quantitative tool, and it was not read in detail.

## Q4. Which partial-order reduction is simplest to implement correctly, and how do these projects test that POR does not miss bugs?

### Takeaway
Only loom implements POR among the six: a DPOR driven by vector clocks, with a "last dependent access" per object, a conservative "explore all threads" fallback, and an extra backtrack point under preemption bounding. Stateright, Shuttle, the P C# checker, PEx and TLC rely on bounds, symmetry, state caching or randomness instead.

No project directly compares POR against unreduced search in a test. The nearest patterns are:
- loom's outcome-set tests (assert the exact set of observable results);
- Stateright's reduced-vs-unreduced symmetry test, which pins both state counts and requires the same property verdicts.

For `mentalvm` v1, bounded DFS with an exact visited set and a preemption bound may be enough. If POR is added, start with sleep sets over static footprints, behind a differential test against unreduced search on the whole corpus.

### Cited Findings
- loom's `Execution::schedule()` ("Implementation of the DPOR algorithm"):
  - For every thread with a pending operation, it fetches `objects.last_dependent_access(operation)`.
  - If that access does not happen-before the thread's `dpor_vv` vector clock, it calls `path.backtrack(access.path_id(), th_id)`.

  — [LO src/rt/execution.rs L134-L160](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/execution.rs#L134-L160)
- Dependence in loom is per object and per action kind. For atomics, a load depends on `last_non_load_access`; anything else depends on `last_access`. Each `Access` stores `(path_id, VersionVec)`, and `happens_before` is `dpor_vv <= version`. — [LO src/rt/atomic.rs L828-L847](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/atomic.rs#L828-L847); [LO src/rt/access.rs](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/access.rs)
- At the backtrack point, loom marks the racing thread for exploration if it is enabled there; otherwise it marks all threads (`for th in &mut self.threads { th.explore(); }`). Under a preemption bound it skips points whose `preemptions == bound`. — [LO src/rt/path.rs L493-L520](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs#L493-L520)
- "Preemption bounded DPOR requires conservatively adding another backtrack point to cover cases missed by the bounds." loom walks back to the most recent schedule point where the active thread changed and adds a backtrack there too. — [LO src/rt/path.rs L336-L385](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs#L336-L385)
- loom keeps preemptions to a minimum by default: the initial choice at each schedule point is the currently active thread, if it is still runnable ("It's important to avoid pre-emption as much as possible"). The `preemptions` count is carried per `Schedule` branch. — [LO src/rt/execution.rs L163-L200](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/execution.rs#L163-L200); [LO src/rt/path.rs L241-L330](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs#L241-L330)
- loom's docs claim that "setting the thread pre-emption bound to 2 or 3 is enough to catch most bugs" (an experience claim, not measured in the repo). They credit the CDSChecker paper as the basis of the implementation. — [LO src/lib.rs L271-L291, L339-L347](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/lib.rs#L271-L291)
- loom's C11 litmus tests collect every observed outcome into a set and assert it exactly. In `acq_rel`, `seen` must contain 0, 1 and 2 and have length 3. A reduction that dropped a needed interleaving would fail this. `relaxed` is `#[ignore]`d with the note "loom cannot fully model `Ordering::Relaxed`", an incompleteness the test suite records openly. — [LO tests/spec.rs L1-L130](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/tests/spec.rs)
- Stateright has no POR. Its reductions are symmetry plus checker-level dedup. `examples/2pc.rs` checks the 5-RM system both ways: unreduced (`unique_state_count() == 8_832`) and with `.symmetry()` (`== 665`). Both call `assert_properties()`. — [SR examples/2pc.rs L183-L198](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/examples/2pc.rs#L183-L198)
- Shuttle has no POR. Its exhaustive `DfsScheduler` enumerates every interleaving, and its tests pin the exact iteration counts (1, 2, 15, 4, 7, 6, 252, 100, 50050, 20 executions in `tests/basic/dfs.rs`). — [SH shuttle/tests/basic/dfs.rs L30-L279](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/dfs.rs)
- PEx prunes with state caching only after the backtrack choice number ("perform state caching only if beyond backtrack choice number") and uses the create-machine-first heuristic. Neither is POR in the textbook sense. — [P …/ExplicitSearchScheduler.java L249-L260, L552-L568](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PEx/src/main/java/pex/runtime/scheduler/explicit/ExplicitSearchScheduler.java)

### Inferences
- **Why loom's DPOR does not transfer.** loom's DPOR is tightly coupled to its runtime objects (each object type implements `last_dependent_access`/`set_last_access`) and to stateless re-execution. It cannot be lifted out as a module. A DPOR in `mentalvm` would need vector clocks per activation, last-access tracking per state variable, lock and channel, and the conservative fallbacks loom adds. That is a lot of subtle code for v1.
- **Simplest correct option: no POR in v1.** Mental's actual bounds (1 request, 2 workers, 1 duplicate, 1 preemption) are small. "Activations run until they wait" already removes the intra-activation interleavings, which is the coarse step granularity P uses. The visited set merges the different orders of independent steps that reach the same configuration. Bounded DFS with an exact visited set, plus a preemption bound counted the way loom counts it, is complete for the stated bounds with no POR code at all.
- **If POR is needed: sleep sets over static footprints.**
  - Mental's choices carry footprints (reads, writes, channels, locks), which are also used for atomicity (spec §8.4, §8.7).
  - Two `sched` choices of different activations are independent if neither writes what the other reads or writes, and neither enables or disables the other.
  - Sleep sets need no backtracking analysis; each DFS node passes on a set of choices not to try.
  - The known catch: combining sleep sets with a visited set needs care (store the sleep set with the state, or revisit when the new sleep set is smaller). That point comes from the POR literature, not from code read here.
- **The test that keeps POR honest.** For every corpus model and bound, POR on and off must produce the same set of outcome kinds and violations at the same nodes. Exact state counts may differ, and both get pinned as snapshots, as in Stateright's 2pc (8,832 vs 665). Run it on every CI build. Add proptest-generated tiny models (2-3 activations, 2-3 shared variables) so the differential covers shapes nobody wrote by hand.
- **Count preemptions as loom does.** A preemption is a `sched` choice of a different activation while the current activation is still runnable (it has not reached a wait). Each DFS node carries a `preemptions_left` counter, which then has to be part of the visited key (Q3).

### Gaps
- No studied project has a test that runs DPOR and full search side by side. Only loom has DPOR, and it relies on litmus outcome sets.
- I did not find sleep-set code in any of the six repos, so the sleep-set recommendation rests on the literature, not on a code precedent.

## Q5. How do they minimize a counterexample schedule?

### Takeaway
None of the five checkers or testers has a schedule shrinker in its source:
- Stateright and TLC get short witnesses by construction from BFS.
- Shuttle ships no minimizer; it relies on running inside proptest, and its `shrink.rs` test only replays hand-shrunk schedules.
- P and PEx have none.

proptest's `ValueTree` gives the cleanest algorithm to borrow: delete elements one at a time, then simplify each remaining element, and step back with `complicate` when a candidate stops failing. For Mental, find the witness with shortest-first search, then run a delete-one-choice pass that re-checks each candidate by replay.

### Cited Findings
- Stateright documents that BFS "will find the shortest `Path` to each discovery if checking is single threaded", while DFS does not. — [SR src/checker.rs, `spawn_bfs` doc](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs)
- Stateright's BFS originally rebuilt paths from parent fingerprints (`DashMap<Fingerprint, Option<Fingerprint>>`) and now also stores `Vec<usize>` per job. The comment says "with action indices encoding, this is not possible, so we must store a Vec (like in dfs.rs)". — [SR src/checker/bfs.rs L20-L37](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/bfs.rs#L20-L37)
- TLC rebuilds traces from (predecessor pointer, fingerprint) records. For each record it calls `tool.getState(fp, predecessor.state)`, which regenerates the predecessor's successors and keeps the one whose fingerprint matches. BFS order makes this trace shortest. — [TL src/tlc2/tool/TLCTrace.java L201-L230](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/src/tlc2/tool/TLCTrace.java#L201-L230)
- Shuttle's `tests/basic/shrink.rs` replays a 9-step failing schedule `[0,0,1,1,1,2,2,2,2]` and a hand-minimized 3-step one `[0,0,2]`, and asserts both still panic. There is no shrinker in `shuttle-engine/src`. The only mention is a span-reset helper "to enable correct spans when doing proptest minimization". — [SH shuttle/tests/basic/shrink.rs](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/shrink.rs); [SH shuttle-engine/src/runtime/runner.rs L15-L22](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/runtime/runner.rs#L15-L22)
- Shuttle's causal-slice replay (`set_target_clock`) drops steps that are concurrent with the failure. It is a cheap minimization of the scheduled steps, as long as the failure's vector clock is known. — [SH shuttle-schedulers/src/replay.rs L60-L115](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/replay.rs#L60-L115)
- proptest's `ValueTree` contract: `simplify()` moves "current" halfway toward the simplest value, and `complicate()` partly undoes the last simplification. Both must be safe to call again after returning false. — [PT strategy/traits.rs L580-L620](https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/strategy/traits.rs#L580-L620)
- proptest's `VecValueTree::simplify` first tries `DeleteElement(ix)` for each index (respecting `min_size`), then `ShrinkElement(ix)` for each kept element. `complicate` undoes only the previous shrink. — [PT collection.rs L591-L650](https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/collection.rs#L591-L650)
- proptest's shrink loop keeps the candidate while it still fails and calls `complicate()` when a candidate passes. It is bounded by `max_shrink_iters` and `max_shrink_time`, and on bail-out "Move back to the most recent failing case". — [PT test_runner/runner.rs L769-L880](https://github.com/proptest-rs/proptest/blob/70cf37d46b7d83393b6ed94c611f8c52209f585d/proptest/src/test_runner/runner.rs#L769-L880)

### Inferences
- **Define "minimal" so it can be tested.** Order candidates lexicographically by:
  1. fewest choices that are not forced (choices made where `enabled` had one element do not count, as Shuttle's PCT skips steps with `runnable.len() == 1`, [SH pct.rs L141](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src/pct.rs#L141));
  2. fewest preemptions;
  3. fewest `env` faults;
  4. smallest `Choice` order.

  A witness is minimal when no single deletion or replacement keeps the same outcome kind at the same node.
- **Algorithm (about 40 lines).**
  1. Find the witness with search that tries fewer preemptions first. Either run iterative deepening on the preemption bound (0, then 1) and use BFS or depth-ordered DFS inside it, or record the shortest witness per outcome.
  2. Repeat until nothing changes: for each index `i`, try the schedule with choice `i` removed, under a replay that treats `infeasible` choices as skipped and fills any other choice point with the default (the non-preempting choice). Keep the result if the outcome kind and node match.

  This is proptest's delete-element phase with a replay oracle. Shuttle's causal slice is an optional pre-pass once activations carry causes (spec §8.6 `causes`).
- **The acceptance test can pin the result.** The report-jobs witness (spec §8.6 schedule, 7 choices) can be snapshot-tested with insta: the minimizer must reproduce exactly that schedule or a shorter one.

### Gaps
- None of the codebases measures minimizer quality. The expected minimal size for report-jobs comes only from the spec's example schedule.

## Q6. How do model checkers test themselves?

### Takeaway
Six patterns recur:
1. A known-bug corpus with expected errors, and a correct corpus that must pass (P: 145 `DynamicError` programs).
2. Golden state or execution counts (TLC: 282 test files assert `TLC_STATS`; Stateright: `unique_state_count`; Shuttle: DFS iteration counts).
3. Differential runs across strategies or reductions (Stateright BFS vs DFS give equal counts; symmetric vs plain give the same verdicts).
4. Exact outcome-set assertions (loom litmus tests).
5. Witness validation by replay rather than exact match (Stateright `assert_discovery`).
6. Replay round-trip and determinism checkers (Shuttle). Shuttle's replay round-trip tests are currently disabled because the replay mechanism broke, which shows why that test must gate CI.

### Cited Findings
- P's regression suite is laid out by feature (`Feature1SMLevelDecls` … `Feature5ModuleSystem`, `Combined`, `Liveness`), each with `Correct/`, `DynamicError/` and `StaticError/` folders. 145 `DynamicError` programs (a known-bug corpus) and the `Correct` programs are loaded by `TestCaseLoader.FindTestCasesInDirectory(..., new[] { "DynamicError", "Correct" })`. Expected checker logs live in `Tst/CorrectLogs`. — [P Tst/RegressionTests](https://github.com/p-org/P/tree/260da928f67569ca7a8c0039a02d0bad7cde6096/Tst/RegressionTests); [P Tst/UnitTests/RegressionTests.cs L59, L91](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Tst/UnitTests/RegressionTests.cs); [P Tst/UnitTests/PCheckerLogGeneratorTests.cs L28-L31](https://github.com/p-org/P/blob/260da928f67569ca7a8c0039a02d0bad7cde6096/Tst/UnitTests/PCheckerLogGeneratorTests.cs)
- TLC tests assert exact statistics through a message recorder, for example `recorder.recordedWithStringValues(EC.TLC_STATS, "950", "555", "89")` (generated, distinct, left). 282 test files under `test/tlc2/tool` reference `TLC_STATS`, against about 511 spec directories in `test-model`. — [TL test/tlc2/tool/EvalExceptionLivenessTest.java L50-L51](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/test/tlc2/tool/EvalExceptionLivenessTest.java#L50-L51)
- TLC also asserts the exact error trace (`assertTraceWith(recorder.getRecords(EC.TLC_STATE_PRINT2), expectedTrace)`). — [TL test/tlc2/tool/ActionCompositionBTest.java L61-L67](https://github.com/tlaplus/tlaplus/blob/d8fb231de4e4b5d02c1b2ed1ac759c086f694021/tlatools/org.lamport.tlatools/test/tlc2/tool/ActionCompositionBTest.java#L61-L67)
- Stateright pins state counts per checker on the same model: `assert_eq!(checker.unique_state_count(), 256 * 256)` appears in the BFS (L450), DFS (L474) and on-demand (L516) tests. The examples pin counts too (2pc: 288, 8,832, 665; single-copy-register: 93, 54). — [SR src/checker/bfs.rs L450](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/bfs.rs#L450); [SR src/checker/dfs.rs L474-L584](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/dfs.rs#L474); [SR examples/2pc.rs L183-L198](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/examples/2pc.rs#L183-L198)
- Stateright's `assert_discovery(name, actions)` replays the given action sequence with `Path::from_actions` and accepts it if it really violates or satisfies the property. A test can state the expected witness without depending on which equivalent witness the search found first. — [SR src/checker.rs L534-L560](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs#L534-L560)
- Shuttle's PCT tests reproduce the figures of the PCT paper and justify iteration counts by the probability guarantee in comments, for example "n=2, k=20, d=2, so probability of finding the bug in one iteration is at least 1/(2*20) … in 300 iterations … > 99.9%". — [SH shuttle/tests/basic/pct.rs L101-L195](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/pct.rs#L101-L195)
- Shuttle's `check_replay_roundtrip` runs a failing test, parses the printed schedule, replays it, and asserts the same failure and schedule. Its uses in `tests/basic/replay.rs` are all `#[ignore = "replay mechanism is broken because the schedule is not emitted in the panic output. reintroduce once replay mechanism is fixed."]`. — [SH shuttle/tests/mod.rs L29-L70](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/mod.rs#L29-L70); [SH shuttle/tests/basic/replay.rs L32-L51](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic/replay.rs#L32-L51)
- Shuttle has a dedicated test file for its nondeterminism checker (`tests/basic/uncontrolled_nondeterminism.rs`) and a `portfolio.rs` test of running several schedulers. — [SH shuttle/tests/basic/](https://github.com/awslabs/shuttle/tree/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle/tests/basic)
- loom's tests include the C11 spec litmus tests with exact outcome sets (Q4), plus regression tests named after bugs (`rwlock_regression1.rs`, `double_panic_in_drop.rs`). — [LO tests/](https://github.com/tokio-rs/loom/tree/9d549cd5357a5485900f4c53951fc1726319659f/tests)
- Stateright's path rebuilding panics with tests for that panic (`panics_if_unable_to_reconstruct_init_state`, `panics_if_unable_to_reconstruct_next_state`), so its nondeterminism diagnostic is tested too. — [SR src/checker/path.rs L221-L244](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker/path.rs#L221-L244)

### Inferences
**Test suite for `mentalvm`, in order of value:**
1. **Corpus** (`crates/mentalvm/tests/corpus/`, P-style): tiny models, each with a sidecar `expect` giving the outcome kind, the node, and an insta snapshot of the minimal witness and of the counts (configs, transitions, max depth). It needs one model per outcome kind:
   - normal end
   - model gap (unhandled message)
   - deadlock
   - illegal write
   - zero-time cycle
   - invariant violated

   Plus report-jobs before and after the fix, and "correct" models that must report "none within bounds".
2. **Differential checks:**
   - `explore` with POR on and off (and symmetry on and off, later) gives the same set of `(outcome kind, node)`;
   - BFS and DFS find the same violations;
   - every sampled run's outcome is in the exhaustive outcome set for the same bounds.
3. **Replay round-trip:** for every corpus witness, `replay(model, schedule)` gives all `matched` verdicts and the same outcome, and serde round-trip of the run file is identity. Never `#[ignore]`d (the Shuttle lesson).
4. **Determinism:** run each strategy twice in-process and assert byte-identical serialized reports. Add a test that the visited set gives identical results in exact and fingerprint modes, once fingerprints exist.
5. **Changed-model replay:** report-jobs with the CAS fix, where the stored race schedule must give a `diverged` verdict at `worker[1]/n:claim`, and a fresh `explore` must report no violation (spec §8.9, F-S11). A deletion test checks `infeasible`.
6. **proptest properties over generated small models:**
   - `fire` never panics;
   - every element of `enabled(c)` fires;
   - `run` with a replay chooser reproduces any schedule produced by `explore`;
   - the minimizer's output still fails and is no longer than its input.
7. **cargo-mutants** on `fire`, `enabled` and the search loop. It catches assertions that a weak corpus never makes fail.

**No probability-based tests for v1.** Use fixed seeds instead. Shuttle's probability-justified iteration counts suit PCT, which v1 does not need.

### Gaps
- None of the projects documents a mutation-testing or coverage gate. The "100% diff coverage" and cargo-mutants requirements have no precedent in these repos.

## Q7. What mentalvm should copy and avoid, and candidate guidelines

### Takeaway
Copy:
- Stateright's step interface and its sorted action list;
- Shuttle's single scheduler trait;
- P's explicit, value-carrying schedule entries, named replay failures, and replay-then-continue;
- PEx's exact-mode visited set and its cycle check;
- TLC's collision honesty;
- loom's preemption counting;
- proptest's delete-then-simplify shrinking.

Avoid:
- index or seed-based schedules;
- ahash, `StdRng`, XOR hashing;
- hidden reductions inside the step function;
- multi-threaded checkers;
- runtime-coupled DPOR;
- replay tests that are allowed to be ignored.

Flagged as bloat for v1:
- Stateright's web explorer, threads, DashMap job broker and on-demand checker;
- Shuttle's about 20 wrapper crates, PCT, URW and annotations;
- loom's relaxed-memory model and checkpoint files;
- P's Q-learning, feedback-guided, POS and liveness-temperature strategies;
- PEx's five hash modes;
- TLC's disk FPSet, distributed mode, symmetry and VIEW.

v1 needs concrete runs, bounded-exhaustive DFS and replay. Sampled search is about 30 lines on top (a seeded `Chooser`) and is optional.

### Cited Findings
- Stateright's checker builder pulls in multi-threading (`threads(n)`, `JobBroker`, `DashMap`/`DashSet`, `nohash-hasher`), an HTTP explorer (`tiny_http`, `serve()`), timeouts and visitors. Its dependencies include `ahash`, `dashmap`, `tiny_http`, `parking_lot`, `rand`, `serde_json`. — [SR src/checker.rs L58-L300](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs); [SR Cargo.toml](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/Cargo.toml)
- Shuttle's scheduler crate alone has 12 files: `random`, `pct`, `dfs`, `replay`, `round_robin`, `urw`, `annotation`, `uncontrolled_nondeterminism`, `check`, and more. The repository adds wrapper crates for tokio, dashmap, parking_lot and collections. The core trait is 26 lines. — [SH shuttle-schedulers/src/](https://github.com/awslabs/shuttle/tree/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-schedulers/src); [SH shuttle-engine/src/scheduler/mod.rs](https://github.com/awslabs/shuttle/blob/72bdbdb612c2311905c84bbc73d59f12a5b17e43/shuttle-engine/src/scheduler/mod.rs)
- P's C# strategies directory holds Exhaustive (DFS), Probabilistic (Random, PCT, QLearning, ProbabilisticRandom), Feedback (PCT, POS, Random, ScenarioSteering, TimelineObserver), Liveness and Special (Combo, Replay): about 3,770 lines in total. — [P Src/PChecker/CheckerCore/SystematicTesting/Strategies](https://github.com/p-org/P/tree/260da928f67569ca7a8c0039a02d0bad7cde6096/Src/PChecker/CheckerCore/SystematicTesting/Strategies)
- loom's `Load` branches record up to `MAX_ATOMIC_HISTORY` candidate stores per atomic load, to model C11 weak memory. Mental's sequentially consistent model needs none of it. — [LO src/rt/path.rs L45-L55](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs)
- loom caps branches with a fixed-capacity store and a clear error: "Model exceeded maximum number of branches. This is often caused by an algorithm requiring the processor to make progress, e.g. spin locks." — [LO src/rt/path.rs `assert_path_len!`](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/path.rs)

### Inferences: candidate guidelines for `mentalvm`

Each rule gives its rationale, source, and how it is enforced.

**State and step**

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G1 | The core exposes `initial`, `enabled`, `fire` as pure functions over `&Model` and `&Config`. `fire` returns a new `Config` and one `Step`, or an `Outcome`. No `&mut Config` in any public signature. | Configurations as values make snapshots, stepping backwards, DFS backtracking and equality "however reached" (spec §8.1) free. | Stateright `Model` (SR lib.rs L158-L179); PEx copy and restore cost (StepState.java) | Reviewer; a public-API snapshot test (`cargo public-api` or an insta snapshot of `rustdoc` JSON) |
| G2 | `Config` derives `Clone, Eq, Ord, Hash, Serialize, Deserialize` and contains only `Vec`, `BTreeMap`, `BTreeSet`, `Arc<T>` and plain values. Per-instance and per-activation parts sit behind `Arc` and change through `Arc::make_mut`. | Cheap clone-on-step with deterministic iteration; no undo logs. | Stateright `Vec<Arc<A::State>>` + `Cow` (SR model_state.rs L15-L23; model.rs L402-L450) | Clippy `disallowed-types` (HashMap, HashSet, RefCell, Cell in `mentalvm` core); a proptest that `serde` round-trip of a `Config` is identity |
| G3 | The step function contains no pruning. Every legal choice appears in `enabled` and every enabled choice fires. Reductions (visited set, POR, preemption bound, symmetry) live only in strategies and each can be switched off. | Hidden pruning in the step changes semantics silently. | Stateright's no-op `None` with its own caveat (SR model.rs L418-L440); PEx create-first rule (ExplicitSearchScheduler.java L552-L568) | Proptest: for every generated config, every `c in enabled(cfg)` gives `Fired::Next` or `Fired::End`, never a panic. Reviewer. |
| G4 | Every run outcome (model gap, deadlock, illegal write, zero-time cycle, invariant violated, normal end) is a value of `enum Outcome` returned by `fire`/`run`. `panic!`/`unwrap` is reserved for internal invariants. | Outcomes are data to report, minimize and replay. Unlike Shuttle and P, Mental runs no real code that throws. | P unhandled-event assert (StateMachine.cs L1043-L1070); Shuttle panic-based failure (replay.rs L82, L122) | Clippy `unwrap_used`, `expect_used`, `panic` = deny in core (allow with a comment for invariants); one corpus model per outcome kind |

**Choices, schedules and replay**

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G5 | All nondeterminism is a `Choice` enum (`Sched`, `Env`, `Data`, `Pick`) named by semantic IDs and carrying its value. A schedule is `Vec<Choice>`: never action indices, never "regenerate from seed". | Replay on a changed model must match by name; stored values survive model edits. | Stateright index paths break on change (path.rs L21-L90); Shuttle `Random` steps depend on the seed (scheduler/mod.rs); P stores values (ScheduleStep.cs) | Changed-model replay test (report-jobs pre- and post-fix); reviewer |
| G6 | `enabled` returns choices sorted by `Choice: Ord`, deduplicated. | Bit-for-bit reproducibility. Stateright had to add sorting after `HashableHashSet` order broke path rebuilding. | SR model.rs L342-L386; path.rs L28-L75 | A `debug_assert!(is_sorted)` in `enabled`; determinism test (G14) |
| G7 | Strategies are a single `Chooser` trait with one method (`choose(&Config, &[Choice]) -> Option<Choice>`) driving one `run` loop. `explore` is a separate function that owns its DFS stack. There are no separate bool, int or schedule choice methods. | One choice point, one stack. P's three parallel DFS stacks show the cost of splitting. | Shuttle `Scheduler` (scheduler/mod.rs L75-L100); Stateright `Chooser` (simulation.rs L22-L39); P DFSStrategy.cs L27-L49 | Reviewer; grep in CI that no other code path calls an RNG |
| G8 | `replay` gives a verdict per choice: `matched`; `infeasible` (the named choice is not in `enabled`); `diverged` (the kind or position no longer lines up, or the run ends before or after the schedule). An optional continuation strategy takes over after the first non-match. | Mental §8.9. P's three named "not reproducible" cases map onto it. Replay-then-continue is P's suffix strategy. | P ReplayStrategy.cs L93-L107 and suffix fallback; Shuttle `allow_incomplete` (replay.rs) | Unit tests, one per verdict, in `replay/tests.rs`; insta snapshot of the report-jobs post-fix replay |

**Search**

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G9 | v1 bounded-exhaustive search is a single-threaded, stateful DFS over cloned configs with an **exact** visited set (full `Config`, no fingerprint). The visited key includes the remaining bound budgets (dups, preemptions, depth), or else the search keeps the best budget per config. | Sound by construction, deterministic, no collision analysis, small code. Budgets in the key avoid unsound pruning. | PEx `Exact` mode (StateCachingMode.java); Stateright dedups by state while depth lives in the job (dfs.rs L31, L329) | Corpus counts pinned with insta; a test with a model where pruning without budgets would miss a violation |
| G10 | A preemption is a `Sched` choice of another activation while the current one can still step. The default (first-tried) choice is the non-preempting one. | Matches Mental's bounds language ("1 preemption") and gives shortest-preemption witnesses first. | loom execution.rs L163-L200, path.rs Schedule.preemptions | Unit tests on the counter; report-jobs witness must show exactly 1 preemption |
| G11 | Zero-time cycle = a config (with `now`) revisited on the current DFS path. A config revisited off the current path is pruned. | One mechanism gives both pruning and cycle detection. | PEx ExplicitSearchScheduler.java L249-L290 | Corpus model `zero-time-cycle` |
| G12 | Fingerprints, if introduced, hash a canonical serialization with an algorithm whose output is specified and version-pinned (not ahash, not `DefaultHasher`). Keep ≥64 bits, report a collision bound as TLC does, and keep exact mode as a test oracle. | Portable across native and wasm and across versions. ahash says its output differs by machine. | SR lib.rs L371-L390; ahash README L22; TLC FP64 and ModelChecker.java L962-L972; PEx ComputeHash XOR (avoid) | Differential test exact vs fingerprint on the whole corpus; wasm test of identical fingerprints (wasm-bindgen-test or wasmtime) |
| G13 | Sampled search uses one named PRNG (PCG or SplitMix64) seeded from the run file. The core never reads entropy; the CLI prints the seed it picked. | Reproducibility. Stateright carries a FIXME about `StdRng` instability. | SR simulation.rs L46-L48; SH pct.rs L50 | Clippy `disallowed-methods` (`rand::thread_rng`, `OsRng`, `getrandom`) in core; a test that the same seed gives byte-identical reports |

**Determinism, minimization and reduction**

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G14 | Determinism is tested, not assumed: every strategy is run twice in-process on the corpus with byte-identical serialized output, and every witness from `explore` replays to all-`matched` with the same outcome. | All three runtime projects ship a nondeterminism detector, because it bites. | Shuttle UncontrolledNondeterminismCheckScheduler; Stateright path-rebuild panic; loom "Is the model fully deterministic?" | Integration test in `tests/determinism.rs`, CI-gating |
| G15 | The witness minimizer is a delete-one-choice fixpoint over the schedule. Each candidate is re-checked by replay, where skipped choices fall back to the default non-preempting choice. It keeps candidates with the same outcome kind at the same node and returns the best found (proptest bail-out rule). Search order (fewest preemptions first) supplies a near-minimal witness to start from. | No precedent shrinker exists in the checkers; proptest's `VecValueTree` delete phase is the simplest proven loop. BFS shortest-first is the cheap half. | PT collection.rs L591-L650, runner.rs L769-L880; Stateright BFS shortest doc; TLC trace rebuilding | Proptest: output fails, is no longer than input, and is a fixpoint. Insta snapshot of the report-jobs minimal witness. |
| G16 | POR is not in v1 unless report-jobs bounds turn out too slow. If added, it is sleep sets over static footprints and ships only with a differential test: same set of `(outcome kind, node)` with POR on and off on every corpus model plus proptest-generated tiny models, with both state counts pinned. | Only loom has DPOR, and it is coupled to its runtime. No project tests POR against full search, so Mental must. | loom execution.rs L134-L160, path.rs L336-L385, L493-L520; Stateright 2pc 8,832 vs 665 (examples/2pc.rs L183-L198); loom litmus outcome sets (tests/spec.rs) | CI differential test; cargo-mutants on the independence function |

**Testing and scope**

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G17 | The corpus is laid out like P's: `tests/corpus/{bug,correct}/<name>/` with model, bounds, and an insta snapshot of outcome, minimal witness and counts. Every outcome kind has at least one bug model; every bug fixed in `mentalvm` adds one. | Known-bug corpora with golden counts are how P, TLC and Stateright test themselves. | P Tst/RegressionTests (145 DynamicError); TLC TLC_STATS asserts in 282 test files; Stateright `unique_state_count` pins | CI; a reviewer checklist item "bug fix adds a corpus case" |
| G18 | Tests check witnesses by replay validity (`assert_discovery` style) as well as by snapshot, so a legitimate change in search order shows up as a reviewed snapshot diff, not a false failure of correctness. | Separates "a violation exists and this witness proves it" from "the search found this exact one". | SR checker.rs L534-L560 | Test helper `assert_witness(model, schedule, outcome)` |
| G19 | Replay round-trip tests (`explore` → serialize run file → parse → `replay` → same outcome, all `matched`) are CI-gating and may not be `#[ignore]`d. | Shuttle's round-trip tests are disabled today ("replay mechanism is broken"), which shows replay rots silently. | SH tests/basic/replay.rs L32-L51 | CI grep forbidding `#[ignore]` under `crates/mentalvm/` |
| G20 | v1 scope: concrete `run`, bounded-exhaustive `explore`, `replay`, minimizer; sampled search only if it is a seeded `Chooser` under about 50 lines. No threads, no PCT, no symmetry, no fingerprint modes, no web explorer, no checkpoint files. | "Compact code, no bloat"; the listed features are where the studied codebases grew. | SR checker.rs / Cargo.toml; SH shuttle-schedulers; P Strategies (about 3,770 lines); loom `Load`/`checkpoint`; TLC fp/distributed | Reviewer; `cargo tree` dependency allowlist check in CI |
| G21 | The core must build for `wasm32-unknown-unknown` with no `std::fs`, `std::time`, `std::thread` or `std::env`. Bounds replace time limits; the CLI owns files, clocks and the env-var seed. | Spec §8 (no I/O, WebAssembly). Stateright, Shuttle and loom read env vars and wall clocks in their checkers (`timeout`, `LOOM_MAX_*`, `SHUTTLE_RANDOM_SEED`). | SR checker.rs `timeout`; LO model.rs L39-L120; SH pct.rs `seed_from_env` | CI job `cargo build -p mentalvm --target wasm32-unknown-unknown`; clippy `disallowed-methods` for `std::env::var`, `Instant::now`, `SystemTime::now` |

### Gaps
- None of the projects carries a path condition, symbolic forking or lifecycle-machine checks on writes. Those parts of `mentalvm` have no code precedent here; the related precedents are the semantics report's (Coyote, Rebeca, BIP), which this note did not re-read in code.
- The exact interplay of sleep sets with a budgeted visited set (G9 + G16) needs a design note before implementation; no studied code shows it.
- I did not read the P C# `PCTStrategy` or TLC's liveness checking in detail; neither is needed for v1.
