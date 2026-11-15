# Paths, Input Classes, Loops and States: State of the Art (Nov 2026) for Scenario Graphs, Scripts and Assumptions

Scope note for the report writer. Sources marked [S] were seen in a search result or fetched page during this session (2026-11-11). Sources marked [K] are well-known primary papers or tool pages cited from the researcher's background knowledge. Their URLs are canonical DOI, venue or tool links, but they were not re-fetched this session, so spot-check a few before publication. Numbers are only given where a source stated them.

---

## Q1. Symbolic and concolic execution: how do they handle path explosion, and can path conditions be lifted to human-meaningful input classes?

### Takeaway
Classic engines (KLEE, SymCC, angr, Manticore, SPF) fight path explosion in four ways: search heuristics, state merging or veritesting, function and loop summaries, and concolic and compositional execution. None of them produces human-meaningful input classes on its own. Path conditions come out as solver formulas over bytes or bit-vectors. The newest line of work (2025–2026) keeps path constraints as code or natural language and lets an LLM reason over them, as in AutoBug (OOPSLA 2025). That is the closest fit to "input classes", but it is approximate. Lifting a path condition into a readable class therefore needs an explicit abstraction step: predicates named in domain terms. Raw solver output is not that step.

### Cited Findings
**Foundations and engines**
- Symbolic execution keeps a path condition (PC), a conjunction of branch predicates over symbolic inputs, for each explored path. Solving the PC yields a concrete test for that path. [K] — [King 1976, CACM](https://dl.acm.org/doi/10.1145/360248.360252)
- *Partition analysis* (1985) already proposed splitting the input domain into subdomains by combining path conditions with specification conditions. It is the direct ancestor of "input classes from path conditions". [K] — [Richardson & Clarke, IEEE TSE 1985](https://ieeexplore.ieee.org/document/1702003)
- Concolic (dynamic symbolic) execution runs the program concretely, collects the PC, and negates one branch at a time to reach new paths. [K] — [DART, PLDI 2005](https://dl.acm.org/doi/10.1145/1065010.1065036); [CUTE, FSE 2005](https://dl.acm.org/doi/10.1145/1081706.1081750); [SAGE, NDSS 2008](https://www.ndss-symposium.org/ndss2008/automated-whitebox-fuzz-testing/)
- KLEE runs symbolic execution on LLVM bitcode with environment models and search heuristics. It is still the reference academic engine. [K] — [Cadar, Dunbar, Engler, OSDI 2008](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/cadar/cadar.pdf); [KLEE site](https://klee-se.org/)
- SymCC compiles symbolic handling into the target binary instead of interpreting IR, and reports large speedups over KLEE and QSYM. [K] — [Poeplau & Francillon, USENIX Security 2020](https://www.usenix.org/conference/usenixsecurity20/presentation/poeplau)
- angr is a Python binary-analysis framework with pluggable "exploration techniques", including veritesting, loop limiting and DFS. [K] — [Shoshitaishvili et al., IEEE S&P 2016](https://ieeexplore.ieee.org/document/7546500); [angr docs](https://docs.angr.io/)
- Manticore does symbolic execution of native binaries and Ethereum smart contracts. [K] — [Mossberg et al., ASE 2019](https://arxiv.org/abs/1907.03890)
- Path explosion, loops, memory modelling and the environment are the main open problems. The standard survey catalogs the mitigations. [K] — [Baldoni et al., ACM CSUR 2018](https://arxiv.org/abs/1610.00502)

**Path-explosion mitigations**
- State merging joins states at control-flow join points into one state with disjunctive or ite constraints. *Query count estimation* decides when merging pays off, and *dynamic state merging* reconciles merging with search heuristics. [K] — [Kuznetsov et al., PLDI 2012](https://dl.acm.org/doi/10.1145/2254064.2254088)
- Veritesting alternates between dynamic symbolic execution (per path) and static symbolic execution (whole acyclic regions encoded as one formula), which merges paths through branchy, loop-free code. [K] — [Avgerinos et al., ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568293)
- Compositional symbolic execution computes function summaries (pre/post pairs over PCs) and reuses them at call sites instead of re-exploring callees. [K] — [Godefroid, POPL 2007 (SMART)](https://dl.acm.org/doi/10.1145/1190216.1190226); [Anand, Godefroid, Tillmann, TACAS 2008](https://link.springer.com/chapter/10.1007/978-3-540-78800-3_28)
- Differential symbolic execution compares the summaries of two program versions, which is a precedent for "diff two scripts after a fix". [K] — [Person et al., FSE 2008](https://dl.acm.org/doi/10.1145/1453101.1453131)
- Probabilistic symbolic execution uses model counting over PCs to give each path a probability. That gives a principled "most likely default branch". [K] — [Geldenhuys, Dwyer, Visser, ISSTA 2012](https://dl.acm.org/doi/10.1145/2338965.2338972)

**Language coverage relevant to TS/Python/Go/Java services**
- JavaScript: ExpoSE does dynamic symbolic execution of JS, including regex support. [K] — [Loring, Mitchell, Kinder, SPIN 2017](https://dl.acm.org/doi/10.1145/3092282.3092295). Java: Symbolic PathFinder. [K] — [jpf-symbc](https://github.com/SymbolicPathFinder/jpf-symbc). Python: CrossHair checks contracts symbolically. [K] — [CrossHair](https://github.com/pschanely/CrossHair). The researcher knows of no mature Go source-level symbolic engine (see Gaps).

**LLM-era symbolic execution (2025–2026)**
- AutoBug (OOPSLA 2025) is "LLM-powered symbolic execution". It splits analysis into path-based subtasks and represents path constraints in "a generic code-based representation that the LLM can directly reason over", with no translation to SMT. It is language-agnostic and helps smaller LLMs most. [S] — [Li, Meng, Duck, arXiv 2505.13452](https://arxiv.org/abs/2505.13452)
- PALM (2025) pairs symbolic path enumeration with LLM test generation. The LLM infers the constraints of library calls, so those APIs need no hand-written symbolic models. [S] — [arXiv 2506.19287](https://arxiv.org/pdf/2506.19287)
- An empirical study (Nov 2025) tests LLMs and reasoning models on solving path constraints for test generation and feasibility classification. [S] — [arXiv 2511.18288](https://arxiv.org/abs/2511.18288)
- A study of whether LLMs can imitate KLEE's output found that fine-tuned GPT-4o identified the most constrained path only about 20% of the time. The authors conclude LLMs cannot yet replace symbolic execution tools. [S] — [arXiv 2511.08530](https://arxiv.org/pdf/2511.08530)

**Lifting path conditions to readable form**
- DeltaDoc combined symbolic execution with summarization heuristics to generate natural-language descriptions of the path predicates under which behavior changed. It is an early precedent for PC-to-prose. [K] — [Buse & Weimer, ASE 2010](https://dl.acm.org/doi/10.1145/1858996.1859005)
- A Sandia report addresses turning angr's low-level constraints into human-readable path constraints, which shows the readability gap with binary-level engines. [S] — [OSTI 1767091](https://www.osti.gov/servlets/purl/1767091)

### Inferences
- The tool does not need full symbolic execution. It needs per-function symbolic summaries keyed by named predicates, a compositional, summary-based design in the style of SMART. A behavior pack's "graph per public function keyed by input classes" is a function summary whose PC disjuncts have been given names.
- Path conditions become explainable when each atomic predicate is anchored to a source expression (`token.exp < now`) and given a domain name ("expired token"). Classes are then Boolean combinations of named predicates. An LLM can propose the names, and the anchor keeps them checkable.
- Veritesting and state merging map onto the product's collapse rule. A branchy region that does not affect state or output can be merged and shown as one step with a disjunctive condition. A region that does affect them is kept as a fork.
- AutoBug's code-as-constraint representation suggests a hybrid for web stacks with no solver-grade engine (TS, Python, Go). Keep the predicate as a source snippet plus a name, let the LLM judge feasibility, and use a solver only where types are simple (ints, enums, string equality).
- Probabilistic symbolic execution and model counting give a principled default for unbound branches: the happy path is the class with the most valid inputs, or the one marked "success" by the return type.

### Gaps
- No mature open-source symbolic engine for Go source or for idiomatic async TypeScript at service scale was found this session. The landscape of Go and TS symbolic tools in 2026 is unverified.
- No benchmark found that measures how readable or correct LLM-named input classes are when derived from path conditions.

---

## Q2. Input space partitioning, equivalence classes, grammar inference, and web-API input specifications

### Takeaway
Category-partition and input space partitioning give the vocabulary the product needs: parameters, categories, choices, and constraints between choices. Path-condition partitioning grounds that vocabulary in code. For structured inputs, grammar mining (Mimid, Arvada, Kedavra, symbolic parsing) plus semantic constraints (ISLa or Fandango) define classes over syntax. For web services, OpenAPI-driven tools (RESTler, Schemathesis, EvoMaster) and the 2025–2026 LLM REST testers infer parameter classes and producer–consumer dependencies. Those dependencies are exactly the cross-request state edges a scenario graph needs.

### Cited Findings
**Category-partition and ISP**
- The category-partition method splits each functional unit's parameters and environment into categories. Each category is split into choices, and constraints (properties and selectors such as `[error]`, `[single]`, `[if X]`) prune infeasible combinations. A test frame is one choice per category. [K] — [Ostrand & Balcer, CACM 1988](https://dl.acm.org/doi/10.1145/62959.62964)
- Input space partitioning distinguishes interface-based characteristics (by parameter) from functionality-based ones (by behavior). Coverage criteria include each-choice, base-choice and pair-wise. *Base choice* is a designated default per characteristic that other choices vary from. [K] — [Ammann & Offutt, Introduction to Software Testing](https://cs.gmu.edu/~offutt/softwaretest/)
- Combinatorial (t-way) testing tools such as NIST ACTS generate covering arrays over choices with constraints. [K] — [NIST ACTS](https://csrc.nist.gov/projects/automated-combinatorial-testing-for-software)

**Grammar and input-structure inference**
- Mimid mines context-free grammars by tracking how a recursive-descent parser's control flow consumes input. [K] — [Gopinath, Mathis, Zeller, ESEC/FSE 2020](https://arxiv.org/abs/1912.05937). A later comparison on the Panini benchmark reports ~97% precision, ~81% recall and a 67% success rate. [S] — [arXiv 2503.08486](https://arxiv.org/pdf/2503.08486)
- Arvada learns grammars black-box from sample inputs and a parse oracle. [K] — [Kulkarni, Lemieux, Sen, ASE 2021](https://arxiv.org/abs/2108.13340). Follow-ups criticize its accuracy, speed and readability, and name Kedavra as the black-box state of the art. [S] — [Kedavra, arXiv 2408.16706](https://arxiv.org/pdf/2408.16706)
- Static inference of regular grammars for ad hoc parsers (Panini) extracts input formats without running the program. [S] — [Schröder & Cito preprint](https://mcschroeder.github.io/files/panini_preprint.pdf)
- *Symbolic parsing* infers input grammars from code (2025, ACM journal DOI 10.1145/3776743). [S] — [arXiv 2503.08486](https://arxiv.org/pdf/2503.08486); [ACM](https://dl.acm.org/doi/10.1145/3776743)
- 2026 work learns context-free grammars with an agentic LM loop and guarantees, and reports that Arvada needs ~63k oracle calls and ~14.6 h per run in its setup. [S] — [arXiv 2608.05493](https://arxiv.org/html/2608.05493). Readability-focused black-box inference also exists. [S] — [arXiv 2509.26616](https://arxiv.org/pdf/2509.26616)
- ISLa attaches first-order semantic constraints to grammars, which allows classes such as "a request whose `length` field matches the body". [K] — [Steinhöfel & Zeller, ESEC/FSE 2022](https://dl.acm.org/doi/10.1145/3540250.3549139). Its successor Fandango (CISPA, 2025) uses evolutionary search over grammar plus constraints. [K] — [Fandango](https://fandango-fuzzer.github.io/)

**Web API input specifications**
- RESTler analyzes an OpenAPI spec to infer producer–consumer dependencies (a response field feeds a later request parameter). It generates stateful request sequences and learns from response codes. [K] — [Atlidakis, Godefroid, Polishchuk, ICSE 2019](https://www.microsoft.com/en-us/research/publication/restler-stateful-rest-api-fuzzing/)
- Schemathesis derives property-based fuzzers from OpenAPI/GraphQL schemas, and uses OpenAPI links for stateful sequences. [K] — [Hatfield-Dodds & Dygalo, ICSE 2022](https://arxiv.org/abs/2112.10328); [docs](https://schemathesis.readthedocs.io/)
- EvoMaster does white-box search-based REST/GraphQL test generation (JVM, with SQL-aware heuristics). [K] — [EvoMaster](https://github.com/WebFuzzing/EvoMaster)
- Morest builds a "RESTful-service property graph" of operations, schemas and dependencies, and updates it dynamically. [K] — [Liu et al., ICSE 2022](https://dl.acm.org/doi/10.1145/3510003.3510133). An empirical comparison of REST testers found coverage still limited by dependencies and input values. [K] — [Kim et al., ISSTA 2022](https://dl.acm.org/doi/10.1145/3533767.3534401)
- LLM-era REST testing:
  - AutoRestTest combines a semantic property-dependency graph, LLM agents and multi-agent RL. [S] — [arXiv 2501.08600](https://arxiv.org/pdf/2501.08600)
  - LlamaRestTest fine-tunes a small Llama3-8B on parameter values and inter-parameter dependencies, and refines them from server responses. [S] — [arXiv 2501.08598](https://arxiv.org/pdf/2501.08598)
  - APIPilot (2026) treats LLM-inferred producer–consumer dependencies as *hypotheses* and verifies them with live calls before building sequences. Evaluated on 16 services against DeepREST, AutoRestTest, RESTifAI and KAT. A commentary flags that results on stateful services that are not reset may reflect one session's state. [S] — [arXiv 2608.17546](https://arxiv.org/html/2608.17546v2); [Pith commentary](https://pith.science/paper/2608.17546)

### Inferences
- An input in the product's model maps onto category-partition directly. Each input port (route, CLI, UI event) has characteristics, such as credentials, token and payload shape. Each characteristic has choices ("valid", "expired", "malformed", "missing"), and selector constraints say which choices are error-only or depend on others. Classes carry a `[error]`-style tag and a designated base choice, which becomes the default binding for unbound branches, recorded as an assumption.
- Choices should come from three places, each recorded as provenance: (1) the schema (OpenAPI/JSON Schema/zod/pydantic/protobuf: required, enum, format, min/max); (2) code path conditions (validator branches, `if (!user)`); (3) LLM naming. Schema-derived classes are cheap and reliable. Path-derived classes catch what the schema leaves implicit.
- RESTler and APIPilot dependency edges (a POST /orders response feeds GET /orders/{id}) are cross-request data-flow edges. In the scenario graph they are state-carrying edges between scripts, and APIPilot's verify-the-LLM-hypothesis loop is a template for grading assumptions.
- Grammar mining matters mostly for behavior packs below the app layer (parsers, protocol messages). For typical JSON services, schema types plus validators are enough.

### Gaps
- No study found that evaluates LLM-generated equivalence classes against category-partition specs written by experts.
- How well TS validator libraries (zod, io-ts) and Python pydantic models translate automatically into ISP characteristics is unstudied in the sources found. It is likely straightforward engineering.

---

## Q3. Specification mining and typestate inference (Daikon, Perracotta, Synoptic/CSight, k-tails, LLM variants)

### Takeaway
Specification mining is mature for traces (k-tails, Synoptic, CSight, GK-tail/MINT for extended FSMs with data guards) and for invariants (Daikon). Typestate or API-protocol mining from code or traces is well established but mostly dates from 2002–2018. The LLM era (2024–2026) has moved effort to generating pre/postconditions and formal specs from code (SpecGen, nl2postcond and successors) and to state machines from protocol code or RFCs (Q4). No mature LLM typestate miner was found. For the product, the most reusable ideas are GK-tail/MINT style EFSMs, where transitions carry data guards inferred from invariants, and Synoptic's invariant-preserving refinement.

### Cited Findings
- Ammons, Bodík and Larus coined "specification mining": learning temporal API rules as probabilistic automata from traces. [K] — [POPL 2002](https://dl.acm.org/doi/10.1145/503272.503275)
- Daikon infers likely invariants (pre/postconditions, object invariants) from execution traces by checking a grammar of candidate properties. [K] — [Daikon](https://plse.cs.washington.edu/daikon/); [Ernst et al., Sci. Comput. Program. 2007](https://doi.org/10.1016/j.scico.2007.01.015)
- k-tails merges states whose future suffixes up to length k agree, and is the classic passive FSM inference from traces. [K] — [Biermann & Feldman, IEEE TC 1972](https://ieeexplore.ieee.org/document/5008985)
- Whaley, Martin and Lam statically and dynamically extract per-class method-call protocol FSMs, splitting states by which fields a method reads and writes. That is a direct precedent for state-field-based typestate. [K] — [ISSTA 2002](https://dl.acm.org/doi/10.1145/566172.566212)
- Alur et al. synthesize Java class interface specifications (typestate) with L* and model checking. [K] — [POPL 2005](https://dl.acm.org/doi/10.1145/1040305.1040314)
- Perracotta mines temporal properties (alternating patterns such as acquire/release) from imperfect traces at scale. [K] — [Yang et al., ICSE 2006](https://dl.acm.org/doi/10.1145/1134285.1134325)
- GK-tail infers EFSMs, FSMs whose transitions carry data constraints inferred with Daikon, from interaction traces. [K] — [Lorenzoli, Mariani, Pezzè, ICSE 2008](https://dl.acm.org/doi/10.1145/1368088.1368157). MINT infers EFSMs with guards learned by data classifiers. [K] — [Walkinshaw, Taylor, Derrick, EMSE 2016](https://link.springer.com/article/10.1007/s10664-015-9367-7)
- Synoptic mines temporal invariants from logs, then refines a coarse model until it satisfies them (CEGAR-style), producing compact, invariant-respecting FSMs from logs. [K] — [Beschastnikh et al., ESEC/FSE 2011](https://dl.acm.org/doi/10.1145/2025113.2025151)
- CSight infers *communicating* FSMs (one per process, linked by channels) from distributed-system logs. [K] — [Beschastnikh et al., ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568246)
- Deep Specification Miner (DSM) uses RNNs plus clustering to infer FSMs from traces. [K] — [Le & Lo, ISSTA 2018](https://dl.acm.org/doi/10.1145/3213846.3213876)
- Survey of API property inference (sequencing, typestate, behavioral). [K] — [Robillard et al., IEEE TSE 2013](https://ieeexplore.ieee.org/document/6311409)
- Typestate was introduced as states attached to types that restrict which operations are legal. [K] — [Strom & Yemini, IEEE TSE 1986](https://ieeexplore.ieee.org/document/6312929)
- LLM spec generation:
  - SpecGen generates JML specifications with an LLM and refines them with a verifier. [K] — [Ma et al., ICSE 2025, arXiv 2401.08807](https://arxiv.org/abs/2401.08807)
  - nl2postcond turns natural-language intent into checkable postconditions. [K] — [Endres et al., FSE 2024, arXiv 2310.01831](https://arxiv.org/abs/2310.01831)
  - A 2025 evaluation tests whether LLMs can infer formal specifications. [S] — [arXiv 2503.04779](https://arxiv.org/html/2503.04779)
  - Spec generation improved with program slicing and "logical deletion". [S] — [arXiv 2509.09917](https://arxiv.org/pdf/2509.09917)
  - A 2026 study of logical constructs in LLM-generated specs. [S] — [arXiv 2602.00715](https://arxiv.org/pdf/2602.00715)
  - KBSpec builds formal specs with an evolving knowledge base. [S] — [arXiv 2606.21339](https://arxiv.org/pdf/2606.21339)
  - Specs synthesized from tests alone. [S] — [arXiv 2608.13240](https://arxiv.org/pdf/2608.13240)

### Inferences
- Mined typestates are entity lifecycle states. For a class or ORM model, states are equivalence classes of field valuations, and transitions are the methods or routes that change them. Whaley's field-partition heuristic and GK-tail's data guards together give "state + guarded transition", which is what an EFSM is.
- Daikon-style invariants at each scenario-graph node are a good way to attach "what is true here" annotations, and to check LLM-claimed input classes against observed traces in producer 2.
- Synoptic and CSight are the right precedent for producer 2: refine the static scenario graph with observed logs while keeping the mined invariants. CSight's communicating FSMs match the product's multi-service lanes.

### Gaps
- No 2024–2026 paper was found that targets *LLM-based typestate or API-protocol mining* for general libraries. LLM work concentrates on Hoare-style specs and protocol FSMs. This looks like a real research gap rather than a search miss, but one search is not conclusive.

---

## Q4. Protocol state machine inference: active learning, passive learning, state-aware fuzzing, implementations and RFCs

### Takeaway
Active automata learning (L*, TTT, L#) via LearnLib or AALpy is the gold standard for black-box protocol state machines. It found real bugs in TLS, TCP, SSH and DTLS, but it needs a hand-built abstract input alphabet and mapper. State-aware fuzzing (AFLNet, StateAFL, SGFuzz, ChatAFL) infers states on the fly, either from response codes or from enum-typed state variables in the code. Since 2024, LLMs extract state machines from RFCs (RFCNLP, then PSMBench, FlowFSM, RFC2TLA+) and from implementation source (ProtocolGPT). Quality varies by protocol, and the best 2026 pipelines use multi-pass extraction constrained by tool calls. For the product's RFC behavior packs, LLM extraction from the RFC plus verification against an implementation (active learning or traces) is the state of the art.

### Cited Findings
**Active learning**
- L* learns a minimal DFA from membership and equivalence queries. [K] — [Angluin 1987](https://doi.org/10.1016/0890-5401(87)90052-6). A practitioner overview of model learning appeared in CACM. [K] — [Vaandrager, CACM 2017](https://dl.acm.org/doi/10.1145/2967606)
- L# is a newer active learning algorithm based on apartness. [K] — [Vaandrager et al., TACAS 2022](https://arxiv.org/abs/2107.05419)
- LearnLib is the main Java library (L*, TTT and others). [K] — [Isberner, Howar, Steffen, CAV 2015](https://link.springer.com/chapter/10.1007/978-3-319-21690-4_32); [learnlib.de](https://learnlib.de/)
- AALpy is a Python library for active learning of DFAs, Mealy and Moore machines with L*, KV and L#, plus visibly-pushdown and stochastic models (MDPs, stochastic Mealy). The latest release seen is 1.5.3 (Dec 2025), and a 2025 extension adds passive state-merging learning. [S] — [PyPI aalpy](https://pypi.org/project/aalpy/); [passive extension](https://www.researchgate.net/publication/393948259_Extending_AALpy_with_Passive_Learning_A_Generalized_State-Merging_Approach). One comparison notes LearnLib is slightly faster for deterministic automata. [S] — (same search; [wcventure overview](https://github.com/wcventure/Active-Automata-Learning/blob/master/README.md))
- Recent algorithmic work includes compositional learning through alphabet refinement [S] — [arXiv 2504.16624](https://arxiv.org/pdf/2504.16624), and learning with advice. [S] — [arXiv 2508.10535](https://arxiv.org/pdf/2508.10535)
- Symbolic automata (transitions labeled by predicates rather than letters) can be learned actively. [K] — [Argyros & D'Antoni, CAV 2018](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_23)

**Implementation case studies (foundational)**
- *Protocol state fuzzing* of TLS implementations with LearnLib found state-machine bugs in several TLS libraries (e.g., GnuTLS, OpenSSL). [K] — [de Ruiter & Poll, USENIX Security 2015](https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/de-ruiter)
- The same approach was applied to TCP [K] — [Fiterău-Broştean et al., CAV 2016](https://link.springer.com/chapter/10.1007/978-3-319-41540-6_25), SSH [K] — [SPIN 2017](https://dl.acm.org/doi/10.1145/3092282.3092289), and DTLS. [K] — [USENIX Security 2020](https://www.usenix.org/conference/usenixsecurity20/presentation/fiterau-brostean)

**State-aware fuzzing**
- AFLNet treats server response codes as states, builds a state model while fuzzing, and prioritizes rarely visited states. [K] — [Pham, Böhme, Roychoudhury, ICST 2020](https://mboehme.github.io/paper/ICST20.AFLNet.pdf)
- StateAFL infers states from snapshots of long-lived memory. [K] — [Natella, EMSE 2022](https://arxiv.org/abs/2110.06253)
- SGFuzz identifies *state variables* statically as variables of enum type assigned named constants, then builds a state-transition tree at runtime. This is a cheap, effective heuristic for "status field = state". [K] — [Ba et al., USENIX Security 2022](https://www.usenix.org/conference/usenixsecurity22/presentation/ba)
- ChatAFL uses an LLM to extract message grammars and state hints from RFC knowledge to guide AFLNet. [K] — [Meng et al., NDSS 2024](https://www.ndss-symposium.org/ndss-paper/large-language-model-guided-protocol-fuzzing/)
- Controlled static loop analysis can extract protocol message formats as state machines from parser code. [S] — [arXiv 2305.13483](https://arxiv.org/pdf/2305.13483)

**From implementation source with LLMs**
- ProtocolGPT (IWQoS 2025) chunks protocol source code and prompts an LLM to identify states, message types and transitions. On six implementations it reports over 90% precision, beats baselines by more than 30%, raises fuzzing coverage by over 20%, and found two 0-days. [S] — [arXiv 2405.00393](https://arxiv.org/abs/2405.00393v2); [ACM DL](https://dl.acm.org/doi/10.1109/IWQoS65803.2025.11143461)

**From RFCs and specs**
- RFCNLP (S&P 2022) used NLP (BERT-based tagging) to extract FSMs from RFC text (TCP, DCCP, among others) and synthesize attacks. [K] — [Pacheco et al., arXiv 2202.09470](https://arxiv.org/abs/2202.09470)
- Hermes extracted FSMs from 4G/5G NAS/RRC specifications. [K] — [Al Ishtiaq et al., USENIX Security 2023](https://www.usenix.org/conference/usenixsecurity23/presentation/al-ishtiaq)
- PSMBench / RFC2PSM (NeurIPS 2025 Datasets & Benchmarks) pairs 1,580 pages of RFC text with 108 manually validated states and 297 transitions across 14 protocols. LLM output is scored with fuzzy-matching metrics. [S] — [NeurIPS poster](https://neurips.cc/virtual/2025/loc/san-diego/poster/121835); [paper PDF](https://proceedings.neurips.cc/paper_files/paper/2025/file/521bd9583b3a39e92f662ee57e81e5ce-Paper-Datasets_and_Benchmarks_Track.pdf)
- FlowFSM (2025) is an agentic prompt-chaining framework for FSM extraction from RFCs (e.g., FTP, RFC 959). [S] — [arXiv 2507.11222](https://arxiv.org/abs/2507.11222). There is also a RAG plus chain-of-thought variant. [S] — [arXiv 2502.15727](https://arxiv.org/pdf/2502.15727)
- 3GPP spec FSM extraction with domain prompts and LLM ensembles notes that specs use vague state references ("any state"). [S] — [arXiv 2510.14348](https://arxiv.org/pdf/2510.14348)
- RFC2TLA+ (ASE 2026; Ding, Li, Grishchenko, Lie, U. Toronto) extracts communicating FSMs and invariants from RFCs in three passes (states, transitions, invariants). The LLM is constrained by tool calls, and the output targets TLA+ for verification. The search snippet reports average state F1 of 0.97. Its bibliography cites PSMBench, AALpy, LearnLib and CPAchecker. [S] — [paper PDF](https://security.csl.toronto.edu/wp-content/uploads/2026/08/gding-ase2026-rfc2tla.pdf) (full text could not be extracted, so transition F1 and protocols are unverified)
- RFCLLM (Sept 2026) evaluates LLM reasoning over protocol state machines (TCP, SCTP, DCCP, RTSP). LLM-extracted results are notably weaker than manual extraction, especially for RTSP. [S] — [arXiv 2609.13389](https://arxiv.org/html/2609.13389)

**Passive learning from logs**
- State-merging tools (RPNI and EDSM/blue-fringe) are packaged in flexfringe. [K] — [Verwer & Hammerschmidt, ICSME 2017](https://github.com/tudelft-cda-lab/FlexFringe). See also Synoptic and CSight in Q3.

### Inferences
- RFC behavior packs (TLS 1.3, TCP, HTTP/2 streams) should be built with a RFC2TLA+-style pipeline: multi-pass, tool-constrained LLM extraction into a communicating FSM. Then *verify against implementations*, by active learning on a real library (AALpy/LearnLib) or by comparing with fuzzer-observed states. Disagreements are themselves useful pack content (implementation-specific deviations).
- The mapper (abstract to concrete inputs) that active learning needs is the same thing as the product's input classes. One artifact can serve both: a class such as "ClientHello with an unsupported group" is an abstract input symbol.
- SGFuzz's enum-state-variable heuristic and ProtocolGPT's chunked LLM extraction together form a practical static recipe for user code: find enum or status-typed fields, collect their assignment sites, and let the LLM label the guards.

### Gaps
- No work found that applies active automata learning to *multi-service web applications* end to end (beyond single REST APIs). The input-alphabet and reset problems for stateful web services are noted in the APIPilot commentary but not solved.
- RFC2TLA+ transition-level accuracy and the protocol list are unverified (PDF text not extractable this session).

---

## Q5. Static detection of persistent state and its transitions; shared mutable state and races

### Takeaway
Persistent "data state" is best found from declared schemas (ORM models, migrations, Prisma/SQLAlchemy/JPA/GORM) plus enum or status-typed columns and their write sites. Research covers *data constraints* in ORM apps (Yang et al. ICSE 2020, ConstrOpt) but not automated extraction of status-field state machines. Where libraries declare state machines (XState, django-fsm, AASM, Spring Statemachine, python-statemachine), extraction is direct. For races, RacerD (static, compositional, lock- and ownership-based) and lockset and happens-before detectors (Eraser, FastTrack, TSan, Go's -race) are mature for threads. For web systems the relevant races are mostly *database-level* (feral concurrency, ACIDRain) and *event-loop* races (NodeRacer). Those need different models.

### Cited Findings
**Persistent state and ORM**
- In a study of real ORM web apps, data constraints are expressed in the DB schema, in ORM validations and in front-end code. Most were added after the first commit, and 138 constraints conflicted between application code and schema. [S] — [Yang et al., ICSE 2020](https://people.cs.uchicago.edu/~shanlu/paper/ICSE20.pdf)
- ConstrOpt statically extracts candidate data constraints from web-app source, mainly from ORM validation APIs, to optimize queries. [S] — [arXiv 2205.02954](https://arxiv.org/pdf/2205.02954)
- Older work formalizes *dynamic* (transition) constraints in ORM as static constraints over the transaction history. [S] — [Modeling Dynamic Rules in ORM](https://www.researchgate.net/publication/220830071_Modeling_Dynamic_Rules_in_ORM). Smalldb proposed REST "state machines over SQL" and noted that static analysis of the state function could check stored states against the machine definition (design proposal, older). [S] — [Restful State Machines and SQL Database](https://www.researchgate.net/publication/336156559_Restful_State_Machines_and_SQL_Database)
- Libraries that declare state machines make extraction trivial:
  - python-statemachine can build states from an Enum. [S] — [docs](https://python-statemachine.readthedocs.io/en/latest/states.html)
  - Laravel enum state machines declare allowed transitions per enum. [S] — [TamkeenTech](https://github.com/TamkeenTech/laravel-enum-state-machine)
  - Ruby stateful_enum puts transitions on ActiveRecord enums. [S] — [stateful_enum](https://github.com/amatsuda/stateful_enum)
  - Other examples. [K] — [XState/Stately docs](https://stately.ai/docs); [django-fsm](https://github.com/viewflow/django-fsm); [AASM](https://github.com/aasm/aasm); [Spring Statemachine](https://spring.io/projects/spring-statemachine)
- No tool was found that statically *infers* state machines from undeclared enum or status fields in business code. Results were limited to declared-in-code libraries. [S] — (search result set above)
- The closest static heuristic is SGFuzz's enum-typed state-variable identification (Q4) [K] — [Ba et al. 2022](https://www.usenix.org/conference/usenixsecurity22/presentation/ba), and Whaley et al.'s field-based state partitioning (Q3). [K] — [ISSTA 2002](https://dl.acm.org/doi/10.1145/566172.566212)

**Shared mutable state and races**
- Eraser's lockset algorithm flags shared variables not consistently protected by a common lock. [K] — [Savage et al., ACM TOCS 1997](https://dl.acm.org/doi/10.1145/265924.265927)
- FastTrack is an efficient precise happens-before (vector clock) detector. [K] — [Flanagan & Freund, PLDI 2009](https://dl.acm.org/doi/10.1145/1542476.1542490). ThreadSanitizer is the production hybrid detector. [K] — [Serebryany & Iskhodzhanov, WBIA 2009](https://dl.acm.org/doi/10.1145/1791194.1791203). Go ships a TSan-based dynamic race detector (`-race`). [K] — [Go race detector](https://go.dev/doc/articles/race_detector)
- RacerD is a static, compositional race detector (in Infer) for Java/C++. It uses ownership and lock summaries without whole-program alias analysis, is deployed at Meta on Android code, and favors few false positives. [K] — [Blackshear et al., OOPSLA 2018](https://dl.acm.org/doi/10.1145/3276514)
- Node.js event races (async callbacks interleaving on one thread) need event-level happens-before. NodeRacer detects them dynamically. [K] — [Endo & Møller, ICST 2020](https://cs.au.dk/~amoeller/papers/noderacer/paper.pdf)
- Application-level ("feral") concurrency control, such as Rails uniqueness and association validations, is unsafe under concurrent requests without DB constraints. [K] — [Bailis et al., SIGMOD 2015](https://dl.acm.org/doi/10.1145/2723372.2737784). ACIDRain finds concurrency anomalies in database-backed e-commerce apps by analyzing transaction traces at weak isolation levels. [K] — [Warszawski & Bailis, SIGMOD 2017](https://dl.acm.org/doi/10.1145/3035918.3064037)

### Inferences
- The product should treat persistent state as first-class *state entities*. Each table or collection, cache key or queue is an entity whose ports are read, write and subscribe. Status columns (enum, string with a small closed set, or a boolean cluster such as `verified_at IS NULL`) become lifecycle state machines. Transitions are found by static search for write sites (`order.status = 'PAID'`, `UPDATE ... SET status`) with their guarding path conditions, which gives guarded EFSM transitions per entity.
- Races in web systems are mostly check-then-act across requests on shared DB rows (feral concurrency), not memory races. The product's "parallel lanes" should therefore flag pairs of scripts that read and write the same state entity without a DB constraint or transaction guarding the condition. That is a lockset-style rule lifted to DB rows, with ACIDRain and feral concurrency as the evidence base. RacerD suits Java services, Go's -race suits observed runs, and Node needs event-race reasoning.

### Gaps
- No peer-reviewed tool for static inference of status-field state machines in TS/Python/Go/Java business code was found. Treat this as an open niche the product would fill with LLM help.
- No static race detector for cross-service DB anomalies in TS, Python or Go was verified this session.

---

## Q6. Which formalism best captures "finite graph with cycles + branches labeled by input-class conditions + summarized loops"?

### Takeaway
No single textbook formalism covers it. The best fit is a **hierarchical, communicating extended/symbolic automaton**. Each entity's scenario graph is a control-flow automaton or EFSM whose edges carry guards drawn from a finite vocabulary of named predicates (symbolic automata over an input-class alphabet). It is hierarchical in the statecharts sense, so collapsed segments and loop summaries are composite states with a summary relation. Composition across services is communicating machines (CSight, P, RFC2TLA+), or global choreographies that project to per-service local behavior. A script is a run: a partial order (an MSC/pomset) through the product of these machines under a binding of predicates.

### Cited Findings
- **EFSMs** add variables and guards and updates to FSM transitions. GK-tail and MINT infer them (Q3), and they are the standard model in protocol and model-based testing. [K] — [Lorenzoli et al., ICSE 2008](https://dl.acm.org/doi/10.1145/1368088.1368157)
- **Symbolic automata** label transitions with predicates from an effective Boolean algebra, so alphabets can be huge or infinite and transitions stay finite. Minimization and learning algorithms exist. [K] — [D'Antoni & Veanes, CAV 2017](https://link.springer.com/chapter/10.1007/978-3-319-63387-9_3); [Argyros & D'Antoni, CAV 2018](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_23)
- **Control-flow automata** (CFAs) with predicate abstraction and CEGAR are the representation in BLAST and CPAchecker. Nodes are program locations and edges are assignments or assumptions (`[x>0]`). The word "assume" edge is literally the product's notion. [K] — [Henzinger et al., POPL 2002 (lazy abstraction)](https://dl.acm.org/doi/10.1145/503272.503279); [CPAchecker](https://cpachecker.sosy-lab.org/)
- **Trace partitioning** in abstract interpretation keeps separate abstract states per chosen partition of execution history (e.g., per branch outcome) while staying finite. It is the formal version of "keep forks that matter, merge the rest". [K] — [Rival & Mauborgne, TOPLAS 2007](https://dl.acm.org/doi/10.1145/1275497.1275501)
- **Statecharts** add hierarchy, orthogonal regions (parallelism) and history to FSMs, and SCXML is their standard interchange format. Modern JS/TS tooling (XState) implements them. [K] — [Harel 1987](https://doi.org/10.1016/0167-6423(87)90035-9); [W3C SCXML](https://www.w3.org/TR/scxml/); [Stately docs](https://stately.ai/docs)
- **Communicating state machines**: the P language models distributed systems as communicating state machines and is used in industry (AWS). [K] — [P language](https://p-org.github.io/P/); [Desai et al., PLDI 2013](https://dl.acm.org/doi/10.1145/2491956.2462184). CSight infers CFSMs from logs. [K] — [ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568246). RFC2TLA+ targets communicating FSMs plus invariants in TLA+. [S] — [ASE 2026 PDF](https://security.csl.toronto.edu/wp-content/uploads/2026/08/gding-ase2026-rfc2tla.pdf)
- **Multiparty session types** give a global protocol (choreography) that projects to each participant's local type. That is a precedent for one system-level scenario graph that projects onto per-entity lanes. [K] — [Honda, Yoshida, Carbone, POPL 2008](https://dl.acm.org/doi/10.1145/1328438.1328472)
- **Symbolic execution trees** with summaries (King 1976, SMART) are the natural *path-condition tree*, but they are acyclic, so loops must be cut or summarized. [K] — [King 1976](https://dl.acm.org/doi/10.1145/360248.360252); [SMART](https://dl.acm.org/doi/10.1145/1190216.1190226)
- **Loop summarization** in symbolic execution:
  - LESE introduces trip-count variables and links them to input fields such as lengths. [K] — [Saxena et al., ISSTA 2009](https://dl.acm.org/doi/10.1145/1572272.1572299)
  - Partial loop summaries are inferred as pre/postconditions over induction variables. [K] — [Godefroid & Luchaup, ISSTA 2011](https://dl.acm.org/doi/10.1145/2001420.2001425)
  - Proteus classifies loops by path-interleaving pattern and summarizes them as disjunctive formulas or automata. [K] — [Xie et al., FSE 2016](https://dl.acm.org/doi/10.1145/2950290.2950340)
  - The CSUR survey covers loop handling broadly. [K] — [Baldoni et al. 2018](https://arxiv.org/abs/1610.00502)

### Inferences
- **Recommended core formalism:** a *hierarchical guarded control-flow automaton per entity*.
  - Locations are steps or segments.
  - Edges carry a guard (a Boolean formula over named predicates) and an effect (port send/receive, state write).
  - Composite locations hold collapsed segments and loops, each with a summary: entry guard, exit condition, trip-count class, aggregate effects.
  - Entities compose by message passing over ports (CFSM/P-style).
  - Cycles stay as back-edges, and unrolling is on demand.
- **Guards over named predicates, not raw formulas**: a symbolic automaton whose Boolean algebra is the finite set of input-class and state predicates. The same structure then supports SMT checks (feasibility, conflicting assumptions), model counting (likelihood) and learning (L*/symbolic L*) during producer 2.
- **Scripts** are runs of the product automaton under a *binding*, a partial assignment of predicates to true or false. A run is represented as a pomset/MSC (events plus happens-before), which matches the `after:` field. Unbound predicates encountered on the run become assumptions, each defaulting to a base choice.
- Statecharts and SCXML are a good *interchange and visualization* target for lifecycle and protocol states because skins and tools already understand them. They are a poor fit for data-flow-heavy scenario graphs, which are better as CFAs.

### Gaps
- No existing formalism or tool found that unifies data-flow-rich scenario graphs, lifecycle state machines and cross-service partial orders in one artifact. The recommended combination is a design inference, not a validated standard.

---

## Q7. Concrete recommendations for representing input classes, branch conditions, assumptions, loops and states

### Takeaway
Represent everything over a shared, anchored **predicate vocabulary**. Input classes, branch guards, assumptions and states are all formulas over named predicates, and each predicate is tied to a source expression and a provenance. That makes the model analyzable (SMT, model counting, diffing, learning) and explainable (names plus anchors). The recommendation is synthesized from the findings above, so each point names what it rests on.

### Cited Findings
(Recommendations are inferences. Each lists the findings it rests on.)
- **Predicates**: `{id, name: "token expired", expr: "payload.exp < now()", anchor, type: input|config|state|env, provenance, source: schema|path|llm|pack}`. This follows anchored path conditions (King; DeltaDoc), AutoBug's code-form constraints, and CFA assume-edges. — [AutoBug](https://arxiv.org/abs/2505.13452); [DeltaDoc](https://dl.acm.org/doi/10.1145/1858996.1859005); [BLAST](https://dl.acm.org/doi/10.1145/503272.503279)
- **Input classes** are category-partition *choices*: each class is a conjunction of predicates on one input characteristic, with tags `[error]`, `[base]` and `[single]` and inter-choice constraints. Characteristics are seeded from schemas (OpenAPI, zod, pydantic, protobuf) and refined from validator path conditions. — [Ostrand & Balcer](https://dl.acm.org/doi/10.1145/62959.62964); [Schemathesis](https://arxiv.org/abs/2112.10328); [RESTler](https://www.microsoft.com/en-us/research/publication/restler-stateful-rest-api-fuzzing/)
- **Branch conditions** are guards over predicates. Branches whose arms neither affect state, output nor ports, nor carry an assumption, are merged in the style of veritesting or state merging and displayed as one step with a disjunctive note. — [Veritesting](https://dl.acm.org/doi/10.1145/2568225.2568293); [Kuznetsov et al.](https://dl.acm.org/doi/10.1145/2254064.2254088); [trace partitioning](https://dl.acm.org/doi/10.1145/1275497.1275501)
- **Assumptions** are explicit `{predicate, value, reason: default|base-choice|llm|config-unresolved, confidence, likelihood?}` records attached to a script binding. A solver checks for contradictory assumption sets, and model counting or priors can rank defaults. Like APIPilot dependencies, assumptions are hypotheses to verify (producer 2). — [Probabilistic SE](https://dl.acm.org/doi/10.1145/2338965.2338972); [APIPilot](https://arxiv.org/html/2608.17546v2)
- **Loops and recursion** are composite nodes with a summary: `{entry_guard, exit_guard, trip_class: 0|1|many|bounded(n)|input-dependent(len(items)), per-iteration body graph, aggregate effects}`. Recursion is summarized like a function summary (SMART-style pre/post). — [LESE](https://dl.acm.org/doi/10.1145/1572272.1572299); [Godefroid & Luchaup](https://dl.acm.org/doi/10.1145/2001420.2001425); [Proteus](https://dl.acm.org/doi/10.1145/2950290.2950340); [SMART](https://dl.acm.org/doi/10.1145/1190216.1190226)
- **States** come in three kinds sharing one EFSM schema `{entity, state_var(s), states as predicates, transitions: {trigger port/route, guard, effect, anchor}}`:
  - Lifecycle states come from enum or status fields plus write sites (SGFuzz and Whaley heuristics).
  - Protocol states come from packs (RFC2TLA+-style extraction, verified by AALpy or LearnLib).
  - Persistent data states come from ORM schema, constraints and status columns (Yang et al.; ConstrOpt).
  - Sources: [SGFuzz](https://www.usenix.org/conference/usenixsecurity22/presentation/ba); [Whaley et al.](https://dl.acm.org/doi/10.1145/566172.566212); [GK-tail](https://dl.acm.org/doi/10.1145/1368088.1368157); [RFC2TLA+](https://security.csl.toronto.edu/wp-content/uploads/2026/08/gding-ase2026-rfc2tla.pdf); [Yang et al.](https://people.cs.uchicago.edu/~shanlu/paper/ICSE20.pdf)
- **Races** are pairs of script lanes with a check-then-act on the same state entity and no guarding constraint or transaction. RacerD and lockset reasoning cover in-process threads, and feral concurrency and ACIDRain cover the DB level. — [RacerD](https://dl.acm.org/doi/10.1145/3276514); [Eraser](https://dl.acm.org/doi/10.1145/265924.265927); [Bailis et al.](https://dl.acm.org/doi/10.1145/2723372.2737784); [ACIDRain](https://dl.acm.org/doi/10.1145/3035918.3064037)
- **Producer 2 alignment**: refine graphs from logs while preserving mined invariants (Synoptic, CSight). Steps outside the graph are counterexamples, in the CEGAR sense. — [Synoptic](https://dl.acm.org/doi/10.1145/2025113.2025151); [CSight](https://dl.acm.org/doi/10.1145/2568225.2568246)
- **Fix checking**: diff two versions' summaries, in the style of differential symbolic execution. — [Person et al.](https://dl.acm.org/doi/10.1145/1453101.1453131)

### Inferences
- Keeping one predicate table per scope (function, entity, system) gives stable IDs for the override layer. Flipping an assumption is "set predicate p := ¬v", re-solving which edges are enabled, and lazily regenerating only the downstream subgraph keyed by the content hashes of the affected nodes.
- LLM roles that fit the evidence: naming predicates and classes, proposing classes the schema and code leave implicit, labeling states, and judging feasibility in languages without a solver (as in AutoBug and PALM). Roles to *avoid* trusting without checks: exact path enumeration and constraint solving. The 2511.08530 study found ~20% accuracy on most-constrained-path identification.
- Behavior packs should store per-function summaries in the same guarded-automaton format: input-class-keyed disjuncts with effects and links to lower packs. That matches compositional symbolic execution summaries and the Infer and CodeQL precedent the brainstorm already cites.

### Gaps
- No empirical work found on how *users* understand assumption-flipping or fork views over path-condition models. The explainability of these representations is unvalidated.
- No benchmark exists for whole-system (multi-service) scenario-graph extraction against which the product's generator could be scored. PSMBench (protocol FSMs) and REST testing benchmarks (APIPilot's 16 services) are the closest reusable evaluation sets.
