# Recovering architecture, entities, entry points and inter-service interactions from code and IaC (state as of Nov 2026)

Scope note: research done 2026-11-11 with a limited tool budget (about 18 search/fetch calls). Numbers below come from primary papers or arXiv abstract/HTML pages unless marked otherwise. Items marked "(not re-verified this session)" are well-known tools whose repo URLs are given, but whose descriptions come from prior knowledge rather than a page fetched during this research. The report writer should treat those as lower confidence.

## 1. Architecture recovery research: classic clustering techniques and microservice-specific recovery

### Takeaway

Classic module-clustering recovery (ACDC, ARC, Bunch, LIMBO, WCA, ZBR, RELAX) works poorly on monoliths: averaged MoJoFM is only about 38-47%, and only ACDC reliably beats k-means. Microservice recovery is a different and much easier problem, because service boundaries are explicit in build and deploy artifacts. The best static tool, Code2DFD, reaches F1 0.98 on components and 0.87 on connections, but only 0.66 on endpoints, measured on 17 mostly Java/Spring apps. The hard parts are endpoints and edges, not boxes.

### Cited Findings

#### Classic recovery (mostly 2013-2019; partly superseded as a research focus, but still the baseline)

- Garcia, Ivkovic and Medvidovic (ASE 2013) compared six recovery techniques against eight independently verified ground-truth architectures. Two techniques routinely beat the rest, "but even the best of the lot has surprisingly low accuracy." — [researchr: GarciaIM13](https://researchr.org/publication/GarciaIM13); [ASE 2013 record](https://search.isc.ac/inventory/49/660580.htm)
- The two winners were ARC and ACDC, and "even the accuracy of these techniques showed significant room for improvement" (follow-up by the same group). — [ARCADE MSR 2015](https://jgarcia.ics.uci.edu//wp-content/uploads/arcade_msr_2015.pdf)
- Lutellier et al. (ICSE 2015; TSE extension) compared nine variants of six techniques. Only ACDC consistently beat a k-means baseline, and overall accuracy was low. Symbol-level dependencies gave higher accuracy than include-level dependencies, so the quality of the input dependency graph matters a lot. — [Lutellier et al. ICSE 2015](https://www.cs.purdue.edu/homes/lintan/publications/archrec-icse15.pdf); [TSE version](https://www.cs.purdue.edu/homes/lintan/publications/archrec-tse17.pdf); [UWSpace thesis](https://uwspace.uwaterloo.ca/handle/10012/9554?show=full)
- A 2023 information-fusion study reports averaged MoJoFM of ACDC 47, ARC 41 and Bunch-NAHC 38, against a2a scores of 81, 82 and 81 (Table 5). a2a is far more lenient than MoJoFM. — [Software Architecture Recovery with Information Fusion, arXiv 2311.04643](https://arxiv.org/pdf/2311.04643)
- RELAX (concern-oriented recovery, 2019) reports per-system MoJoFM where ACDC and ARC trade places. ACDC is higher on ArchStudio (87.68 vs 76.28) and Hadoop (62.92 vs 54.28); ARC is higher on Linux-D (51.47 vs 36.31) and OODT (48.48 vs 46.01). — [Recover and RELAX, arXiv 1903.06895](https://arxiv.org/pdf/1903.06895)
- Bunch's accuracy "was shown to be low in recent empirical studies." — [summarized in search result, arXiv 2311.04643](https://arxiv.org/pdf/2311.04643)
- 2025-2026 work layers LLMs on top of classic recovery, for example "Semantic-Enhanced Automatic Refinement of Architecture Recovery Results Using LLMs" and lightweight Java module recovery that embeds software intent. Only titles and abstracts were seen; no numbers were verified. — [arXiv 2607.23774](https://arxiv.org/pdf/2607.23774); [arXiv 2512.15980](https://arxiv.org/pdf/2512.15980)

#### Microservice-specific static recovery

- Code2DFD (Schneider and Scandariato, JSS 202:111722, 2023) extracts security-annotated dataflow diagrams from Java/Spring microservices by keyword and pattern search over source and configuration files (build files, docker-compose, application properties). It reports precision 93% and recall 85% on a curated set of 17 open-source apps, and the dataset is released. — [arXiv 2304.12769](https://arxiv.org/abs/2304.12769v1); [TUHH record](https://tore.tuhh.de/signposting/describedby/9e9162e6-4baa-4202-b007-84883526ec9a); [repo tuhh-softsec/code2DFD](https://scriptagc.wasmer.app/https_GitHub_com/tuhh-softsec/code2DFD/tree/0af4b1683f611eecdb13b1c457801a5904aad150)
- **Key benchmark:** "Comparison of Static Analysis Architecture Recovery Tools for Microservice Applications" (Schneider, Bakhtin, Li, Soldani, Brogi, Cerny, Scandariato, Taibi). Its method was a registered report at MSR'24 (arXiv 2403.06941), and it is listed at ICSA 2026. A multivocal review found 13 tools; 9 were executable. The ground truth is manually verified DFDs of 17 mostly Java/Spring apps, plus 63 endpoints found via Spring mapping annotations. — [arXiv 2412.08352 abstract](https://arxiv.org/abs/2412.08352); [HTML full text](https://arxiv.org/html/2412.08352); [ICSA 2026 listing (via search)](https://conf.researchr.org/details/icsa-2026/icsa-2026-papers/27/LLMs-for-Architectural-Refactoring-An-Exploratory-study-on-Monoliths-to-Microservice)
  - F1 by tool (components / connections / endpoints / all), from Table 5 — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352):
    - Code2DFD: 0.98 / 0.87 / 0.66 / **0.86**
    - MicroDepGraph: 0.87 / 0.67 / – / 0.74
    - Attack Graph Generator: 0.80 / 0.54 / – / 0.60
    - microMiner: 0.71 / – / – / 0.71
    - RAD: – / 0 / 0.79 / 0.37
    - RAD-source: – / 0.04 / 0.67 / 0.29
    - Prophet: 0.23 / 0.02 / 0 / 0.08
    - MicroGraal: – / 0 / 0 / 0
    - ContextMap (the only proprietary tool): – / 0 / 0 / 0
  - Code2DFD missed only 5 of 182 ground-truth components: component recall 0.98, precision 0.97. — [arXiv 2412.08352](https://arxiv.org/pdf/2412.08352)
  - Combining four tools reached F1 0.91 overall. The combination appears to be Code2DFD + MicroDepGraph + RAD + RAS-source, but that is inferred from the table IDs and not stated explicitly. — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352)
  - Many tools detected few or no connections. The main sources of false positives were inaccurate heuristics and implementation flaws. Reproducibility was poor: several tools needed modifications or manual steps. — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352)
- Prophet (Cerny group, Baylor) statically analyzes Java microservices to produce service views and domain views (context maps, inter-service communication diagrams) without runtime data. It feeds the Microvision AR visualization. — [Bushong, Das, Cerny 2022, "Reconstructing the Holistic Architecture of Microservice Systems using Static Analysis"](https://www.scitepress.org/Papers/2022/110321/); [NSF PAR copy](https://par.nsf.gov/servlets/purl/10393890)
- A different "Prophet" plugin inside MicroGraal traverses the GraalVM native-image IR to collect method calls and constants such as URL strings. The two Prophets are easy to confuse. — [CEUR Vol-4278 paper10](https://ceur-ws.org/Vol-4278/paper10.pdf)
- Cerny and colleagues list persistent limitations of reconstruction: dependence on language and framework, ambiguous endpoint matching, incomplete runtime coverage, and the lack of a shared intermediate representation. — [CEUR Vol-4278 paper10 (as summarized by search)](https://ceur-ws.org/Vol-4278/paper10.pdf)
- MicroART (Granchelli et al., ICSA 2017), Arcan (architectural smells), MicroDepGraph (Rahman/Taibi) and the Cerny et al. mapping study "Microservice architecture reconstruction and visualization techniques and tools" (JSS 2022) are the standard references. They were not re-fetched this session; MicroDepGraph's accuracy is in the benchmark above. — [arXiv 2412.08352](https://arxiv.org/html/2412.08352) (MicroDepGraph only)

### Inferences

- Our problem looks like the microservice case (services are explicit in Dockerfiles and compose/K8s manifests), not the monolith-clustering case. Clustering such as ACDC belongs only *inside* a large service, for grouping modules into sub-components. Never use it to find services.
- Even the best tool drops to F1 0.66 on endpoints. Combining tools gains about 5 F1 points (0.86 to 0.91). The evidence therefore supports an ensemble of deterministic extractors plus reconciliation, which is the role our LLM should play.
- The benchmark is almost entirely Java/Spring. There is no evaluated tool for polyglot stacks such as Node/Express, Next.js, FastAPI or Go net/http. That is a real gap, and an opportunity.

### Gaps

- MicroART and Arcan accuracy numbers, and the 2025-2026 microservice-reconstruction surveys, were not fetched.
- Per-tool precision/recall (as opposed to F1), and how the benchmark counted message-broker connections, were not visible in the part of the paper that was read. The final 55k characters of the HTML were not read.
- No benchmark was found that measures recovery of physical topology (machine, container, process) or port delegation chains. None appears to exist.

## 2. Inter-service edge detection (HTTP client-to-route matching, gRPC, GraphQL, queues, meshes, cross-service taint)

### Takeaway

Static edge detection is the weakest link in published tools: connection F1 is 0 to 0.67 for everything except Code2DFD (0.87). The best recent work on cross-service *dataflow* is MScan (IEEE S&P 2025). It stitches per-service taint analyses at HTTP and RPC boundaries and reports 59 zero-days at about 72% precision, against CodeQL's 23 at under 40%. Agentic follow-ups such as NEO (2026) combine CodeQL with LLM reasoning.

### Cited Findings

- Connection-detection F1 in the 2024-2026 benchmark: Code2DFD 0.87, MicroDepGraph 0.67, Attack Graph Generator 0.54, RAD-source 0.04, Prophet 0.02, and RAD, MicroGraal and ContextMap 0. — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352)
- Ambiguous endpoint matching is a known open problem in microservice reconstruction. — [CEUR Vol-4278 paper10](https://ceur-ws.org/Vol-4278/paper10.pdf)
- MicroGraal works around dynamic URLs by reading constant URL strings from GraalVM native-image IR, which is a form of constant propagation over a compiler IR. — [CEUR Vol-4278 paper10](https://ceur-ws.org/Vol-4278/paper10.pdf)
- MScan (Fudan / Alibaba, IEEE S&P 2025) detects taint-style vulnerabilities across microservice boundaries. It uses distance-guided, context-sensitive taint analysis so it does not blow up memory or time out the way whole-program analysis does. It was evaluated on 25 open-source and 5 industrial Java microservice apps. — [MScan paper PDF](https://yuanxzhang.github.io/paper/mscan-oakland25.pdf); [Fudan SecSys listing](https://secsys.fudan.edu.cn/publications/content/2025-12-09-008)
- In the Black Hat USA 2025 version of the work, MScan reports 59 zero-days at about 72% precision, against CodeQL's 23 findings at under 40% precision. — [Black Hat 2025 BH25-040 listing](https://conference-intel.pages.dev/c/blackhat-usa-2025/BH25-040/)
- A third-party talk summary says the tool uses LLMs to find externally reachable APIs from gateway configs. This is **unconfirmed** against the paper, and the summary also misspells the tool's name. — [Pulse summary](https://www.pulse.bot/cybersecurity/video/black-hat-usa-2025-detecting-taint-style-vulnerabilities-in-microservice-structured-web-apps-c768f6c1-cb65-4818-b5e7-26b1ab61433f/)
- NEO (preprint; S&P 2026 per the authors) is an agentic framework built on CodeQL primitives plus LLM reasoning for privilege-escalation bugs in microservices. It reports 24 zero-days, 81.0% precision and 85.0% recall on a ground-truth set, and notes that MScan does not reason about privileges. — [NEO PDF](https://peng-hui.github.io/data/paper/sp26:neo.pdf); [arXiv 2605.15569](https://arxiv.org/html/2605.15569)
- Industry commentary (Gecko Security, January 2026): taint analysis misses authorization and business-logic flaws that depend on cross-service context. LLM-based "semantic" analysis is pitched as the fix. This is vendor opinion. — [NHI Mgmt Group summary](https://nhimg.org/articles/semantic-code-analysis-is-exposing-business-logic-flaws-sast-misses/)
- protoc-gen-scip, a SCIP indexer for protobuf, was among the 13 identified tools but was excluded from execution as not applicable. That makes it a candidate building block for gRPC edges rather than a full recovery tool. — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352)

### Inferences

- For our pipeline, edges come in two strengths, which map naturally to `inferred` and `assumed` provenance:
  - **Exact contracts:** proto service/method, GraphQL schema field, OpenAPI operationId, Kafka or RabbitMQ topic/queue name string. These can be matched deterministically.
  - **Heuristic contracts:** HTTP method plus normalized path template (with base URL resolved through env and config), or client SDK generated from a spec. These need a normalization step and a confidence score.
- MScan's approach (analyze each service, then stitch summaries at boundaries) is the scalable pattern. Run per-repo analysis, export boundary summaries (routes provided, calls required), then join across repos. This matches our behavior-pack and summary design.
- Unresolved base URLs (`process.env.AUTH_URL`) should be resolved by joining against the IaC environment (compose `environment:`, K8s `env`/ConfigMap, Helm values) and service DNS names. That join is what turns a client call into a service-to-service edge.

### Gaps

- No paper with accuracy numbers was found for cross-language matching of HTTP client calls to server routes (for example a TS `fetch` to a Go handler), or for queue-topic matching. Searches were limited.
- No published evaluation was found for static extraction of service-mesh routes (Istio VirtualService, Envoy) as edges.
- GraphQL-specific inter-service edge recovery was not researched.

## 3. Entry points and attack surface (framework routes, OpenAPI from code, frontend handlers)

### Takeaway

Extracting route tables from server code is now close to solved for mainstream frameworks. OOPS (JSS 2026), an LLM agent with an API dependency graph, reports F1 above 98% for endpoint methods across 5 languages and 8 frameworks. Respector (ICSE 2024) is the static and symbolic baseline, but it supports only Spring Boot and Jersey. Linking frontend events (forms, buttons) to API calls has no evaluated tool that we found.

### Cited Findings

- Respector (ICSE 2024) is the first static and symbolic analysis that generates OpenAPI specs from REST API code. It finds endpoint methods, then derives URI, HTTP verb, parameters and response types. It was evaluated on 15 real-world APIs, produced specs more accurate than other generators, and exposed mismatches between hand-written specs and the code. It supports only Spring Boot and Jersey. — [Respector paper (NSF PAR)](https://par.nsf.gov/servlets/purl/10537946); [ICSE 2024 replication artifact](https://conf.researchr.org/details/icse-2024/icse-2024-artifact-evaluation/14/A-Replication-of-Generating-REST-API-Specifications-through-Static-Analysis-)
- A later study found limitations in both Respector and Prophet when generating specs. — [arXiv 2410.23873](https://arxiv.org/html/2410.23873v1)
- **OOPS** (Chen et al., JSS vol. 239, 2026, art. 112914; arXiv 2601.12735) is a technology-agnostic, LLM-based static generator of OpenAPI 3.0 specs.
  - It works in two stages, endpoint-method extraction and then spec generation, over an API dependency graph that links related files and keeps context small. Multi-stage generation and self-refinement reduce hallucination.
  - It was evaluated on 12 real-world APIs in 5 languages and 8 frameworks.
  - Average F1: above 98% for endpoint methods, 97% for request parameters and responses, 92% for parameter constraints.
  - Average input below 5.6K tokens (max 16.13K); average output below 0.9K (max 7.63K).
  - [arXiv 2601.12735](https://arxiv.org/abs/2601.12735)
- AutoOAS (Lercher et al. 2024) is another automatic OpenAPI inference approach. It was not fetched. — [Lercher2024-AutoOAS PDF](https://pinzger.github.io/papers/Lercher2024-AutoOAS.pdf)
- In the microservice benchmark, endpoint detection is the weakest category: Code2DFD 0.66 F1, RAD 0.79, RAD-source 0.67. The ground truth itself was built from Java mapping annotations. — [arXiv 2412.08352 HTML](https://arxiv.org/html/2412.08352)

### Inferences

- Use deterministic, framework-specific route extractors first. Express `app.get`/`router.use` mount prefixes, FastAPI decorators and `include_router(prefix=)`, Django `urlpatterns`, Spring `@RequestMapping`, Go `http.HandleFunc`/chi/gin, and Next.js file-system routes (`app/**/route.ts`, `pages/api`) are all syntactic patterns that tree-sitter queries can match. Add the LLM only for dynamic registration and middleware composition, in the OOPS style: a dependency-graph-scoped context of about 5K tokens per endpoint.
- Where a framework emits OpenAPI at runtime (FastAPI's `/openapi.json`, springdoc), run the app or import its module to dump the spec. That ground truth is cheap. This is our proposal, not evaluated in a source.
- Frontend to API: link JSX `onSubmit`/`onClick` handlers through call graphs to `fetch`/axios/tRPC/GraphQL client calls, then match them to server routes as in section 2. This is the first hop of the "press submit on the login form" scenario, and we found no evaluated tool for it.

### Gaps

- No evaluation was found of route extraction for Express, Next.js or Go specifically, or of frontend-event-to-API linking.
- OOPS's per-framework breakdown and baseline comparisons were not visible in the abstract.

## 4. Infrastructure-as-code analysis (Dockerfile, compose, K8s/Helm, Terraform, proxies, cloud LBs)

### Takeaway

Tooling for IaC graphs exists, but it is fragmented and mostly shows resource-dependency graphs, not network delegation chains. That covers Terraform visualizers (terraform graph, inframap, rover), asset graphs (Cartography), and policy graphs (Checkov). Research on "deployment model recovery" (EDMM/DeMAF, µTOSCA mining from K8s plus Istio) maps technology-specific IaC into technology-agnostic topology models. That is the closest analogue to our physical facet, but it is all prototypes.

### Cited Findings

- `terraform graph` emits a DOT graph of resource dependencies. Inframap builds provider-aware diagrams from Terraform state or HCL, shows only the "important" resources, and can prune secrets. Rover offers an interactive resource map and dependency graph via Docker. — [phoenixNAP review](https://phoenixnap.com/kb/terraform-visualization); [Spacelift: terraform graph](https://spacelift.io/blog/terraform-graph); [Spacelift visualization tools](https://spacelift.io/blog/terraform-visualization)
- DeMAF transforms technology-specific deployment models, including Kubernetes configs, into the technology-agnostic Essential Deployment Metamodel (EDMM). — [Springer SN Computer Science s42979-023-01721-4 (as summarized by search)](https://link.springer.com/article/10.1007/s42979-023-01721-4)
- µTOSCA mining (Soldani/Brogi line of work) reconstructs microservice architecture specs offline from existing Kubernetes manifests plus Istio/Kiali data, with no special test deployment needed. The search result attributing this was not fetched, so which paper it is remains uncertain. — [Springer s42979-023-01721-4](https://link.springer.com/content/pdf/10.1007/s42979-023-01721-4.pdf); related microMiner F1 0.71 on components in [arXiv 2412.08352](https://arxiv.org/html/2412.08352)
- Model-to-IaC (the reverse direction) is more mature: MiCADO compiles TOSCA templates into Kubernetes, Terraform and Ansible, and Telecom SudParis generates Compose files and Dockerfiles from TOSCA. — [Westminster FIST2022](https://westminsterresearch.westminster.ac.uk/download/d8fd8d1ca6c75555a632b17b33c9eeabfb3582656b36849471bb392ff197ac92/125148/FIST2022-v2.1.pdf); [SIMBAD DevOps-generation](https://www-inf.telecom-sudparis.eu/SIMBAD/tools/DevOps-generation/)
- Code2DFD reads deployment and config files (docker-compose, Spring application properties, gateway and discovery config) to infer components and connections. That is why it leads the benchmark. — [arXiv 2304.12769](https://arxiv.org/abs/2304.12769v1)
- Tools not re-verified this session, given as pointers only:
  - Cartography (CNCF) ingests cloud and K8s assets into a Neo4j graph. — [github.com/cartography-cncf/cartography](https://github.com/cartography-cncf/cartography)
  - Checkov builds a resource graph for cross-resource policies over Terraform, K8s, Helm and Dockerfiles. — [github.com/bridgecrewio/checkov](https://github.com/bridgecrewio/checkov)
  - KubeLinter lints K8s YAML and Helm. — [github.com/stackrox/kube-linter](https://github.com/stackrox/kube-linter)
  - Inframap repo. — [github.com/cycloidio/inframap](https://github.com/cycloidio/inframap)
  - Rover repo. — [github.com/im2nguyen/rover](https://github.com/im2nguyen/rover)

### Inferences

- Port-delegation chains (`host:443 → container:8443 → process listen :8080`) are mostly *deterministic* from IaC and do not need an LLM. Each hop maps to a source:

  | Hop | IaC source |
  | --- | --- |
  | Host port to container port | compose `ports:`, `docker run -p` |
  | Container port to process listen port | Dockerfile `EXPOSE` (advisory), plus the app's `listen()` resolved via env |
  | Cluster exposure | K8s Service `port`/`targetPort`/`nodePort`, plus Pod `containerPort` |
  | Ingress routes and TLS termination | Ingress or Gateway-API `HTTPRoute` host/path rules; nginx `server`/`listen`/`location`/`proxy_pass`; Envoy listener, route and cluster |
  | Cloud load balancers | Terraform `aws_lb_listener`/`aws_lb_target_group`, or GCP forwarding rules |

  Render engines first: `helm template`, `kustomize build`, `terraform plan -json` / `terraform show -json`. Then parse the resolved output, not raw templates.
- The kernel-level hops (iptables DNAT, veth, bridge) are not written in any repo. They come from Docker and Kubernetes semantics, so they belong in platform behavior packs marked `assumed`. This matches checkpoint-01's "kernel is the floor".
- EDMM and TOSCA are a useful prior-art vocabulary for our physical-facet types (Compute, Container, SoftwareComponent, hostedOn and connectsTo relations). Reusing their relationship types eases interop.

### Gaps

- No accuracy evaluation was found for IaC-based topology recovery, such as ingress-to-service-to-pod chains or TLS termination points. None appears to exist.
- Cartography, Checkov's graph, nginx/Envoy config parsers (for example crossplane or nginx-config parsers) and cloud LB extraction were not verified in this session.

## 5. Runtime and observability topology as validation

### Takeaway

Runtime service graphs come mainly from traces (the OpenTelemetry Collector `servicegraph` connector pairs client and server spans into edge metrics) or from eBPF capture (Pixie builds service maps from captured HTTP traffic without agents). They give observed edges we can compare against statically inferred ones, but they only see traffic that actually ran.

### Cited Findings

- The OpenTelemetry Collector `servicegraphconnector` derives edges by pairing client and server spans and emits request, failure and latency metrics per edge. It represents uninstrumented peers such as databases as virtual nodes via `virtual_node_peer_attributes`. Unpaired spans are held in an in-memory store that must be tuned. — [pkg.go.dev servicegraphconnector](https://pkg.go.dev/github.com/open-telemetry/opentelemetry-collector-contrib/connector/servicegraphconnector); [OneUptime guide, Feb 2026](https://oneuptime.com/blog/post/2026-02-06-otel-service-dependency-graphs-traces/view); [OneUptime connector config](https://oneuptime.com/blog/post/2026-02-06-service-graph-connector-opentelemetry-collector/markdown)
- Pixie is open-source Kubernetes observability. It uses eBPF to collect telemetry without language agents and builds a service map from captured HTTP traces. — [New Relic Pixie docs](https://docs.newrelic.com/docs/auto-telemetry-pixie/get-started-auto-telemetry-pixie/)
- µTOSCA/microFreshener-style mining combines static K8s manifests with Istio/Kiali runtime data. This is a precedent for static-plus-runtime fusion. — [Springer s42979-023-01721-4](https://link.springer.com/article/10.1007/s42979-023-01721-4)
- Kiali (Istio service-mesh graph built from Envoy telemetry in Prometheus) was not re-verified. — [kiali.io](https://kiali.io)

### Inferences

- Validation metric for producer 2: precision and recall of statically recovered edges against servicegraph edges, over a window of traffic. A static edge with no observed traffic is "unexercised", not necessarily wrong. An observed edge with no static counterpart is a "surprise", meaning the graph is incomplete, exactly as checkpoint-01 says.
- The `code.filepath`/`code.lineno` span attributes noted in checkpoint-01 allow alignment at step level, not just edge level.

### Gaps

- No paper was found that measures static-versus-runtime topology agreement for microservices with numbers.

## 6. LLM-based architecture recovery and documentation generation (2024-2026)

### Takeaway

LLM-based recovery is active but weakly evaluated. Wiki products (DeepWiki, Google Code Wiki, CodeBoarding) have no independent accuracy measurement; the best numbers come from competitors' own benchmarks (CodeWiki 68.79% vs DeepWiki 64.06% on rubric scoring). Automatic metrics disagree with human ranking. Research systems (ArchAgent, ICASSP 2026; RE plus LLM for architecture descriptions) all combine static analysis with LLM synthesis, which supports a "grounded LLM" design.

### Cited Findings

- **ArchAgent** (Pan, Mao, Ma, Ling; ICASSP 2026; arXiv 2601.13007) combines static analysis, adaptive code segmentation, contextual pruning and LLM synthesis to reconstruct multi-view, business-aligned architectures across multiple repositories. An ablation shows dependency context improves accuracy on production repos. The abstract gives no numbers; a benchmark is at github.com/panrusheng/arch-eval-benchmark. — [arXiv 2601.13007](https://arxiv.org/abs/2601.13007)
- "Generating Software Architecture Description from Source Code using Reverse Engineering and Large Language Model" (arXiv 2511.05165) is a semi-automated method that feeds reverse-engineering outputs to an LLM. Not fetched. — [arXiv 2511.05165](https://arxiv.org/html/2511.05165v1)
- An LLM-from-requirements microservice design study reports F1 of about 0.82 for recovering communication links with few-shot prompting. This is *design synthesis*, not recovery from code, so it is only indirectly relevant. — [arXiv 2607.28307](https://arxiv.org/html/2607.28307v1)
- AI-for-software-architecture literature review (2025). — [arXiv 2504.04334](https://arxiv.org/pdf/2504.04334)
- **CodeWiki benchmark** (arXiv 2510.24428): hierarchical rubrics are derived from official, developer-maintained docs, and three LLM judges score the output. CodeWiki (generated with Claude Sonnet 4) scores 68.79% against DeepWiki's 64.06%; DeepWiki was better on C/C++ repos. The authors built both the system and the benchmark. — [arXiv 2510.24428](https://arxiv.org/html/2510.24428v5)
- **Repository-documentation evaluation via QA and feature-driven development** (arXiv 2604.06793) compares DeepWiki, DocAgent, AutoDoc and RepoAgent. DeepWiki reached 47.37% on issue solving against RepoAgent's 52.63%. Automatic-metric rankings differ from human rankings, so they "are not reliable enough to replace human evaluation." — [arXiv 2604.06793](https://arxiv.org/pdf/2604.06793)
- Maintainers have reported DeepWiki hallucinating convincingly, for example when names no longer match behavior. These are anecdotes. — [as summarized via search; see Bitovi comparison](https://www.bitovi.com/blog/comparing-ai-documentation-engines-deepwiki-vs-code2tutorial)
- **Google Code Wiki** (preview Nov 2025) is a Gemini-powered, continuously regenerated wiki per repo, with architecture, class and sequence diagrams and links to every symbol.
  - The public preview covers public repos; a Gemini CLI extension for private repos is waitlisted.
  - Critics doubt an LLM's context suffices to piece together a whole codebase's architecture.
  - [Google Developers Blog](https://developers.googleblog.com/es/introducing-code-wiki-accelerating-your-code-understanding/); [InfoQ](https://infoq.com/news/2025/11/google-code-wiki/); [The Register](https://www.theregister.com/2025/11/17/google_previews_code_wiki/)
- LLMs extracting UML class diagrams from Java and Python programs: an empirical study at STAF/A-MDE 2025. Numbers were not fetched. — [STAF 2025](https://conf.researchr.org/details/staf-2025/a-mde-2025-papers/6/Using-LLMs-to-Extract-UML-Class-Diagrams-from-Java-and-Python-Programs-An-Empirical-)
- CodeBoarding (open-source, static analysis plus LLM, interactive diagrams) and Archify were not verified this session. — [CodeBoarding write-up cited in checkpoint-01](https://blog.brightcoding.dev/2026/02/16/codeboarding-ai-powered-interactive-diagrams-for-codebases)

### Inferences

- No product publishes accuracy on *runtime topology* (entities, ports, edges). They evaluate prose documentation quality. Our claims should be measured against a DFD-style ground truth (as in arXiv 2412.08352) rather than rubric-based doc scores.
- The consistent pattern across ArchAgent, OOPS and NEO is static analysis to scope context, then LLM synthesis or verification. That is the pattern to copy. Unconstrained "read the repo and draw a diagram" is where the hallucinations come from.

### Gaps

- No head-to-head of LLM-based versus static microservice topology recovery on the same ground truth was found. The Schneider et al. benchmark excludes LLM tools.
- ArchAgent, CodeBoarding and Code Wiki component-level accuracy numbers were not available.

## 7. Recommendations: a concrete pipeline for entities, ports, delegation and inter-service edges

### Takeaway

Deterministic extractors should build the skeleton: entities and port mappings from rendered IaC, route tables from framework queries, exact contracts from proto, GraphQL and topic names. The LLM should do the bounded jobs: resolving dynamic URLs and config, matching ambiguous client calls to routes, naming and grouping, and filling unwired ports as `assumed`. Verify against the published DFD benchmark, and later against OpenTelemetry service graphs.

### Cited Findings

- Combining static tools beats any single tool (F1 0.86 to 0.91), and connections and endpoints are where single tools fail. — [arXiv 2412.08352](https://arxiv.org/abs/2412.08352)
- Config and deployment files are the strongest signal in the best tool. — [Code2DFD, arXiv 2304.12769](https://arxiv.org/abs/2304.12769v1)
- Better input dependencies (symbol-level rather than include-level) improve recovery. — [Lutellier et al. ICSE 2015](https://www.cs.purdue.edu/homes/lintan/publications/archrec-icse15.pdf)
- Context bounded by a dependency graph keeps LLM route extraction accurate and cheap: over 98% F1 at under 5.6K input tokens on average. — [OOPS, arXiv 2601.12735](https://arxiv.org/abs/2601.12735)
- Analyzing per service and stitching at boundaries scales cross-service flow analysis where whole-program analysis times out. — [MScan](https://yuanxzhang.github.io/paper/mscan-oakland25.pdf)

### Inferences (proposed pipeline; our design, not from any source)

1. **Inventory (deterministic).** Per repo, detect languages and frameworks from manifests and lockfiles. Find every Dockerfile, compose file, Chart.yaml, kustomization, `*.tf`, nginx/Envoy/Traefik config, `*.proto`, `*.graphql` and OpenAPI file.
2. **Render IaC before parsing.** Run `helm template`, `kustomize build`, `docker compose config` and `terraform show -json` (on a plan), so that values and overlays are resolved. Unresolvable values become assumption slots.
3. **Physical and logical entities, plus port delegation (deterministic).**
   - Map compose services and K8s Deployments to container and replica entities, and K8s Service/Ingress/Gateway, nginx and cloud LBs to boundary entities.
   - Build delegation wires from the port fields listed in section 4. Each wire is anchored to a file and line and marked `inferred`. Kernel hops (DNAT, veth) come from platform packs and are marked `assumed`.
4. **Entry points (deterministic first).** Run tree-sitter or Semgrep-style queries per framework for routes, including mount prefixes. Ingest an OpenAPI file or a runtime spec dump where one exists. Use an OOPS-style LLM pass only for the leftovers: dynamic registration, decorators-by-factory, middleware. Link each route to its handler symbol via a SCIP or LSP index.
5. **Outbound calls (deterministic plus constant propagation).**
   - Find HTTP client calls (fetch, axios, requests, http.Client, RestTemplate/Feign/WebClient), gRPC stubs, GraphQL operations, and producer/consumer calls for Kafka, RabbitMQ, SQS and NATS, plus DB drivers.
   - Resolve URL, topic and host strings by constant propagation, then join with the env from step 3.
6. **Edge matching (deterministic, then LLM).**
   - Exact keys first: proto `package.Service/Method`, topic name, GraphQL field, operationId.
   - Then HTTP: resolve host to a service by DNS name, env var or ingress host, and match method plus a normalized path template against the route table.
   - Send the remaining ambiguous candidates to the LLM with only the call site, the candidate routes and the env, and ask for a choice plus a rationale. Record `inferred`, or `assumed` with confidence.
7. **User-facing services.** These are the entities reachable from an external port: an ingress or LB with a public host, a published host port, or a frontend bundle. For a web scenario, start from the frontend routes and form handlers and follow them through step 6.
8. **LLM consolidation.** The LLM names services, groups them into logical systems, marks trivial collapse candidates, and proposes fills for every unwired port. This applies the checkpoint's validation rule: "open entity must account for every port."
9. **Verification.**
   - Regression-test on the 17-app DFD dataset from Code2DFD and arXiv 2412.08352, scoring F1 per category (components, connections, endpoints); aim above Code2DFD's 0.86.
   - Build our own polyglot gold set, including the hand-written login script.
   - Consistency checks: every client edge resolves to a route or is flagged, and every exposed port is wired.
   - Later, compare against the OpenTelemetry servicegraph or Pixie edges from section 5.

### Gaps

- No source evaluates this specific combination. Its expected accuracy beyond the Java/Spring benchmark is unknown.
- There is no published polyglot (TS, Python, Go) ground-truth dataset for microservice topology. We will likely need to build one.
