# mentalc: the LLM loop (task, backend, answer, check, repair), the call ledger, and the MCP server

Research date 2026-11-14. Every repository was shallow-cloned and read, not built. Commits:

| Repo | Commit | What it is |
|---|---|---|
| modelcontextprotocol/rust-sdk (rmcp 3.10.0) | `c644b75a` | the official Rust MCP SDK: 59,664 lines in `crates/rmcp/src` plus 1,841 in `rmcp-macros`, built on tokio |
| modelcontextprotocol/modelcontextprotocol | `0175d20f` | the MCP specification, revisions 2024-11-05 through 2026-07-28 |
| openai/codex (`codex-rs`) | `e9919f4b` | OpenAI's coding agent in Rust: about 2.36M lines in total (**bloat**: a whole platform) |
| Aider-AI/aider | `5dc9490b` | Python coding agent: edit formats, a "reflection" repair loop, LiteLLM for model calls |
| specula-org/Specula | `1a86e7e5` | Python: coding agents write TLA+ specs, then checkers and a trace validator drive repair. About 88k lines of Python (**bloat**) |
| specula-org/SysMoBench | `827899e4` | Python benchmark for LLM-written TLA+, with a syntax-feedback correction loop and an MCP `submit_spec` tool |
| GREsau/schemars (1.2.3) | `4791bb73` | derives JSON Schema from Rust types: 8,237 lines |
| mochi-neko/clust | `81fda7c6` | unofficial async Rust client for the Anthropic API: 8,550 lines, last commit 2025-03-02 |

URL prefixes used below:

- RMCP = `https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca`
- SPEC = `https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e`
- CODEX = `https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs`
- AIDER = `https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c`
- SPECULA = `https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65`
- SMB = `https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9`
- SCH = `https://github.com/GREsau/schemars/blob/4791bb7352555d339e45658ca09c092ed9fa1f5d`
- CLUST = `https://github.com/mochi-neko/clust/blob/81fda7c644a6e01971c378e53f4d0afdb873a9b9`

The dependency counts come from `cargo tree -e normal`, run on 2026-11-14 against crates.io in a separate probe project in the scratchpad, never inside a clone. Each count is the number of unique crates and includes the probe crate itself.

Facts about the Anthropic API come from the Anthropic `claude-api` reference bundled with Claude Code (version 2.1.322: `shared/tool-use-concepts.md` § Structured Outputs, `shared/error-codes.md`, `curl/examples.md`). Their public counterpart is <https://platform.claude.com/docs> (not fetched).

---

## 1. What is the smallest correct design for Task → Backend → Answer → Check → Repair with deterministic tests?

### Takeaway

Every tool studied puts the same small loop around the model. The prompt is rendered from data. The model answers, a deterministic checker validates the answer, and a failure becomes a message naming exactly what failed. A hard cap ends the loop: 3 in Aider and SysMoBench, 5 in Specula's stop gate.

For mentalc, the smallest design that serves both the push backends (`api`, `local`, `agent-cli`) and the pull backend (`agent`) has two parts:

- a **resumable build state machine on disk**, with two operations: `next_task()` and `accept(task_id, raw_answer) -> Verdict`;
- a **synchronous** `Backend::complete(&RenderedTask) -> RawAnswer` that only the push backends implement.

The `async fn answer` in spec §6.5 is neither needed nor consistent with "no async in v1".

### Cited Findings

#### Aider (repair loop and edit-format validation)

