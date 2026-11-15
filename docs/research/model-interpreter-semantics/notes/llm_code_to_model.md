# LLM code-to-executable-model translation and model-vs-implementation validation

Scope note: these notes build on `docs/research/static-analysis-llm-system-models/report.md` and do not repeat what it covers (IRIS, LLMDFA, SemTaint, RepoAudit, ProtocolGPT, RFC2TLA+, SysMoBench headline numbers, the 2606.05792 TLA+ benchmark). In the project's framing, extractors plus an LLM *compile* code into a model of entities, ports, wires, state, predicates, guarded transitions and per-port behaviors; an engine interprets it. These notes cover how to make that compiler reliable and how to check its output against the code. Research date: 2026-11-11. Unless marked, figures are self-reported by the authors.

## 1. LLMs generating executable models, specifications, simulators and mocks from code

### Takeaway
As of late 2026, code-to-model translation works when it is agentic and has a checker in the loop, and fails when it is one-shot. The leading system, Specula (July 2026), gets 100% on SysMoBench's conformance metric by validating every model against traces from the instrumented code. One-shot prompting on large systems such as etcd Raft gets 0–15% conformance. The recurring errors are wrong grain of atomicity, oversimplified concurrency, omitted behaviors, "textbook" behavior copied from training data where the code deliberately deviates, and repair that cheats by weakening guards. LLM-simulated services (MIRAGE) reach 94–99% status fidelity, but only for HTTP/JSON and only on three systems.

### Cited Findings

**SysMoBench (TLA+ from system code), details beyond the prior report**
- SysMoBench v3 (28 Jan 2026) has 11 artifacts: concurrent (Asterinas Spinlock, Mutex, Rwmutex, Ringbuffer), distributed (etcd Raft, Redis Raft, Xline CURP, ZooKeeper FLE), and PGo-generated (dqueue, locksvc, raftkvs) — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- Four metrics are applied in sequence:
  - Syntax: a SANY check; 100% for a full pass, 50% when only every per-action check passes. Only 100%-syntax models go on.
  - Runtime: the fraction of actions TLC covers without a runtime error.
  - Conformance: the fraction of instrumented code actions that TLC covers without a conformance error. The system code is instrumented with logging, its traces are fed to TLC together with the model, missing events are inferred, and an LLM maps model names to code names.
  - Invariant: the fraction of LLM-concretized invariant templates that hold.
  — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- Three agents were compared: a basic modeling agent (prompt with the code), a code translation agent (statement-by-statement translation, later reorganized into control flow), and a trace-learning agent. The trace-learning agent's results were omitted because it failed the runtime checks. Each agent ran 5 times with the best run kept, up to 3 feedback iterations, no human help, and no cost reported. The LLMs were Claude-Sonnet-4, GPT-5, Gemini-2.5-Pro and DeepSeek-R1 (these generations are now superseded) — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- On Spinlock, nearly every agent/LLM pair scores 100/100/80–100/86–100 (syntax/runtime/conformance/invariant). On etcd Raft the best result, translation agent with Claude-Sonnet-4, is 100/66.67/**15.38**/92.31. The basic agent with Claude-Sonnet-4 reaches only 7.69% conformance, and most other pairs never get past syntax or runtime — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- Error taxonomy reported:
  - Syntax: math symbols in place of ASCII operators, Python syntax mixed into TLA+, wrong operator arity.
  - Runtime: inconsistent TLC configs (missing constants), and type misuse such as applying set operations to records.
  - Invariants: 8.3% of safety properties violated against 41.9% of liveness properties.
  - Qualitative: models imitate classic TLA+ examples from training data and miss system-specific content; they leave topics out or add extra ones, mismatch fairness, and hard-code values.
  — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- Basic agents do badly on the PGo-generated systems, whose code is repetitive and uses synthetic variable names. This points to reliance on familiar names and priors rather than on the code itself — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)
