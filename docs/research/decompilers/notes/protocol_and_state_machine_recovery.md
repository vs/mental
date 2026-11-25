# Protocol reverse engineering and state-machine recovery

Evidence tags on every claim: **[read]** read in the primary source this session (full text or abstract page, as stated); **[search]** seen only in a search-result summary; **[training]** from training knowledge, not checked this session (no URL is given for these, so none is invented). Dates are given so older work is visible; most of this field is 2002 to 2016, with a 2023 to 2026 tail.

| System / work | One line |
|---|---|
| Polyglot (2007) | Dynamic taint tracking in a server binary recovers one message's fields (keywords, delimiters, length fields) from how the code reads them. [training] |
| Dispatcher (2009) | Polyglot extended to messages a program *sends*, used on botnet command-and-control. [training] |
| AutoFormat (2008) | Uses the call-stack context of each byte's processing to build a hierarchical field tree. [training] |
| Tupni (2008) | Recovers record sequences, loops and constraints of an input format from binary execution. [training] |
| Discoverer (2007) | Network traces only: tokenizes, clusters and aligns messages into formats. [training] |
| Prospex (2009) | Clusters messages into types by format and *server reaction*, builds a prefix tree of sessions, labels states, merges with Exbar into a minimal state machine. [read] |
| ReverX (2011) | Traces to a message automaton plus format, by prefix tree and state merging. [training] |
| Netzob (2012-2014) | Open-source tool: alignment-based vocabulary inference plus active learning of the protocol automaton. [training] |
| Shi, Xu, Zhang (2023) | *Static* analysis of a parser binary: each loop iteration is a state, inter-iteration dependences are transitions; over 90% precision and recall. [read, abstract] |
| L* / LearnLib / AALpy | Active automata learning: query a black box, learn a Mealy machine, approximate equivalence by testing. [read via de Ruiter; AALpy search] |
| de Ruiter and Poll (2015) | Learned state machines of nine TLS stacks; extra states and transitions exposed new flaws in GnuTLS, JSSE, OpenSSL. [read] |
| Fiterau-Brostean et al. (2016-2023) | Learned models of TCP, SSH, DTLS; checked by model checking, later by a catalogue of bug-pattern automata. [read NDSS 2023; rest search] |
| Ammons, Bodik, Larus (2002) | "Mining specifications": API-call traces, scenario extraction by dependence, probabilistic automaton learning. [training] |
| k-tails (1972) | Merge states whose next-k futures agree; the base of most passive FSM inference. [training] |
| Perracotta (2006) | Mines alternating temporal rules (a then b) from imperfect traces with a satisfaction threshold. [search mention; detail training] |
| Synoptic / CSight / InvariMint / Texada (2011-2015) | Log-based inference: invariants, refinement, k-tails coarsening; CSight for distributed logs as communicating FSMs; InvariMint as declarative composition; Texada for arbitrary LTL templates. [search] |
| Static typestate inference (Alur et al. 2005 and others) | Infer a class's legal call order from code, by predicate abstraction and learning. [training] |
| Engler et al. "Bugs as deviant behavior" (2001) | Infer beliefs from what code usually does; the rare deviation is the bug. [training] |
| Godel Checker / "Fencing off Go" (2017-2018) | Infer behavioural types from Go SSA, model-check them for safety and liveness. [search] |
| P at AWS, PObserve | State-machine modelling language; PObserve checks production logs against the P spec. [search] |
| Specula (2025-2026) | LLM agents write TLA+ models from system code, validated by traces; 48 projects, 249 bugs. [read, abstract] |
| Process mining event abstraction (van Zelst et al. 2021 survey) | Taxonomy of lifting low-level events to activities. [search] |
| Guided Process Discovery (Mannhardt et al. 2018) | Activity patterns written by experts over low-level events, matched into high-level activities. [search] |
| Conformance checking / alignments | Align a log trace to a model with sync, log-only and model-only moves; the non-sync moves are the deviations. [training] |

## Network protocol reverse engineering (dynamic and static)

### Takeaway