- Aider's loop is minimal. `max_reflections = 3`. `run_one` sends the message, and if a checker has set `self.reflected_message`, sends that as the next user message. After 3 rounds it warns "Only 3 reflections allowed, stopping." — [AIDER/aider/coders/base_coder.py L100-101, L932-944](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/base_coder.py#L932-L944)
- A malformed answer is any `ValueError` raised while parsing or applying edits. Aider counts it (`num_malformed_responses += 1`) and reflects the error text back verbatim. A dry-run applies every edit before anything is written: `get_edits` → `apply_edits_dry_run` → `prepare_to_edit` → `apply_edits` — [AIDER/aider/coders/base_coder.py L2295-2315](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/base_coder.py#L2295-L2315)
- Aider's repair message is precise, and the model for "a repair task naming exactly what failed". It gives:
  - the count of failed blocks;
  - each failed SEARCH block, verbatim, with the file it was meant for;
  - "Did you mean to match some of these actual lines", from `find_similar_lines`;
  - "Are you sure you need this SEARCH/REPLACE block? The REPLACE lines are already in {path}";
  - the rule that was broken;
  - "The other N blocks were applied successfully. Don't re-send them." — [AIDER/aider/coders/editblock_coder.py L41-123](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/editblock_coder.py#L41-L123)
- Lint and test failures go through the same channel: `self.reflected_message = lint_errors` / `test_errors` — [AIDER/aider/coders/base_coder.py L1596-1622](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders/base_coder.py#L1596-L1622)
- Aider's prompts are Python classes, one `*_prompts.py` per edit format (`editblock_prompts.py`, `wholefile`…), not versioned data files — [AIDER/aider/coders/](https://github.com/Aider-AI/aider/tree/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders)

#### SysMoBench (checker-driven correction)

- `DirectCallMethod` generates a spec, runs the language backend's syntax validator (SANY), and on failure feeds the errors back, for up to `max_correction_attempts = 3` rounds — [SMB/tla_eval/methods/direct_call/method.py L1-35](https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tla_eval/methods/direct_call/method.py)
- The agent variant puts the checker behind an MCP tool. `submit_spec(spec_content, config_content)` validates in two phases (SANY syntax, then a TLC run) and returns a JSON result with `remaining_attempts`. After `SYSMOBENCH_MAX_ATTEMPTS` (default 3) it refuses. The attempt counter lives **in server memory** behind an `asyncio.Lock` — [SMB/tools/submit_spec/src/submit_spec/server.py L35, L60-215](https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tools/submit_spec/src/submit_spec/server.py)
- Errors (for example `{"error": "spec_content is required"}`) come back as ordinary `TextContent`, not with `isError: true` — same file, `_handle_submit_spec`.

#### Specula (agent pipeline with a gated stop)

- Specula enforces one contract in two layers:
  - an *execution layer*: an agent Stop hook blocks the agent from ending its turn and returns corrective instructions;
  - an *acceptance layer*: the launcher re-checks the same contract from outside after the agent exits, so "a run that slipped through is loudly FAILED".

  A fuse (`SPECULA_STOP_GATE_CAP`, default 5) bounds the blocks per run and then writes `FAILED-HOOK-CAP`. "The contract forces an ACCOUNT, never an outcome": an agent that cannot finish writes `BLOCKED.md` — [SPECULA/src/specula/stop_gate.py L1-48](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/stop_gate.py)
- Specula's gate is explicitly **fail-open**: "an internal exception, malformed hook input, or missing workspace allows the stop". This is the opposite of Mental's fail-closed rule — same file, L41-45.
- The output contract lists each required file together with the code that consumes it. Each file must be non-empty, and each TLA+ module must parse with SANY before the handoff is accepted — [SPECULA/src/specula/output_contract.py L1-40](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/output_contract.py)
- Backends are adapters over agent CLIs: `adapters/claude_code.py` (821 lines), `opencode.py` (235), `pi.py` (93) — [SPECULA/src/specula/adapters/](https://github.com/specula-org/Specula/tree/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/adapters)

#### Codex (model calls with a schema-constrained final answer)

- `codex exec --output-schema <file>` sends the schema as `text.format = {type: json_schema, strict: true, name: "codex_output_schema"}`. Enforcement is left to the provider's constrained decoding, and Codex does no local re-validation on this path — [CODEX/codex-api/src/common.rs L443-461](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/codex-api/src/common.rs#L443-L461); [CODEX/core/src/client_common.rs L56](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/src/client_common.rs#L56)

#### Anthropic API constraints relevant to the loop

- Structured outputs (`output_config.format = {type: "json_schema", schema}`) constrain the reply. Supported: object, array, string, integer, number, boolean, null, `enum`, `const`, `anyOf`, `allOf`, `$ref`/`$defs`, a few string formats. Required: `additionalProperties: false` on every object. Not supported: **recursive schemas**, numeric constraints (`minimum`/`maximum`/`multipleOf`), string length constraints, complex array constraints. Even under the schema, the output may not match it when `stop_reason` is `refusal`, and may be incomplete when it is `max_tokens` — Anthropic claude-api reference, `shared/tool-use-concepts.md` § Structured Outputs.
- On API version `2026-10-21`, sending `temperature`, `top_p` or `top_k` returns a 400 on every model — claude-api reference, Thinking & Effort section. Determinism for tests therefore cannot come from `temperature: 0`; it has to come from recorded answers.

### Inferences

- **The pull backend inverts control, so the core must be a state machine, not a function call.** With `agent`, `mentalc` is called (by `task next` / `task submit`) instead of calling out, so `Backend::answer(task) -> Answer` cannot be the core abstraction. The smallest design that fits both modes:
  - `Build` (on disk under `.mental/`) has `next_task() -> Option<Task>` and `accept(task_id, raw: &str) -> Verdict`, where `Verdict = Accepted | Repair(Task) | Failed(Error)`.
  - The `api`, `local` and `agent-cli` backends are a loop: `while let Some(t) = build.next_task() { let raw = backend.complete(&render(&t))?; build.accept(t.id, &raw)?; }`.
  - The `agent` backend exposes the same two calls as CLI subcommands and MCP tools.
  - All checking, repair, caching and ledger writes live in `accept`, so every backend is judged identically (spec §6.5 already requires that).
- **Push backends need only one synchronous method:** `fn complete(&self, task: &RenderedTask) -> Result<RawAnswer, BackendError>` plus `fn caps(&self)`. Make it sync. Blocking HTTP is enough for 15 calls per build (F-S10). Spec §6.5's `async fn answer` should change to match the "no async in v1" decision.
- **Repair is just another Task** (role `repair`). It carries the original inputs, the previous raw answer, and a structured failure list: check name, record ID, rule, evidence such as an anchor quote not found plus the closest actual lines. That keeps it self-contained (spec §6.5 rule), and it follows Aider's best practice of quoting the failure, showing the nearest real text, and saying what passed.
- **Ask for the whole corrected answer under the same schema, not a patch.** Aider asks only for the failed blocks because its edits are independent. Mental's checks (coverage, guard algebra) are global over the answer, so a full re-answer keeps checking uniform. This is an inference; measure repair success rate both ways if cost matters.
- **Cap repairs per task in config (default 3, matching Aider and SysMoBench).** On exhaustion, fail the build with every failure attached. Never fail open (unlike Specula's stop gate).
- **Treat `stop_reason` of `refusal` and `max_tokens` as backend errors, not repair triggers.** The schema guarantee does not hold for them, and repairing a refusal wastes calls.
- **Answer schemas must stay inside the Anthropic subset:** non-recursive, `additionalProperties: false` everywhere, no numeric or length constraints. Guards and expressions must therefore be flat (for example a string in a tiny grammar that mentalc parses, or a list of literals), and every constraint the schema cannot express becomes a mentalc check.

### Gaps

- No public source code was found for Lemur (ICLR 2024, LLM invariant synthesis with a verifier loop) or for any "Event-B agent". Only the Lemur paper exists ([arXiv 2310.04870](https://arxiv.org/abs/2310.04870)). The `gh search repos` queries for Lemur, Loopy and Event-B LLM returned nothing.
- No source compares repair-success rates for "send the whole answer again" against "send only the failed parts".
- It is unknown whether Anthropic strict structured outputs require every property to be listed in `required`, which matters for `Option<T>` fields. The reference names `required` only for strict tool use.

---

## 2. How do these tools record and replay LLM interactions for tests and for audit (call ledger)?

### Takeaway

Nobody studied uses VCR-style recorded cassettes of real model traffic. Tests rely on three tools:

- **scripted fakes**: a mock HTTP server with hand-built events (Codex), a monkey-patched `send` (Aider), a deterministic fixture adapter plus a fake checker (Specula), an "existing spec" adapter (SysMoBench);
- **golden files and snapshots** of what was sent to the model (Codex `insta` context snapshots, Aider gold fixtures);
- **a handful of real recorded responses kept for edge cases** (Specula's recorded Claude refusal).

For audit, every agent writes an append-only JSONL or text log: Codex rollouts, Aider's `--llm-history-file`, Specula's `gate.log`, SysMoBench's per-attempt summary.

### Cited Findings

- **Codex, a mock HTTP server with builder-made SSE events.** `core/tests/common/responses.rs` wraps `wiremock::MockServer`. It provides `sse(events)`, `ev_response_created`, `ev_assistant_message(id, text)`, `ev_completed_with_tokens`… to build replies, and a `ResponsesRequest` wrapper with assertions on what the client *sent* (`instructions_text()`, `message_input_texts(role)`, `function_call_output(call_id)`, `has_function_call`) — [CODEX/core/tests/common/responses.rs L52-458, L798-911](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/tests/common/responses.rs)
- **Codex, snapshots of the request context with normalization.** `context_snapshot/normalize.rs` replaces volatile values (working directories, temp dirs, prompt cache keys, UUIDs) with stable labels, so `insta::assert_snapshot!` on the rendered request stays deterministic. 70 files in `core/tests/suite` use insta — [CODEX/core/tests/common/context_snapshot/normalize.rs](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/tests/common/context_snapshot/normalize.rs)
- **Codex keeps tests beside modules** as `*_tests.rs` files (`client_tests.rs`, `compact_tests.rs`, `retry`…), matching Mental's `tests.rs`-beside-module decision — [CODEX/core/src/](https://github.com/openai/codex/tree/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/core/src)
- **Codex audit log.** The rollout recorder "Persist[s] Codex session rollouts (.jsonl) so sessions can be replayed or inspected later", one file per session under `~/.codex/sessions/rollout-<timestamp>-<id>.jsonl` — [CODEX/rollout/src/recorder.rs L1, L79-99](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/rollout/src/recorder.rs)
- **Aider, a fake `send`.** Tests replace `coder.send` with a function that sets `coder.partial_response_content` to a canned LLM reply. Then they run `coder.run(with_message="hi")` and assert on the files on disk — [AIDER/tests/basic/test_editblock.py L323-361](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/tests/basic/test_editblock.py#L323-L361)
- **Aider, a gold-file test of the parser.** `tests/fixtures/chat-history.md` is a recorded transcript. Its parse must equal `chat-history-search-replace-gold.txt`, and a unified diff is shown on mismatch — [AIDER/tests/basic/test_find_or_blocks.py L76-99](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/tests/basic/test_find_or_blocks.py#L76-L99)
- **Aider audit and metrics.** `--llm-history-file` appends every message sent and received — [AIDER/aider/io.py L754](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/io.py#L754). The benchmark aggregates `num_malformed_responses`, `syntax_errors`, `indentation_errors` and `exhausted_context_windows` per run — [AIDER/benchmark/benchmark.py L490-526](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/benchmark/benchmark.py#L490-L526)
- **Aider's request "hash" is not a cache key.** It is SHA-1 over the kwargs computed *before* `messages` are added, used only as an identifier — [AIDER/aider/models.py L1015-1023](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/models.py#L1015-L1023)
- **Specula, a deterministic fixture adapter that replaces the agent.** "Real dispatch, never an LLM". Behavior switches are files next to the script: `repair`, `sany-error` (writes a module the fake SANY rejects), `blocked`, `fail-main-N`, `fix-on-rejection` and others. A `fake_sany.py` stands in for the checker — [SPECULA/tests/e2e/fixtures/single_session_adapter.py L1-30](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/e2e/fixtures/single_session_adapter.py)
- **Specula, one real recorded response kept as a fixture:** `tests/unit/fixtures/claude-cyber-refusal.result.json`, used by `test_adapters.py`, `test_confirmlib.py` and `test_policy.py` — [SPECULA/tests/unit/fixtures/](https://github.com/specula-org/Specula/tree/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/unit/fixtures)
- **Specula audit trail.** Gate state lives under `.stop-gate/` (block counter, `FAILED-HOOK-CAP` marker, `gate.log`) — [SPECULA/src/specula/stop_gate.py L36-39](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/stop_gate.py)
- **SysMoBench, a no-LLM adapter.** `ExistSpecAdapter` ("doesn't actually call any LLM") serves existing specs through the same `ModelAdapter` interface. That makes it a replay backend — [SMB/tla_eval/models/exist_spec_adapter.py](https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tla_eval/models/exist_spec_adapter.py). Each submission's result is appended and `save_summary` is written after every attempt — [SMB/tools/submit_spec/src/submit_spec/server.py](https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tools/submit_spec/src/submit_spec/server.py)

### Inferences

- **For mentalc, the ledger, the cache and the test fixtures can be one mechanism:**
  - `calls.jsonl` holds one line per call (spec §6.6), including failed calls and repairs (`repair_of`).
  - The raw answer is stored by content hash (for example `.mental/answers/<h>.json`) rather than inline, so the ledger stays small and diffable.
  - A **replay backend** (`Backend` impl, test-only or `--backend replay`) answers a task by looking up its cache key in a directory of recorded answers. It fails loudly when a key is missing, in line with fail-rather-than-degrade.
  - Recording is then just running once with `api` or `agent` and copying the answers directory into `tests/fixtures/<repo>/answers/`.
- **Golden tasks: snapshot the rendered task with insta** (prompt text, input anchors and hashes, answer schema) for the report-jobs fixture, with volatile fields normalized the way Codex does. A diff in a golden task is exactly the signal that a cache key changed.
- **Fault-injected and invalid answers are fixtures too.** Keep hand-written bad answers (bad anchor, missing obligation, overlapping guards, unknown field, invented ID) next to the good ones, so every repair path is covered without a model. This is Specula's switch-file approach, expressed as data.
- **Do not rely on wiremock-style HTTP mocks for the core.** Mental's `Backend` trait boundary makes an in-process fake simpler. Keep HTTP-level tests only for the `api` client's request and response mapping: serialize the request and snapshot it, deserialize recorded response bodies, and test the retry classification as a pure function.

### Gaps

- None of the tools studied records model, prompt version and input hash per call the way Mental's ledger does. Codex rollouts are session transcripts, not per-call provenance records. So no external precedent exists for "rebuild the parts produced by LLM X".
- Codex pairs wiremock with tokio. Whether a blocking `ureq` client can be tested against a tiny `std::net::TcpListener` stub without new dev-dependencies was not checked in code. It is likely, but unverified.

---

## 3. rmcp vs hand-written stdio JSON-RPC: lines of code, dependency weight, async requirement, spec-compliance risk

### Takeaway

**Write a small synchronous JSON-RPC-over-stdio server.** rmcp works and tracks the spec, but:

- it makes tokio a mandatory dependency (the `tokio` dependency is not optional), and its `server` feature pulls in schemars and uuid;
- it resolves to 61 crates for the minimal stdio server;
- it is 61.5k lines of source;
- it shipped three major versions (1.0.0 → 3.0.0) between 2026-03-03 and 2026-07-28;

all for 3-4 tools. The real risk of writing it by hand is the spec churn of 2026-07-28, which removed the `initialize` handshake. A dual-era server answers both `initialize` (2025-11-25 clients) and `server/discover` (2026-07-28 clients), and I estimate it at roughly 250-400 lines. A fully synchronous implementation is possible: stdio is one newline-delimited channel, and 2026-07-28 forbids servers from sending requests anyway.

### Cited Findings

- **rmcp size:** `crates/rmcp/src` is 59,664 lines (largest files: `transport/auth.rs` 10,742, `model.rs` 6,285, `service/client.rs` 2,942); `rmcp-macros` is 1,841 — counted from the clone at [RMCP/crates/rmcp/src](https://github.com/modelcontextprotocol/rust-sdk/tree/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/src). Tests are 33,648 lines.
- **rmcp dependencies:** `tokio = { version = "1", features = ["sync","macros","rt","time"] }` is a non-optional dependency, alongside `futures`, `tokio-util`, `tracing`, `indexmap`, `sha2`, `chrono` and others. The `server` feature is `["transport-async-rw", "schemars", "dep:pastey", "uuid"]`, and the defaults are `["base64","macros","server"]` — [RMCP/crates/rmcp/Cargo.toml](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/Cargo.toml)
- **Measured dependency weight** (`cargo tree`, crates.io, 2026-11-14):

  | Selection | Unique crates |
  |---|---|
  | `rmcp 3.10` with `default-features=false, features=["server","transport-io"]` (includes tokio, mio, socket2, futures-*, schemars, chrono, uuid, sha2, tracing, two versions of syn) | 61 |
  | rmcp with default features + `transport-io` | 68 |
  | `serde` + `serde_json` + `thiserror` alone | 14 |

  — scratchpad dependency probe.
- **rmcp's stdio transport is tokio stdin/stdout.** It runs on Tokio's blocking pool, and "A pending read on tokio::io::Stdin cannot be cancelled … can therefore still leave Tokio runtime shutdown waiting for stdin input or EOF" — [RMCP/crates/rmcp/src/transport/io.rs L1-25](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/src/transport/io.rs)
- **rmcp's tool definition is compact.** `#[tool_router(server_handler)] impl Calculator { #[tool(description=…)] fn sum(&self, Parameters(SumRequest{a,b}): Parameters<SumRequest>) -> String }`, where `SumRequest: Deserialize + JsonSchema`. Serving is `Calculator.serve(stdio()).await?.waiting().await?` under `#[tokio::main]`. Tool handlers may be plain sync `fn` — [RMCP/examples/servers/src/common/calculator.rs](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/examples/servers/src/common/calculator.rs); [RMCP/examples/servers/src/calculator_stdio.rs](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/examples/servers/src/calculator_stdio.rs)
- **rmcp input schemas** come from schemars `SchemaSettings::draft2020_12()`, cached per type — [RMCP/crates/rmcp/src/handler/server/common.rs L15-42](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/src/handler/server/common.rs)
- **rmcp supports both eras:** `V_2026_07_28` (LATEST, `NO_INITIALIZE`) and `LATEST_WITH_INITIALIZE = V_2025_11_25`, with discover handling in `service/server.rs` — [RMCP/crates/rmcp/src/model.rs L169-201](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/src/model.rs#L169-L201)
- **rmcp API churn:** 1.0.0 on 2026-03-03, 2.0.0 on 2026-06-27, 3.0.0 on 2026-07-28, 3.10.0 on 2026-11-12 — [RMCP/crates/rmcp/CHANGELOG.md](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/CHANGELOG.md). Codex pins rmcp to a git revision, `=3.5.1` — [CODEX/Cargo.toml L478](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/Cargo.toml#L478)
- **rmcp quality signal:** a 2026-02-25 self-audit rated rmcp Tier 3, with server conformance at 83.3% (25/30) — [RMCP/conformance/results/2026-02-25-rust-sdk-assessment.md](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/conformance/results/2026-02-25-rust-sdk-assessment.md). CI now runs `@modelcontextprotocol/conformance` server suites for both 2025-11-25 and 2026-07-28, **over HTTP** (`--url http://127.0.0.1:8001/mcp`) — [RMCP/.github/workflows/conformance.yml L53-67](https://github.com/modelcontextprotocol/rust-sdk/blob/c644b75aafa0a437e5d814b5672d82ed6b4899ca/.github/workflows/conformance.yml)
- **MCP 2026-07-28 made the protocol stateless:**
  - it removed `initialize`/`notifications/initialized` and `ping`;
  - every request carries `_meta["io.modelcontextprotocol/protocolVersion"]` and client capabilities;
  - servers MUST implement `server/discover`;
  - all results carry `resultType` (`"complete"`);
  - `tools/list` results require `ttlMs` and `cacheScope`;
  - Sampling, Roots and Logging are deprecated ("integrate directly with LLM provider APIs instead of Sampling").

  — [SPEC/docs/specification/2026-07-28/changelog.mdx](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/changelog.mdx)
- **The stdio transport rules:** one JSON-RPC message per line with no embedded newlines; stdout carries only MCP messages; stderr is free for logs; the server MUST NOT write requests to stdout; it SHOULD exit when stdin reaches EOF; cancellation is `notifications/cancelled`, after which the server MUST NOT send further messages for that request — [SPEC/docs/specification/2026-07-28/basic/transports/stdio.mdx](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/basic/transports/stdio.mdx)
- **Backward compatibility:** a dual-era client probes with `server/discover` and falls back to `initialize` on any non-modern error. A server MAY implement both. A "Legacy client / Dual-era server" pair works because "the server answers `initialize` and serves the client according to the negotiated legacy revision" — [SPEC/docs/specification/2026-07-28/basic/versioning.mdx L126-171](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/basic/versioning.mdx)
- **The legacy minimum (2025-11-25):**
  - `initialize` MUST be first;
  - the server echoes the client's protocol version when it supports it, otherwise answers with its latest;
  - the client then sends `notifications/initialized`;
  - `ping` MUST get a prompt empty response.

  — [SPEC/docs/specification/2025-11-25/basic/lifecycle.mdx L40-172](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2025-11-25/basic/lifecycle.mdx); [SPEC/docs/specification/2025-11-25/basic/utilities/ping.mdx L29](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2025-11-25/basic/utilities/ping.mdx)
- **Tool rules:**
  - `tools/list` "MUST NOT vary per-connection or as a side effect of other requests" and SHOULD be deterministic in order;
  - state across calls goes in "explicit, server-minted handles passed as ordinary tool arguments", and an unknown or expired handle should return a tool execution error;
  - input validation and business errors use `isError: true` in a normal result ("Clients SHOULD provide tool execution errors to language models to enable self-correction");
  - unknown tools and malformed requests use JSON-RPC errors (for example `-32602`).

  — [SPEC/docs/specification/2026-07-28/server/tools.mdx L60-66, L690-785](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/server/tools.mdx)
- **Specula and SysMoBench use Python's `FastMCP` / `mcp.server.Server`.** Specula's `tools/tlc_tools/mcp_server.py` is 18 lines — [SPECULA/tools/tlc_tools/mcp_server.py](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tools/tlc_tools/mcp_server.py)

### Inferences

- **A synchronous server is possible and simpler.** The loop: `for line in stdin.lock().lines() { let msg = parse(line); if let Some(resp) = handle(&mut build, msg) { writeln!(out, "{}", resp)?; out.flush()?; } }`, returning on EOF. Processing is sequential, so a `notifications/cancelled` can only be read after the response for that request has already been written. That satisfies "MUST NOT send further messages" trivially. Make the server generic over `BufRead` and `Write`, so tests feed it a transcript in memory.
- **Methods to implement (dual era):**
  - `initialize`: reply `{protocolVersion, capabilities: {tools: {}}, serverInfo, instructions}`;
  - `notifications/initialized`: ignore;
  - `ping`: reply `{}`;
  - `server/discover`: reply `{resultType, supportedVersions: ["2026-07-28"], capabilities, _meta.serverInfo, ttlMs, cacheScope}`;
  - `tools/list`: add `resultType`, `ttlMs` and `cacheScope` when the request carries the 2026 `_meta` version;
  - `tools/call`;
  - any other method gets `-32601`; malformed JSON gets `-32700`; bad params get `-32602`.

  Estimate: 250-400 lines plus tests. This figure is mine, not measured.
- **The tool surface follows from "tools/list must not vary":**
  - `mental_task_next()` returns the task, its `task_id` handle and its answer schema in the result text and `structuredContent`;
  - `mental_task_submit(task_id: string, answer: object)`;
  - optionally `mental_build_status()`.

  The answer schema cannot be the submit tool's per-task `inputSchema`, because the tool list must stay fixed. So the schema travels inside the task, and mentalc validates on submit. A failed check returns `isError: true` with the repair text, so the agent self-corrects in the same session. An unknown or stale `task_id` returns `isError: true` saying so.
- **The handle model matches Mental's design.** State lives on disk and is keyed by `task_id`. That is exactly what the stateless 2026 spec asks for, unlike SysMoBench's in-memory attempt counter, which a server restart loses.
- **The dependency argument is decisive.** A hand-written server adds 0 crates (serde_json is already needed). rmcp adds about 47 crates and an async runtime, used only by `mentalc mcp`. Wrapping rmcp in `Runtime::block_on` would confine async to one module, but would still bring the whole tree in.
- **How to contain the compliance risk:**
  1. Keep all MCP code in one module with version constants.
  2. Add insta transcript tests for each era built from the spec's own JSON examples: initialize → tools/list → tools/call, discover → tools/call with `_meta`, unknown method, malformed line, EOF exit.
  3. Run a manual smoke test with Claude Code and Codex at each release.
  4. Revisit the choice if MCP grows features mentalc must support (none planned: no sampling, no subscriptions, no tasks extension).

### Gaps

- It is not known which MCP revision Claude Code and Codex send in November 2026 (probe with `server/discover`, or `initialize` only). The dual-era design hedges this, but it was not verified against the clients.
- Whether `@modelcontextprotocol/conformance` can drive a **stdio** server was not determined. rmcp runs it over HTTP only. If it cannot, a CI conformance gate for a stdio-only server would need a small stdio-to-HTTP shim, or must be dropped.
- The hand-written server's line count is an estimate. No minimal hand-written Rust stdio MCP server was located to measure.

---

## 4. How to keep prompts as versioned data files and make prompt changes invalidate exactly the right cache entries?

### Takeaway

Store each prompt template as a data file that is compiled in with `include_str!`, as Codex does with its `prompts/templates/*.md`, and give each a declared version. Key the answer cache on role, template name, template **content hash**, answer-schema hash, and the content hashes of the exact input slices placed in the prompt. Then edits to unrelated code, or a model change, invalidate nothing, while any byte change to a template or schema invalidates exactly its own tasks. A CI lockfile test forces a version bump whenever a template's hash changes, so the human-readable version in the ledger never lies.

### Cited Findings

- Codex keeps prompts as Markdown files compiled into the binary: `pub const SUMMARIZATION_PROMPT: &str = include_str!("../templates/compact/prompt.md");`, and the same for realtime, permissions and persistent mode — [CODEX/prompts/src/compact.rs L1-2](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/prompts/src/compact.rs); [CODEX/prompts/templates/](https://github.com/openai/codex/tree/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/prompts/templates)
- Specula keeps its orchestrator prompts as `.md` files (`src/specula/prompts/incremental.md`, `ci-initialization.md`) and puts stage knowledge in skill folders — [SPECULA/src/specula/prompts/](https://github.com/specula-org/Specula/tree/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/src/specula/prompts)
- Aider keeps prompts in Python classes (`editblock_prompts.py` and others) with no version field — [AIDER/aider/coders/](https://github.com/Aider-AI/aider/tree/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/coders)
- The Mental spec already records `prompt: {template, version, hash}` and `input_hash` per call. It states that "the compile cache is keyed by the unit's content and the prompt version, not by the LLM, so changing the LLM invalidates nothing by itself" — /Users/vs/workspace/mental/docs/specification.md §6.6.
- Anthropic schema compilation is cached server-side for 24 hours per schema ("New schemas incur a one-time compilation cost"). Stable, deterministic schema bytes therefore also save latency — claude-api reference, `shared/tool-use-concepts.md` § Structured Outputs, Important Notes.
- MCP asks for deterministic `tools/list` order to "improve LLM prompt cache hit rates", the same principle at the tool level — [SPEC/docs/specification/2026-07-28/server/tools.mdx L68-71](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/server/tools.mdx)

### Inferences

- **Cache key** = `H(role ‖ template_name ‖ H(template_bytes) ‖ H(canonical_json(answer_schema)) ‖ sorted [(input_kind, anchor, H(slice_bytes))])`.
  - Keying on the template *hash* rather than the declared version alone means a forgotten version bump cannot serve stale answers.
  - Keying on *slice* hashes rather than file hashes means an edit elsewhere in the same file does not invalidate the unit.
  - The LLM ID is deliberately absent (spec §6.6).
  - Summaries of callees enter as inputs by their own content hash, so a callee's changed summary invalidates its callers bottom-up. That is the correct incremental behavior for §6.4's bottom-up order.
- **The check version is not part of the key.** Instead, re-run the checks on every cache hit: they are cheap and deterministic. Then a stricter checker invalidates exactly the answers that now fail (they become repair tasks), and nothing else.
- **Only accepted answers enter the cache,** under the original task's key. Repair attempts are ledger lines (`repair_of`), not cache entries.
- **A `prompts.lock` checked into the repo** lists `template → (version, sha256)`. A unit test recomputes the hashes and fails when a hash changed while the version did not, or when the lock is stale. A second test snapshots every template's rendered output for a fixed fixture (insta). Reviewers then see both the version bump and the exact prompt diff.
- **Canonicalize JSON before hashing** (sorted keys, no insignificant whitespace). Mental's records are serialized by serde from typed structs, so struct field order is stable. Hashing `serde_json::Value` maps needs the `preserve_order` feature off (BTreeMap), or an explicit sort.

### Gaps

- None of the studied tools keys an answer cache on the prompt's content. Aider's request hash excludes the messages, Codex has no answer cache, and SysMoBench has no answer cache. There is no external precedent to compare against. The scheme above is derived from Mental's own spec and general content-addressing practice.

---

## 5. How to test the `agent` backend's pull protocol (task next / submit with on-disk state) end to end?

### Takeaway

Drive the real on-disk state machine with a **scripted fake agent** that answers from recorded and hand-written answers, through three layers:

1. library calls, with crash-and-reopen after every step;
2. the real `mentalc` binary as a subprocess for `task next` / `task submit`;
3. a JSON-RPC transcript piped through `serve(reader, writer)` for MCP.

Specula's fault-injection e2e suite is the closest precedent. It runs the real controller with a deterministic fixture adapter, delivers real SIGKILL/SIGTERM at named fault points, and checks the recovery invariants.

### Cited Findings

- **Specula's fault injection:** "Each test runs the real `specula run` controller with the deterministic single-session fixture adapter, holds a process at a named fault point …, delivers a real SIGKILL or SIGTERM, and attaches with `specula run --run-id` to check the recovery invariants … No model or agent CLI runs." Kill modes are `agent` (SIGKILL the agent only) and `crash` (SIGSTOP then SIGKILL the whole tree) — [SPECULA/tests/e2e/test_single_session_faults.py L1-12](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/e2e/test_single_session_faults.py)
- **Specula's fixture adapter switches:** `block-POINT` publishes `reached-POINT.json` and waits for `release-POINT`; `fail-main-N`; `unconsumed`; `fix-on-rejection` — [SPECULA/tests/e2e/fixtures/single_session_adapter.py L19-30](https://github.com/specula-org/Specula/blob/1a86e7e5ca7ff8567dbdadd39bbe2d953c63ed65/tests/e2e/fixtures/single_session_adapter.py)
- **SysMoBench's `submit_spec`** has a status tool (`get_submission_status`: attempts used, remaining, previous results), but keeps that state in memory only — [SMB/tools/submit_spec/src/submit_spec/server.py L196-215](https://github.com/specula-org/SysMoBench/blob/827899e489fb382b5a37c3c55a5ceb7bf38372c9/tools/submit_spec/src/submit_spec/server.py)
- **rmcp's tests connect client and server in-process** over `tokio::io::duplex` (for example `tests/test_client_initialization.rs`) — [RMCP/crates/rmcp/tests/](https://github.com/modelcontextprotocol/rust-sdk/tree/c644b75aafa0a437e5d814b5672d82ed6b4899ca/crates/rmcp/tests)
- **The MCP spec on restarts:** "If the server process exits unexpectedly, the client SHOULD restart it. Because the protocol is stateless, any in-flight requests are simply lost and the client can retry them" — [SPEC/docs/specification/2026-07-28/basic/transports/stdio.mdx](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/0175d20f8b2c49e0402c37c37ed39ae2be47277e/docs/specification/2026-07-28/basic/transports/stdio.mdx)

### Inferences

- **Layer 1, library and property tests (most of the coverage).** A `ScriptedAgent` loops `next_task` / `accept` against a temp `.mental/`. It looks answers up by cache key in `tests/fixtures/<repo>/answers/`, plus scripted bad answers to force repairs. With proptest, generate sequences of operations:
  - `Next`
  - `Submit(valid | invalid_k)`
  - `SubmitStale(old_id)`
  - `SubmitTwice`
  - `Reopen` (drop every in-memory struct and reload from disk)
  - `CrashMidWrite` (truncate the last ledger line or temp file)

  Invariants:
  - the final model equals the model from an uninterrupted run;
  - every `calls.jsonl` line is well-formed;
  - each accepted task appears once;
  - the repair count never exceeds N;
  - after N failed repairs the build is `Failed`, never "done with gaps";
  - a stale or duplicate submit is rejected with a clear error and changes nothing.
- **Make crash safety structural, so the property test can pass.** Write each state file to a temp path and `rename` it. Append ledger lines with a single `write_all` of a full line. On reopen, ignore a torn last line (or fail with a clear error), and treat the ledger as the source of truth for which calls happened.
- **Layer 2, the CLI.** A few tests run `env!("CARGO_BIN_EXE_mentalc")` with `std::process::Command` (no `assert_cmd` dependency needed). They run `task next --json`, then `task submit <id> --file answer.json`, and check the exit codes and JSON outputs (insta snapshots) for accepted, repair and failed. Killing the process between commands is free, because every command is a fresh process. That gives Specula's "attach after kill" property by construction.
- **Layer 3, MCP.** `serve(&mut Cursor::new(transcript_jsonl), &mut Vec<u8>)` over a temp build. Snapshot the response lines for each era's transcript: `initialize → tools/list → tools/call next → tools/call submit(bad) → isError repair → submit(good)`, and the discover variant. One test spawns `mentalc mcp` as a subprocess, writes the transcript and closes stdin, then asserts a clean exit on EOF.
- **The CLI and MCP must share a single core function per operation** (`task::next(&Build)`, `task::submit(&mut Build, id, raw)`), so layers 2 and 3 only test the mapping. Mutation testing (cargo-mutants) then concentrates on layer 1.

### Gaps

- No studied project uses property-based testing of an LLM pull protocol. The proptest design above is an inference, not an observed practice.

---

## 6. Candidate guidelines for mentalc's LLM-facing code (enforceable rules)

### Takeaway

Fourteen rules follow from the evidence above. Each is stated with its rationale, source and enforcement. Most can be enforced by tests, because every LLM boundary becomes a pure function over data (rendered task in, raw answer out) once the backend is a trait and the build is a state machine on disk.

### Cited Findings

(Sources are those cited in sections 1-5; the short tags are reused in the table below.)

- [A1] Aider reflection cap 3 and precise failure messages — AIDER base_coder.py L932-944; editblock_coder.py L41-123
- [S1] Specula two-layer gate, fuse cap, BLOCKED account, fail-open — SPECULA stop_gate.py L1-48
- [S2] Specula deterministic fixture adapter and fault-injection e2e — SPECULA tests/e2e/…
- [B1] SysMoBench correction loop (3), MCP submit tool with in-memory state — SMB direct_call/method.py; submit_spec/server.py
- [C1] Codex `include_str!` prompt templates — CODEX prompts/src/compact.rs
- [C2] Codex wiremock fakes plus insta context snapshots with normalization — CODEX core/tests/common/…
- [C3] Codex retry: retry 429, 5xx and transport errors; exponential backoff with ±10% jitter; honor Retry-After; `max_attempts` — [CODEX/codex-client/src/retry.rs](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/codex-client/src/retry.rs)
- [M1] MCP 2026-07-28 stateless, discover, `isError`, handles, fixed `tools/list` — SPEC changelog, tools.mdx, stdio.mdx
- [R1] rmcp: 61 crates, tokio mandatory, 61.5k lines, 3 majors in 5 months — RMCP Cargo.toml, CHANGELOG
- [X1] Anthropic structured-output subset (no recursion, `additionalProperties: false` everywhere, no numeric or length constraints); `refusal`/`max_tokens` void the schema guarantee; sampling params 400 on 2026-10-21; retryable codes 429/500/529, the `retry-after` header — claude-api reference `shared/tool-use-concepts.md`, `shared/error-codes.md`
- [SC1] schemars emits `additionalProperties: false` only under `#[serde(deny_unknown_fields)]` ([SCH/schemars_derive/src/schema_exprs.rs L85, L264](https://github.com/GREsau/schemars/blob/4791bb7352555d339e45658ca09c092ed9fa1f5d/schemars_derive/src/schema_exprs.rs)). It emits `oneOf` for tagged enums and `anyOf` for untagged ones (same file, L470-484). Its default meta-schema is Draft 2020-12, and a `Transform` trait rewrites generated schemas ([SCH/schemars/src/generate.rs L30-136](https://github.com/GREsau/schemars/blob/4791bb7352555d339e45658ca09c092ed9fa1f5d/schemars/src/generate.rs); [SCH/schemars/src/transform.rs](https://github.com/GREsau/schemars/blob/4791bb7352555d339e45658ca09c092ed9fa1f5d/schemars/src/transform.rs)). Weight: 17 crates, mostly shared with serde_derive (syn, quote, proc-macro2).
- [V1] A JSON Schema *validator* (`jsonschema`, `default-features=false`) resolves to 64 crates (**bloat**) — scratchpad dependency probe.
- [H1] Blocking HTTP: `ureq 3.5.2` resolves to 33 crates with default features (rustls, ring, webpki-roots) and 16 without TLS — scratchpad probe. clust, the closest small Anthropic client, is 8,550 lines on async `reqwest`. It hard-codes a `ClaudeModel` enum of 2024 model IDs and was last updated 2025-03-02 (**bloat and staleness**) — [CLUST/src/messages/claude_model.rs L8](https://github.com/mochi-neko/clust/blob/81fda7c644a6e01971c378e53f4d0afdb873a9b9/src/messages/claude_model.rs); [CLUST/Cargo.toml](https://github.com/mochi-neko/clust/blob/81fda7c644a6e01971c378e53f4d0afdb873a9b9/Cargo.toml). Anthropic publishes no official Rust SDK (the anthropics GitHub org lists SDKs for Python, TypeScript, Java, Go, Ruby, C#, PHP; checked with `gh repo list anthropics`).

### Inferences

**Proposed rules** (status: Proposed; each one could become a line in a project rules file):

| # | Rule | Rationale | Source | Enforcement |
|---|---|---|---|---|
| G1 | The LLM core is a state machine: `Build::next_task()` and `Build::accept(task_id, raw) -> Verdict`. Backends never call checks, the cache or the ledger. | One judge for every backend (spec §6.5). The pull backend inverts control. | §1; [S1], [B1] | Reviewer, plus a module boundary (backends in `backend/` cannot import `check`/`cache`/`ledger`). A test asserts every backend goes through `accept`. |
| G2 | `Backend` is synchronous: `fn complete(&self, &RenderedTask) -> Result<RawAnswer, BackendError>`. No `async`, no tokio anywhere in mentalc v1. Fix spec §6.5's `async fn`. | 15 calls per build do not need concurrency; tokio would add a runtime and dozens of crates. | [R1], [H1]; F-S10 | CI: `cargo tree -e normal -i tokio` must fail (no match); clippy `disallowed-types`/grep gate for `async fn` in `crates/mentalc`. |
| G3 | The MCP server is hand-written, synchronous, line-delimited JSON-RPC over `BufRead`/`Write`, and dual-era (2025-11-25 `initialize`/`ping` plus 2026-07-28 `server/discover` and `_meta`, with `resultType`, `ttlMs`, `cacheScope`). No rmcp. | Zero added crates against 61. Stable against SDK major-version churn. Testable in memory. | [R1], [M1] | Insta transcript tests per era built from spec examples; a subprocess EOF-exit test; a manual Claude Code and Codex smoke check at each release (reviewer checklist). |
| G4 | The MCP tool list is fixed and ordered: `mental_task_next`, `mental_task_submit(task_id, answer)`, optionally `mental_build_status`. Per-task schemas travel inside the task, never as dynamic `inputSchema`. Check failures, unknown or stale handles, and exhausted repairs return `isError: true` with the repair text. Only protocol faults are JSON-RPC errors. | The spec forbids `tools/list` varying as a side effect, recommends handles, and says clients feed `isError` results back to the model for self-correction. | [M1]; SysMoBench returned errors as plain text [B1] | Snapshot test of `tools/list`; a test that every check failure maps to `isError: true`. |
| G5 | Every repair task names each failure: check, record ID, rule, evidence (for anchors: the quote, the file, the closest actual lines), and what passed. It asks for the whole answer again under the same schema. | The model can only fix what it is shown. Aider's message is the proven pattern. | [A1] | Insta snapshots of repair tasks for each check kind; a unit test that every `CheckFailure` variant renders a non-empty `evidence`. |
| G6 | Repairs are capped per task (config, default 3). On exhaustion the build fails with every failure attached. No fail-open path, no "model with gaps". | Fail rather than degrade. Specula's fail-open gate is the counter-example. | [A1], [B1], [S1]; memory "fail rather than degrade" | Proptest invariant: the final state is `Failed` after N+1 bad answers. cargo-mutants must kill a mutant that skips the cap. |
| G7 | Answer types are Rust structs with `#[serde(deny_unknown_fields)]`. Deserialization *is* the structural validator. The JSON Schema sent to the LLM is derived from the same types with schemars, then passed through one `Transform` that rejects anything outside the Anthropic subset (recursion, numeric or length keywords, objects without `additionalProperties: false`) and rewrites `oneOf` → `anyOf`. Constraints the schema cannot express are mentalc checks. No JSON Schema validator crate. | One source of truth for shape. Anthropic rejects recursion and constraint keywords. A validator crate costs 64 crates for what serde already does. | [SC1], [X1], [V1] | A test over every role's schema: the subset check passes and an insta snapshot of the schema exists. A grep or clippy gate: every type in `answer/` derives `Deserialize` with `deny_unknown_fields`. Proptest: random JSON → deserialize never panics, and an unknown field is always rejected. |
| G8 | Prompt templates are files under `prompts/<role>/<name>.md`, embedded with `include_str!`, with a declared version. `prompts.lock` pins `(version, sha256)` per template. | Prompts are reviewable data. The version in the ledger must mean something. | [C1]; spec §6.6 | A unit test fails when a template's hash changed without a version bump, or when the lock is stale. Insta snapshot of each rendered template on the fixture. |
| G9 | Cache key = H(role, template name, template hash, schema hash, sorted input slice hashes). The LLM is never part of the key. Only checked, accepted answers are cached. Every cache hit is re-checked. | Exact invalidation: unrelated edits and model changes invalidate nothing, while prompt, schema or input changes invalidate exactly their tasks. Stricter checks surface stale answers as repairs. | §4; spec §6.6 | Property test: changing a byte outside every slice leaves the key unchanged, and changing a byte inside a slice changes it. A test that a hit whose answer now fails a check yields a repair, not acceptance. |
| G10 | Every call, including failed and repair calls, appends exactly one complete JSON line to `calls.jsonl` *before* its records are used. Raw answers are stored by content hash. A torn last line is detected on open. | Audit and LLM comparison (spec §6.6). Crash safety for the pull backend. | Codex rollout JSONL [C2]; Aider `--llm-history-file`; Specula `gate.log` [S1] | Proptest with `CrashMidWrite`/`Reopen`; a test that ledger line count equals the number of backend calls; a schema test of each ledger line (deny_unknown_fields struct). |
| G11 | Tests never call a live model. Use the `replay` backend, which serves answers by cache key from `tests/fixtures/<repo>/answers/` and errors on a miss, plus hand-written invalid answers for every check kind. Golden tasks are insta snapshots of rendered tasks with volatile values normalized. | Determinism: sampling params are now rejected, so answers can only be made deterministic by recording them. Repair paths must be covered without a model. | [S2], [C2], [X1]; SysMoBench `ExistSpecAdapter` | CI has no API key and fails if a test reaches the network (the `api` backend refuses to run without a key; fail, don't degrade). Diff coverage at 100% on `check/`, `repair`, `cache`, `ledger`. |
| G12 | The `agent` pull protocol is tested at three layers: a library proptest with reopen and crash, a CLI subprocess test via `CARGO_BIN_EXE_mentalc`, and an MCP transcript through `serve(reader, writer)`. CLI and MCP share one core function per operation. | Specula shows that crash and resume invariants are where pipelines break. Sharing the core keeps the I/O layers thin. | [S2]; §5 | The tests themselves, plus cargo-mutants on `task.rs` and `mcp.rs` (no surviving mutants in dispatch). |
| G13 | The `api` backend is a small blocking client (ureq, rustls) that sends the raw Messages API: `x-api-key`, `anthropic-version`, `model`, `max_tokens`, `system`, `messages`, `output_config.format = json_schema`, and `effort`. It never sends `temperature`/`top_p`/`top_k`, and never hard-codes model IDs (they come from config). It retries only 408/409/429/5xx/529 and transport errors, with exponential backoff and jitter, honoring `retry-after` and a max attempt count. `stop_reason` of `refusal` or `max_tokens` is a `BackendError`, not a repair. | No official Rust SDK exists. clust shows the bloat and staleness risk of a typed wrapper. Codex's retry is about 100 lines. | [H1], [C3], [X1] | Insta snapshot of the serialized request; unit tests of response parsing on recorded bodies (success, refusal, max_tokens, 429 with `retry-after`, 529); pure-function tests of the retry classification; `cargo tree` crate budget in CI. |
| G14 | Track a dependency budget. mentalc's LLM and MCP layer may add at most `ureq` (with rustls) and `schemars` on top of serde, serde_json and thiserror. Any further crate needs a written justification. | The owner's goals: compact, no bloat. The measured gap is 14 + ~19 + ~3 crates, against 61 for rmcp or 64 for jsonschema. | [R1], [V1], [H1], [SC1] | A CI script compares the `cargo tree -e normal --prefix none \| sort -u \| wc -l` count with a checked-in number; a reviewer approves any increase. |

**Bloat flagged:**

- Codex's 2.36M lines;
- rmcp's 61.5k lines and 61 crates for 3 tools;
- clust's 8.5k lines and stale model enum;
- the `jsonschema` validator at 64 crates;
- Specula's 88k lines of Python;
- Aider's dependence on LiteLLM (not measured here).

None of these should be adopted. Their *patterns* should be: Aider's failure message, Specula's account-not-outcome gate and fault injection, Codex's prompt files, snapshot normalization and retry policy, and SysMoBench's checker-as-MCP-tool.

### Gaps

- It is unverified whether Anthropic strict structured outputs accept optional properties (missing from `required`). This affects whether answer structs may have `Option<T>` fields, or must use explicit `null` unions.
- Whether `oneOf` is accepted by Anthropic structured outputs is not stated: the supported list names `anyOf`/`allOf` only, so G7 rewrites `oneOf` to be safe.
- The OpenAI-compatible request shape (`response_format: {type: "json_schema", json_schema: {strict: true, schema}}` on `/v1/chat/completions`) was not verified from source in this pass. Codex uses the Responses API `text.format` instead ([CODEX/codex-api/src/common.rs L443-461](https://github.com/openai/codex/blob/e9919f4b4ef9aa4b15af92589c7f44581d1ad26a/codex-rs/codex-api/src/common.rs#L443-L461)).
- The exact line counts for a hand-written MCP server (est. 250-400) and a blocking Messages client (est. 150-250) are estimates, not measurements.