- Alloy and PAT results were near zero, which the authors attribute to LLM unfamiliarity with those languages. Target-language familiarity therefore matters a great deal — [SysMoBench HTML v3](https://arxiv.org/html/2509.23130v3)

**Specula (state of the art, July 2026)**
- Pipeline:
  1. Infer protocol-level and code-level invariants, each with cited evidence from code, comments, issues, commits and tests.
  2. Write a reference TLA+ model whose abstraction level is chosen by those invariants.
  3. Generate evidence-backed scenarios.
  4. Project the reference model into per-scenario models by action selection, coarsening and serialization.
  5. Instrument the code automatically, collect traces, and check them with a TLC replay harness.
  6. Repair the model or the instrumentation until the code conforms.
  7. Model-check repaired models against the protocol invariants to catch overfitting.
  8. Replay violations at code level as tests.
  — [Specula, arXiv 2607.25333](https://arxiv.org/html/2607.25333v1)
- Scale: 48 projects (36 distributed, 12 concurrency) in 7 languages, 2K–95K LoC, including MongoDB, etcd, ScyllaDB, libgomp and SONiC. It found 249 bugs (207 new); 68 confirmed and 24 fixed. 80.3% surfaced through model checking, and the median counterexample length is 9 steps — [Specula](https://arxiv.org/html/2607.25333v1)
- Baselines on 5 systems:
  - Bugs found: Specula 62, plain Claude Code ("Agent-Raw") 2, Claude Code with TLA+ skills/MCPs ("Agent-TLA+") 3.
  - SysMoBench overall: Specula with Opus-4.8 100%, Sonnet-4.6 93%, Haiku-4.5 47%; Agent-TLA+ 82%, Agent-Raw 81%.
  - Conformance: Specula 100%, Agent-TLA+ 75%, Agent-Raw 70%.
  — [Specula](https://arxiv.org/html/2607.25333v1). An independent summary notes that the comparison does not isolate which component causes the gain — [alphaXiv](https://www.alphaxiv.org/abs/2607.25333)
- Cost: 1.43–9.86 hours end to end (median 3.69 h) and **$19–$168 per project (median $57)** in tokens — [Specula](https://arxiv.org/html/2607.25333v1)
- Observed LLM error categories:
  - hallucinated execution paths and oversimplified concurrency;
  - omitted behaviors alongside admitted unreachable states, i.e. under- and over-approximation at once;
  - textbook invariants applied where the implementation deliberately deviates;
  - overfitting during repair: weakened guards, permissive transitions, hard-coded trace updates;
  - syntax slips (missing UNCHANGED, duplicate assignments, malformed configs), caught by a custom static analyzer.
  — [Specula](https://arxiv.org/html/2607.25333v1)
- Stated limitation: agents struggle to tell a model error from a real code bug, and the loop may not converge if the agents misjudge the invariants — [Specula](https://arxiv.org/html/2607.25333v1)

**Other TLA+/Quint/formal-model generation**
- The TLA+ community maintains agent skills, including `tlaplus-from-source`, which derives a high-level TLA+ model from C, C++, Rust and similar code in an analyze → abstract → specify → propose workflow — [tlaplus/AgentSkills](https://github.com/tlaplus/AgentSkills)
- TLA-Prover fine-tunes for TLA+ synthesis with preference optimization over LoRA (snippet only; not read in full) — [arXiv 2606.06133](https://arxiv.org/pdf/2606.06133)
- Informal Systems (Nov 2025) used an LLM on the Malachite BFT engine. The LLM edited a Quint spec from an English protocol description, with `quint parse` and `quint typecheck` run after each edit; humans validated the spec by querying reachable scenarios; the LLM then changed the code from the old spec, the new spec and their diff, and wrote model-based-testing glue that replays spec scenarios against the implementation in CI. Spec work took about 2 days, code and tests about 1 week, against a months-long estimate. Validation found 2 bugs in the English spec in one afternoon. Their lesson: "validation is the bottleneck, not generation"; "LLMs don't think, they translate" — [Quint blog](https://quint.sh/posts/llm_era)
- A 2025 paper with Informal Systems authors used an LLM to fill stubs of a Quint model and catalogued its syntactic and semantic hallucinations (snippet only) — [arXiv 2501.12972](https://arxiv.org/html/2501.12972v1)

**State machines from code (beyond the prior report)**
- State-inferer combines program slicing, an LLM and model checking. It reports a macro strict transition-level F-score of 95.2, against 78.9 for the best baseline, on 7 TLS/IKEv2/SSH implementations. It is an unreviewed Research Square preprint (Nov 2026) — [State-inferer](https://www.researchsquare.com/article/rs-11391337/v1)
- An architecture-description study (Nov 2025) found that LLM-generated state diagrams from code keep the source terminology but almost entirely miss **self-transitions**, which are often timeouts. This is a concrete "missing transitions" error class — [arXiv 2511.05165](https://arxiv.org/html/2511.05165v1)
- On spec-style inputs, the best transition F1 was 0.75 (Claude 3.5 Sonnet) while guard F1 dropped to 0.23 (GPT-4o): guards are the weakest element. These are older model generations — [arXiv 2604.00275](https://arxiv.org/html/2604.00275)

**LLM-simulated services and mocks (service virtualization)**
- MIRAGE (2026) has an LLM answer each dependency request online, keeping cross-request state, instead of generating static mocks. It contrasts this with Prism, which builds interface-faithful but not behavior-faithful mocks from OpenAPI (for example, it returns 200 where the real service declines a payment) — [MIRAGE, arXiv 2604.04806](https://arxiv.org/html/2604.04806v3)
- MIRAGE fidelity over 110 scenarios on 3 systems:

  | Approach | Status fidelity | Body-shape fidelity |
  |---|---|---|
  | Record-replay | 62% | 16% |
  | Traces only | 92% | 53% |
  | Black-box (caller code + traces) | 94% | 75% |
  | White-box (adds dependency source) | 99% | 99% |

  On error-handling scenarios replay scored 0% and MIRAGE 100%. Values were not checked, only status codes and top-level keys — [MIRAGE](https://arxiv.org/html/2604.04806v3)
- MIRAGE cost: about 3 s per request with Opus 4.6 and 1.5 s with Sonnet 4.6, $0.16–$0.82 per dependency, about $2.24 per CI run with 14 dependencies. Black-box failures cluster in stateful lifecycle and pre-seeded-data assumptions. Only HTTP/JSON was tested; gRPC and queues were not — [MIRAGE](https://arxiv.org/html/2604.04806v3)
- MirrorAPI fine-tunes LLMs on request–response pairs from 7,000+ RapidAPI APIs as tool-environment simulators. Its authors claim results "comparable to real-world APIs" as a StableToolBench environment (Findings of ACL 2025) — [MirrorAPI, arXiv 2503.20527](https://arxiv.org/pdf/2503.20527)

### Inferences
- For our compiler, the right reference point is Specula, not SysMoBench's basic agent. Specula's key moves map onto our design:
  - Its evidence-backed invariants play the role of our anchored predicates.
  - Its reference model plus per-scenario projections is our model plus scripts.
  - Its model-to-code mapping recorded during generation is our anchors.
  - Its trace-validation loop is our tier 8 ("observed"), and it is the only thing that pushed conformance to 100%.
- The error taxonomy across sources is consistent, and each class can become a targeted check:
  - (a) wrong grain of atomicity: Specula's "oversimplified concurrency"; the Raft/EWD998 cases in Cirstea et al.
  - (b) missing transitions, especially self-loops, timeouts and error paths.
  - (c) invented or textbook behavior: SysMoBench's imitation of classic specs; Specula's "textbook invariants".
  - (d) weak guards: guard F1 0.23 on spec inputs.
  - (e) repair overfitting: weakened guards, permissive transitions.
  - (f) under- and over-approximation at once.
- MIRAGE's ablation (dependency source alone gives 100% on two benchmarks; without it body-shape fidelity is 53–69%) supports our choice to fetch real library source rather than rely on priors.
- MIRAGE also shows that an LLM can itself serve as a co-simulation fallback for an unmodeled port. Its outputs would have to be marked `assumed`, and it is not deterministic, which conflicts with reverse stepping.

### Gaps
- No published system compiles code into a *statechart/XState* or *DEVS* model and validates it against the implementation. Search found protocol FSM extraction and TLA+/Quint/P, not DEVS or XState targets.
- No paper reports LLM generation of **P** models from code with conformance numbers. PObserve is used with hand-written P specs.
- MirrorAPI's fidelity figures (89.9 BLEU, 94.1% cosine) came only from an aggregator snippet and were not verified.
- Specula's §5.4–5.5 (loop effectiveness, iteration counts) were not read; the per-iteration contribution of repair is unknown.

## 2. Validating a model against an implementation

### Takeaway
There are three mature families of techniques:
- **Trace validation**: implementation traces checked against the model, as TLC constrained model checking (Cirstea/Kuppe/Merz 2024, CCF, etcd) and as P/PObserve on logs at AWS.
- **Model-based test generation**: model to tests to real code (MongoDB Realm Sync at 100% branch coverage; Quint/Modelator; ioco theory).
- **Mutation-based adequacy checks** on generated specifications.

The industrial record shows that trace validation finds discrepancies every time it is applied. It is only practical when the model's grain of atomicity is chosen to match what can be logged; MongoDB abandoned trace-checking against a highly abstract spec. The payoff is the discrepancy analysis, not a proof.

### Cited Findings

**Trace validation against TLA+**
- Cirstea, Kuppe, Loillier and Merz (2024) built:
  - a Java library that logs operations as TLA+ transitions and variable updates;
  - a trace spec that constrains the original spec by the recorded trace;
  - a TLC run against it, framed as constrained model checking.
  They found spec/implementation discrepancies in **every** case study — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Case studies: two-phase commit, a snapshot-isolation KV store, EWD998 termination detection, two unnamed Raft implementations, and Microsoft CCF — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Discrepancy causes:
  - an implementation shortcut (counting "Prepared" messages instead of tracking which RMs sent them, so a resent message triggers a premature commit);
  - an over-strict spec (a node forbidden to send itself a message);
  - **grain-of-atomicity** mismatches: in EWD998, token passing split into two logged transitions; in Raft, the implementation updates the term and appends entries in one step where the spec has two.
  Fixes: log several transitions, use TLC's action composition, or treat steps that do not change the abstract state as stuttering — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Handling partial traces and checking acceptance:
  - Traces may omit variables, event names or arguments, and TLC fills the gaps non-deterministically.
  - Acceptance is `TraceAccepted` (TLC's diameter equals the trace length). On failure there is no counterexample; breakpoints in the TLA+ debugger locate the mismatch.
  - Overly imprecise logs can accept erroneous traces.
  — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Performance and logging granularity:
  - Logging events together with variable values explores the fewest states.
  - Logging only variables, or only event names, explodes the state space (many runs hit the 1-hour timeout).
  - Well-chosen partial logs are enough, and an event-names-only trace still found the 2PC bug in under a minute.
  - DFS bounded by trace length can beat BFS.
  — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Their instrumentation advice: log every change to shared state (send, receive, lock, stable storage); trace only what matters; decide deliberately how to handle atomicity mismatches; run trace validation in CI so spec and code stay in sync, as CCF does — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- CCF "smart casual verification" (NSDI 2025) found 6 bugs (5 safety, 1 liveness) before production. The team reports that the implementation and spec used different grains of atomicity, which complicated matching, and that trace validation now runs in CI — [arXiv 2406.17455](https://arxiv.org/pdf/2406.17455v2); [Decentralized Thoughts](https://decentralizedthoughts.github.io/2025-05-23-smart-casual-verification)
- Kuppe's 2024 TLA+ talk reports spec/implementation divergences across seven systems, lists trace validation of vanilla etcd-raft (etcd-io/raft issue #111), and notes that some failures came from test infrastructure dropping messages — [Kuppe, TLA+ Conf 2024](https://conf.tlapl.us/2024/MarkusAKuppe-ValidatingSystemExecutionsWithTheTLAPlusTools.pdf)
- TLC documents trace validation as a supported workflow — [TLA+ docs: trace validation](https://docs.tlapl.us/_export/xhtml/using:tlc:trace_validation)
- ModelFuzz uses a TLA+ model to guide fuzzing of the implementation and found 13 new bugs in etcd-raft and RedisRaft — [arXiv 2410.02307](https://www.arxiv.org/pdf/2410.02307v2)

**Industrial negative result and positive MBT result (MongoDB, VLDB 2020)**
- Model-based trace-checking of the MongoDB Server's replication protocol against a highly abstract spec was judged **impractical** and cancelled as "more effort than it was worth". Model-based test-case generation for MongoDB Realm Sync was "highly successful" and reached 100% branch coverage of the specified algorithm — [Davis, Hirschhorn, Schvimer, arXiv 2006.00915](https://arxiv.org/abs/2006.00915v1); [Davis blog](https://emptysqua.re/blog/extreme-modelling-in-practice/)

**P / PObserve (AWS)**
- PObserve (2023) checks P specifications, typically global invariants, against service logs parsed into time-stamped events, in both testing and production, without extra instrumentation. P itself has been developed at AWS since 2019 — [Brooker & Desai, CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services); [PObserve docs](https://p-org.github.io/P/advanced/pobserve/pobserve); [P case studies](https://p-org.github.io/P/casestudies)

**Model-based testing (model → tests → code)**
- Quint's approach: since the spec is executable, the same scenarios run against spec and implementation and the results are compared (MBT). Trace validation checks real-environment traces against the spec. Quint Connect is a Rust library that replays generated traces against an implementation — [Quint MBT docs](https://quint-lang.org/docs/model-based-testing); [Quint blog](https://quint-lang.org/blog)
- Modelator takes TLA+ models and uses Apalache to generate tests that run against an implementation. Its last release (0.4.2) dates from January 2022, so it is dormant and superseded by Quint tooling — [modelator on docs.rs](https://docs.rs/crate/modelator/latest)
- Informal Systems' Tendermint light-client MBT used TLA+ and Apalache — [tendermint-rs MBT](https://github.com/informalsystems/tendermint-rs/blob/HEAD/light-client/tests/support/model_based/Abstract.md)
- ioco (Tretmans): an implementation conforms if it never produces an output, *quiescence included*, that the specification could not produce after the same trace. Implementations must be input-enabled; specifications need not be. Test generation is sound and exhaustive. Tools include TGV, TestGen and TorX — [Janssen/Tretmans via arXiv 1909.13604](https://arxiv.org/pdf/1909.13604v1.pdf); [Bijl, Rensink, Tretmans, compositional testing with ioco](https://ftp.cs.ru.nl/CSI/SoftwEng.FunctLang/papers/2003/bijm2003-compositional_testing_ioco.pdf)
- Known ioco limitations: it is hard to reconcile with stable-failures refinement, and for underspecified inputs it supports only positive testing, which rests on optimistic assumptions — [CONCUR 2023](https://drops.dagstuhl.de/storage/00lipics/lipics-vol279-concur2023/LIPIcs.CONCUR.2023.4/LIPIcs.CONCUR.2023.4.pdf); [arXiv 1606.09035](https://arxiv.org/pdf/1606.09035)
- ioco decomposes across components under conditions studied in "Decomposability in ioco" — [arXiv 1303.1009](https://arxiv.org/pdf/1303.1009)

**Mutation-based validation of generated specifications**
- LLM-generated RTL assertions are refined until an "effective kill rate" (invalid mutants excluded) exceeds 85%. Runs typically converge to 90–100%, and surviving mutants are fed back to the LLM — [arXiv 2606.21451](https://arxiv.org/pdf/2606.21451)
- Mutated implementations serve as negative tests to detect *over-permissive* LLM-generated specifications that would accept faulty code (snippet-level) — [arXiv 2608.13077](https://arxiv.org/pdf/2608.13077)
- Mutation scores expose weak oracles: some LLM test suites reach 100% coverage but a 4% mutation score (snippet-level) — [arXiv 2506.02954](https://arxiv.org/pdf/2506.02954v6)
- Meta uses mutation-guided LLM test generation in production (ACH) — [arXiv 2501.12862](https://arxiv.org/pdf/2501.12862)

### Inferences
- Trace validation detects only *over-restriction*: the code did something the model does not allow, i.e. a missing transition or a wrong guard. It cannot detect over-approximation (the model allows things the code never does). Model-to-test generation, in the MBT/ioco direction, covers the other half. A faithful-compiler check needs both directions: traces → model (completeness) and model → tests on the code (soundness of the model's claims).
- The MongoDB failure and the Cirstea/CCF atomicity findings point the same way: trace validation is cheap only if the model's step granularity can be logged. Our refinement levels help, because the coarse level can be validated against coarse logs (HTTP spans, queue events) and the fine level against function-level tracing. Stutter steps (forwarding-only) are exactly what Cirstea et al. treat as stuttering.
- ioco's quiescence is the formal version of our "stuck configuration" check. A model that predicts no output where the code responds (or the reverse) is a conformance failure.
- Mutation of the *code* (does the model notice when the code changes behavior?) and mutation of the *model* (do the existing traces and tests kill the mutant model?) are both cheap adequacy metrics for a compiled model, although no paper applies them to code-to-model compilers specifically.

### Gaps
- No public, quantified report of etcd-raft trace-validation findings (only issue and PR pointers).
- No study of trace validation for *web/microservice* systems (HTTP, queues, DBs) as opposed to consensus protocols. All the evidence is from distributed protocols.
- No source applies ioco or MBT to LLM-generated models specifically.
- GraphWalker/ModelJUnit were not researched in detail. They are classic graph-walk MBT tools, but no 2025–2026 LLM evaluation using them was found.

## 3. Iterative repair loops driven by checker feedback

### Takeaway
Checker-in-the-loop repair is necessary but shows sharply diminishing returns: most gains come in the first 3–4 iterations, and workflow and feedback design matter more than the model. The specific danger in model repair is *overfitting*: the LLM weakens guards or hard-codes trace updates to make a trace pass. The countermeasure is to re-check repaired models against independent invariants, as Specula does. Verifier loops are the closest analogues: AutoVerus passes over 90% of 150 Verus proof tasks, and Clover's Dafny consistency check accepts 87% of correct instances.

### Cited Findings
- Kiecker et al. (July 2026, NIER) studied repair loops across code generation, test generation and translation with low-cost LLMs. The first **3–4 iterations capture most of the achievable gains**, later ones add only marginal improvement, and repair behavior depends more on workflow orchestration and feedback design than on the model. They argue repair budgets should be an explicit experimental variable — [arXiv 2607.05197](https://arxiv.org/abs/2607.05197)
- SysMoBench caps feedback at 3 iterations, takes the best of 5 runs, and still gets low runtime pass rates on etcd Raft for most models (2025 models) — [SysMoBench](https://arxiv.org/html/2509.23130v3)
- Specula's repair loop fixes either the model or the instrumentation. To block overfitting, repaired models are model-checked against the protocol invariants; a Kudu-Raft case shows a repaired model violating State Machine Safety. The overfitting patterns observed are weakened guards, permissive transitions and hard-coded trace updates — [Specula](https://arxiv.org/html/2607.25333v1)
- AutoVerus (OOPSLA 2025) orchestrates LLM agents in three phases that mimic expert proof construction (generation, refinement, debugging with verifier feedback). It proves over 90% of 150 non-trivial Verus tasks; one leaderboard lists 137/150 — [arXiv 2409.13082](https://arxiv.org/abs/2409.13082v3); [SPLASH 2025](https://2025.splashcon.org/details/OOPSLA/200/AutoVerus-Automated-Proof-Generation-for-Rust-Code)
- Clover (SAIV 2024, GPT-4 with Dafny) checks consistency among code, docstrings and formal annotations, accepting up to 87% of correct instances. It found 6 incorrect programs in the human-written MBPP-DFY-50. Its dataset is textbook-level and its model generation is superseded — [arXiv 2310.17807](https://arxiv.org/abs/2310.17807v4)
- SpecGen mutates LLM-generated specifications and keeps the variants that verify, using mutation as a *repair* operator — [arXiv 2401.08807](https://arxiv.org/pdf/2401.08807v4)
- The prior project report found static guardrails with reflection fixing 92% of extraction errors within the first retry (PARSE) — [arXiv 2510.08623](https://arxiv.org/abs/2510.08623v1)
- Older self-repair evidence (2023, GPT-3.5/4 era, superseded): Olausson et al. found self-repair gains modest once the extra sampling cost is counted, with feedback quality as the bottleneck. Cited from prior knowledge and not re-fetched for these notes — [Is Self-Repair a Silver Bullet?, arXiv 2306.09896](https://arxiv.org/abs/2306.09896)
- A 2026 preregistered study suggests self-repair feedback effects in small frozen code models may be largely placebo-like. Only the title and snippet were seen, so this is unconfirmed — [arXiv 2606.31511](https://arxiv.org/pdf/2606.31511)

### Inferences
- Budget per compilation unit: about 3 repair iterations, then escalate (a stronger model, a k-sample vote, or mark the record `assumed` and surface it).
- The repair loop must have an *anti-overfitting oracle* independent of the failing check. For us that is the coverage obligations (every extracted branch and effect must still be accounted for), the guard-algebra checks, and the original anchored predicates. A repair that makes a trace pass by turning a guard into `true` or deleting a case should fail tier 3/4. Repairs should be scored as diffs: "guard weakened" and "case removed" deserve extra scrutiny.
- Specula's open problem (is it a model bug or a code bug?) is exactly our `provenance` question. A divergence between model and trace should create a record ("model gap" or "code bug candidate") rather than an automatic repair.

### Gaps
- No found source gives per-iteration success curves for TLA+/model repair specifically. Specula's loop-effectiveness section was not read.
- The cost per repair iteration for model compilation is unreported, apart from Specula's whole-pipeline $19–$168.

## 4. Granularity and compositionality

### Takeaway
LLMs succeed locally and fail on composition. DafnyComp (ICLR 2026) shows 95.7% syntactic correctness but only 3.7% verification on multi-function programs, with failure growing faster than linearly in the number of functions. Successful systems decompose. Specula splits one reference model into per-scenario projections; SysMoBench's translation agent works statement by statement; State-inferer feeds program slices; MIRAGE models one dependency at a time. Composition is verified mechanically rather than trusted. Direct, quantified modular-versus-whole comparisons for *model* generation were not found.

### Cited Findings
- DafnyComp: 300 multi-function Dafny programs. Models do well on single functions but drop sharply when composing: about 95.7% syntactic correctness against 3.7% verified, and failure grows superlinearly with the number of functions. The failure modes are specification fragility, implementation–proof misalignment and reasoning instability — [Local Success Does Not Compose, ICLR 2026](https://arxiv.org/html/2509.23061v1); [ICLR proceedings](https://proceedings.iclr.cc/paper_files/paper/2026/hash/c04d37be05ba74419d2d5705972a9d64-Abstract-Conference.html)
- NLForge/"Natural Language based Specification and Verification" (May 2026) generates natural-language function summaries bottom-up (callee to caller) and reuses them for compositional verification. Only "preliminary results ... promising" is stated; no numbers were found — [arXiv 2605.11315](https://arxiv.org/abs/2605.11315)
- FM-Agent derives function specifications top-down from caller usage (Hoare-style) to scale formal methods to large systems (snippet only) — [arXiv 2604.11556](https://www.alphaxiv.org/abs/2604.11556)
- Specula decomposes by *scenario*: a reference model is projected per scenario by action selection, coarsening and serialization, which keeps model checking tractable. 93.5% of model-checking bugs were found by BFS, with short counterexamples (median 9 steps) — [Specula](https://arxiv.org/html/2607.25333v1)
- SysMoBench's statement-by-statement translation agent beats whole-code prompting on etcd Raft (conformance 15.38 against 7.69 with Claude-Sonnet-4) — [SysMoBench](https://arxiv.org/html/2509.23130v3)
- State-inferer's slicing-plus-LLM-plus-model-checking pipeline reaches 95.2 transition F vs 78.9 for baselines (unreviewed preprint) — [State-inferer](https://www.researchsquare.com/article/rs-11391337/v1)
- MIRAGE simulates one dependency at a time, and timeout risk rises with serial chains deeper than about 5 dependencies — [MIRAGE](https://arxiv.org/html/2604.04806v3)
- ioco has compositional results: under conditions, conformance of components implies conformance of the composition — [Bijl/Rensink/Tretmans](https://ftp.cs.ru.nl/CSI/SoftwEng.FunctLang/papers/2003/bijm2003-compositional_testing_ioco.pdf); [arXiv 1303.1009](https://arxiv.org/pdf/1303.1009)

### Inferences
- "One port's behavior per LLM call", with callee summaries supplied as already-checked records, is well supported. It is the locally-succeeds regime. DafnyComp's lesson is that *summaries passed between units are the fragile part*. Each callee summary must be validated on its own before callers consume it (bottom-up checking), and the caller's call must be checked to use only cases the callee summary declares.
- Compositional conformance (ioco-style) gives a principled reason to validate per port: if each port's behavior conforms to its traces and tests, composition along wires inherits conformance under input-enabledness conditions. That argues for making every provided port input-enabled in the model, with an explicit `reject` or `unknown` case.

### Gaps
- No measured accuracy-versus-cost comparison of per-component and whole-system *model* generation was found.
- No study measures how the quality of callee summaries affects caller-model accuracy for LLM-generated behavioral models.

## 5. Incremental re-compilation and spec drift

### Takeaway
There is little research on keeping LLM-generated models in sync with changing code. The practice that works is to run conformance checks in CI (CCF trace validation; Quint MBT glue in CI) and use deterministic code-path-to-spec mapping to flag drift, with LLMs proposing updates rather than auto-overwriting. Semantic drift builds up over successive LLM-driven updates. LLMs are unreliable as standalone drift detectors.

### Cited Findings
- Cirstea et al. recommend running trace validation in CI to keep spec and implementation in sync, as CCF does — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2); [CCF smart casual verification](https://decentralizedthoughts.github.io/2025-05-23-smart-casual-verification)
- Informal Systems ran the code change from the spec diff (old spec, new spec, diff to the LLM), with MBT glue in CI as the regression check — [Quint blog](https://quint.sh/posts/llm_era)
- A 2026 paper on a verified spec layer for LLM-driven codebase maintenance reports that semantic drift between spec graph and code builds up over successive updates. It measures drift by regenerating a reference spec from the code and comparing fidelity (snippet-level) — [arXiv 2609.06383](https://arxiv.org/pdf/2609.06383)
- A study of LLM trust allocation across conflicting artifacts finds LLMs useful for Javadoc auditing but unreliable as standalone code-drift detectors. Conflicts that exist only in the implementation stay underdetected, and the authors recommend pairing LLMs with static, dynamic or symbolic analysis (snippet-level) — [arXiv 2604.03447](https://arxiv.org/pdf/2604.03447)
- Practitioner tools: SpecGov maps code paths to governing docs and flags PRs that change code without the expected spec change, deliberately without an LLM equivalence judgment ([DEV post](https://dev.to/paladini/your-specs-are-lying-how-to-detect-documentation-drift-in-every-pull-request-41p7)). An OpenSpec discussion argues that code-to-spec reconciliation is mechanical but spec overwrite is a decision, and proposes post-commit drift detection with clause-to-symbol links ([OpenSpec #169](https://github.com/Fission-AI/OpenSpec/discussions/169)). These are not peer-reviewed.
- Specula notes that bug counts for most systems come from earlier Specula versions because of budget, so re-running the whole pipeline per version is expensive (median $57 per project) — [Specula](https://arxiv.org/html/2607.25333v1)

### Inferences
- Whole-pipeline regeneration per commit is unaffordable at Specula-like cost, so incremental compilation (the Salsa-style memoization from the prior report) is necessary, not an optimization.
- A cheap drift signal has two layers:
  1. Deterministic: anchors re-resolved, and AST hashes of anchored slices changed.
  2. Behavioral: re-run the stored conformance suite, i.e. model-generated tests and recorded traces, against the new code.
  Only units whose anchors changed *or* whose conformance tests now fail need recompilation. A conformance failure without an anchor change points to a missed dependency (an unanchored effect), which is itself a useful compiler bug signal.

### Gaps
- No peer-reviewed system was found that incrementally regenerates only the affected parts of an LLM-generated formal model on code change, or that measures model-staleness rates over commit histories.

## 6. Benchmarks and datasets for code-to-model translation

### Takeaway
SysMoBench (11 artifacts, TLA+, automated conformance via trace validation) is the only benchmark found that scores *model-to-implementation conformance* automatically. Specula already saturates it: 100% with Opus-4.8, and 93% with Sonnet-4.6. Adjacent benchmarks score spec generation (DafnyComp, Verus-Bench, CloverBench) or simulator fidelity (MIRAGE, MirrorAPI-Bench). None covers web or microservice systems as executable models, polyglot stacks, or port and behavior models like ours.

### Cited Findings
- SysMoBench: 11 artifacts (concurrency primitives, Raft variants, ZooKeeper FLE, PGo-generated systems), TLA+, with automated syntax, runtime, conformance and invariant metrics — [SysMoBench](https://arxiv.org/html/2509.23130v3)
- Specula at 100% on SysMoBench with Opus-4.8 indicates saturation as of mid-2026 — [Specula](https://arxiv.org/html/2607.25333v1)
- DafnyComp (300 compositional Dafny programs) — [arXiv 2509.23061](https://arxiv.org/html/2509.23061v1). Verus-Bench (150 proof tasks) — [AutoVerus](https://arxiv.org/abs/2409.13082v3). CloverBench (textbook Dafny) — [Clover](https://arxiv.org/abs/2310.17807v4)
- MIRAGE's 110-scenario, 3-system service-simulation suite measures status and body-shape fidelity against the real service — [MIRAGE](https://arxiv.org/html/2604.04806v3). MirrorAPI-Bench covers 7,000+ APIs — [MirrorAPI](https://arxiv.org/pdf/2503.20527)
- Transition-level state-machine benchmarks: an OpenReview paper notes a gap in automated, at-scale measurement of graph-level transition-system accuracy — [OpenReview PDF](https://openreview.net/pdf/78c94ca9b034762063c81018a1fadc7d312a6e73.pdf)
- PSMBench (named in the assignment): **not found** in these searches; it could not be verified.

### Inferences
- We will need our own benchmark: a few gold models (login, queue-and-worker, CLI) with instrumented reference implementations, so conformance can be computed SysMoBench-style (the fraction of instrumented code actions admitted by the model) and per port.
- Contamination is a known risk: SysMoBench's PGo results show models leaning on familiar names. The gold set should include code with renamed identifiers to measure prior-driven hallucination, like the perturbation study in the prior report.

### Gaps
- No benchmark for code-to-statechart/XState, code-to-DEVS, or IaC-to-executable-model translation.
- No benchmark measures over-approximation (model behaviors the code cannot exhibit) automatically. SysMoBench's conformance measures only whether code behavior is admitted.

## 7. Recommendations for the compiler stage and its verification loop

### Takeaway
Treat the compiler like Specula scaled down to one port at a time: per-port units with checked callee summaries; evidence-anchored predicates and invariants before behaviors; a model-to-code mapping emitted at generation time so that instrumentation and trace alignment are mechanical; two-directional conformance (traces into the model, model-generated tests on the code); a repair loop capped at about 3 iterations and guarded against overfitting; and incremental recompilation keyed on anchors plus conformance regressions. The metrics should separate syntax, runtime and stuckness, conformance, and over-approximation, as SysMoBench does, plus repair, cost and drift metrics.

### Cited Findings (bases for the recommendations)
- Evidence-backed invariants first, which then set the abstraction level — [Specula](https://arxiv.org/html/2607.25333v1)
- A model-to-code mapping recorded during generation enables automated instrumentation — [Specula](https://arxiv.org/html/2607.25333v1). An LLM-produced model-name-to-code-name mapping is used in SysMoBench conformance — [SysMoBench](https://arxiv.org/html/2509.23130v3)
- Partial logs work if they record shared-state changes, but logging too little explodes the search — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2)
- Grain-of-atomicity mismatch is the dominant source of false divergences — [arXiv 2404.16075](https://arxiv.org/html/2404.16075v2); [CCF](https://decentralizedthoughts.github.io/2025-05-23-smart-casual-verification)
- MBT from an abstract model succeeded at MongoDB where trace-checking an abstract spec failed — [arXiv 2006.00915](https://arxiv.org/abs/2006.00915v1)
- 3–4 repair iterations capture most gains — [arXiv 2607.05197](https://arxiv.org/abs/2607.05197). Repair overfitting is real and is countered by independent invariant checks — [Specula](https://arxiv.org/html/2607.25333v1)
- Local success does not compose — [DafnyComp](https://arxiv.org/html/2509.23061v1)
- Real dependency source lifts simulator fidelity from 53–69% to 100% — [MIRAGE](https://arxiv.org/html/2604.04806v3)
- Mutation kill rate as a refinement trigger (85%) — [arXiv 2606.21451](https://arxiv.org/pdf/2606.21451)

### Inferences (concrete recommendations)

**A. Unit of compilation and order**
1. Compile one provided port at a time, bottom-up within a service: callees first, and a callee summary must pass its own checks before a caller may consume it. Go outside-in across services. Each unit's input is the handler slice, the extracted obligations (branches, effects, state writes), already-checked callee summaries, and packs.
2. Two passes per unit, in Specula's order: (i) predicates and invariants with anchors, i.e. what must be true and why, citing code, tests or comments; (ii) cases and transitions that refer only to those predicates. Invariants become the anti-overfitting oracle for repair.
3. Emit a model-to-code map with every transition, making the anchor the instrumentation point: a function entry or exit, a send or receive site, a state write site, and the abstract variables observable there. This is what turns trace validation into a mechanical step rather than a second LLM task. It is SysMoBench's name-mapping step done at compile time.

**B. Intermediate representations**
4. Keep the constrained record DSL from the prior report as the compiler target; do not target TLA+ directly. SysMoBench and Specula show syntax and config errors consume early iterations, and DSL schema checks remove that class. Keep an *export* to TLA+/Quint/P for exhaustive exploration and for reuse of TLC trace validation. Quint's typechecker, used after each edit, is the model for a fast compiler front end.
5. Make the grain of atomicity explicit per transition. Mark `atomic` and the lock, transaction or none, and allow declared stutter steps and composite steps, so that coarse logs can be aligned to fine models and the reverse. This is the direct remedy for the most common divergence cause across Cirstea, CCF, SysMoBench and Specula.
6. Make every provided port input-enabled: an explicit `reject` or `unknown` case for unmatched inputs. That way ioco-style quiescence and stuckness checks are meaningful and compositional conformance arguments apply.

**C. Checker-in-the-loop repair**
7. Order the checks cheapest first: schema → anchors → coverage obligations → guard algebra → interpreter run (stuck configurations = SysMoBench's "runtime") → conformance. Cap repair at 3 iterations per unit, then escalate to a stronger model or k-sample voting, or downgrade to `assumed` and surface the record.
8. Guard against repair overfitting: diff each repair, and reject or flag repairs that weaken a guard, add a catch-all transition, delete a case or hard-code values, unless the coverage obligations and invariants still hold. Record each repair's kind in `checks` or provenance.
9. When the model and the code diverge, do not auto-repair blindly. Classify the divergence as one of: model gap (fix the model), instrumentation or logging error, atomicity mismatch (fix the granularity), or code-bug candidate (show the user). This is Specula's stated open problem.

**D. Validation against the real code (two directions)**
10. Completeness, code → model: run existing test suites and recorded executions with lightweight instrumentation at the anchors, then align the traces in the interpreter in lockstep (PObserve/TLC style). A trace event with no enabled transition is a missing transition or guard.
11. Soundness, model → code: generate tests from the model (one per case or per input class, plus Specula-style scenario projections), execute them against the real code with stubs at required ports, and compare the observable outputs at ports. This is MBT/ioco and catches over-approximation and invented behavior. MongoDB's experience says this direction is the more practical one for abstract models.
12. Mutation adequacy: mutate the *model* (drop a case, negate a guard, merge two steps) and check that the conformance suite kills each mutant. Mutate the *code* at anchored sites and check that the model-based tests detect the change. Use the kill rate as a refinement trigger, with 85% as the one published threshold.
13. For ports whose implementation is unavailable (third-party services), an LLM simulator in the MIRAGE style can stand in during co-simulation, marked `assumed`. Prefer packs built from the real source where possible, given the 53–69% versus 100% body-shape fidelity gap.

**E. Incrementality**
14. Recompile a unit when its anchors' AST hashes change *or* its stored conformance tests or traces now fail. A failure without an anchor change is a compiler dependency bug, so log it as such. Run the conformance suite in CI as CCF and Quint do.

**F. Metrics to track** (per unit and per system; adapted from SysMoBench and Specula)
- Syntax/schema pass rate on first emission; check failures by tier.
- Stuck-configuration rate in interpreter runs (the analogue of SysMoBench runtime).
- Conformance: the fraction of instrumented code actions or trace events admitted by the model (SysMoBench M_c), per port.
- Over-approximation: the fraction of model-generated tests whose predicted outcome the real code contradicts.
- Coverage: extracted branches, effects and state writes accounted for; self-transitions, timeouts and error cases present (the known blind spots).
- Divergence taxonomy counts: atomicity, missing transition, wrong guard, invented behavior, textbook behavior, instrumentation error.
- Repair: iterations to pass; the fraction of repairs that weakened guards or removed cases; escalation rate; assumption rate.
- Mutation kill rate, for both model mutants and code mutants.
- Cost: tokens and wall time per unit and per system (Specula's $19–$168 per project is the only external reference).
- Drift: units invalidated per commit; conformance regressions without anchor change; ID and override survival.
- Robustness: conformance on identifier-renamed variants of the gold code (contamination and priors).

### Gaps
- None of these recommendations has been tested on HTTP/queue/DB application systems. All the conformance evidence comes from consensus protocols, concurrency primitives and three microservice demo systems (MIRAGE).
- No source reports how much instrumentation overhead or developer effort per-port trace capture costs in web stacks. OpenTelemetry spans may suffice at the coarse level, but this is unverified.