The field lifts in two stages, message *format/type* then *session state machine*, and its central lesson is that a message's type is best defined by how the program reacts to it, not by its bytes; the state machine is then a generalization from positive examples, which needs domain knowledge to avoid collapsing to one state. Static variants exist but recover parsers (formats), rarely session logic.

### Cited Findings

- Prospex (Milani Comparetti, Wondracek, Kruegel, Kirda, IEEE S&P 2009) takes observed application sessions, infers message formats, then clusters messages into types; its similarity metric assumes "messages of the same type share similar message formats and that the server 'reacts' in a similar fashion upon receiving them", so features come from the execution trace, not only the bytes. Clustering is partitioning around medoids. [read] — [Prospex PDF](https://sites.cs.ucsb.edu/~chris/research/doc/oakland09_prospex.pdf)
- Prospex represents sessions as an augmented prefix tree acceptor (APTA), "an incompletely-specified deterministic finite state automaton ... each branch represents an application session", then finds "the minimal DFA that is consistent with T" with Exbar, which merges pairs of states. [read] — [Prospex PDF](https://sites.cs.ucsb.edu/~chris/research/doc/oakland09_prospex.pdf)
- Prospex cites Gold: "a regular language cannot be learned from positive examples only". Since all observed prefixes are accept states, a plain minimal DFA would be "an over-general DFA with only a single state"; Prospex adds a heuristic state-labeling step so that "only states with the same label may be merged". [read] — [Prospex PDF](https://sites.cs.ucsb.edu/~chris/research/doc/oakland09_prospex.pdf)
- Prospex was evaluated on SMTP, Samba (SMB), SIP and the Agobot C&C protocol, and its output was turned into Peach fuzzer specifications that found vulnerabilities; the authors extended Peach's statefulness for this (Peach 2.2). Ground truth for evaluation was manual message labeling ("we manually label every message in the training set with its actual message type"). [read] — [Prospex PDF](https://sites.cs.ucsb.edu/~chris/research/doc/oakland09_prospex.pdf)
- Prospex states that it, like prior traffic-based work, cannot handle encrypted traffic, and that observing server behaviour could overcome this. [read] — [Prospex PDF](https://sites.cs.ucsb.edu/~chris/research/doc/oakland09_prospex.pdf)
- A later paper (MACE, Cho et al.) characterizes Prospex's machines as incomplete, "possibly missing transitions", because they only cover observed traffic. [search] — [MACE PDF](https://ptolemy.berkeley.edu/projects/truststc/pubs/854/Cho.pdf)
- PISE (NDSS BAR workshop 2023) applies an extended L* directly to binaries (symbolic execution as the query engine). [search] — [PISE](https://www.ndss-symposium.org/wp-content/uploads/2023/03/bar2023-23002-paper.pdf)
- Shi, Xu and Zhang (arXiv 2305.13483, 2023, v4 2024; no peer-reviewed venue listed on the page) argue dynamic format inference has low coverage, and instead analyze parser code statically: each loop iteration is a state, dependences between iterations are transitions, and designed path-merging rules keep path sensitivity without path explosion. They report inference in about five minutes with over 90% precision and recall, 20% to 230% more fuzzer coverage and ten more zero-days than baselines. [read, abstract] — [arXiv 2305.13483](https://arxiv.org/abs/2305.13483)
- Polyglot (Caballero, Yin, Liang, Song, CCS 2007): dynamic binary taint analysis, field boundaries and keywords from how the server's code processes input bytes. Dispatcher (Caballero et al., CCS 2009): the same for messages the program builds and sends ("buffer deconstruction"), used on the MegaD botnet. AutoFormat (Lin, Jiang, Xu, Zhang, NDSS 2008): call-stack context gives a field hierarchy. Tupni (Cui, Peinado, Chen, Wang, Irun-Briz, CCS 2008): record sequences and constraints. Discoverer (Cui, Kannan, Wang, USENIX Security 2007): traffic only, recursive clustering and type-based alignment. ReverX (Antunes, Neves, Verissimo, 2011): prefix-tree acceptors over traces, merged into a state machine. Netzob (Bossert and others, from about 2012): sequence alignment for vocabulary plus active learning of the automaton. [training]
- Surveys: Narayan, Shukla, Clancy, "A survey of automatic protocol reverse engineering tools", ACM Computing Surveys 2015; Kleber, Maile, Kargl, survey of network protocol reverse engineering tools, IEEE Communications Surveys and Tutorials 2018; Duchene et al., state of the art of network protocol reverse engineering tools, Journal of Computer Virology and Hacking Techniques 2018. [training; not opened this session]

### Inferences

- Prospex's "type = similar server reaction" is the same idea as mentalc's input class and case ("all inputs that behave alike", "all paths that end alike"). mentalc gets the reaction directly from code, so it does not need clustering: the type of a message is the case its handler takes.
- The Gold result explains why this whole field leans on heuristics: from traces you only see positive examples and must guess the generalization. mentalc has the code, which is the whole language, so for mentalc lifting is *matching and abstraction*, not *learning*. That removes the main source of guesswork in these tools.
- The static loop-as-state technique is the closest static analogue to what mentalc does for lifecycle machines (write sites as transitions); it shows path-merging rules are the practical crux of keeping a static state-machine derivation precise.

### Gaps

- Did not open the three surveys or the original Polyglot/Dispatcher/AutoFormat/Tupni/Discoverer/ReverX/Netzob papers this session; their descriptions are training knowledge.
- Prospex's numerical results (states recovered vs reference, false merges) were in the PDF but not extracted.
- No static tool found that recovers *session* state machines (not formats) from server source in a high-level language.

## Model learning and active automata learning (TLS, TCP, SSH, DTLS)

### Takeaway

Here "lifting" is a hand-written mapper from concrete packets to an abstract alphabet; L* then learns a Mealy machine over that alphabet, completeness is only as good as a bounded equivalence test, and bugs show up as states or transitions that a correct implementation would not have. The 2023 step replaces eyeballing with small bug-pattern automata checked against the learned model, each hit confirmed by a test on the real system.

### Cited Findings

- de Ruiter and Poll (USENIX Security 2015) "use state machine learning to infer state machines from protocol implementations, using only black-box testing, and then inspect the inferred state machines". They used LearnLib, "a modified version of Angluin's L* algorithm"; the learner needs an input alphabet and a reset command, and "a test harness is needed to translate abstract messages from the input alphabet to concrete messages". [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- Equivalence "has to be approximated, with what is effectively a form of model-based testing", an improved Chow W-method, which is correct only "if the actual state machine does not have more states than the number of found states plus the specified depth". They sped it up using the domain fact that after "Connection closed" every output is the same. [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- Server alphabet: ClientHello (RSA), Certificate (empty), ClientKeyExchange, ChangeCipherSpec, Finished, ApplicationData (regular and empty), HeartbeatRequest. Nine implementations analysed; TLS 1.0 and 1.2 always gave the same model. A script simplified models "by combining transitions with the same responses". [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- Analysis was manual: "every superfluous state or transition is undesirable and a reason for closer inspection. The presence of such superfluous states or transitions is typically easy to spot visually." Detecting spurious behaviour is "almost fully automatic", deciding exploitability "does require manual investigation". [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- Bugs found as unexpected paths: GnuTLS 3.3.8 had a "shadow" path entered by a HeartbeatRequest during the handshake, ending in an internal-error alert; JSSE (Java 1.8.0_25) had a second path to application data with the client's ChangeCipherSpec omitted, so the server accepted plaintext from the client while encrypting its own data; OpenSSL 1.0.1g's model showed the known Early ChangeCipherSpec flaw (keys computed from an empty master secret when no ClientKeyExchange was sent). mbed TLS showed strange but apparently unexploitable empty-ApplicationData behaviour. New flaws in 3 of 9 implementations; models differ per stack, so they fingerprint implementations. [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- Fiterau-Brostean, Jonsson, Sagonas, Taquist (NDSS 2023): "A bug pattern is a (typically small) automaton specifying a particular error in an implementation's control logic, for instance sending two specific messages in the wrong order, or omitting an authentication step. Dually, bug patterns can also be seen as encoding specific requirements." Patterns come from RFC requirements, bugs reported in other implementations, and protocol knowledge. [read] — [NDSS 2023 paper](https://www.ndss-symposium.org/wp-content/uploads/2023-68-paper.pdf)
- The technique finds sequences the (possibly inaccurate, learned) model can perform and the pattern accepts, then turns them into tests on the real implementation "to find a witness for the bug or filter out false alarms". Results: three SSH servers, nine DTLS implementations; all previously reported bugs reproduced with witnesses, plus new bugs, two new vulnerabilities and non-conformance issues. [read] — [NDSS 2023 abstract page](https://www.ndss-symposium.org/ndss-paper/automata-based-automated-detection-of-state-machine-bugs-in-protocol-implementations)
- Patterns are DFAs with implicit transitions to a sink (no bug) and "other symbol" sets (for example U = Sigma minus {CertReq}); an extra transition back to the start avoids false alarms from renegotiation. Named SSH patterns include Early Service Accept, Unauthenticated Client, Rekey Fail Before/After Auth, Continue After Disconnect, Multiple USERAUTH_SUCCESS, Missing NEWKEYS; DTLS patterns include ChangeCipherSpec before CertificateVerify, Continue After Closure Alert, Early Finished, Insecure Renegotiation. Fifteen patterns were defined from the bugs of the earlier DTLS paper. [read] — [NDSS 2023 paper](https://www.ndss-symposium.org/wp-content/uploads/2023-68-paper.pdf)
- The same paper found that detection with one large "general" automaton of all allowed sequences was inferior to small specific patterns (a general pattern failed to produce witnesses for all bugs after a day), and that learned models reach 133 states (JSSE client) to 387 (OpenSSL), "very difficult to effectively analyze ocularly"; ocular inspection "can miss errors". [read] — [NDSS 2023 paper](https://www.ndss-symposium.org/wp-content/uploads/2023-68-paper.pdf)
- Earlier: learned SSH models were model-checked against security and conformance properties (SPIN 2017); DTLS state fuzzing of thirteen implementations (USENIX Security 2020); TCP models of Linux, Windows, FreeBSD showed RFC 793 violations. [search] — [SPIN 2017 PDF](https://www.cs.ru.nl/~erikpoll/papers/spin2017.pdf); [USENIX Sec 2020](https://www.usenix.org/conference/usenixsecurity20/presentation/fiterau-brostean)
- The TCP work (CAV 2016, "Combining model learning and model checking to analyze TCP implementations") used a mapper with abstraction of sequence numbers and NuSMV model checking; mappers as a formal abstraction layer come from Aarts, Jonsson, Uijen, Vaandrager. [training]
- AALpy supports deterministic learning of DFA, Mealy and Moore machines (L*, KV, which needs fewer queries than L*), passive RPNI, abstracted nondeterministic Mealy machines and stochastic Mealy machines; workflow is alphabet, system under learning, equivalence oracle, algorithm. [search] — [AALpy on PyPI](https://pypi.org/project/aalpy/1.3.3/); [AALpy paper](https://link.springer.com/doi/10.1007/s11334-022-00449-3)

### Inferences

- The bug-pattern automaton is exactly the "near-miss as finding" idea in a proven form: a pattern describes the bad shape, it is checked against the system's own model, and every hit is turned into a witness or discarded. In mentalc the model is not learned but compiled, so the "possibly inaccurate model" problem shrinks to unbound predicates, and mentalvm plays the role of the real-implementation test.
- Their finding that many small patterns beat one big "allowed behaviour" automaton argues for a catalogue of small, named patterns per level, each with its own finding text, rather than one grand specification.
- The mapper is the lifting, and it is hand-written per protocol. In mentalc the pack plays the mapper's role (it fixes which library calls are which effects), so this part already exists.
- "Superfluous state or transition" maps to mentalc's derived lifecycle machine: a write that the derived machine does not explain is an illegal write, the same signal.

### Gaps

- L* applied to a compiled model (mentalvm as the membership oracle) was not found in the literature searched.
- Exact CVE identifiers from the NDSS 2023 paper were not extracted.

## Specification mining of state machines and temporal rules

### Takeaway

Spec mining has two families: automaton inference from traces (Ammons, k-tails, Synoptic, CSight) and template instantiation (Perracotta, Texada, InvariMint), where fixed rule *shapes* are instantiated with events and kept if they hold. The template family is deterministic and explainable, and InvariMint shows that inference algorithms themselves can be written as compositions of mined rule instances.

### Cited Findings

- CSight (Beschastnikh, Brun, Ernst, Krishnamurthy, ICSE 2014) mines logs into a communicating finite state machine (CFSM); it drops the usual assumption that executions are totally ordered, using vector timestamps added to logs; evaluated on stop-and-wait, TCP opening/closing handshakes and Voldemort replication; a 39-student study found CFSMs as useful as time-space diagrams for finding bugs. [search] — [CSight abstract](https://homes.cs.washington.edu/~mernst/pubs/concurrent-models-icse2014-abstract.html); [CSight paper](https://homes.cs.washington.edu:443/~mernst/pubs/concurrent-models-icse2014.pdf)
- Synoptic needs only regular expressions to parse existing logs; it uses k-tails as a final coarsening step. [search] — [Synoptic tool demo](https://www.cs.ubc.ca/~bestchai/papers/esecfse2011-tools-final.pdf); [InvariMint TSE 2015](https://homes.cs.washington.edu/~mernst/pubs/fsm-inference-declarative-tse2015.pdf)
- Synoptic mines three invariant kinds (always followed by, always precedes, never followed by), starts from a coarse partition graph, refines it by counterexample-guided refinement until all mined invariants hold, then coarsens with k-tails. [training]
- InvariMint is declarative: the user lists property types, instances are mined from the log, each instance is an FSM, and instances are combined with FSM union and intersection; existing algorithms (k-tails, Synoptic) were re-expressed this way, and a user can remove a merge by editing a property instance. [search] — [InvariMint TR](https://www.cs.ubc.ca/~bestchai/papers/invarimint_tr.pdf)
- Texada mines instances of any user-given LTL property type from logs, events defined by regular expressions, without new code per type. [search] — [Texada ASE 2015 demo](https://www.cs.ubc.ca/~bestchai/papers/ase15-texada-tool-demo.pdf)
- Perracotta (Yang, Evans, Bhardwaj, Bhat, Das, ICSE 2006) mines alternating rules from imperfect traces, keeping rules above a satisfaction threshold and chaining them into larger machines; applied to the Windows kernel and JBoss. [search mention in Texada slides; detail training] — [Texada slides](https://www.cs.ubc.ca/~bestchai/papers/texada-ase15_final-slides.pdf)
- Ammons, Bodik, Larus, "Mining specifications" (POPL 2002): extract scenarios (dependence-related API calls on the same objects) from traces, learn a probabilistic FSA (sk-strings), then a human cleans the result with a tool (Cable). Biermann and Feldman's k-tails (1972) merges states whose length-k futures agree. [training]
- Static typestate inference: Whaley, Martin, Lam (ISSTA 2002) extract component interfaces statically and dynamically; Alur, Cerny, Madhusudan, Nam (POPL 2005) synthesize Java class interfaces with L* plus predicate abstraction run against the code as an oracle; Shoham, Yahav, Fink, Pistoia (ISSTA 2007) mine specs statically with automata-based abstractions. [training]
- Engler, Chen, Hallem, Chou, Chelf, "Bugs as deviant behavior" (SOSP 2001): infer "beliefs" from code (this lock always guards this variable, this call is always checked); a statistically rare violation of a belief that most code follows is reported, ranked by a z-statistic. [training]

### Inferences

- mentalc's lifecycle machine (each write a transition from the value a prior test established) is a static typestate derivation; Learning in the style of Alur et al. is not needed because write sites are enumerable.
- Template mining (Texada, Perracotta) matches the owner's pattern idea at the effect level: a pattern is a temporal template over effect kinds (read X; branch on X; write X), instantiated per state variable. InvariMint's insight that a model is the intersection of rule automata suggests each level's IL could be defined by a small set of composable pattern automata.
- Engler's deviance is a second, statistical, kind of near-miss: "nine of ten claims on this table use a conditional update; this one does not". It is a belief, not a fact, so in mentalc it could only rank or label a prediction, never create a record.
- k-tails and threshold-based mining generalize from incomplete evidence; mentalc does not have incomplete evidence about its own code, so these generalizations would add guesses.

### Gaps

- Did not open Ammons 2002, Perracotta, Alur et al. 2005 or Engler 2001 this session.
- No work found that mines spec templates over a *static* effect graph of a web application.

## Static extraction of models of distributed systems from code

### Takeaway

The deterministic static route exists only for languages with explicit channels (Go behavioural types, Erlang abstractions); for mainstream services the 2025-2026 frontier is LLM agents that write TLA+ from code and then check conformance by trace validation, which inverts mentalc's rule that the LLM never writes facts.

### Cited Findings

- Specula (Cheng, Pial, Tang, Su, Ma, Hackett, Beschastnikh, Huang, Xu; arXiv 2607.25333, July/August 2026) is "a push-button agentic system that generates high-quality formal specifications for large, complex system code": LLM coding agents write TLA+ models and invariants, with "self-evolving loops" against reward hacking and hallucination; applied to 48 open-source system projects, 249 bugs. [read, abstract] — [arXiv 2607.25333](https://arxiv.org/abs/2607.25333)
- Specula's pipeline (from search summaries of the paper and its repository): a code-analysis phase writes a modeling brief; generated artifacts include a model following the code's control flow, a bounded model-checking spec, a trace-validation spec and an instrumentation spec; a control-flow analyzer is paired with the LLM generator; failing trace validation triggers repair of the model or the instrumentation. It won the 2025 TLA+ GenAI challenge. [search] — [Specula repo](https://github.com/specula-org/specula); [tlaplus group post](https://groups.google.com/g/tlaplus/c/VSk8wF4jYvc)
- A SysMoBench evaluation reports LLMs reliably write syntactically valid TLA+ but often fail conformance to real implementations and invariants. [search] — [SIGOPS blog](https://sigops.org/2026/can-llms-model-real-world-systems-in-tla)
- Godel Checker (Lange, Ng, Toninho, Yoshida, ICSE 2018) translates Go to SSA, infers a behavioural type of the program's channel communication, and model-checks and termination-checks it for safety and liveness; "Fencing off Go" (POPL 2017) restricts channel usage so a program has finitely many communication patterns. [search] — [ICSE 2018 page](https://mrg.cs.ox.ac.uk/publications/a-static-verification-framework-for-message-passing-in-go-using-behavioural-types/)
- P is a state-machine language AWS has developed since 2019; it was used for S3's move to strong read-after-write consistency; its checker explores interleavings of messages and failures; PObserve checks structured production logs against a P specification after the fact; P 3.0 adds a verifier. [search] — [P casestudies](https://p-org.github.io/P/casestudies); [CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services)
- Trace validation of distributed programs against TLA+ specs (Cirstea, Kuppe, Merz and others, 2024), Modelator (model-based testing from TLA+ for Tendermint/Cosmos), Mocket (EuroSys 2023, spec-guided testing by mapping code variables to TLA+ variables), Soter (Erlang to an abstract actor model, SAS 2013), and dingo-hunter (Go, global-graph synthesis for deadlock detection, 2016) are related; Gobra is a Viper-based Go *verifier*, not a model extractor. [training]

### Inferences

- mentalc's model already is the "control-flow-following model" Specula asks the LLM to write, built mechanically. Specula's trace-validation step is the part that transfers: it is a conformance check between a model and runs, which mentalc could use later for grounding (PObserve is the production version).
- Behavioural-type inference is the static, deterministic precedent for lifting effect graphs to an exchange/activation level; it works because Go channels are explicit. In Python services the channels are library calls, which packs already name.

### Gaps

- "Tango" and "RAFT model extraction" as named in the brief: no specific system was found or verified; Tango may refer to a 1990s Estelle trace-analyzer generator [training, uncertain].
- No static, LLM-free extractor of message-passing models from Python services was found.

## Business-process and log-level lifting (process mining event abstraction)

### Takeaway

Process mining has the closest analogue to the owner's per-level pattern idea: Guided Process Discovery lets an expert write *activity patterns* over low-level events and matches them into high-level activities, and conformance checking (alignments) reports exactly where a trace and a pattern disagree.

### Cited Findings

- van Zelst, Mannhardt, de Leoni, Koschmider, "Event abstraction in process mining: literature review and taxonomy", Granular Computing 6(3), 2021 (online 2020): source systems record events at different granularity, so abstraction is a key preprocessing step; the paper reviews techniques and gives a taxonomy. [search] — [Springer](https://link.springer.com/article/10.1007/s41066-020-00226-2)
- Guided Process Discovery (Mannhardt, de Leoni, Reijers, van der Aalst, Toussaint, Information Systems 76, 2018) groups low-level events by behavioural activity patterns that "capture domain knowledge on the relation between high-level activities and low-level events" over control flow, time, resource and data; tested on two real logs (a Norwegian hospital's digital whiteboard among them). [search] — [TU/e record](https://research.tue.nl/en/publications/guided-process-discovery-a-pattern-based-approach/)
- Mannhardt and Tax (2017) proposed an unsupervised variant using Local Process Model discovery, because writing a pattern per activity is tedious. [search] — [arXiv 1704.03520](https://arxiv.org/pdf/1704.03520v2.pdf)
- GPD composes the patterns into one abstraction model and computes an alignment between each low-level trace and that model; events that align with a pattern become instances of its activity, and events that cannot be aligned are reported as unmatched, with a matching-error measure. [training]
- The survey's taxonomy dimensions include supervised vs unsupervised, whether the technique needs domain knowledge, and whether it handles interleaved activity instances. [training; full text returned 403]
- Object-centric event logs (OCEL, van der Aalst 2019 onward) attach one event to several objects. [training]

### Inferences

- An activity pattern is a "higher-level IL instruction" defined over lower-level events, and GPD's alignment is the matcher. Applied to mentalc, patterns would be matched against behavior paths (from the code) rather than logs, so the matching is total and deterministic.
- Interleaving, a named difficulty in the taxonomy, is the case mentalc cares about most: two activations interleaving inside one move. Patterns must match per activation (mentalc's traces carry the activation, which logs usually lack).
- Object-centric logs match the spec's object-centric outcomes; the process-mining precedent says a goal touching two subjects belongs to both.

### Gaps

- Survey full text was refused (HTTP 403), so the taxonomy above is partly from memory.

## How these techniques report unmatched or anomalous behavior

### Takeaway

Four reporting forms recur: superfluous states/transitions in a learned model (TLS), hits of a bug-pattern automaton confirmed by a witness (SSH/DTLS 2023), non-synchronous moves in an alignment (process mining), and statistical deviations from a majority belief (Engler). All four have a direct mentalc counterpart; only the first two are already proven to find real bugs in protocol code.

### Cited Findings

- TLS: "every superfluous state or transition is undesirable and a reason for closer inspection". [read] — [tls_fuzzing.pdf](https://www.cs.ru.nl/~erikpoll/papers/tls_fuzzing.pdf)
- SSH/DTLS: a hit is a sequence the model can perform that the bug automaton accepts; it is tested on the real system to get a witness or discarded as a false alarm; some found bugs "could not validate" and are marked separately. [read] — [NDSS 2023 paper](https://www.ndss-symposium.org/wp-content/uploads/2023-68-paper.pdf)
- Learned models may be inaccurate; hence the validation step. [read] — [NDSS 2023 abstract page](https://www.ndss-symposium.org/ndss-paper/automata-based-automated-detection-of-state-machine-bugs-in-protocol-implementations)
- Alignments (Adriansyah and van der Aalst, 2011 onward; Carmona et al., "Conformance Checking", 2018): an optimal alignment pairs a trace with a model run using synchronous moves, log-only moves (the trace did something the model does not allow) and model-only moves (the model required a step the trace skipped); a cost function, usually computed by A*, picks the cheapest. A model-only move is precisely "the pattern expected a step that is missing". [training]
- Engler 2001: deviations from majority beliefs, ranked statistically. [training]
- Synoptic/CSight show a counterexample path when a mined invariant fails in the model. [training]

### Inferences

- A near-miss in mentalc is an alignment of a behavior path against a pattern with exactly one model-only move: "claim a job" = read status, branch on PENDING, *guarded* write RUNNING; the report-jobs claim aligns with the guard step as a model-only move, so the finding names the missing step and its anchor. Alignment costs give a principled "how near" measure and a deterministic tie-break.
- The SSH/DTLS split between "found" and "found but could not validate" maps to mentalc's prediction vs witness-confirmed finding.

### Gaps

- No work found that uses partial pattern matches as findings in static analysis of web services specifically.

## Fit for mentalc

Criteria: static, pure, deterministic, every record anchored, fail rather than degrade, no LLM for facts. Ranked best fit first.

1. **Bug-pattern automata checked against the model (Fiterau-Brostean et al. 2023).** Small DFAs over an abstract alphabet, with sink and "other" sets, intersected with the system's model; each hit is a prediction until a run confirms it. Fits fully: mentalc's alphabet is the effect instruction set (or a level's IL), the model is compiled not learned so there are fewer false alarms, intersection is deterministic, a hit is anchored by the nodes it passes, and mentalvm supplies the witness. Their evidence that many small named patterns beat one large automaton fits a per-level catalogue.
2. **Activity patterns plus alignment (Guided Process Discovery, conformance checking).** The most direct model for "an IL per level, matched by patterns": a pattern per upper step class, alignment against each behavior path, synchronous moves become the lift, model-only moves become near-miss findings with a cost. Deterministic with a fixed cost function and tie-break, anchored via the aligned nodes. Risk: alignment over all paths of a large behavior can be expensive; restrict to per-case paths and state a bound, and stop the build rather than drop paths.
3. **Template mining and declarative composition (Texada, InvariMint).** Fixed temporal rule shapes instantiated per state variable, and a level's IL defined as a composition of rule automata. Deterministic and explainable when mined over the model's paths instead of logs. The threshold-based "mostly holds" variants (Perracotta) are out.
4. **Behaviour as the type of a message (Prospex) and static loop-as-state extraction (Shi et al. 2023).** Confirm the existing design: a message type is the case its handler takes; path-merging rules are the practical key to a precise static state machine. Reuse as design guidance for lifecycle machines and input classes, not as code.
5. **Behavioural-type inference (Godel, MPST work).** Static and deterministic lifting of communication into a session type; fits the activation and exchange candidates. Needs explicit channels, which mentalc gets from packs; the theory is heavier than the current need.
6. **Deviance mining (Engler 2001).** Static and deterministic, but its output is a statistical belief. Usable only to rank or label predictions ("this claim differs from the other nine"), never to create or remove a record.
7. **Active automata learning (L*, LearnLib, AALpy).** Needs a running system, and mentalc runs nothing; for mentalc itself it does not fit. With mentalvm as the membership oracle it could learn a subject's upper-level machine from explored runs and compare it with the static derivation, which is a ready-made method for the static-vs-explored levels experiment; equivalence would be bounded, which matches how mentalvm states results.
8. **Trace validation and PObserve.** Conformance of real logs to a model. Fits grounding later (checking the model against production), not the compiler.
9. **Passive inference from traces (k-tails, Synoptic, CSight, Ammons) and dynamic protocol reverse engineering (Polyglot, Dispatcher, AutoFormat, Tupni, Discoverer, ReverX, Netzob).** Dynamic, need observations mentalc does not take, and generalize by heuristic merging from positive examples only (the Gold limit), which is guessing. Ideas transfer (prefix trees, merge constraints); the tools do not.
10. **LLM pipelines that write the model (Specula and similar).** The LLM writes the facts, which mentalc forbids; reported conformance failures confirm the risk. Only the conformance-loop structure (model, instrumentation, trace validation, repair) is worth borrowing, with the mechanical model in the LLM's place.

What I skipped or could not check: the protocol RE surveys, the pre-2009 protocol RE papers, Ammons 2002, Perracotta, Alur et al. 2005, Engler 2001 and the process-mining survey full text were not opened this session (tagged training or search); "Tango" and "RAFT model extraction" could not be identified.
