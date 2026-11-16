# Sources: how people perceive levels, and how practice and LLMs build them

Evidence tags: **[S]** seen in a search result this session, 2026-11-16 (link given; mostly abstracts and secondary summaries, not full texts); **[K]** background knowledge, not re-verified; **[M]** from Mental's own trials and experiments.

## One line per source

**How people perceive levels and events**

| Source | What it says in one line |
|---|---|
| Rasmussen's abstraction hierarchy | A work domain is described at five levels linked by means and ends: going up answers *why*, going down answers *how*. |
| Action identification theory | People describe an action at the highest level that works, and drop to a lower level when it gets hard or fails. |
| Event segmentation theory | People cut continuous activity into events, coarse ones containing fine ones, at points where goals, actors, places or causes change. |
| Causal network model of stories | Events on the causal chain to the outcome are recalled, summarised and rated important; dead ends are not. |
| Basic-level categories | There is one level at which people name things fastest; experts' preferred level sits lower than novices'. |
| Working-memory capacity | About four chunks can be held in mind at once, fewer than the older "seven plus or minus two". |
| Expertise reversal effect | Guidance that helps novices slows experts down. |
| Program comprehension models | Developers build a control-flow model, a domain-level model and a top-down model, and switch between them all the time. |
| Whyline | Letting developers ask "why did" and "why didn't" about a run made debugging much faster in small studies. |
| Trace visualization experiments | A trace view beside the IDE cut task time by 22% and raised correctness by 43%. |
| Fisheye views | Show each item by its importance minus its distance from the focus. |
| Physics of Notations | Diagrams need explicit complexity management (modules and hierarchy) and a dialect per audience. |

**How practice builds levels**

| Source | What it says in one line |
|---|---|
| Cockburn's goal levels | Use cases sit at cloud, kite, sea, fish or clam level; "sea level" is one actor, one sitting, one goal; ask *why* to go up and *how* to go down. |
| Hierarchical task analysis | Goals are redescribed into subgoals with plans; stop expanding when probability times cost of error is acceptable. |
| EventStorming | Three workshop levels (big picture, process, software design) with the same notation and different participants. |
| Domain Storytelling | A story's scope has three settings: coarse or fine, as-is or to-be, and *pure* (no software) or *digitalized*. |
| APQC Process Classification Framework | Business work in five nested levels: category, process group, process, activity, task. |
| ArchiMate and ISO 42010 | Business, application and technology layers; a viewpoint names its stakeholders and the concerns it frames. |
| C4 model | Four zoom levels (context, container, component, code), each for a different audience. |
| Lipton's reduction | A run of statements shaped right-movers, at most one non-mover, left-movers can be treated as one atomic action without changing what is reachable. |

**What LLMs do with levels**

| Source | What it says in one line |
|---|---|
| Hierarchical code summarization (TCS, 2025) | Summaries built bottom-up miss business context unless the prompt is grounded in it. |
| CodeWiki, DeepWiki, RepoAgent | Repository documentation generated over a module tree, parents written from children. |
| LLM event abstraction in process mining | An LLM labels low-level events as activities, given examples. |
| Mental's rq experiment | LLM-written abstraction functions were too fine and unstably named; their invariants were useful. |

## Perception: the claims and their strength

