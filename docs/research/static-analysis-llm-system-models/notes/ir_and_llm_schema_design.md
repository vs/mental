# Code IRs and schema design for LLM generation and verification (state as of November 2026)

Scope: (a) existing code IRs and indexes the mental-model tool can ground on and anchor to; (b) evidence-based principles for schemas that LLMs generate reliably, with stable identity, incremental updates and mechanical verification. Project context: /Users/vs/workspace/mental/docs/brainstorm/checkpoint-01.md (entities, ports, delegation, scenario graphs, scripts, provenance `inferred|assumed|observed`, lazy zoom cached by code content hash, override layer keyed by stable IDs, rule "an open entity must account for every one of its ports").

Research note: about 30 search and fetch calls. Primary sources were fetched for SCIP (scip.proto), the SCIP announcement, the Kythe schema, the Glean engineering post, the Salsa algorithm page, the stack graphs paper abstract and the "Structure Tax" AACL 2026 page. Other findings rest on search-result summaries of primary papers and are marked where precision matters.

---

## 1. Code IRs and indexes: what each represents, granularity, identity, incrementality, coverage

### Takeaway

For the anchoring problem, the best fit is **SCIP**: a document-oriented Protobuf index with human-readable, package-qualified string symbols, ranges per occurrence, and range plus enclosing-range for definitions. **Tree-sitter** fills the syntactic gaps, for languages without a SCIP indexer and for fast incremental re-parsing on edit. Glean, Kythe, CodeQL and Joern CPG are richer but heavier. They suit specific deep queries (dataflow, taint, call graphs) better than a universal identity layer. GitHub's stack graphs are archived (Sep 2025), and LSIF is superseded by SCIP. Neither should be a new dependency.

### Cited Findings

#### SCIP (Sourcegraph), current

