# Extractors Build Skeletons, LLMs Name Behavior

> **Note added after this report:** the project has since adopted a model-interpreter framing ([specification](../../specification.md#p6-the-model-is-a-program)). In those terms, the record graph proposed here is the program, the generation pipeline is the compiler, and script instantiation is the interpreter. The research content below is unchanged.

The state of the art as of November 2026 converges on one division of labor, and the model proposed here is built around it: deterministic tools own structure and exhaustiveness, the LLM owns local semantic judgment and naming, and a mechanical checker decides what is shown as fact. Static call graphs still beat LLM call graphs (about 85% against 60% completeness on Python), LLM-inferred specifications fed to a static engine double its real-world detections (27 to 55 CVEs in IRIS), and LLM-decides filters drop 27.5–45% of confirmed bugs where LLM-as-context plus a sound analyzer drops none. No production tool for TypeScript, Python or Go emits path-condition-partitioned function summaries, no tool statically infers state machines from undeclared status fields, and no SAST tool links services across repositories, so the scenario graph has to be assembled from a thin layer over existing extractors rather than bought. The recommended model is a flat, ID-referenced record graph with eight record families: entities with typed ports and wires (SysML v2 conjugation, binding as delegation, AADL-style allocation between the logical and physical facets); a shared vocabulary of anchored predicates; inputs as category-partition characteristics with choices; state as stores plus guarded lifecycle machines; behaviors as per-port guarded automata that carry Pulse-style disjunctive cases at summary level; scripts as partial orders of events under a binding; assumptions as first-class unbound predicates; and overrides. The scenario graph is not stored as one global object: it is the composition of local behaviors at ports, which is what makes user code and behavior packs one format and avoids the realizability pitfalls of global specifications. The design stays inside the decisions of checkpoint #1 and argues for four changes: semantic-path IDs in place of positional step IDs, assumptions as records instead of strings, a second provenance axis (source and checks) beside the three-value enum, and a wider "never hide" rule that covers non-atomic interleavings. The whole-pipeline accuracy is unmeasured anywhere in the literature, so the first thing to validate is per-claim precision on a hand-written gold model of a non-web-form system.

## Six research fields point to the same split between tools and model

### Data-flow engines are mature; call graphs and library models are the bottleneck

The core interprocedural algorithms date from 1995 to 2019 and are not the constraint. IFDS reduces context-sensitive data-flow to graph reachability over an exploded supergraph ([IFDS, POPL'95](https://dl.acm.org/doi/10.1145/199448.199462)), SPDS synchronizes two pushdown systems to avoid k-limited access paths ([Späth et al., POPL'19](https://ris.uni-paderborn.de/record/20539)), and Pinpoint slices path-feasibility queries on demand and solves them with SMT, checking roughly 2 MLOC of MySQL in 1.5 hours ([Pinpoint, PLDI'18](https://www.cse.ust.hk/~charlesz/papers/pinpoint.pdf)). Work since 2019 scales these engines: SparseDroid averaged a **22x speedup** over FlowDroid ([ASE'19](https://2019.ase-conferences.org/details/ase-2019-papers/1/Performance-Boosting-Sparsification-of-the-IFDS-Algorithm-with-Applications-to-Taint-)), and CodeQL shipped overlay-based incremental analysis for every language by mid-2026, combining a cached whole-codebase database with a small database of changed code ([GitHub changelog, Mar 2026](https://github.blog/changelog/2026-03-24-faster-incremental-analysis-with-codeql-in-pull-requests/); [Jun 2026](https://github.blog/changelog/2026-06-10-incremental-analysis-for-go-c-c-and-codeql-cli)).

The weak link for the stacks this product targets is the call graph. Jelly's approximate interpretation raised JavaScript call-graph recall from about **75.9% to 88.1%** on 36 projects ([PLDI'24](https://cs.au.dk/~amoeller/papers/approx)); Jarvis improves on PyCG for Python mainly by covering more language features ([arXiv 2305.05949](https://arxiv.org/abs/2305.05949v5)); Go's VTA is sound only "modulo reflection and unsafe" ([vta docs](https://pkg.go.dev/golang.org/x/tools/go/callgraph/vta)). Inversion of control makes this worse: without framework models, Doop reached about 14.5% in-app reachability on enterprise Java, and JackEE's Datalog rules for Spring and friends lifted that to about 58% ([JackEE summary](https://blog.acolyer.org/2020/09/25/jackee)). The Soundiness manifesto states the general condition plainly: practical analyses knowingly skip reflection, eval and dynamic features, and should document that ([CACM 2015](https://cacm.acm.org/magazines/2015/2/182650-in-defense-of-soundiness/fulltext)). The consequence for a tool that shows flows is that a missing edge becomes a silently missing branch, so unsoundness has to be a visible record, not an absence.

Library behavior is handled everywhere by summaries, and the formats are the most reusable prior art for behavior packs. CodeQL's models-as-data are declarative rows (`type, access path, input, output, kind`) with access-path tokens such as `Member[name]`, `Argument[n]`, `ReturnValue`, `Awaited` and `GuardedRouteHandler` ([CodeQL JS docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)), a provenance of the form `{origin}-{verification}`, neutral rows that state "nothing happens here", and precedence of manual over generated models ([CodeQL Java/Kotlin guide](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-java-and-kotlin)). Those rows describe flows only. The richer shape comes from Infer Pulse, whose summaries are lists of disjuncts with conditions on parameters kept as *latent* and discharged at call sites, where a caller's context prunes contradicting disjuncts ([Pulse docs](https://fbinfer.com/docs/checker-pulse)). Pulse does not support Python, TypeScript or Go, and none of the examined tools publishes summaries per library *version*; both are open niches rather than solved problems.

### Path conditions exist, but nothing turns them into named input classes

Symbolic execution keeps a path condition per path ([King 1976](https://dl.acm.org/doi/10.1145/360248.360252)), and partition analysis proposed splitting the input domain by those conditions as early as 1985 ([Richardson & Clarke](https://ieeexplore.ieee.org/document/1702003)). The engines fight path explosion with state merging ([Kuznetsov et al., PLDI 2012](https://dl.acm.org/doi/10.1145/2254064.2254088)), veritesting ([Avgerinos et al., ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568293)) and compositional function summaries ([SMART, POPL 2007](https://dl.acm.org/doi/10.1145/1190216.1190226)), and summarize loops with trip-count variables or disjunctive formulas ([LESE](https://dl.acm.org/doi/10.1145/1572272.1572299); [Proteus](https://dl.acm.org/doi/10.1145/2950290.2950340)). Their output is solver formulas over bytes, not "expired token". The vocabulary the product needs comes from testing theory instead: the category-partition method splits each parameter and environment condition into categories and *choices*, with `[error]` and `[single]` tags and constraints between choices ([Ostrand & Balcer, CACM 1988](https://dl.acm.org/doi/10.1145/62959.62964)), and input space partitioning adds a designated *base choice* per characteristic ([Ammann & Offutt](https://cs.gmu.edu/~offutt/softwaretest/)).

The LLM-era results set the boundary of what the model can be trusted with. AutoBug keeps path constraints as code the LLM reasons over directly, without translating to SMT, and is language-agnostic ([arXiv 2505.13452](https://arxiv.org/abs/2505.13452)); LLMDFA has the LLM write scripts that call a solver for path feasibility and reaches 87.10% precision and 80.77% recall on average ([arXiv 2402.10754](https://arxiv.org/abs/2402.10754v2)). Against that, a study of LLMs imitating KLEE found the most-constrained path identified only about **20%** of the time ([arXiv 2511.08530](https://arxiv.org/pdf/2511.08530)), and execution-path prediction was the hardest task on REval ([arXiv 2403.16437](https://arxiv.org/pdf/2403.16437)). Naming and grouping guards is an LLM job; enumerating paths and deciding feasibility is not.

### State inference works for traces and protocols, not for business status fields

Specification mining is mature for traces: k-tails, Synoptic's invariant-preserving refinement ([ESEC/FSE 2011](https://dl.acm.org/doi/10.1145/2025113.2025151)), CSight's communicating FSMs from distributed logs ([ICSE 2014](https://dl.acm.org/doi/10.1145/2568225.2568246)) and GK-tail's extended FSMs whose transitions carry data guards ([ICSE 2008](https://dl.acm.org/doi/10.1145/1368088.1368157)). Protocol state machines are learned actively with LearnLib or AALpy, which found real bugs in TLS implementations ([de Ruiter & Poll, USENIX Security 2015](https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/de-ruiter)), and since 2024 are extracted by LLMs from RFCs and source: ProtocolGPT reports over 90% precision on six implementations ([arXiv 2405.00393](https://arxiv.org/abs/2405.00393v2)); RFC2TLA+ extracts communicating FSMs and invariants in three tool-constrained passes ([ASE 2026 PDF](https://security.csl.toronto.edu/wp-content/uploads/2026/08/gding-ase2026-rfc2tla.pdf); its reported state F1 of 0.97 comes from a search snippet and its transition accuracy is unverified); and RFCLLM finds LLM extraction still notably weaker than manual extraction on some protocols ([arXiv 2609.13389](https://arxiv.org/html/2609.13389)).

For application code the picture is thinner. Where a library declares a machine (XState, django-fsm, AASM, python-statemachine) extraction is direct ([python-statemachine docs](https://python-statemachine.readthedocs.io/en/latest/states.html); [Stately docs](https://stately.ai/docs)). **No tool was found that statically infers state machines from undeclared enum or status fields in business code.** The two usable heuristics are SGFuzz's rule that state variables are enum-typed variables assigned named constants ([USENIX Security 2022](https://www.usenix.org/conference/usenixsecurity22/presentation/ba)) and Whaley et al.'s partitioning of states by which fields a method reads and writes ([ISSTA 2002](https://dl.acm.org/doi/10.1145/566172.566212)). Races in web systems are mostly check-then-act across requests on shared database rows, which is what feral concurrency control and ACIDRain document ([Bailis et al., SIGMOD 2015](https://dl.acm.org/doi/10.1145/2723372.2737784); [ACIDRain, SIGMOD 2017](https://dl.acm.org/doi/10.1145/3035918.3064037)), not memory races of the kind RacerD finds ([OOPSLA 2018](https://dl.acm.org/doi/10.1145/3276514)).

### Architecture recovery finds boxes easily and edges badly

Classic module clustering is the wrong tool: averaged MoJoFM sits at 38–47% ([arXiv 2311.04643](https://arxiv.org/pdf/2311.04643)), and only ACDC consistently beats k-means ([Lutellier et al., ICSE 2015](https://www.cs.purdue.edu/homes/lintan/publications/archrec-icse15.pdf)). Service-level recovery is easier because boundaries are explicit in build and deploy artifacts. In the benchmark of nine executable static recovery tools on 17 mostly Java/Spring applications, Code2DFD reached **F1 0.98 on components, 0.87 on connections and 0.66 on endpoints**, most other tools found few or no connections, and combining four tools reached 0.91 overall ([arXiv 2412.08352](https://arxiv.org/html/2412.08352)). Code2DFD leads because it reads deployment and configuration files and emits a traceability file linking each element to code evidence ([arXiv 2304.12769](https://arxiv.org/abs/2304.12769v1)). The hard parts are endpoints and edges, and no evaluated tool covers polyglot stacks.

Entry points are closer to solved. OOPS, an LLM pipeline over an API dependency graph, reports F1 above **98% for endpoint methods** and 92% for parameter constraints across 5 languages and 8 frameworks, at under 5.6K input tokens on average ([arXiv 2601.12735](https://arxiv.org/abs/2601.12735)); Respector is the static and symbolic baseline for Spring Boot and Jersey ([NSF PAR](https://par.nsf.gov/servlets/purl/10537946)). For cross-service flow, MScan analyzes each service and stitches results at HTTP and RPC boundaries, where whole-program analysis times out ([MScan PDF](https://yuanxzhang.github.io/paper/mscan-oakland25.pdf)). Infrastructure-as-code tooling yields resource-dependency graphs (terraform graph, inframap, rover), and research prototypes map Kubernetes configurations into technology-agnostic deployment metamodels ([DeMAF/EDMM](https://link.springer.com/article/10.1007/s42979-023-01721-4)); no accuracy evaluation of port-delegation chains or TLS termination points was found. LLM wiki products are evaluated on prose quality, not topology: CodeWiki scores 68.79% against DeepWiki's 64.06% on a rubric the CodeWiki authors built ([arXiv 2510.24428](https://arxiv.org/html/2510.24428v5)), and automatic rankings of repository documentation disagree with human rankings ([arXiv 2604.06793](https://arxiv.org/pdf/2604.06793)).

### LLMs win as specification inferrers and lose as global reasoners

The neuro-symbolic results are consistent enough to act on. On CWE-Bench-Java, CodeQL alone found 27 of 120 real vulnerabilities and IRIS with GPT-4 found **55** by having the LLM infer source and sink specifications ([arXiv 2405.17238](https://arxiv.org/html/2405.17238v3)). SemTaint treats static analysis as the source of truth, requires every LLM-proposed fact to **bind to a CodeQL data-flow node** or be rejected, runs each agent three times, and recovers 106 of 162 vulnerabilities CodeQL missed; demand-driven invocation cut agent calls by 94.5% ([arXiv 2601.10865](https://arxiv.org/html/2601.10865)). RepoAudit explores a repository function by function with a validator that re-checks data-flow facts and path conditions, at 78.43% precision and about $2.54 per project ([arXiv 2501.18160](https://arxiv.org/abs/2501.18160)). Decomposition beats end-to-end prompting by 0.35 F1 in LLMDFA's own comparison ([arXiv 2402.10754](https://arxiv.org/html/2402.10754v2)).

The failures are equally consistent. Across 24 LLMs, static tools "consistently outperform" on call graphs, with PyCG at 84.9% completeness against 60.3% for the best LLM, while LLMs beat static tools on type inference ([arXiv 2410.00603](https://arxiv.org/abs/2410.00603)). Evident shows that letting GPT-5.5 decide which alarms are false misses **27.5–45.0% of confirmed bugs**, whereas using the LLM only to build a harness for a sound analyzer filtered 48.8–63.8% of false alarms with zero misses ([arXiv 2606.15122](https://arxiv.org/abs/2606.15122)). Code summaries hallucinate on unfamiliar code: perturbing code raised one model's hallucination rate from 17% to 97% ([FSE 2026](https://conf.researchr.org/details/fse-2026/fse-2026-research-papers/66/Hallucinations-in-LLM-based-Code-Summarization-Unveiling-Detection-and-Mitigation)), and checking every entity named in a summary against the code detects hallucinations at 73% F1 ([ETF, arXiv 2410.14748](https://arxiv.org/html/2410.14748v4)). Most of these numbers were measured on 2023–2025 models and are self-reported; the architectural lesson has held across generations, but how much the decomposition advantage shrinks on late-2026 models is unknown.

### Formalisms and representations: constrained DSLs with a checker in the loop

No single formalism covers structure, behavior, state and partial order together. LLMs get syntax right far more often than semantics: free-form TLA+ from natural language reached at best 26.6% syntactic and **8.6% semantic** correctness across 30 models ([arXiv 2606.05792](https://arxiv.org/abs/2606.05792)), while TLA+ modeling of real code with checker feedback reaches invariant scores up to 81%, with the characteristic error being wrong atomicity, merging multi-step operations into one guard ([SysMoBench, arXiv 2509.23130](https://arxiv.org/pdf/2509.23130)). SysML v2 output slips into v1 syntax or invented keywords ([arXiv 2606.23395](https://arxiv.org/pdf/2606.23395)); state-machine extraction from requirements reached F1 0.90 on states but 0.75 on transitions ([arXiv 2604.00275](https://arxiv.org/html/2604.00275v1)); and ProMoAI succeeds by having the LLM write an intermediate language that cannot express unsound nets ([arXiv 2412.00023](https://arxiv.org/html/2412.00023v1)).

On the representation side, SCIP is the best anchoring substrate: human-readable, package-qualified symbol strings with typed ranges and an `enclosing_range`, though its spec makes no promise of stability across commits and embeds the package version in the symbol ([scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)). GitHub's stack graphs are archived ([repo](https://github.com/github/stack-graphs)) and LSIF is superseded ([Sourcegraph](https://sourcegraph.com/blog/announcing-scip)). Schema-enforced output guarantees shape, not content; the measured cost of structure comes from schema design, with reasoning-first field order matching or exceeding free-form accuracy and answer-first causing steep drops ([Structure Tax, AACL 2026](https://aclanthology.org/2026.aacl-long.209/)). Graph encodings matter by 4.8 to 61.8 accuracy points, with neighbor-list encodings best on most tasks ([Talk like a Graph](https://arxiv.org/pdf/2310.04560)). LLM-built graphs duplicate nodes heavily without an entity-resolution pass, 30.53% in one GraphRAG baseline ([CORE-KG, arXiv 2510.26512](https://arxiv.org/pdf/2510.26512)). Mechanical citation checks (file exists, range valid) are the cheapest verification layer, with one preprint reporting 92% citation accuracy in its best configuration ([arXiv 2512.12117](https://arxiv.org/pdf/2512.12117)), and static guardrails with reflection fixed 92% of extraction errors within the first retry in PARSE ([arXiv 2510.08623](https://arxiv.org/abs/2510.08623v1)). Salsa supplies the incremental semantics: memoized queries with recorded dependencies, early cutoff when a recomputed value is unchanged, and durability tiers ([Salsa algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)).

## The semantic core borrows one idea from each formalism

The model is a deliberately small synthesis. Each concept below is taken from the formalism that defines it most cleanly, and each choice is a design inference resting on the cited source, not something any single source endorses as a whole.

| Concept in the model | Borrowed from | Why this one |
| --- | --- | --- |
| Port with direction and a compatibility rule | SysML v2 port conjugation: A connects to B iff B is the conjugate of A ([Verified-MBSE](https://chantakan.github.io/verified-mbse/VerifiedMBSE/Core/Port.html)) | A one-line, mechanically checkable typing rule for every wire |
| Delegation | SysML v2 *binding* (outer port is the inner port) and UML delegation as forwarding ([UML connector](https://www.uml-diagrams.org/composite-structure-diagrams/connector.html)), extended with a declared adapter | UML and SysML delegation is pure forwarding; "messages change form at boundaries" needs a typed transformer on the wire |
| Boundary entity (LB, gateway, router) | Wright's first-class connector with roles and glue ([Allen & Garlan](https://doi.org/10.1145/258077.258078)) | Gives "the gateway's routes cover the ports it fronts" as a check |
| Two containment facets | AADL software/platform hierarchies linked by binding ([SAE AS5506](https://www.sae.org/standards/content/as5506d/)); C4 deployment views ([c4model.com](https://c4model.com)) | The established pattern is two trees plus an allocation relation, never one forced tree |
| Predicate and guard | Control-flow automata with assume edges ([lazy abstraction](https://dl.acm.org/doi/10.1145/503272.503279)); symbolic automata over a Boolean algebra of predicates ([D'Antoni & Veanes](https://link.springer.com/chapter/10.1007/978-3-319-63387-9_3)) | A finite named vocabulary keeps graphs finite and supports solver checks, diffing and later learning |
| Input class | Category-partition choices with `[error]`, `[single]` and a base choice ([Ostrand & Balcer](https://dl.acm.org/doi/10.1145/62959.62964)) | Human-meaningful partitions with a principled default for unbound branches |
| Behavior summary | Pulse disjuncts with latent conditions ([Pulse docs](https://fbinfer.com/docs/checker-pulse)); compositional symbolic summaries ([SMART](https://dl.acm.org/doi/10.1145/1190216.1190226)); CodeQL access paths and provenance ([CodeQL docs](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/)) | Binding inputs at a call site prunes cases; an unbound latent condition is exactly an assumption |
| Lifecycle and protocol state | Extended FSMs with guarded transitions ([GK-tail](https://dl.acm.org/doi/10.1145/1368088.1368157)); communicating state machines as in P ([P docs](https://p-org.github.io/P/)); SCXML run-to-completion ([W3C](https://www.w3.org/TR/scxml/)) | One pinned semantics avoids the statechart-variant problem and matches RFC machines |
| Script | Pomset ordered by happens-before ([Pratt](http://boole.stanford.edu/pub/ijpp.pdf); [Lamport](https://lamport.azurewebsites.net/pubs/time-clocks.pdf)) | `after:` is the covering relation; lanes are chains; concurrency is incomparability |
| Fork and assumption | Conflict relation of prime event structures ([Winskel](https://www.cl.cam.ac.uk/~gw104/EvStr.pdf)) | An untaken branch is a conflicting event; an assumption is an unresolved conflict |
| Collapse in time | Refinement mappings and stuttering ([Abadi & Lamport](https://lamport.azurewebsites.net/pubs/abadi-existence.pdf)); Lipton reduction ([CACM 1975](https://doi.org/10.1145/361227.361234)) | Forwarding-only steps are stuttering; a segment that touches shared state is atomic only if reducible |
| Layered messages | Nested protocols in session types ([Demangeon & Honda](https://mrg.cs.ox.ac.uk/publications/nested-protocols-in-session-types/subsessioncam.pdf)) | A higher-layer session runs on a lower one that other parties cannot see |

One structural decision follows from this table and deserves its own argument. Two research notes pull in different directions: the path-analysis literature suggests a guarded automaton *per entity*, composed by message passing, while the choreography literature suggests one *global* interaction graph that projects onto participants. Global specifications carry hard realizability problems; safe realizability is EXPSPACE-complete for bounded HMSCs and undecidable in general ([Lohrey, CONCUR 2002](https://www.fi.muni.cz/concur2002/papers/46.html)). A graph extracted from code has the opposite risk, spurious global paths that no combination of local behaviors can take. The model therefore **stores behavior locally, one guarded automaton per provided port, and derives the scenario graph as the composition of those automata across wires**. This is what makes "user-code graphs and behavior packs share one format" literally true (a pack is a set of local behaviors), it matches the MScan pattern of analyze-per-service-then-stitch, and it keeps every stored record anchored to one piece of code. The checkpoint's statement that static analysis yields a scenario graph stays true; the graph is a derived, cached view.

## Eight record families make up the model

All records are flat, carry `id`, `kind` and usually `parent`, and reference each other by ID; nesting is derived. The wire format the LLM emits is JSON under a strict schema, and YAML is used here for reading. Field order inside LLM-emitted records puts free text (`analysis`) before decisions, following the Structure Tax result.

### Identity, anchors and provenance

IDs are semantic paths minted by the system, never by position: `parent-id/kind:slug`, where the slug comes from the anchor symbol's last descriptor when one exists and from a short LLM-proposed kebab slug otherwise. The checkpoint's sketch uses positional step IDs such as `s3.2`; those renumber whenever a step is inserted and would orphan the override layer, so order moves into `after` and `order` fields. Content hashes are stored beside IDs as cache keys and never serve as identity, because a content-hash ID changes on every edit. The reasoning rests on why SCIP replaced LSIF (opaque numeric IDs blocked debugging and partial updates, [Sourcegraph](https://sourcegraph.com/blog/announcing-scip)); the specific scheme is a design inference with no empirical backing yet.

Anchors are multi-key and resolved in priority order: the SCIP symbol with its version component stripped, so a lockfile bump does not rename everything; a scope fingerprint (enclosing symbol plus a normalized tree-sitter AST hash), the idea behind GitLab's scope-offset fingerprinting, which the company reports cuts futile re-auditing by about 30% ([GitLab blog](https://about.gitlab.com/blog/improved-scope-offset-fingerprinting)); `(full commit SHA, path, range)` mapped forward by diff, with the commit pinned by hash as Kythe recommends ([Kythe schema](https://kythe.io/docs/schema/)); and a one-line `quote` checked by substring match. Anchor status is `exact`, `drifted` (only positional or textual keys resolve) or `broken`, recomputed per commit.

Provenance keeps the checkpoint's three values, with `observed` reserved for runtime evidence. One research note proposed labeling index facts `observed`; that would blur the "imagined versus actual" view, so this report rejects it. Instead every record carries a second axis: `source` (`extractor:<name>`, `llm:<model>`, `pack:<id@ver>`, `spec:<ref>`, `trace`, `user`) and `checks` (which verification tiers passed). A compose port mapping is `inferred` with `source: extractor:compose`; an LLM-resolved dispatch target is `inferred` only if it binds to index evidence, otherwise `assumed`. This mirrors CodeQL's `{origin}-{verification}` provenance without changing the user-facing enum.

```yaml
anchor:
  repo: reports
  commit: 9c1e4b7a52d04c6f8a13e0b7d2f4a6c81e3b5d70
  path: worker/tasks.py
  range: [38, 0, 91, 0]            # 0-based, half-open, utf-8
  symbol: "scip-python python reports . `worker.tasks`/generate_report()."
  scope_fp: "`worker.tasks`/ :: ast:4be1..."
  quote: "def generate_report(self, report_id: str):"
  status: exact                     # exact | drifted | broken (recomputed, never emitted by the LLM)
```

### Entities, ports, wires and allocation

An entity is a type, an instance, a boundary of ports and an interior of parts. Ports carry a direction (`provides` or `requires`), a protocol, a layered message type, and an `exposure` that marks whether the port faces outside the modeled system. Wires come in two kinds: `assembly` between peers and `delegation` from an outer port to an inner one, the latter optionally carrying an `adapter` that names the transformation (DNAT, TLS termination, HTTP parse, deserialize). Each entity has at most one parent per facet, and an `allocation` record links a logical instance to the physical instance it runs on. Boundary entities are ordinary entities with `on_boundary_of` set, whose outbound ports split one outer port into several delegations. An open question in the checkpoint asks whether network topology is a third facet; the allocation mechanism generalizes to more trees, but the evidence found supports only the two-tree pattern, so this stays open.

```yaml
- id: reports/worker
  kind: entity
  type: Service                     # type hierarchy lives in domain packs
  facets: {logical: reports, physical: host-b/ctr-worker}
  provenance: inferred
  source: extractor:compose
  anchor: {path: docker-compose.yml, range: [41, 2, 58, 0], quote: "worker:", status: exact}

- id: reports/worker/port:task-generate-report
  kind: port
  parent: reports/worker
  direction: provides
  protocol: celery-task             # resolved against pack pypi/celery@5.4
  message: {layer: amqp-like, payload: "json:{report_id: str}"}
  exposure: internal                # external | internal
  trigger: queue-consumer           # how work arrives; see the input section
  handler: "scip-python python reports . `worker.tasks`/generate_report()."

- id: reports/api/port:queue-out/wire
  kind: wire
  wire: assembly                    # assembly | delegation
  from: reports/api/port:enqueue-generate          # requires
  to: reports/broker/port:queue-reports-generate   # provides
  match: {key: "queue:reports.generate", strength: exact}   # exact | heuristic
  provenance: inferred
  source: extractor:celery-routes

- id: host-a/port:eth0-443/wire
  kind: wire
  wire: delegation
  from: host-a/port:eth0-443
  to: host-a/ctr-api/port:8000
  adapter: {kind: dnat, pack: "platform/docker-bridge@27"}   # hops inside come from the pack, assumed
  provenance: inferred
  source: extractor:compose
```

The checkpoint's validation rule becomes three mechanical checks: every port of an open entity has a wire or an `assumed` marker; every assembly wire joins a `requires` port to a conjugate `provides` port; and every route of a boundary entity delegates to an existing port.

### Predicates: the shared vocabulary

Guards, input classes, state names and assumptions are all formulas over one table of named, anchored predicates. A predicate has a name in domain terms, the source expression it stands for, the thing it is *about* (`over`), and an origin. The `over` field is what generalizes the model beyond web forms: a predicate ranges over a message field arriving at a port, a configuration key, a state variable in a store, the response of a required port (a downstream failure is an input), or time and scheduling. Guards use a closed formula grammar over predicate IDs (`&`, `|`, `!`), so they can be validated, diffed and, where the underlying expressions are simple, handed to a solver.

```yaml
- id: reports/worker/pred:status-is-pending
  kind: predicate
  name: "report is still pending"
  expr: "report.status == ReportStatus.PENDING"
  over: {state: reports/db/store:report/var:status}
  origin: path                      # schema | path | pack | llm
  provenance: inferred
  anchor: {path: worker/tasks.py, range: [44, 4, 44, 48], quote: "if report.status != ReportStatus.PENDING:", status: exact}

- id: reports/worker/pred:upload-ok
  kind: predicate
  name: "object store accepts the upload"
  expr: "s3.put_object(...) does not raise ClientError"
  over: {response: reports/worker/port:s3-put}
  origin: path
```

### Inputs, characteristics and input classes

An input is not a value; it is a set of *characteristics* attached to a trigger port, each with a small set of *choices*. An input class is one choice: a conjunction of predicates, tagged `base` (the default), `error` or `single`. Characteristics are seeded from three origins, recorded on each class. Schema-derived choices (required, enum, format, bounds from OpenAPI, zod, pydantic, protobuf, argparse or clap definitions) are cheap and reliable; path-derived choices come from validator and guard branches in the handler; LLM-proposed choices cover what both leave implicit. Ambient characteristics belong to the scenario rather than to a port: pre-existing state ("a report with this key already exists"), configuration, and the behavior of required ports ("the object store is down"). RESTler-style producer–consumer dependencies, where a response field of one operation feeds a parameter of a later one ([RESTler](https://www.microsoft.com/en-us/research/publication/restler-stateful-rest-api-fuzzing/)), are recorded as `feeds` links between ports and are what connect one script's output to the next script's state precondition.

```yaml
- id: reports/api/port:post-reports/in:date-range
  kind: characteristic
  parent: reports/api/port:post-reports
  about: "body.start, body.end"
  choices:
    - id: reports/api/port:post-reports/in:date-range/class:within-limit
      name: "range of at most 366 days"
      when: "!pred:range-too-large & pred:payload-valid"
      tags: [base]
      origin: path
    - id: reports/api/port:post-reports/in:date-range/class:too-large
      name: "range longer than 366 days"
      when: "pred:range-too-large"
      tags: [error]
      origin: path
    - id: reports/api/port:post-reports/in:date-range/class:malformed
      name: "missing or unparseable dates"
      when: "!pred:payload-valid"
      tags: [error]
      origin: schema
```

### State: stores, variables and machines

State lives in *store* entities: a table or collection, a cache keyspace, a queue or topic, a file or bucket, or an in-process location such as a module-level singleton, a connection pool or a circuit breaker. Stores are ordinary entities whose ports are read, write and subscribe, which keeps "shared state is an interaction that is not a message" inside the port model. Each store declares *state variables*, the abstractions worth tracking, and a variable with a small closed set of values gets a *machine*: an extended FSM whose states are predicates over the variable and whose transitions name the triggering behavior node, the guard and the anchor of the write site. The same record type holds protocol machines from packs (TCP, TLS, HTTP/2 streams, a Celery task's delivery states), under SCXML-style run-to-completion semantics.

```yaml
- id: reports/db/store:report/var:status
  kind: state_var
  parent: reports/db/store:report
  domain: [PENDING, RUNNING, DONE, FAILED]
  scope: per-row                    # per-row | per-key | singleton | per-connection | per-message
  detected_by: [orm-enum-column, write-sites]

- id: reports/db/store:report/var:status/machine
  kind: machine
  initial: PENDING
  transitions:
    - {from: PENDING, to: RUNNING, by: "reports/worker/beh:generate-report/n:claim",
       guard: "pred:status-is-pending", atomic: false}
    - {from: RUNNING, to: DONE,    by: "reports/worker/beh:generate-report/n:mark-done"}
    - {from: RUNNING, to: PENDING, by: "reports/worker/beh:generate-report/n:mark-retry",
       guard: "!pred:upload-ok & !pred:attempts-exhausted"}
    - {from: RUNNING, to: FAILED,  by: "reports/worker/beh:generate-report/n:mark-failed",
       guard: "!pred:upload-ok & pred:attempts-exhausted"}
    - {from: RUNNING, to: PENDING, by: "reports/sweeper/beh:requeue-stuck/n:reset", guard: "pred:stale-15m"}
    - {from: FAILED,  to: PENDING, by: "reports/cli/beh:retry/n:reset"}
```

The answer to "what are the states in the system" is therefore a three-part inventory produced by different means: data state from declared schemas (ORM models, migrations) and their status columns; lifecycle state from enum-typed or small-closed-set fields plus their write sites, using the SGFuzz and Whaley heuristics with the LLM labeling guards; and protocol or runtime state from packs. A fourth kind is in-flight state, the contents of queues and pending calls, which the script's partial order already represents.

### Behaviors: guarded automata with a summary level

A behavior belongs to one provided port and describes everything the code behind that port can do. It exists at two levels of detail that share one format. The `summary` level is a list of *cases*: each case has a guard over predicates, the observable effects in order (sends on required ports, state writes, the reply), and latent conditions the caller or environment must settle. The `full` level is a graph of nodes and guarded edges. Node kinds are a closed set: `action` (tagged with one of the checkpoint's event kinds), `call` (a send on a required port, which is where composition happens), `branch`, `loop` (a composite with entry guard, exit guard, a trip class of `0`, `1`, `many`, `bounded(n)` or `input-dependent`, and aggregate effects), `par` (fork and join), `slot` (a hole for user code in a framework behavior), and `unknown` (an unresolved callee, reflection, dynamic import or unmodeled framework hook). The refinement obligation between levels is mechanical: every path through the full graph must map to exactly one case, with the same effects at ports.

```yaml
- id: reports/worker/beh:generate-report
  kind: behavior
  port: reports/worker/port:task-generate-report
  lod: summary                      # summary | full ; full adds nodes[] and edges[]
  analysis: "Loads the row, returns early unless PENDING, claims it without a row lock,
             pages through orders, uploads CSV, marks DONE; ClientError triggers self.retry."
  cases:
    - id: .../case:skip
      when: "!pred:row-exists | !pred:status-is-pending"
      effects: [{kind: read, port: port:db, store: store:report}]
      outcome: ack
    - id: .../case:done
      when: "pred:row-exists & pred:status-is-pending & pred:upload-ok"
      effects:
        - {kind: write, store: store:report, var: status, to: RUNNING, node: n:claim}
        - {kind: loop,  node: n:page-orders, trip: input-dependent, reads: store:orders}
        - {kind: send,  port: port:s3-put}
        - {kind: write, store: store:report, var: status, to: DONE}
        - {kind: send,  port: port:webhook, when: "pred:webhook-configured"}
      outcome: ack
    - id: .../case:retry
      when: "pred:row-exists & pred:status-is-pending & !pred:upload-ok & !pred:attempts-exhausted"
      effects:
        - {kind: write, store: store:report, var: status, to: RUNNING}
        - {kind: send,  port: port:s3-put, result: error}
        - {kind: write, store: store:report, var: status, to: PENDING}
        - {kind: send,  port: port:enqueue-generate, via: "pack:pypi/celery@5.4#Task.retry"}
      outcome: retry
    - id: .../case:failed
      when: "pred:row-exists & pred:status-is-pending & !pred:upload-ok & pred:attempts-exhausted"
      effects: [{kind: write, store: store:report, var: status, to: FAILED}]
      outcome: ack
  unknowns: []                      # each entry is a visible marker, never dropped
  provenance: inferred
  source: llm:<model>
  deps: [{key: "`worker.tasks`/generate_report().", ast_hash: "4be1..."}]
  gen_key: "h:a93f..."
```

### Scripts, assumptions and overrides

A script is a binding plus a pomset. The binding is a partial assignment of predicates, chosen by naming input classes and ambient conditions. Steps are instances of behavior nodes, each with an actor, an anchor inherited from its node, an event kind, and `after` edges; the engine closes `after` transitively and checks that every receive follows its send. Predicates met on the path but absent from the binding become assumption records that default to the base choice; flipping one is a single-field edit followed by re-solving which edges are enabled. Segments stay as in the checkpoint, with one added field: `atomicity`, computed, with values `stutter` (forwarding only), `isolated` (no shared-state access concurrent with another lane), `guarded` (inside a lock or transaction) or `non-atomic`. A `non-atomic` segment is never auto-collapsed. This widens the checkpoint's proposed rule to "a collapsed segment never hides an error, an assumption or a non-atomic interleaving", on the strength of the Lipton condition and of SysMoBench's finding that wrong atomicity is the characteristic LLM modeling error.

```yaml
- id: scr:large-report-s3-flaky
  kind: script
  title: "User requests a report; first upload fails; retry succeeds"
  entry: reports/api/port:post-reports
  binding:
    classes: [".../in:date-range/class:within-limit", ".../in:idempotency/class:new-key"]
    predicates: {"reports/worker/pred:upload-ok": [false, true]}   # per delivery attempt
  assumptions: [scr:large-report-s3-flaky/a:webhook-off, scr:large-report-s3-flaky/a:single-delivery]

- id: scr:large-report-s3-flaky/a:single-delivery
  kind: assumption
  predicate: "pack:pypi/celery@5.4/pred:redelivered"
  bound_to: false
  reason: base-choice               # base-choice | config-default | config-unresolved | llm | user
  alternatives: [true]
  provenance: assumed

- id: ovr:0007
  kind: override
  target: scr:large-report-s3-flaky/seg:broker-hop
  field: collapsed
  value: false
  set_by: user
  base_gen_key: "h:a93f..."         # re-applied if the target ID survives; flagged orphaned otherwise
```

## Generation runs extractors first, then one bounded LLM call per unit

The pipeline has three properties taken from the evidence: the LLM is invoked on demand and per unit, never end to end; every LLM output is a set of anchored, typed records; and nothing the LLM says can remove a branch.

### Deterministic extractors build the skeleton

The first pass uses no LLM. It inventories languages, frameworks, lockfiles, IaC and contract files; renders IaC before parsing (`helm template`, `kustomize build`, `docker compose config`, `terraform show -json`), with unresolvable values becoming assumption slots; builds SCIP indexes where an indexer exists and tree-sitter facts everywhere; and builds a call graph with a static tool for the language, with dynamic-language hints where available. From these it emits physical entities, replicas and port-delegation wires from compose, Kubernetes, proxy and load-balancer configuration, all anchored to file and line. Kernel-level hops are not written in any repository and come from platform packs marked `assumed`. This ordering reflects the benchmark result that configuration and deployment files are the strongest signal for components and connections.

Finding user-facing services and their inputs is mostly deterministic and follows from the port model. Framework-specific queries enumerate *trigger ports* of every kind, which is what generalizes past web forms:

| Trigger kind | Where the extractor looks | Input characteristics seeded from |
| --- | --- | --- |
| HTTP, GraphQL or gRPC operation | route registrations with mount prefixes, file-system routes, `.proto`, schema files, a dumped OpenAPI document | request schema, path and query parameters, auth middleware |
| CLI command | argparse, click, clap, cobra definitions; `main` | flags, positional arguments, stdin, environment variables, files named by arguments |
| Queue or topic consumer | consumer and task decorators, subscription config | message schema, delivery attributes (redelivered, attempt count) |
| Schedule | cron expressions, beat schedules, Kubernetes CronJobs | time, and the state the job scans |
| Webhook or callback endpoint | routes registered with third parties | provider payload schema from a pack |
| UI event | form and click handlers followed through the call graph to client calls | form fields, client-side validation |
| Library public API | exported symbols, package entry points | parameter types, documented preconditions |
| OS-level | signal handlers, file watchers, socket listeners | signal kind, file contents |

A trigger port is *external* when no entity inside the model requires it, or when an ingress, published host port or public load balancer delegates to it; an entity is a user-facing service when it owns at least one external trigger port or is reached from one through delegation only. The LLM does not decide exposure. For each external port the pipeline creates an *actor* entity (user, operator, clock, external system) holding the conjugate `requires` port, so every script has a first sender. The same pass extracts outbound calls (HTTP clients, gRPC stubs, producers, database drivers, SDKs), resolves URL, topic and host strings by constant propagation joined with the rendered environment, and matches edges by exact contract first (proto method, topic name, operation ID), then by method plus normalized path template.

Three more extractors feed the behavioral records. A schema extractor reads request models, ORM models, migrations and declared state-machine libraries. A guard extractor lists every branch point in each handler's slice with its condition expression and anchor, plus every statically visible effect: outbound call sites, store reads and writes, raises and returns. A state-variable extractor finds enum-typed or small-closed-set fields and all their write sites. These lists matter less as answers than as *obligations*: they are what the LLM's output is checked against.

### What the LLM is asked for, per call

| Call | Unit and trigger | Input given | Output records |
| --- | --- | --- | --- |
| A. Entry-point leftovers | one unresolved registration (dynamic routes, decorator factories), on demand | the registration site and files linked by the dependency graph, about 5K tokens | port records, each bound to a handler symbol |
| B. Edge resolution | one ambiguous client call | call site, candidate provided ports, resolved environment | one wire with `match.strength: heuristic` and a rationale, or "none" |
| C. Consolidation | once per system | entity and port lists, no code | names, logical grouping, actor entities, proposals for unwired ports |
| D. Behavior summary | one provided port, top two levels eagerly | handler slice, extracted guards and effects, schemas, summaries of callees and packs, IDs in scope | `analysis`, predicates, characteristics and classes, cases, unknowns |
| E. State machine | one state variable | its write sites with enclosing guards, predicates already minted | machine transitions bound to behavior nodes |
| F. Scenario binding | one plain-language scenario | characteristics and classes on the entry port and reachable ambient predicates | a binding; no steps |
| G. Full expansion | one behavior or one `call` node, on zoom | the code of that callee only, plus parent case and bound predicates | nodes and edges at `lod: full` |
| H. Presentation | one script | the instantiated steps | segment proposals, event-kind parameters, collapse reasons |

Call D is the heart of the generator and is where granularity matters. It processes one port per call, with callee behavior supplied as already-generated summaries rather than code, which is the compositional pattern of function summaries and answers the finding that multi-hop state propagation, not context length, is the repository-level bottleneck ([RepoReasoner, via secondary overview](https://www.alphaxiv.org/overview/2607.25996)). Its output order is fixed: free-text analysis, then predicates (each bound to an extracted guard or declared `origin: llm`), then characteristics and classes, then cases, then unknowns. The model may reference only IDs present in its context or declared earlier in the same response. Order across the system is bottom-up within a service where callee summaries are needed and outside-in across services, starting from external ports, so that the first two levels are ready before anything is shown.

Script instantiation is deliberately not an LLM task. Given a binding from call F, the engine walks the composed behaviors: at each guard it evaluates the formula over bound predicates, takes the enabled edge, and where the guard is undetermined it takes the edge consistent with base choices and mints an assumption. At each `call` node it follows the wire to the callee's behavior, at summary level unless the user has zoomed. This is constraint matching over a finite vocabulary, the role the evidence says should not be given to the LLM.

### Verification decides what is shown as inferred

Checks run cheapest first on every write, and failures return to the model as a targeted error list for one retry before surfacing to the user.

| Tier | Check | Basis |
| --- | --- | --- |
| 1 Structural | schema and closed enums; unique IDs; no dangling references, with fuzzy suggestions for near misses; `after` acyclic; send precedes receive | graph validators that reject dangling edges and duplicate IDs ([EviGraph](https://arxiv.org/pdf/2608.04738)); LLMs misspell node names and create cycles ([arXiv 2407.09311](https://arxiv.org/pdf/2407.09311)) |
| 2 Anchor | file exists at commit; range in bounds; quote matches; symbol occurrence present in the index; every identifier, route or config key named in a title or summary resolves | bind-to-analyzer-node rejection in SemTaint; entity tracing |
| 3 Coverage | every extracted branch point in the slice maps to a predicate or is declared merged with a reason; every extracted effect appears in some case or is declared irrelevant; every port of an open entity is wired or `assumed` | design inference, extending the checkpoint's port rule to guards and effects |
| 4 Guard algebra | sibling case guards are pairwise disjoint and jointly exhaustive over the predicate table, else an `otherwise` case with an `unknown` marker is added; class guards within a characteristic partition it; assumption sets are consistent | solver-checked feasibility in LLMDFA and RepoAudit |
| 5 Composition | assembly wires join conjugate ports; a branch observed by several entities is decided by one and communicated by a message; full-level graphs refine their summaries case for case | conjugation rule; local-choice conditions from the session-type and MSC literature |
| 6 Atomicity and races | segment atomicity recomputed; two happens-before-incomparable accesses to the same state variable with at least one write, and no enclosing transaction or constraint, are flagged as a race candidate | Lipton reduction; feral concurrency and ACIDRain |
| 7 Agreement | k samples for deep levels; disagreement downgrades a record to `assumed` and shows a fork; an adversarial "find the line that contradicts this" pass for records about to be shown as `inferred` | three-run voting in SemTaint; self-refutation in production security review ([Anthropic](https://www.anthropic.com/news/claude-code-security)) |
| 8 Observed | align traces and logs to steps by anchor and order; a step outside the graph is a counterexample | Synoptic and CSight refinement; PObserve-style log conformance ([P case studies](https://p-org.github.io/P/casestudies)) |

Two rules sit above the tiers. Only a solver, an analyzer or an observed trace may prune a branch as infeasible; LLM narration never does, which is the Evident result applied directly. And counts such as a segment's `contains: {assumptions, errors}` are always recomputed, never trusted from the model.

## Packs compose by pruning cases, and regeneration is a cached query

### Behavior packs are local behaviors with slots

A pack is keyed by `ecosystem/package@version` plus a content hash of the source it was generated from, and contains the same records as user code at `lod: summary`: entity types and ports, predicates, behaviors with cases, machines, and links to lower packs. Callable paths use CodeQL-style access paths (`Member[sign].Argument[2].Member[algorithm]`) so call sites can be matched syntactically. Composition at a call site proceeds in three moves. Constant propagation and configuration resolution bind the pack's predicates where they can (`algorithm` is the literal `"RS256"`), which prunes cases exactly as Pulse prunes disjuncts. Remaining latent predicates become assumptions in the calling script, defaulting to the pack's base choices. `slot` nodes in a framework pack are filled by the user behaviors the extractor bound to them: a route handler, a middleware chain, a task body. This is the mechanism JackEE's rules and CodeQL's `GuardedRouteHandler` token implement for reachability, lifted to behavior.

Precedence follows CodeQL: a curated pack beats a verified generated pack, which beats an unverified generated pack, which beats on-the-fly analysis of the fetched dependency source; `neutral` entries state that a callable has no relevant behavior and override generated ones. Pack provenance uses the origin-and-verification pair, with a `trace-checked` level added. Packs are generated from the library's actual source at that version, not from model priors, and validated by running the same pipeline against the library's own tests where traces are available. The perturbation result (17% to 97% hallucination) is the reason for the content hash: a fork or patched build that looks like a familiar library is where a pack will be confidently wrong, so a hash mismatch demotes the pack to on-the-fly analysis. RFC packs follow the multi-pass, tool-constrained extraction pattern of RFC2TLA+ and are checked against an implementation by active learning; the abstract input alphabet that active learning needs is the same artifact as the pack's input classes. Exporting a pack or a script neighborhood to P or TLA+ for interleaving exploration is a plausible optional backend, off the rendering path.

### Regeneration is Salsa over LLM calls

Each generation call is a memoized derived query. Its key combines the normalized AST hashes of the code slices it read, the `gen_key`s of the callee summaries and packs it consumed, the bound predicates, and the prompt, schema and model version. Its recorded dependency set is the list of anchors it cited. On a new commit, anchors are re-resolved; an inverted index from anchor to generation gives the fanout, the same quantity Glean computes by query to bound its incremental cost ([Meta engineering](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/)). Only invalidated calls re-run. Three adaptations make this work for a non-deterministic producer. An unchanged key is never re-run, so the same input cannot yield a different picture. After a re-run, new records are matched to old ones by anchor symbol, then scope fingerprint, then slug and title similarity, and old IDs are carried forward; this is the entity-resolution pass that the knowledge-graph literature finds indispensable ([DEG-RAG](https://arxiv.org/html/2510.14271v1)). Early cutoff then compares normalized outputs (IDs matched, lists sorted, prose fields ignored): if a port's cases are equivalent to before, its callers are not regenerated, which bounds cascades up the zoom hierarchy. Packs and lockfile-pinned dependencies are high-durability inputs and validate in constant time per revision.

Overrides live in a separate layer keyed by stable ID with the `gen_key` they were made against; when the target changes they are re-applied if the ID survives and the field is still valid, and flagged `orphaned` otherwise, never silently dropped. Storage is one record per line or per-entity files sorted by ID, layered as base model at a commit, generated overlays, then overrides, the same stack shape as CodeQL's overlay databases. Fix-checking is a diff of two versions' behavior summaries, the idea of differential symbolic execution ([Person et al., FSE 2008](https://dl.acm.org/doi/10.1145/1453101.1453131)). No published system applies Salsa-style incrementality to LLM artifacts; this section is a design inference throughout.

## A worked example: report generation through a queue and a worker

The example is hand-constructed to exercise what the login scenario does not: asynchronous hand-off, a background worker, a CLI, a schedule, a lifecycle machine, a loop, retries and a race. It has not been run through an implementation. The system is a Python service: a FastAPI app exposes `POST /reports` and `GET /reports/{id}`; a Celery worker consumes `generate_report` from a Redis queue, pages through orders in Postgres, uploads a CSV to object storage and optionally calls a customer webhook; a beat schedule runs `requeue_stuck` every five minutes; and `reportctl retry <id>` resets a failed report. The YAML excerpts in the schema section are this system's records.

The extractors produce the skeleton without a model call. The compose file yields five containers and their port mappings; the worker has `deploy.replicas: 2`, so it renders as a stack of two. Route decorators yield two HTTP ports, the task decorator one queue-consumer port, the beat schedule one schedule port and the click definitions one CLI port. Exposure analysis marks the HTTP ports external because the compose file publishes the app's port, the CLI port external because nothing in the model invokes it, and the schedule port external with the clock as its actor; the worker's task port is internal because the API's `delay()` call requires it. The API and the worker are matched through an exact key, the queue name, and both wire to the broker store. The ORM model yields the `report` store and an enum column `status`; the state-variable extractor finds seven write sites across four entities. The object store and the customer webhook are opaque external entities whose interiors come from packs or are `assumed`.

Call D on `POST /reports` receives a handler of about forty lines, the pydantic model, five extracted branch points and three extracted effects. It returns three predicates bound to guards (`payload-valid` from the schema, `range-too-large` from a path condition, `key-seen` over the `report` store), two characteristics with their classes, and four cases: reject malformed with 422, reject too-large with 400, return the existing report for a repeated idempotency key, and the base case that inserts a `PENDING` row, sends on the enqueue port and replies 202. The coverage check finds that the fifth branch, a `try/except` around `delay()`, is unaccounted for; the retry adds a predicate over the broker's response and a fifth case that replies 503 after the insert, which the race and consistency views later show leaves a `PENDING` row with no message in flight. Call D on the worker port yields the four cases shown earlier, with `Task.retry` resolved through the Celery pack and its `attempts-exhausted` predicate bound by constant propagation to the decorator's `max_retries=3`. Call E assembles the machine from the write sites and marks the `PENDING → RUNNING` transition `atomic: false`, because the extractor sees a read of `status`, a comparison and a write with no row lock or conditional update between them.

The script "user requests a report, the first upload fails, the retry succeeds" needs one model call, to turn the sentence into a binding. Instantiation then produces a partial order with three lanes:

```yaml
steps:
  - {id: .../s:validate,     actor: reports/api,    kind: receive,     after: []}
  - {id: .../s:insert-row,   actor: reports/api,    kind: write-state, after: [s:validate]}        # status = PENDING
  - {id: .../s:enqueue,      actor: reports/api,    kind: send,        after: [s:insert-row]}
  - {id: .../s:reply-202,    actor: reports/api,    kind: send,        after: [s:enqueue]}
  - {id: .../s:queued,       actor: reports/broker, kind: queue,       after: [s:enqueue]}
  - {id: .../s:deliver-1,    actor: reports/worker, kind: receive,     after: [s:queued]}
  - {id: .../s:claim-1,      actor: reports/worker, kind: write-state, after: [s:deliver-1]}       # RUNNING, non-atomic
  - {id: .../s:page-orders-1,actor: reports/worker, kind: send,        after: [s:claim-1], loop: {trip: input-dependent}}
  - {id: .../s:upload-1,     actor: reports/worker, kind: error,       after: [s:page-orders-1]}   # pred:upload-ok = false
  - {id: .../s:reset-1,      actor: reports/worker, kind: write-state, after: [s:upload-1]}        # PENDING
  - {id: .../s:requeue,      actor: reports/worker, kind: send,        after: [s:reset-1]}
  - {id: .../s:deliver-2,    actor: reports/worker, kind: receive,     after: [s:requeue]}
  - {id: .../s:upload-2,     actor: reports/worker, kind: send,        after: [s:deliver-2]}       # intermediate steps elided
  - {id: .../s:mark-done,    actor: reports/worker, kind: write-state, after: [s:upload-2]}        # DONE
```

`s:reply-202` and `s:deliver-1` are both after `s:enqueue` and incomparable with each other, so the user's response and the worker's start render as concurrent lanes, which a sequence of numbered steps cannot express. The loop is one composite step that unrolls on demand. The broker hop is a `stutter` segment and collapses by default; the claim is `non-atomic` and does not. Three assumptions are minted and shown as forks: the webhook is not configured (base choice), the message is delivered once (the Celery pack's base choice), and the retry delay elapses without the sweeper intervening. Flipping the second, or composing this script with the `requeue_stuck` script, puts two `claim` steps on different worker replicas in incomparable positions on the same state variable with a non-atomic check-then-act, and the tier 6 check flags the race candidate that the status machine's `atomic: false` transition predicted. Nothing in this trace depends on HTTP: the CLI's `retry` behavior is two cases on one port, with characteristics from the click definitions and a state precondition, and a pure library would be the same records with exported functions as external ports and callbacks as slots.

## What is evidence, what is inference, and what to test first

The recommendations divide cleanly by how well they are supported. Evidence-backed, in the sense that published systems measure the pattern working: static tools own call graphs and enumeration; the LLM infers specifications and names things; LLM facts must bind to analyzer evidence; decomposed, per-unit calls beat end-to-end prompts; configuration and IaC are the primary source for components and connections; deterministic route extraction with a small scoped LLM pass for leftovers; reasoning-first schemas with strict structured output; mechanical anchor and reference checks with one targeted retry; an entity-resolution pass; a checker in the loop for any formal target; and the LLM never certifying infeasibility. Design inferences, assembled from adjacent evidence and endorsed as a whole by no source: local behaviors composed into a derived scenario graph; the predicate table as the single vocabulary for guards, classes, states and assumptions; the summary/full refinement obligation; branch and effect coverage as a validation tier; the atomicity field and its collapse rule; semantic-path IDs and multi-key anchors; Salsa-style memoization with early cutoff for LLM output; and status-field machine inference from write sites.

The changes to checkpoint #1 are four, each argued above. Positional step IDs become semantic paths, because the override layer cannot survive renumbering. Assumptions become records over predicates instead of strings on a step, because flipping one must be a mechanical edit. Provenance keeps three values but gains `source` and `checks`, because "inferred by parsing a compose file" and "inferred by a model" need to be distinguishable without spending `observed` on static facts. And the proposed rule that a collapsed segment never hides an error or assumption extends to non-atomic interleavings. Everything else, the three layers, scenario graph versus script, packs in the user-code format, boundary and interior, both facets, kernel as floor, boundary entities, collapse owned by the script, anchors, lazy generation and the override layer, is kept and given a precise meaning.

The open risks are specific. Predicate canonicalization is the most likely point of failure: the same condition will be named twice in different behaviors, and guards over local variables must be traced back to root predicates over inputs and state, a data-flow question the LLM answers per call without a global view. Case explosion is next; summaries need a cap and a rule for merging cases with identical observable effects, and no authoritative number exists for where to set it. Call-graph recall near 88% for JavaScript, and unmeasured for typed TypeScript and for framework-heavy Python, means some behaviors will end in `unknown` nodes more often than users tolerate. Composition can manufacture spurious global paths when two entities' predicates are correlated but modeled independently. Atomicity analysis at the database level, deciding whether a transaction, constraint or conditional update guards a check-then-act, is an open engineering question in the literature found. Most studies of state-machine and SysML generation used 2024-era models, so their error rates are not current, and several cited figures rest on search snippets or secondary summaries (RFC2TLA+ accuracy, the AWS use of P in the 2025 CACM article, PyCG's maintenance status). And nobody has studied whether users understand fork and assumption-flipping views.

The order of validation follows from which failures would invalidate the schema rather than merely the generator. First, hand-write the gold model for a system like the report example alongside the planned login script, and confirm every concept needed has a record type and every record type is used; the checkpoint's build step 1 should cover a queue-and-worker system and a CLI before the format freezes. Second, measure call D in isolation on repositories outside training data: per-claim precision by type (predicate binding, case guard, effect, state transition), branch-coverage failures caught by tier 3, retry success rate, and the assumption rate, since public benchmark scores are inflated by contamination ([SWE-Bench Illusion](https://arxiv.org/html/2506.12286v4)). Third, measure identity: ID survival across regeneration, anchor drift across a few hundred real commits, and override survival. Fourth, score entity and edge recovery against the released 17-application dataflow-diagram ground truth, aiming above 0.86 overall, then build a small polyglot gold set, because none exists. Fifth, collect one test-run trace per gold scenario; even before producer 2 is built, aligning a single trace to a script is the strongest available check on predicted paths.

## Conclusion

The literature does not contain the product's central artifact. There are flow summaries without branches, path conditions without names, state machines for protocols but not for order statuses, component diagrams without ports, and scenario notations without code anchors. What it does contain is a consistent account of where each producer can be trusted, and that account is enough to design against: a finite vocabulary of anchored predicates is the hinge, because it is the one object an extractor can enumerate, a model can name, a solver can check, a user can flip and a trace can confirm. Treating branch points, effects and ports as obligations the model's output must account for turns the generator's hardest failure, silent omission, into a countable validation error.

The more consequential implication is about where the value accumulates. If behaviors are local, case-structured and keyed by version and content hash, then every verified summary is reusable across projects and every observed trace is a test of a pack rather than of one script. The registry the checkpoint describes then stops being a cache of LLM output and becomes a body of per-version behavioral specifications with measured provenance, which is something no existing analyzer, model library or documentation generator publishes.