**Why and how.** Rasmussen's abstraction hierarchy runs from functional purpose at the top to physical form at the bottom; moving down answers how an element is achieved, moving up why it exists. It is a means-ends hierarchy, and Rasmussen kept it apart from part-whole decomposition, which is a second, independent dimension [S](https://surfingcomplexity.blog/2022/09/) [K]. The evidence is thinner than the idea's popularity: Vicente and Rasmussen (1992) report "preliminary support"; Ham and Yoon (2000) ran three process-control experiments and concluded that information should be shown at all abstraction levels *with the goal-means links preserved* [S](https://dspace.kaist.ac.kr/bitstream/10203/6871/3/2000%5bDong-Han%20Ham%2c%20Wan%20Chul%20Yoon%5dThe%20effects%20of%20Providing%20Functionally%20Abstracted%20information%20in%20process%20control%20tasks.pdf). One practitioner reports never having seen an operator interface that implements the hierarchy well [S](https://surfingcomplexity.blog/2022/09/).

**The level people choose.** Action identification theory (Vallacher and Wegner, 1987): identities of an action range from low-level ones that say how it is done to high-level ones that say why. Successful, familiar actions are identified high; difficult, unfamiliar or failing ones low, and attention then goes to details [S](https://scholar.harvard.edu/sites/scholar.harvard.edu/files/dwegner/files/vallacher_wegner__frederick_1987.pdf).

**Where events begin and end.** Event segmentation theory (Zacks and colleagues): people segment activity as a side effect of predicting what comes next. Fine event models nest inside coarse ones. Boundaries fall where goals of actors, causal relations, characters, objects or locations change, and what happens at a boundary is remembered better [S](https://pmc.ncbi.nlm.nih.gov/articles/PMC2263140) [S](https://pmc.ncbi.nlm.nih.gov/articles/PMC5734104). The prediction-error mechanism is debated (a recent preprint ties boundaries to working memory instead), but the behavioural facts about where boundaries fall are not what is disputed [S](https://davachilab.psychology.columbia.edu/sites/default/files/content/Predictable%20Boundaries.pdf).

**What is remembered.** In the causal network model (Trabasso, van den Broek, Sperry, 1984–85), statements on the causal chain are recalled more often, included more in summaries and rated more important than dead-end statements; an event's importance grows with its number of causal connections [S](https://escholarship.org/content/qt02f7h4gs/qt02f7h4gs.pdf).

**The basic level.** Rosch (1976): novices name fastest at a basic level ("dog", not "animal" or "beagle"). Tanaka and Taylor (1991): for experts, subordinate categories are as fast and as often used as basic ones. Johnson and Mervis (1997): for advanced experts two lower levels work as basic [S](https://www.ece.uvic.ca/~bctill/papers/learning/Tanaka_Taylor_1991.pdf) [S](https://pmc.ncbi.nlm.nih.gov/articles/PMC4123786).

**Capacity.** Cowan (2001): about four chunks on average, 3 to 5 in a later review, in the focus of attention; contested in its exact number [S](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/magical-number-4-in-shortterm-memory-a-reconsideration-of-mental-storage-capacity/44023F1147D4A1D44BDC0AD226838496) [S](https://pmc.ncbi.nlm.nih.gov/articles/PMC2864034). Moody's Physics of Notations asks for explicit complexity management through modularization and hierarchy, and for a visual dialect per task and audience (cognitive fit) [S](https://arxiv.org/pdf/1404.2737).

**Expertise.** Kalyuga and colleagues (2003): worked examples and explanations that help low-knowledge learners make experts slower, because experts must reconcile the support with what they already know; the remedy is fading guidance as knowledge is shown [S](https://en.wikipedia.org/wiki/Expertise_reversal_effect).

**Developers.** Pennington: programmers first build a control-flow *program model*, then a functional *situation model*. Von Mayrhauser and Vans (1995) integrated these with a top-down domain model and observed engineers switching between all of them during enhancement and debugging [S](https://www.cs.kent.edu/~jmaletic/cs69995-PC/papers/von_mayrhauser-1995.pdf) [S](https://arxiv.org/pdf/2310.11301). Sillito, Murphy and De Volder (2006) catalogued 44 kinds of questions programmers ask during change tasks [S](https://dblp.org/rec/conf/sigsoft/SillitoMV06).

**Asking beats watching.** The Whyline (Ko and Myers): in the 2004 study, debugging time fell by nearly a factor of 8 with 9 participants; in 2009, Java developers were twice as successful and twice as fast; 68% of questions were "why didn't". These are small lab studies [S](https://faculty.washington.edu/ajko/slides/ICSE2018MIP.pdf) [S](https://hci.rwth-aachen.de/materials/conferences/CHI2004/1p151.pdf). Cornelissen and colleagues (2011): adding the EXTRAVIS trace view to Eclipse gave 22% less time and 43% more correct answers on eight comprehension tasks; a 2015 replication with ExplorViz found 28% less time and 39–61% better correctness [S](https://azaidman.github.io/publications/cornelissenTSE2010.pdf) [S](https://oceanrep.geomar.de/id/eprint/28324/1/Comparing_Trace_Visualizations_ICPC2015_preprint.pdf). An interview study of distributed-tracing tools (Davidson, Wall, Mace, 2023; six practitioners) names five challenges and eight guidelines; we did not read its full text [S](https://people.mpi-sws.org/~jcmace/papers/davidson2023qualitative.pdf).

**Focus and context.** Furnas's degree of interest: a priori importance minus distance from the focus, with items shown above a threshold [S](https://www.ifs.tuwien.ac.at/~mlanzenberger/teaching/ps/ws06/stuff/p16-furnas.pdf). Shneiderman's "overview first, zoom and filter, details on demand" [K].

## Practice: who already has levels, and what defines them

**Goal levels.** Cockburn: cloud (very high summary), kite (summary), sea (user goal), fish (subfunction), clam (too low). The sea-level test: one actor, one sitting, one measurable outcome; "does the actor go home happy?" Raise the level by asking why, lower it by asking how [S](https://cs.otago.ac.nz/coursework/cosc461/usecases.htm) [S](https://llm-coding.github.io/Semantic-Anchors/anchor/cockburn-use-cases).

**Task analysis.** Hierarchical task analysis redescribes a goal into subgoals with a plan that states order and conditions; the P × C rule stops expansion when the probability of inadequate performance times its cost is acceptable. Stanton (2006) calls the stopping rule a challenging part of the method [S](https://www.bohrium.com/paper-details/hierarchical-task-analysis-developments-applications-and-extensions/812139059903201285-3898).

**Workshop methods.** EventStorming has three levels with the same mechanics and different scope, participants and output: big picture (shape of the domain), process level (one flow), software design (aggregates, commands, events) [S](https://barkingiguana.com/writing/the-workshop-event-storming). Domain Storytelling sets a story's scope by granularity (coarse or fine), point in time (as-is or to-be) and purity (pure stories leave software out; digitalized ones include it), and recommends starting coarse, pure and as-is [S](https://cheatography.com/jacobd79/cheat-sheets/domain-storytelling).

**Process frameworks.** APQC's framework decomposes work into category, process group, process, activity and task; elements are mutually exclusive and collectively exhaustive, and their order does not imply execution order [S](https://www.apqc.org/sites/default/files/files/PCF%20Collateral/Understanding%20the%20PCF%20Elements%20%20-%20FINAL.pdf).

**Architecture frameworks.** ArchiMate's viewpoints follow ISO/IEC 42010: a viewpoint frames named concerns for named stakeholders and fixes the layers it uses (business, application, technology) [S](https://pubs.opengroup.org/architecture/archimate31-doc/chap14.html). The C4 model gives four zoom levels, each "suitable for different audiences" (vendor guides; we found no empirical study of its use) [S](https://icepanel.io/blog/what-is-the-c4-model). Hollnagel's distinction between work-as-imagined and work-as-done [K].

**Atomic moves.** Lipton (1975): a statement is a right mover if it commutes to the right of any step of another thread and a left mover if it commutes to the left; a sequence of right movers, at most one non-mover, then left movers can be reduced to one atomic action [K] ([CACM 18(12)](https://doi.org/10.1145/361227.361234)). Flanagan and Qadeer (PLDI 2003) infer mover types and check block atomicity statically [K] ([paper](https://doi.org/10.1145/781131.781169)). This is the basis of the move verdict proposed in section 5.3 of the report, added after the experiment.

## LLMs and levels

- **Bottom-up summaries lose the business.** The TCS study (LLM4Code 2025) reports that models focus on low-level detail and miss business context for files and packages; hierarchical aggregation improved coverage and grounding prompts in business context raised domain specificity by over 7% (one telecom system; we did not see the full tables) [S](https://arxiv.org/abs/2501.07857).
- **Module-tree documentation.** CodeWiki builds a module tree by static analysis and writes parents from children; its own benchmark scores it 68.79% against DeepWiki's 64.06% (the authors' own comparison) [S](https://arxiv.org/html/2510.24428v5).
- **Event abstraction.** GPT-4 with few-shot examples labelled sensor events as activities at about 90% average accuracy on one participant's 146-day log, 81% to 97% by source; the authors ask for human oversight [S](https://arxiv.org/abs/2409.03478v1).
- **Our own data.** On rq, eight LLM-written abstraction functions had 212–502 abstract states against the reference's 98, and their variable names overlapped between samples with a median Jaccard of 0.54 [M](../../abstraction-function/report.md). The earlier survey found that LLM-generated architecture views work at code-level granularity unless a deterministic loop constrains them [M](../../abstraction-levels/report.md).

## Mental's own evidence used in the report

- **One run at seven levels** (findings F-Z1 to F-Z9): step counts per level 4, 6, 21, 26, 26, 30, 34; mapping by step identity; "each problem has a level where it can be named"; the throwaway viewer switched the whole view to one level at a time and that was judged not a design [M](../../../findings.md).
- **The rq witness** (abstraction-function report, F-A8): a 42-step run with 2 preemptions in which a finished job stays listed as started. Its step listing is the basis of the worked example in the report. The grouping of its steps into upper steps was done by hand for this report; no tool produced it.
- **F-A9**: the oracle in the rq experiment could not be written until the worker's operation was split into take and complete [M](../../abstraction-function/report.md).