- Symbol grammar: `<symbol> ::= <scheme> ' ' <package> ' ' (<descriptor>)+ | 'local ' <local-id>`; `<package> ::= <manager> ' ' <package-name> ' ' <version>`. Descriptor suffixes: namespace `/`, type `#`, term `.`, meta `:`, macro `!`, method `(disambiguator).`, type parameter `[name]`, parameter `(name)`. Names with unusual characters are backtick-escaped — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- Local symbols "MUST only be used for entities which are local to a Document". Global descriptors "should serve as a unique identifier across the package". **The spec makes no statement about symbol stability across commits.** The package version is part of the symbol, so a version bump renames every symbol in that package — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- Occurrences carry typed ranges (`single_line_range` / `multi_line_range`, 0-based, half-open) and an optional `enclosing_range`, "the nearest non-trivial enclosing AST node". `SymbolRole` is a bitset: Definition 0x1, Import 0x2, WriteAccess 0x4, ReadAccess 0x8, Generated 0x10, Test 0x20, ForwardDefinition 0x40. Relationship flags are `is_reference`, `is_implementation`, `is_type_definition` and `is_definition` — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- `Document.relative_path` is required, relative to project root, '/'-separated, and must be a regular file. `position_encoding` is UTF-8, UTF-16 or UTF-32 code units from line start, and "unspecified" should not be used by new indexers. `text` is optional and not included by default — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- The spec recommends streaming the Index "one field value at a time" because of its memory footprint. It does not describe an incremental-indexing protocol — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- Why SCIP replaced LSIF (June 8, 2022): LSIF's graph encoding had no machine-readable schema. Its opaque numeric IDs made payloads hard to debug, and globally incrementing IDs made it hard to "update an existing index with new information for only a subset of the documents". SCIP uses human-readable string IDs and Protobuf. Gzipped LSIF was on average 4x larger than SCIP, and scip-typescript gave a 10x CI speedup over lsif-node, though "not attributed to protocol differences alone". Meta's Don Stewart reported SCIP "8x smaller, and can be processed 3x faster" than LSIF — [Sourcegraph blog](https://sourcegraph.com/blog/announcing-scip)
- Glean ships a SCIP schema, so SCIP data can be ingested into Glean — [glean scip.angle on Hackage](https://hackage.haskell.org/package/glean-0.1.0.0/src/glean/schema/source/scip.angle)

#### LSIF, superseded

- Superseded by SCIP for new indexers since 2022. Sourcegraph still accepted LSIF uploads at the time — [Sourcegraph blog](https://sourcegraph.com/blog/announcing-scip)

#### LSP

- The current released spec is 3.18. 3.19 is "still under development" — [LSP site](https://microsoft.github.io/language-server-protocol/), [3.19 page](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.19/specification)
- LSP 3.18 notes that older versions used the incorrect constant "utf8" for position encoding, to be treated as "utf-8" — [LSP 3.18 spec](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification)

#### Kythe (Google)

- A VName has five fields: language, corpus, root, path, signature. The signature is "a unique string (per corpus, root, path, and language)", generated consistently for the same input, but it "does not necessarily need to be stable across different versions of the input". Indexers may hash long signatures. File nodes "may have greater stability properties" — [Kythe schema](https://kythe.io/docs/schema/)
- Anchors are first-class nodes that "connect concrete syntax to abstract syntax". Each has `loc/start` and `loc/end` byte offsets (end exclusive; 0/0 = whole file), an optional snippet, and edges `defines/binding`, `defines`, `ref` and `ref/call` to semantic nodes. For VCS nodes Kythe recommends the full commit hash over branch names, since "Git branches point to different commits over time" — [Kythe schema](https://kythe.io/docs/schema/)

#### Glean (Meta)

- Stores facts in predicates (tables of rows) with per-language schemas, plus language-neutral derived views. The schema and query language is Angle, a declarative logic language. Simple lookups take about 1 ms, and the first results of complex inheritance queries take a few ms. Storage is RocksDB — [Meta engineering, Dec 2024](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/)
- Incrementality comes from **stacked immutable databases**. Each layer adds or hides facts below it, so several revisions can be viewed without copies. The goal is cost "O(changes) rather than O(repository)"; in practice it is "O(fanout)" (for example, transitive `#include`-ers), and fanout is computed by a Glean query. Languages named include C++, Python, PHP, JavaScript, Rust, Erlang, Thrift and Haskell. RAG for AI coding assistants is listed as a use, without detail — [Meta engineering](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/)
- Incremental *derivation* is implemented only for some query kinds, and generalizing it is described as hard — [Glean blog via search snippet](https://nodejs2025.netlify.app/host-https-glean.software/blog/incremental) (mirror; verify on glean.software)

#### GitHub stack graphs, archived

- Name resolution from "a purely syntactic analysis", with no build step. Graph construction is "file-incremental": each file gets "an isolated subgraph without any knowledge of ... any other file", which removes the cost of reanalyzing file versions already seen. It powered GitHub Precise Code Navigation across "millions of repositories ... thousands of pushes per minute" — [Creager & van Antwerpen, arXiv 2211.01224](https://arxiv.org/abs/2211.01224)
- The github/stack-graphs repo was **archived Sep 9, 2025**, with maintainers stating it won't be maintained. github/semantic was archived Apr 1, 2025 — [GitHub repo](https://github.com/github/stack-graphs) (archive banner per search results)

#### Tree-sitter

- Incremental parsing: apply an `InputEdit` (byte and point positions) to the old tree with `ts_tree_edit`, then reparse with the old tree so unchanged subtrees are reused. Tree cursors are invalidated by edits — [tree-sitter docs, Using parsers / Editing](https://tree-sitter.github.io/tree-sitter/using-parsers) (official page referenced in [GNU bug #60656](https://debbugs.gnu.org/db/60/60656.html); detail from [gotreesitter docs](https://pkg.go.dev/github.com/odvcencio/gotreesitter@v0.13.3))
- Agent tooling precedent: Aider's repo map uses tree-sitter to extract defined and referenced symbols, builds a file dependency graph, ranks it (PageRank-style) and keeps the top symbols within a token budget (default about 1k, `--map-tokens`) — [Aider blog, Oct 2023](https://aider.chat/2023/10/22/repomap.html)

#### CodeQL databases

- **Overlay analysis**: a cached "overlay-base" database for the default branch, with PRs building a small overlay of only the changed files. It requires CLI 2.23.8+, supports only build-mode none (not traced builds), and falls back to full analysis without a cached base. Separately, diff-informed analysis (CLI 2.21.0+) limits results to changed code — [GitHub docs, incremental analysis](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis)
- CodeQL 2.23.5 added initial incremental Python database support via `--overlay-base/--overlay-changes` — [CodeQL 2.23.5 changelog](https://codeql.github.com/docs/codeql-overview/codeql-changelog/codeql-cli-2.23.5/)
- Overlay-informed dataflow was being added in Sep 2025 — [github/codeql PR #20386](https://github.com/github/codeql/pull/20386)

#### Joern Code Property Graph

- The CPG is a "directed, edge-labeled, attributed multigraph" defined in layers (MetaData, AST, CFG, PDG, CallGraph, Type, Method ...). The spec is generated from the implementation's code — [cpg.joern.io](https://cpg.joern.io/), [schema javadoc](https://javadoc.io/static/io.joern/codepropertygraph-schema_3/1.6.22/io/shiftleft/codepropertygraph/schema.html)
- It merges AST, CFG and PDG at statement and predicate nodes, following Yamaguchi et al. 2014 — [Wikipedia: Code property graph](https://en.wikipedia.org/wiki/Code_property_graph)

#### Code graphs built for LLM agents (2024–2026)

- CodexGraph (NAACL 2025): the repo is stored in Neo4j and the agent writes Cypher queries — [arXiv 2408.03910](https://arxiv.org/html/2408.03910v2)
- LocAgent: a directed heterogeneous graph of files, classes and functions with import, invoke and inherit edges. A fine-tuned Qwen-2.5-Coder-32B is comparable to proprietary models on localization, and Pass@10 issue resolution improves by 12% — [arXiv 2503.09089](https://arxiv.org/pdf/2503.09089)
- RepoGraph (ICLR 2025): a line-level repo graph with ego-network retrieval, reported 32.8% relative SWE-bench improvement; KGCompass reports 58.3% on SWE-bench Lite. Both figures are via the related-work summary in [arXiv 2603.27277](https://arxiv.org/pdf/2603.27277), not verified against the originals.

### Inferences

- No index above guarantees symbol identity across commits. SCIP symbols are stable only as long as the package, version and descriptor path don't change. Kythe explicitly disclaims cross-version stability. Identity across regeneration must therefore be our own layer: index symbols are evidence for identity, not identity itself.
- The SCIP symbol string is the right *anchor key* format to adopt or mirror, because it is human-readable and hence LLM-copyable, hierarchical (enclosing symbol is parseable) and cross-repo via package. Drop or normalize the `version` component when computing our stable IDs, so a lockfile bump does not orphan user overrides.
- Glean's stacked DBs and CodeQL overlays are the same pattern: an immutable base per commit plus a small per-change overlay. That pattern fits our "model at commit C + override layer + lazily generated detail layers".
- Stack graphs being archived means cross-file name resolution without a build must come from SCIP indexers (which need builds for some languages), tree-sitter heuristics, or the LLM. Record provenance accordingly: SCIP-resolved is `observed`-grade static fact, while a heuristic or LLM resolution is `inferred`.

### Gaps

- Current per-language SCIP indexer coverage (late 2026) was not fetched. Check github.com/sourcegraph/scip and the indexer list.
- No primary source found on whether SCIP has since gained an official incremental or partial-index protocol.
- No current Joern language-frontend list was fetched.
- Kythe's maintenance status in 2026 was not checked.

---

## 2. Stable identity and robust anchors (symbols, content addressing, re-anchoring, IDs for derived artifacts)

### Takeaway

Industry practice anchors to **(commit, path, range)** plus a **semantic key** (symbol) and re-resolves forward with diff- or blame-based mapping. Line-number-only anchors drift. Robust systems fingerprint by scope plus content (GitLab, 2026) or mark anchors "outdated" only when the exact anchored lines change (GitHub). There is little rigorous academic evaluation of re-anchoring specifically.

### Cited Findings

- GitHub (2018) changed review-comment outdating so a conversation is outdated only if "the exact line of code that anchors the comment" changes — [GitHub changelog](https://github.blog/changelog/2018-09-25-precision-review-comment-outdating/)
- Forgejo tracks review comments on changed lines with `git blame`, mapping forward through later diffs (`git blame --reverse`) — [Forgejo diff_test.go](https://remotebranch.eu/Stowage/forgejo/src/commit/fcce196fb8df8c7a051df4f375e80b38c33b77c0/modules/git/diff_test.go) (per search summary)
- Long-standing user demand to "anchor pull request comments to content ... not to a fixed line number" (Bitbucket Server) — [Atlassian BSERV-4479](https://jira.atlassian.com/browse/BSERV-4479)
- GitLab (Aug 12, 2026) uses "scope offset" fingerprinting for vulnerability tracking: location is identified by enclosing scope plus offset rather than line. It reports about 30% less futile re-auditing than line-based tracking — [GitLab blog](https://about.gitlab.com/blog/improved-scope-offset-fingerprinting)
- Research on re-attaching external annotations to evolving code cites prior "text anchoring" methods that relate metadata in a separate file to code points by text similarity, and proposes an agent that re-tags modified programs — [arXiv 2403.03481](https://arxiv.org/html/2403.03481v1)
- AI review tools drift on large diffs: line references gradually move from the intended location (anecdotal, not a controlled study) — [DEV Community](https://dev.to/shy_the_a91bfb236d4eeb5bb/why-ai-code-review-tools-keep-commenting-on-lines-that-dont-exist-ald)
- Kythe: an anchor's VName drops language and signature to yield its file's VName, so anchors are scoped to files. VCS revisions should use full commit hashes, not branches — [Kythe schema](https://kythe.io/docs/schema/)
- SCIP: the `enclosing_range` field gives the nearest non-trivial enclosing AST node, a natural basis for "scope" anchors — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- Bayesian-network elicitation with LLMs found models misspell node names in final JSON, fixed by nearest-name Levenshtein mapping. Name references produced by LLMs need fuzzy resolution — [arXiv 2407.09311](https://arxiv.org/pdf/2407.09311)

### Inferences

- Use a multi-key anchor, resolved in priority order:
  1. SCIP symbol (version-stripped).
  2. Scope fingerprint: enclosing symbol plus a normalized AST hash of the anchored node (tree-sitter, whitespace and comments stripped).
  3. `(commit, path, range)` mapped forward by diff or blame.
  4. Text-similarity fallback.

  Mark an anchor `drifted` when only keys 3 or 4 resolve, and `broken` when none do. That status is mechanically checkable and can surface as a validation warning.
- Stable IDs for derived artifacts (entities, steps) should be **semantic paths**, not positional counters like `s3.2`. Positional IDs renumber when the LLM inserts a step. A path such as `auth-service/issueToken#sign-jwt` derived from the parent ID, the anchor symbol and a short slug survives insertion. Content hashes should be stored *alongside* IDs, as cache keys and change detectors, never as the ID itself, because a content-hash ID changes on every edit and orphans overrides.
- On regeneration, match new nodes to old ones (anchor symbol equality first, then scope fingerprint, then title similarity) and carry IDs forward. This is entity resolution across versions (see section 4).

### Gaps

- I found no peer-reviewed benchmark of re-anchoring accuracy (comments or annotations) across code evolution. The MSR literature on line-tracking (e.g., origin analysis, LHDiff, "git blame accuracy") was not searched in this pass.
- No source found on stable IDs for *LLM-derived* artifacts across regeneration specifically. The recommendations above are inferred.

---

## 3. LLM structured generation: formats, constrained decoding, schema design, graph generation, reference verification

### Takeaway

Native schema-enforced output is now standard: OpenAI since Aug 2024, Anthropic beta Nov 2025, grammar engines like XGrammar, Guidance and Outlines. It guarantees *syntax*, not *content*. The 2024 claim that format constraints hurt reasoning ("Let Me Speak Freely?") was contested. The 2025–2026 evidence says the cost comes mostly from **schema design**, especially answer-before-reasoning field order, not from structure itself. Format choice (JSON, YAML, XML) matters but is model-dependent. Validators with targeted retry are the consensus answer to dangling references in generated graphs.

### Cited Findings

#### Format restrictions and reasoning

- "Let Me Speak Freely?" (Tam et al., EMNLP 2024 Industry): format restrictions caused a significant drop in reasoning, larger under stricter constraints such as JSON mode, while classification sometimes improved. NL-to-Format (answer freely, then convert) stayed close to free-form — [arXiv 2408.02442](https://arxiv.org/abs/2408.02442), [ACL Anthology](https://aclanthology.org/2024.emnlp-industry.91)
- dottxt's rebuttal "Say What You Mean" (Nov 20, 2024) argues the paper conflated structured generation with JSON mode and used poor prompts. With proper structured generation, results match or beat unstructured output on the tasks the paper flagged worst — [dottxt blog](https://blog.dottxt.ai/say-what-you-mean.html)
- An independent replication found structured output as good or better for LLaMA3-8B-Instruct, but the rebuttal's approach "did not hold for GPT-4o-mini" — [Dylan Castillo](https://dylancastillo.co/posts/say-what-you-mean-sometimes.html)
- "Structure Tax" (Kumar, Kanishka, Mandora, AACL 2026): reasoning-first field order matches or exceeds free-form accuracy, while answer-first causes steep drops, especially in smaller models. Format sensitivity is greater for tasks with little inherent structure. Reasoning-order-preserving schemas also improve calibration. The authors conclude that "schema design, not structure itself" drives the loss — [ACL Anthology 2026.aacl-long.209](https://aclanthology.org/2026.aacl-long.209/) (numbers not extracted; see the PDF)
- Contrary data point: a small informal GPT test found minimal difference between answer-first and reasoning-first — [CO/AI](https://getcoai.com/news/why-field-order-may-not-improve-model-reasoning) (blog, weak)

#### Constrained decoding engines and benchmarks

- JSONSchemaBench (Geng et al., 2025): 10K real-world JSON schemas in 10 datasets. It evaluates Guidance, Outlines, Llamacpp, XGrammar, OpenAI and Gemini on efficiency, constraint coverage and output quality. Constrained decoding improved downstream tasks by up to 4% (even GSM8k) and could speed generation by about 50%. Coverage of schema features varies widely across frameworks — [arXiv 2501.10868](https://arxiv.org/abs/2501.10868v2)
- OpenAI Structured Outputs (Aug 6, 2024): strict mode with gpt-4o-2024-08-06 scored 100% on OpenAI's complex-schema eval, versus under 40% for gpt-4-0613 with prompting. It supports a JSON Schema subset (all fields required, `additionalProperties: false`) — [OpenAI](https://openai.com/index/introducing-structured-outputs-in-the-api/). **Caveat:** the page returned 403 during this research. These figures are from the widely reported announcement and should be re-verified.
- Anthropic Structured Outputs: public beta Nov 2025 (header `structured-outputs-2025-11-13`) for Sonnet 4.5 and Opus 4.1, via `output_config.format` json_schema or strict tool definitions. Format is guaranteed, content is not — [Claude blog](https://claude.com/blog/structured-outputs-on-the-claude-developer-platform), [Tessl summary](https://tessl.io/blog/anthropic-brings-structured-outputs-to-claude-developer-platform-making-api-responses-more-reliable/). Later GA status for 4.5-family models comes from secondary sources only.
- Amazon Bedrock structured outputs use constrained sampling with compiled grammar artifacts — [AWS ML blog](https://aws.amazon.com/blogs/machine-learning/structured-outputs-on-amazon-bedrock-schema-compliant-ai-responses)
- BoundaryML argues schema-enforced outputs "create false confidence": valid shape with wrong content — [BoundaryML](https://boundaryml.com/blog/structured-outputs-create-false-confidence)

#### Format choice (JSON, YAML, XML, other)

- StructEval (TMLR, Jan 2026) benchmarks 18–21 structured formats, both non-renderable (JSON, YAML, CSV ...) and renderable (HTML, React, SVG) — [arXiv 2505.20139](https://arxiv.org/pdf/2505.20139v2)
- Across 13 open-weight models and 7 SLU/NER benchmarks, the output format requested (JSON, XML or inline key-value) changed performance and reliability — [LREC 2026](https://preview.aclanthology.org/ingest-lrec/2026.lrec-main.593/)
- Informal nested-data test (GPT-5 Nano, Llama 3.2 3B, Gemini 2.5 Flash Lite): YAML best for 2 of 3 models, XML consistently worst — [ImprovingAgents blog](https://www.improvingagents.com/blog/best-nested-data-format) (blog, small)
- TOON (a compact format) is compared with JSON, XML and YAML for correctness and energy — [arXiv 2601.12014](https://arxiv.org/html/2601.12014v1)
- A causal-inference study of output format effects on reasoning across seven datasets — [arXiv 2509.21791](https://arxiv.org/pdf/2509.21791v3) (findings not extracted)

#### Schema design choices

- PARSE (Amazon, EMNLP 2025 Industry): an LLM rewrites JSON schemas to be more LLM-usable (ARCHITECT) and adds reflection with static and LLM guardrails (SCOPE). It reports up to 64.7% extraction accuracy gain. On SWDE, about 10% accuracy gain across models and 92% fewer errors within the first retry — [arXiv 2510.08623](https://arxiv.org/abs/2510.08623v1), [ACL Anthology](https://aclanthology.org/2025.emnlp-industry.184)

#### Graphs as text and graph generation

- "Talk like a Graph" (Fatemi et al., ICLR 2024): LLM graph-reasoning performance depends on the encoding method, the task and graph structure. Encoder choice changed accuracy by 4.8%–61.8%, and the "incident" encoding (listing each node's neighbors) was best on most tasks — [arXiv 2310.04560](https://arxiv.org/pdf/2310.04560), [Google Research blog](https://research.google/blog/talk-like-a-graph-encoding-graphs-for-large-language-models/)
- EviGraph (2026): its graph validator rejects dangling edges, duplicate node IDs and unresolved provenance references. A malformed or ungrounded anchor causes rejection and regeneration rather than downstream patching — [arXiv 2608.04738](https://arxiv.org/pdf/2608.04738)
- PEARL (2026) separates structural admissibility (known endpoints, closed edge labels, no self-loops) from semantic validity, with structural repair before global checks — [arXiv 2607.17917](https://arxiv.org/pdf/2607.17917)
- Graph repair with LLMs: template-based encoding generally gives better repair validity — [arXiv 2507.03410](https://arxiv.org/pdf/2507.03410)
- LLMs (GPT-3.5) tend to produce cycles in DAG elicitation and misspell node names — [arXiv 2407.09311](https://arxiv.org/pdf/2407.09311)

### Inferences

- Put free-text fields (`rationale`, `summary`) **before** decision fields (`branch_taken`, `provenance`, `event_kind`) in every schema. That is the one design choice with peer-reviewed support (Structure Tax).
- Use API-enforced JSON Schema for the *wire format* the model emits, with closed enums for `provenance`, `event_kind`, `port.direction` and `set_by`. Allow free reasoning in a preceding field or a prior NL step (the NL-to-Format two-step) for hard branch-resolution decisions. Store and diff as YAML or JSON on disk. The model never has to emit the whole document.
- Emit graphs as **flat node and edge lists with string IDs, one level per call** (node-by-node, lazily). Avoid deep nesting. It fights constrained-decoding schema limits (recursion support varies across frameworks per JSONSchemaBench) and makes partial regeneration and merging hard. Nesting is reconstructed from `parent` references.
- References should be human-readable IDs the model has *seen in context* (SCIP-like symbols, semantic paths), not UUIDs. Validate every reference after generation, fuzzy-resolve near misses (Levenshtein, as in 2407.09311), and retry with a targeted error message (PARSE: most errors fixed on the first retry).

### Gaps

- No controlled study found comparing ID-reference vs nested representations, or short-slug vs UUID IDs, for LLM *generation* error rates. The recommendation is inferred from practitioner reports and the graph-encoding literature.
- Exact numbers for Structure Tax, StructEval and the LREC format paper were not extracted.
- Anthropic's own documentation of its decoding mechanism (constrained vs. other) was not confirmed.

---

## 4. LLM-based knowledge-graph construction from text and code

### Takeaway

Schema-guided extraction plus an explicit **entity-resolution and deduplication pass** is the dominant 2025–2026 recipe. Without resolution, duplication is large: about 30% duplicate nodes in a GraphRAG baseline on legal graphs. Entity resolution matters more than other denoising steps. Incremental KG construction (iText2KG, DIAL-KG) and ontology-first workflows that verify every identifier against the graph (OntoKG) are the closest analogues to our lazily-expanded model. Code-specific KGs for agents (LocAgent, CodexGraph, RepoGraph) build their graphs deterministically from parsers, not LLMs.

### Cited Findings

- KGGen (NeurIPS 2025): two-stage pipeline, LLM triple extraction followed by entity and relation resolution that clusters mentions of the same entity and consolidates equivalent edges. It introduces the MINE benchmark and performs comparably to GraphRAG — [arXiv 2502.09956](https://arxiv.org/pdf/2502.09956), [NeurIPS](https://neurips.cc/virtual/2025/poster/117386)
- A 2026 unified KG-construction benchmark notes that LLM-built graphs remain noisy even after LM-based deduplication — [arXiv 2605.05476](https://arxiv.org/pdf/2605.05476)
- CORE-KG: on 20 legal case graphs, the standard GraphRAG baseline had the highest node duplication at 30.53%. Removing coreference resolution increased duplication — [arXiv 2510.26512](https://arxiv.org/pdf/2510.26512)
- DEG-RAG: ablations show entity resolution and triple reflection are both indispensable, with entity resolution having the larger impact. Entity-type blocking worked best, with caveats about small KGs and LLM-judge bias — [arXiv 2510.14271](https://arxiv.org/html/2510.14271v1)
- iText2KG (2024): incremental construction. The schema is "a blueprint that biases the LLM towards specific classes while maintaining flexibility" — [arXiv 2409.03284](https://arxiv.org/pdf/2409.03284)
- DIAL-KG (Mar 2026): schema-free incremental KG with dynamic schema induction. It supports adding, modifying and retiring knowledge without full reconstruction — [arXiv 2603.20059](https://arxiv.org/html/2603.20059v1)
- OntoKG (Apr 2026): the LLM acts as schema designer with grounding tools "that verify each identifier used in schema decisions against the knowledge graph" — [arXiv 2604.02618](https://arxiv.org/html/2604.02618v1)
- Ontology-driven iterative pipeline (SEMANTiCS 2026): competency questions, **SHACL validation** and convergence checks across iterations — [CEUR Vol-4293](https://ceur-ws.org/Vol-4293/paper59.pdf)
- OntoEKG (Feb 2026): Pydantic-schema extraction followed by hierarchical structuring, serialized to RDF/OWL — [arXiv 2602.01276](https://www.alphaxiv.org/abs/2602.01276)
- Code KGs for agents: LocAgent (files, classes, functions; import, invoke, inherit edges) — [arXiv 2503.09089](https://arxiv.org/pdf/2503.09089); CodexGraph (Neo4j and Cypher) — [arXiv 2408.03910](https://arxiv.org/html/2408.03910v2); a tree-sitter KG exposed over MCP — [arXiv 2603.27277](https://arxiv.org/pdf/2603.27277)

### Inferences

- Split the pipeline:
  1. A **deterministic skeleton** from SCIP and tree-sitter (files, symbols, call and import edges), provenance `observed`-static.
  2. LLM-proposed **abstractions** (entities, ports, delegations, scenario-graph branches), each required to cite skeleton anchors.
  3. A resolution pass that merges LLM-proposed entities with each other and with prior-version entities. Blocking by entity type (DEG-RAG) and by anchor overlap is a natural fit: two proposed entities whose anchor sets overlap heavily are merge candidates.
- Our "open entity must account for every port" rule is the same kind of constraint as SHACL shapes. Express checks declaratively (a SHACL-like or Datalog/Angle-like rule set over the flat graph) so packs and user code share one validator.

### Gaps

- No evaluation found of LLM extraction of *architectural* abstractions (services, ports, delegation) from code, as opposed to text KGs or code-localization graphs.
- No LazyGraphRAG-specific evaluation found.

---

## 5. Grounding and verification loops (citations, mechanical checks, critics)

### Takeaway

Requiring fine-grained citations and **checking them mechanically** (file exists, line range valid, quoted text matches) is the most effective and cheapest verification layer. Retrieval-backed verifiers beat asking an LLM judge whether a citation is valid. Code-specific evidence exists but is preliminary.

### Cited Findings

- Citation-grounded code comprehension (Dec 2025 preprint): 30 Python repos and 180 developer queries. A mechanical checker flags fabricated files, invalid line ranges and incomplete evidence. It reports 92% citation accuracy with zero hallucinations for the best configuration. Across models, citation-format compliance ranged from 88% (2% hallucination) down to 62% (19% hallucination) — [arXiv 2512.12117](https://arxiv.org/pdf/2512.12117) (preprint, not replicated)
- FRONT: select supporting quotes first, then generate conditioned on them. Human evaluation shows quotes significantly aid user verification — [arXiv 2408.04568](https://arxiv.org/pdf/2408.04568)
- CiteGuard: LLM-as-judge assumes the retrieved knowledge is available. Retrieval-augmented validation is more faithful — [arXiv 2510.17853](https://arxiv.org/pdf/2510.17853)
- CITECHECK (2026): grounded retrieval plus structured comparison is more reliable than prompting an LLM to judge citation validity — [arXiv 2605.27700](https://www.alphaxiv.org/abs/2605.27700.md)
- EviGraph: ungrounded or malformed anchors cause rejection and regeneration of the candidate — [arXiv 2608.04738](https://arxiv.org/pdf/2608.04738)
- PARSE: static guardrails plus reflection cut extraction errors by 92% within the first retry on SWDE — [arXiv 2510.08623](https://arxiv.org/abs/2510.08623v1)
- Vendor claim: RAG grounding reduces hallucination by about 68% but does not eliminate it — [GMI Cloud blog](https://www.gmicloud.ai/en/blog/reducing-hallucination-in-production-llm-apps-grounding-citations-and-faithfulness-scoring) (vendor, weak)

### Inferences

- Three-tier verification for every generated element:
  1. **Structural**: schema validity, closed enums, referential integrity (no dangling IDs, unique IDs, `after` relation acyclic within a lane).
  2. **Anchor**: the repo, commit and path exist, the range is within the file, the symbol resolves in SCIP at that range, and an optional `quote` or snippet hash matches.
  3. **Semantic invariants**: every port of an open entity is delegated or marked `assumed`, every unbound branch is an assumption, and every collapsed segment's `contains` counts are recomputed, never trusted from the LLM.

  Tiers 1 and 2 are cheap and deterministic. Run them on every generation, with targeted retry. Use an LLM critic only for tier-3 judgments that cannot be checked mechanically (e.g., "is this the right branch for input class X").
- Make the model emit a short `quote` (the line it relies on) with each anchor. It is cheap to verify by substring match and catches off-by-N line ranges, the dominant drift failure.

### Gaps

- No controlled study found that measures hallucination reduction specifically from *anchor-required* generation over code against unanchored generation, beyond the single preprint.
- Self-consistency for structural extraction (sample N, keep the majority) was not specifically researched.

---

## 6. Incremental and lazy generation (content-hash caching, invalidation, Salsa/Bazel-style computation)

### Takeaway

Incremental computation frameworks give the exact semantics needed: **memoized derived queries with recorded dependencies, revisions, early cutoff (backdating) and durability tiers** (Salsa / rust-analyzer). Code indexes add **immutable base plus overlay** (Glean, CodeQL) and **file-isolated units** (stack graphs). Treating each LLM generation as a Salsa-style derived query, keyed by hashes of the code slices it read, carries these properties over directly.

### Cited Findings

- Salsa: each input set increments the revision. A tracked function's result is cached with the dependencies it read and the revisions in which they last changed. On a new revision, if no dependency changed the cached value is returned, otherwise it re-executes — [Salsa algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- **Backdating (early cutoff)**: if a re-executed dependency produces the same result, the caller is not marked changed ("even though the inputs changed, the output didn't"). For example, adding a comment changes the text but not the AST, so `type_check` is skipped — [Salsa algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- **Durability**: inputs carry durability levels, high for third-party crates and low for the workspace. Salsa tracks the last change per level, so results depending only on high-durability inputs are validated without walking edges — [Salsa algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- Glean: cost "O(changes)", in practice "O(fanout)", with fanout computed by a query over the index. Stacked immutable DBs hide and replace changed units — [Meta engineering](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/)
- CodeQL overlay: base DB per default-branch commit, overlay of changed files per PR — [GitHub docs](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis)
- Stack graphs: per-file isolated subgraphs, so previously seen file versions (by content) never need reanalysis — [arXiv 2211.01224](https://arxiv.org/abs/2211.01224)
- Incremental KG with retire and modify operations — [DIAL-KG, arXiv 2603.20059](https://arxiv.org/html/2603.20059v1)

### Inferences

- Model each lazy level as a derived query, for example `expand(entity_id, depth, scenario_binding)`. Its cache key combines:
  - the hash of the **normalized** code slices it read (tree-sitter AST hash, so a comment or formatting change backdates);
  - the hashes of the child summaries/packs it consumed;
  - the hash of the assumption bindings;
  - the prompt/schema/model version.

  Record the dependency set explicitly as the list of anchors (symbol plus AST hash) the generation cited and read.
- **Early cutoff for LLM output**: when an input hash changes, regenerate. If the regenerated node set is equivalent after ID-matching and normalization, backdate so parents are not regenerated. This bounds cascades up the zoom hierarchy.
- **Durability tiers**: behavior packs and lockfile-pinned library code are high durability, and user repos at HEAD are low. Pack-derived levels are validated in O(1) per revision.
- **Invalidation by fanout**: a code change invalidates the generations whose dependency set includes an anchor in the changed range (an inverted index anchor → generation), plus the ancestors not backdated. This is Glean's fanout query applied to LLM artifacts.
- LLM non-determinism breaks naive early cutoff (the same input yields a different output). Re-running unchanged inputs would cause churn, so cache by input hash and never re-run an unchanged key. Compare *normalized* outputs (matched IDs, sorted lists, ignored prose fields) for backdating.

### Gaps

- No published system found that applies Salsa-style incremental computation to LLM-generated artifacts specifically. This is an inference and a likely novel contribution.
- Bazel's action-cache semantics (content-addressed action keys) were not fetched in this pass. The analogy holds but is uncited.

---

## 7. Recommendations: schema principles and a serialization and identity sketch

### Takeaway

Use a **flat, typed, ID-referenced graph** (nodes plus edges, one record per element) with:

- semantic-path IDs that are independent of content;
- multi-key code anchors modeled on SCIP symbols and Kythe anchors;
- explicit provenance and dependency hashes per element;
- generation one level at a time under API-enforced JSON Schema, reasoning fields first;
- deterministic validators (structure, anchors, invariants) with targeted retry;
- Salsa-style caching with early cutoff;
- user overrides as a separate layer keyed by stable IDs.

### Cited Findings (basis for each principle)

- Human-readable string IDs over opaque numeric IDs. LSIF's numeric IDs blocked debugging and partial updates — [Sourcegraph](https://sourcegraph.com/blog/announcing-scip)
- Anchors as first-class records (file, byte range, snippet) linked to semantic nodes by typed edges (`defines`, `ref`) — [Kythe schema](https://kythe.io/docs/schema/)
- Symbol keys with package and descriptor path, local symbols scoped to a document — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- Use the full commit hash, never branch names — [Kythe schema](https://kythe.io/docs/schema/)
- Reasoning-first field order — [Structure Tax, AACL 2026](https://aclanthology.org/2026.aacl-long.209/)
- Schema rewriting for LLM usability, static guardrails with retry — [PARSE](https://arxiv.org/abs/2510.08623v1)
- Validator rejects dangling edges, duplicate IDs and unresolved provenance — [EviGraph](https://arxiv.org/pdf/2608.04738)
- Entity resolution pass as a required stage — [KGGen](https://arxiv.org/pdf/2502.09956), [DEG-RAG](https://arxiv.org/html/2510.14271v1)
- Neighbor-list ("incident") graph encodings read best for LLMs — [Talk like a Graph](https://arxiv.org/pdf/2310.04560)
- Early cutoff and durability — [Salsa](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- Base plus overlay layering — [Glean](https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/), [CodeQL overlay](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis)
- Mechanical citation checks (file exists, range valid) — [arXiv 2512.12117](https://arxiv.org/pdf/2512.12117)

### Inferences (concrete design sketch)

#### Principles

1. **Flat records, references by ID.** Every entity, port, delegation, step, branch, input class, state and assumption is its own record with `id`, `kind` and `parent`. Containment and nesting are derived. The LLM emits only the records for one expansion: one entity's interior, or one step's children.
2. **IDs are semantic paths, minted deterministically and carried forward.** `id = parent_id + "/" + slug`. The slug comes from the anchor symbol's last descriptor when one exists, else a short LLM-chosen kebab slug. The model *proposes* slugs, but the system mints the final ID and resolves collisions with `~2`. On regeneration, match old to new by anchor symbol, then scope fingerprint, then slug and title similarity, and **reuse old IDs**. Never use positional IDs (`s3.2`); keep order in an explicit `order` or `after` field.
3. **Content hashes are attributes, not identity.** Each record stores `deps: [{anchor_key, ast_hash}]` and `gen_key` (a hash of deps, child summaries, bindings and prompt version). These drive the cache and invalidation, but never the ID.
4. **Multi-key anchors.** `{repo, commit (full SHA), path, range: [start_line, start_col, end_line, end_col], encoding: utf-8, symbol: "<SCIP symbol, version-stripped>", scope_fp: "<enclosing symbol + normalized AST hash>", quote: "<≤1 line>"}`. Anchor status is `exact | drifted | broken`, recomputed on every commit.
5. **Provenance on every record**, from a closed enum: `observed` (index or trace fact), `inferred` (LLM, with anchors), `assumed` (no code anchor; must reference a spec or pack or be an assumption record), plus `source` (`scip | tree-sitter | llm:<model> | pack:<id@ver> | user`). Validator rule: `inferred` requires ≥1 anchor that resolves, and `assumed` requires an `assumption_id` or a spec citation.
6. **Assumptions are first-class records** (`kind: assumption`, `branch_point_id`, `bound_to`, `default_reason`, `alternatives: [branch_ids]`), referenced by ID from steps. The UI's "flip assumption" is then a single-field edit to a binding.
7. **Generation contract per call.** Input: the parent record, its ports, relevant skeleton slices (SCIP symbols with code), the IDs already in scope, and the allowed enums. Output schema, in order: `analysis` (free text) → `records[]` → `edges[]` → `open_questions[]`. Enforce it with the provider's strict JSON Schema. For hard branch resolution, use a two-step NL-then-format. The model may reference only IDs given in context or new IDs it declares in the same response.
8. **Validators, all deterministic and run on every write:**
   - schema;
   - unique IDs;
   - no dangling refs (fuzzy-suggest a fix on near misses);
   - `after` acyclic per lane;
   - anchors resolve (file exists at commit, range in bounds, quote matches, symbol occurrence present in the SCIP index);
   - open entity ⇒ every port has a delegation or an `assumed` marker;
   - every branch point has either a bound input-class constraint or an assumption;
   - segment `contains` counts are recomputed.

   Failures go back to the model as a targeted error list (one retry, then surface to the user).
9. **Override layer** is a separate file of patches keyed by stable ID: `{target_id, field, value, set_by: user|agent, base_gen_key}`. When the target's `gen_key` changes, the override is re-applied if the ID still exists and the field is still valid, else it is flagged `orphaned`. It is never silently dropped.
10. **Storage and merge:** one record per line (JSONL), or per-entity YAML files sorted by ID, for clean git diffs and 3-way merges. Base model at commit, plus generated-level overlays, plus overrides. This is the same stack shape as Glean and CodeQL.
11. **Caching:** a Salsa-style memo table `gen_key → records`. Early cutoff compares normalized outputs. Behavior packs are high durability. Prefetch is just eager evaluation of the neighbors' queries.

#### Record sketch (YAML for reading; the wire format is JSON under strict schema)

```yaml
- id: sys/auth-service/issue-token
  kind: entity              # entity | port | delegation | step | branch | input_class | state | assumption
  type: Function
  parent: sys/auth-service
  provenance: observed
  source: scip
  anchor: {repo: auth, commit: a1b2c3d4e5f6..., path: src/token.ts, range: [41,0,57,1], encoding: utf-8,
           symbol: "scip-typescript npm auth . src/`token.ts`/issueToken().",
           scope_fp: "src/`token.ts`/ :: ast:9f2c...", quote: "export function issueToken(claims: Claims) {"}
  anchor_status: exact
  deps: [{key: "src/`token.ts`/issueToken().", ast_hash: "9f2c..."}]
  gen_key: "h:71ab..."
- id: sys/auth-service/issue-token/port:claims
  kind: port
  parent: sys/auth-service/issue-token
  direction: provides       # provides | requires
  protocol: call-arg
  shape: "{sub, exp, roles}"
  provenance: observed
- id: login/s/sign-jwt
  kind: step
  parent: login/s/issue-token
  actor: sys/auth-service/issue-token
  analysis: "jwt.sign called with alg from config; default RS256 at config/default.ts:12"
  event_kind: transform
  after: [login/s/load-key]
  provenance: inferred
  source: llm:claude-opus
  anchor: {...}
  assumptions: [login/a/jwt-alg]
  children: lazy
  gen_key: "h:..."
- id: login/a/jwt-alg
  kind: assumption
  branch_point: sys/auth-service/issue-token/branch:alg
  bound_to: RS256
  default_reason: "config default (config/default.ts:12); env JWT_ALG unset in scenario"
  alternatives: [HS256, ES256]
  provenance: assumed
```

### Gaps

- No empirical evidence found for the specific choices above (semantic-path IDs, multi-key anchors, Salsa-for-LLM). They are design inferences from the cited systems and should be validated with the planned hand-written gold login script. Measure ID survival rate across regeneration, anchor drift rate across commits, validator-caught error rate per level and retry success rate.
