# Sources: how other fields build and explain multi-level models

Evidence tags: **[S]** seen in a search result this session, 2026-11-17 (link given; these are abstracts, excerpts and secondary summaries, not full texts read); **[K]** background knowledge, not re-verified this session. Sources that the [multi-level models report](../../multi-level-models/notes/sources.md) already covers (Rasmussen's abstraction hierarchy, Cockburn's goal levels, hierarchical task analysis, EventStorming, Domain Storytelling, APQC, ArchiMate, C4, event segmentation, the causal network model) are not repeated here, except where this survey adds something.

## One line per source

### Business processes

| Source | What it says in one line |
|---|---|
| Process architecture levels (L0–L5) | Practice names five or six nested levels from value chain to work instruction, but no two vendors define them the same way. |
| SCOR | Levels 1–3 are a shared industry vocabulary; level 4 is left to each company on purpose. |
| ISO 9001 documentation pyramid | Policy, process, procedure, work instruction, record; a procedure names a hand-off between functions, a work instruction one task at one station. |
| BPMN Method and Style (Silver) | Top-down hierarchy where every subprocess end event names an end state, and the parent's gateway tests exactly those end states. |
| IDEF0 | Every child diagram has 3 to 6 boxes; the arrows into and out of a box must reappear at the edge of its child diagram; every model states its purpose and viewpoint. |
| 7PMG (Mendling, Reijers, van der Aalst) | Seven guidelines from empirical studies of model errors and comprehension, among them verb-object labels and decomposing large models. |
| Studies of modularized process models | Subprocesses help by abstraction and hurt by fragmentation; which wins depends on the question asked. |
| Capability maps versus process maps | A capability is *what* the business does, a process *how*; practitioners say the two can be mapped but are not one hierarchy. |
| Object-centric process mining (OCEL) | An event can belong to several objects; forcing one case notion duplicates events (convergence) or merges unrelated repetitions (divergence). |
| Fuzzy Miner | Process models from logs drawn like road maps: significant activities kept, correlated minor ones aggregated into clusters, the rest dropped. |

### Mathematics

| Source | What it says in one line |
|---|---|
| Lamport's structured proofs (1993, 2011) | A proof is a hierarchy of numbered claims, each proved by the level below; a reader goes as deep as they want. |
| Leron's structured proofs (1983) | An earlier hierarchical proof format, aimed at communication rather than rigour. |
| Weber and Mejía-Ramos (2011, 2014) | Mathematicians read proofs line by line, modularly (by the high-level structure), or by trusting others who checked it. |
| Mejía-Ramos et al. proof comprehension model (2012) | Understanding a proof is tested at a local level (terms, statements, justifications) and a holistic one (high-level ideas, modular structure, transfer). |
| Lean blueprints (Massot; PFR project) | A dependency graph of definitions and lemmas, coloured by status: stated, ready to formalize, statement formalized, proof formalized. |
| Thurston, *On proof and progress* | One idea (the derivative) is understood through many different views; understanding is not one definition. |
| Tao, pre-rigorous / rigorous / post-rigorous | Readers mature through three stages; the post-rigorous reader uses intuition backed by rigour. |

### Engineering and design

| Source | What it says in one line |
|---|---|
| Gajski–Kuhn Y-chart | Hardware has three domains (behaviour, structure, geometry), each with its own levels; synthesis moves between domains at one level. |
| Arcadia / Capella | Four perspectives: operational analysis (no system yet), system need, logical architecture, physical architecture. |
| Architectural drawing sets | Sheets go from the whole at small scale to details at large scale; each callout carries the address of the drawing it refers to; a callout pointing nowhere is a known error. |
| Alexander, *A Pattern Language* | Each of 253 patterns names the larger patterns it helps complete and the smaller ones that complete it. |
| Agrawala et al., assembly instructions (2003) | People conceive assembly hierarchically, with steps separated by new parts; instructions built on that cut assembly time by about a third and errors by half. |

### Science

| Source | What it says in one line |
|---|---|
| Marr's three levels | Computational (what and why), algorithmic (how, as representation and procedure), implementational (how, physically); Marr gave the first primacy. |
| Newell's knowledge level; Dennett's stances | The same three-way split: knowledge / symbol / physical, intentional / design / physical. |
| Craver; mechanistic explanation | Levels of mechanisms exist relative to a phenomenon: a part is at a lower level when it is a working component of that mechanism. |
| Wimsatt's levels of organization | Levels are local maxima of regularity and predictability. |
| Simon, *The Architecture of Complexity* | Systems are nearly decomposable: dense, fast interactions inside a subsystem, sparse, slow ones between; that is why hierarchies can be understood. |
| Hierarchy theory (Allen and Starr; Salthe) | Levels differ by rate and scale; a focal level is explained by the level below (initiating conditions) and the level above (boundary conditions). |
| Noble's biological relativity | No privileged level of causation in biology; a critic answers that levels are privileged relative to a question. |
| Coarse-graining and effective theories in physics | Each scale has its own variables and laws; details below a scale enter only through a few parameters. |

### Operations, safety and command

| Source | What it says in one line |
|---|---|
| Levels of war (US doctrine) | Strategic, operational and tactical levels are defined by the purpose of an action, not by the size of the unit. |
| Commander's intent | Every order restates the intent of the commanders one and two levels up, so subordinates can act when the plan breaks. |
| Jaques, Requisite Organization | Levels of work are defined by time span; when two levels' time spans overlap there is one level too many. |
| AcciMap (Rasmussen) | An accident drawn as one causal diagram across levels from equipment and operators up to regulators and government. |
| STAMP / STPA (Leveson) | A system is a hierarchy of controllers, each constraining the level below and receiving feedback from it. |
| FRAM, work-as-imagined and work-as-done (Hollnagel) | Written procedures and actual practice differ, and the difference is what the analysis looks at. |

### Explaining and teaching

| Source | What it says in one line |
|---|---|
| Minto, *The Pyramid Principle* | Each point raises a *why* or *how* in the reader, answered by the points below it; ideas in one group share one level of abstraction. |
| Reigeluth, elaboration theory | Teach like a zoom lens: start with an epitome (a simple but complete whole), zoom into a part, zoom back out to place it. |
| Bret Victor, *Up and Down the Ladder of Abstraction* | Go up by abstracting over a variable (all runs at once), go down to one concrete run; understanding moves between them. |
| Rhetorical Structure Theory | A text is a tree of nucleus–satellite relations; with satellites deleted the text still makes sense, not the other way round. |

### Music and maps

| Source | What it says in one line |
|---|---|
| Lerdahl and Jackendoff, GTTM (1983) | Grouping of a piece is governed by well-formedness rules (what a valid hierarchy is) and preference rules (which valid one a listener hears); each group has a head that stands for it at the level above. |
| Schenkerian analysis | A piece is read at foreground, middleground and background; each level prolongs the one behind it. |
| Töpfer's radical law | When a map's scale shrinks, the number of objects kept falls with the square root of the scale ratio; the law says how many, not which. |
| Cartographic generalization operators | Selection, simplification, aggregation, displacement, exaggeration: named operations for going from a detailed map to a coarser one. |

## Business processes

**Process architecture levels.** Vendors and consultants describe process hierarchies from L0 or L1 (value chain, enterprise) to L4 or L5 (activity, task, work instruction), but the meaning of each number differs between ServiceNow, Kissflow, Gluu and others [S](https://kissflow.com/workflow/bpm/business-process-hierarchy/) [S](https://gluu.biz/lessons/process-architecture-frameworks/) [S](https://www.servicenow.com/docs/r/5lP9Pa2tE3N8lrzw94z7xA/4O6r6AYX1HZkmXrB9k8bug). Gluu recommends taking the names of L1–L3 from APQC and localising the activity and instruction levels [S](https://gluu.biz/lessons/process-architecture-frameworks/). One commentary argues that value streams, capabilities and processes are different models with no fixed level correspondence [S](https://www.besthub.dev/articles/why-value-streams-capabilities-and-processes-are-not-a-hierarchy-the-mapping-relationship-explained-d8d78c6dd1c7).

**SCOR.** Level 1 is scope (plan, source, make, deliver, return, enable), level 2 configuration, level 3 process elements with metrics (codes such as S1.1); level 4 is company-specific and outside the standard because it cannot be standardised [S](https://en.wikipedia.com/wiki/Scor_(company)) [S](https://umbrex.com/resources/frameworks/strategy-frameworks/scor-model/) [S](https://courses.ischool.berkeley.edu/i243/s06/readings/SCOR-Overview.pdf).

**ISO 9001 pyramid.** Policy or manual, procedures, work instructions, records. Not required by ISO 9001:2015. A practical test: a document that names a hand-off between two functions is a procedure; one that names a single task at a single station is a work instruction [S](https://advisera.com/9001academy/knowledgebase/how-to-structure-quality-management-system-documentation/) [S](https://www.qmsuk.com/news/iso-9001-processes-procedures-and-work-instructions) [S](https://qms2go.com/learn/blog/qms-document-hierarchy-how-to-structure-quality-documentation).

**BPMN Method and Style.** Silver's method is hierarchical top-down. Each subprocess with several outcomes has one labelled end event per end state, and the gateway after it in the parent diagram has gates with the same labels; the matching labels are what make the hierarchy traceable [S](https://www.trisotech.com/bpmn-style-rules/) [S](https://www.businessprocessincubator.com/?p=64685) [S](https://www.trisotech.com/bpmn-101-what-is-a-process/). That a process diagram must describe one kind of instance (one order, one claim) is from Silver's writing [K].

**IDEF0.** The A-0 diagram is one box with its bounding arrows and states the model's purpose and viewpoint; child diagrams have 3 to 6 boxes; the inputs, controls, outputs and mechanisms (ICOM) entering a child diagram must correspond to the arrows of its parent box [S](https://classes.engr.oregonstate.edu/mime/fall2018/ie545-001/Slides/01-3b%20&%2002-3a%20IDEF0%20(Funk).pdf) [S](https://vitechcorp.com/resources/CORE/onlinehelp/desktop/Views/IDEF0.htm) [S](https://notes-aarongilly.vercel.app/articles/Article---IDEF0-Functional-Modeling). The standard is FIPS 183 (1993) [K].

**7PMG.** Mendling, Reijers and van der Aalst, *Information and Software Technology* 52(2), 2010; seven guidelines synthesised from studies relating model structure to errors and comprehension [S](https://www.vdaalst.com/publications/p574.pdf). The specific guidelines used in the report, verb-object labels (G6) and decomposing models above about 50 elements (G7), are from memory of the paper and were not seen in the search results [K].

**Modularization.** Zugal, Pinggera, Weber, Mendling and Reijers (2011–2012) name two opposing forces: abstraction (information hiding, pattern recognition) and fragmentation (split attention between diagrams). A 2024 *Information Systems* study confirms both effects at coarse and fine grain. Turetken et al. find fragmentation raises mental effort. Reviews say the empirical results "hardly converge" into validated guidelines [S](https://orbit.dtu.dk/en/publications/assessing-the-impact-of-hierarchy-on-model-understandability-a-co/) [S](https://link.springer.com/doi/10.1007/s12599-019-00577-4) [S](https://lepo.it.da.ut.ee/~dumas/pubs/bise2016.pdf). The claim that hierarchy helps local questions and hurts global ones is how Zugal et al. frame the trade-off [K]; the search results support only the two forces.

**Capabilities and processes.** Capability maps say what the organisation does; process maps say how. BIZBOK names capabilities with nouns and processes with verb-noun forms; the two are complementary and mapped, not equated [S](https://www.boc-group.com/en/?p=132978) [S](https://brcommunity.com/articles.php?id=b927) [S](https://pubs.opengroup.org/togaf-standard/business-architecture/business-capabilities.html).

**Object-centric process mining.** In an OCEL log an event relates to several typed objects. Flattening to one case notion causes convergence (one event copied into several cases) and divergence (independent repetitions within a case look related) [S](https://www.vdaalst.com/publications/p1450.pdf) [S](https://arxiv.org/pdf/2411.10468) [S](https://1login.easychair.org/publications/preprint/L9zW).

**Fuzzy Miner.** Günther and van der Aalst (BPM 2007) mine process "maps" at several levels of detail using significance and correlation metrics: significant nodes are kept, less significant but highly correlated ones aggregated into clusters, the rest abstracted away [S](https://www.vdaalst.com/publications/p400.pdf) [S](https://processmining.pads.rwth-aachen.de/old-version/fuzzy.html).

## Mathematics

**Lamport.** In *How to Write a Proof* (1993, *Amer. Math. Monthly* 1995), the high-level structure gives the outline, and readers look at as much lower-level detail as they want; the ideal reading tool would show one level and hide the ones below. He cites anecdotal evidence that up to a third of published papers contain errors. The 1993 paper says structured proofs did not seem better than conventional ones at avoiding errors; *How to Write a 21st Century Proof* (2011) says structure makes avoiding errors possible, hard work makes it probable [S](https://www.stat.cmu.edu/~brian/rm/764-2016/mathematical%20proofs/lamport-how-to-write.pdf) [S](https://microsoft.com/en-us/research/wp-content/uploads/2016/12/How-to-Write-a-21st-Century-Proof.pdf) [S](https://blog.acolyer.org/2015/01/12/how-to-write-a-21st-century-proof). TLAPS checks proofs in this hierarchical form, step by step [K].

**Leron.** "Structuring Mathematical Proofs", *Amer. Math. Monthly* 90(3), 174–185, 1983; hierarchical, aimed at communicating proofs better [S](https://scilit.net/publications/31ab4d1e7ee37a71d12069d630971dd9). That Leron's levels present the main line first and the sub-proofs afterwards is from memory [K].

**How mathematicians read.** Weber and Mejía-Ramos (2011, nine mathematicians): three strategies, line-by-line checking, modular reading, and relying on the authority of others; all nine read proofs mainly for ideas useful in their own work. A 2014 survey of 118 mathematicians largely agreed [S](https://sites.math.rutgers.edu/~jpmejia/files/Weber_Mejia_%282011ESM%29.pdf) [S](https://www.researchwithrutgers.com/en/publications/why-and-how-mathematicians-read-proofs-an-exploratory-study/). Weber (2008) studied how eight mathematicians validate proofs [S](https://sites.math.rutgers.edu/~jpmejia/files/Weber_(2008JRME).pdf).

**Proof comprehension.** Mejía-Ramos, Fuller, Weber, Rhoads and Samkoff, *Educ. Stud. Math.* 79, 2012: a seven-dimension model, including summarising via high-level ideas, identifying the modular structure and justifying claims; used to build the first validated proof-comprehension tests [S](https://link.springer.com/doi/10.1007/s10649-011-9349-7) [S](https://aaas-iuse.org/resource/assessing-the-comprehension-of-mathematical-proofs-at-the-undergraduate-level/). The split into a local and a holistic group of dimensions is from memory of the paper [K].

**Lean blueprints.** Massot's template renders labelled items as nodes of a DAG with edges from `\uses`; in Tao's PFR tour, green means fully formalized, blue ready to formalize, a green border on white means statement formalized but proof not yet; since 2024 the graph is transitively reduced and nodes whose whole dependency cone is formalized are darker [S](https://hn.jakechampion.name/reader/https://terrytao.wordpress.com/2023/11/18/formalizing-the-proof-of-pfr-in-lean4-using-blueprint-a-short-tour) [S](https://leanprover-community.github.io/archive/stream/113488-general/topic/blueprint.20changes.html).

**Thurston.** *On proof and progress in mathematics* (Bull. AMS 30, 1994) lists many ways of thinking about the derivative (infinitesimal, symbolic, logical, geometric, rate, approximation, microscopic, …) and says the list could go on [S](https://arxiv.org/pdf/math/9404236).

**Tao.** "There's more to mathematics than rigour and proofs": pre-rigorous, rigorous and post-rigorous stages; rigour should destroy bad intuition and keep good intuition [S](https://en.wikipedia.org/wiki/Mathematical_maturity).

## Engineering and design

**Y-chart.** Gajski and Kuhn (1983), refined by Walker and Thomas (1985). Behavioural, structural and geometric axes; concentric rings for system, algorithmic, register-transfer, logic and circuit levels; high-level and logic synthesis go from behaviour to structure, layout synthesis from structure to geometry [S](https://en.wikipedia.org/wiki/Gajski%E2%80%93Kuhn_chart) [S](https://texample.net/gajski-kuhn-y-chart/).

**Arcadia.** Thales, 2007; Capella is the open-source tool. Operational analysis has no concept of the system; then system need analysis, logical architecture, physical architecture [S](https://Www.wikipedia.org/wiki/Arcadia_(engineering)) [S](https://www.obeosoft.com/ressources/presentations/What-Makes-Capella-Different-SBonnet-Thales.pdf).

**Drawing sets.** Sheets are numbered by series from general to particular; a callout bubble holds the detail number over the sheet number, so the callout is an address; a callout that points to no drawing is a "dead reference", one of the most common set errors [S](https://open-exam-prep.com/study-guides/are/ch17-pdd-documentation-specifications/pdd-cross-referencing-revisions) [S](https://www.studiomatrx.org/students/drawing-fundamentals/the-drawing-set).

**A Pattern Language.** Alexander et al. 1977: 253 patterns from regions and towns to construction details; a pattern "helps to complete those larger patterns which are 'above' it, and is itself completed by those smaller patterns which are 'below' it"; a sequence of patterns is read from larger to smaller [S](https://en.wikipedia.org/wiki/A_Pattern_Language) [S](https://christiantietze.de/library/alexander1977patt/).

**Assembly instructions.** Agrawala et al., SIGGRAPH 2003: people conceive assembly hierarchically, with new objects or significant parts separating higher-level steps and actions on the same object separating finer ones (citing Zacks, Tversky and Iyer 2001); instructions generated on these principles reduced assembly time by about 35% and errors by about 50% in a follow-up study by Heiser et al. [S](https://graphics.stanford.edu/papers/assembly_instructions) [S](https://www.tc.columbia.edu/faculty/bt2158/faculty-profile/files/signingeffectivestepbystepassemblyinstructions.pdf).

## Science

**Marr, Newell, Dennett.** Marr: computational, algorithmic and implementational levels, with primacy for the computational. Newell: the knowledge level emerges from the symbol level and is governed by the principle of rationality. Dennett aligns intentional / design / physical with these [S](https://oecs.mit.edu/pub/ztivn7hx) [S](https://arxiv.org/pdf/1903.10559). The Dennett alignment was seen only in a secondhand quotation.

**Craver, Wimsatt.** Craver's mutual manipulability: a component is constitutively relevant when intervening on it changes the whole's behaviour and intervening on the whole changes it [S](https://philarchive.org/archive/CRACRA-3). Wimsatt: levels are local maxima of regularity and predictability; levels are local to a mechanism, not a universal scale [S](https://open-mind.net/epubs/levels/OEBPS/pt03.html) [S](https://mechanism.ucsd.edu/bill/research/mercier/5thlecture.pdf). Craver's view that levels are defined only within a mechanism is from *Explaining the Brain* (2007) [K].

**Simon.** Near-decomposability (1962): interactions within subsystems are stronger and more frequent than between them; in the short run each subsystem behaves almost independently; in the long run each depends on the others only in aggregate [S](https://izzortsi.github.io/weird-science/concepts/systems-theory/near-decomposability) [S](https://arxiv.org/pdf/2501.12748).

**Hierarchy theory.** Allen and Starr (1982): levels are relatively isolated and differ in time and space scale; the observer's role in defining levels is central [S](https://en.wikipedia.com/wiki/Hierarchy_theory). Salthe's basic triadic system: the focal level, the level below giving initiating conditions, the level above giving boundary conditions; the third level keeps the focal level from being reduced into either neighbour [S](https://www.cspeirce.com/menu/library/aboutcsp/queiroz/meaning.pdf).

**Noble.** No privileged level of causation; downward causation as boundary conditions on lower-level equations. Visser's critique: no universal correct level, but levels can be privileged relative to particular questions [S](https://pmc.ncbi.nlm.nih.gov/articles/PMC5566812) [S](https://www.integralworld.net/visser1410.html).

**Physics.** Renormalization, coarse-graining and effective field theories: each scale has its own degrees of freedom, and what happens below a cutoff enters only through a small number of parameters [K].

## Operations, safety and command

**Levels of war.** US doctrine: whether an action is strategic, operational or tactical depends on the nature of its objective, not on the echelon, unit size or equipment; MCDP 1-2 calls defining the operational level by echelon "erroneous"; JP 3-0 says there are no finite boundaries between the levels [S](https://gwern.net/doc/www/old.reddit.com/ac50f6d0260c0ea73815d2769178feff54123ab9.html) [S](https://www.globalsecurity.org/military/library/policy/usmc/mcdp/1-2/mcdp1-2_ch1.pdf) [S](https://smallwarsjournal.com/node/30702).

**Commander's intent.** Intent must be understood two levels down; an order includes the intent of the next higher commander (paragraph 1b in Marine Corps orders), and templates list the unit two levels up under *Situation*; the idea comes from Auftragstaktik, whose officers were trained to act two levels above their post [S](https://www.globalsecurity.org/military/library/policy/army/fm/71-123/Ch1.htm) [S](https://mca-marines.org/gazette/commanders-intent-defined/) [S](https://www.Wikipedia.org/wiki/Auftragstaktik). A 2025 Canadian Army Journal article argues the separate intent statement is redundant [S](https://www.canada.ca/en/army/services/canadian-army-journal/articles/2025/short-bursts-rolls-commanders-intent-redux.html).

**Jaques.** Time span of discretion: the longest task assigned to a role; in a requisite organisation each level has its own time-span band, and overlapping bands mean too many levels [S](https://en.wikipedia.org/wiki/Elliott_Jaques) [S](https://ninety.io/founders-framework/articles/the-stratified-systems-theory-sst-and-time-span-of-discretion-framework). Contested; sources disagree on the number of strata.

**AcciMap.** Rasmussen 1997; a multi-layer causal diagram with the outcome at the bottom and causes ordered by distance, up to regulators and government; six levels in the original, four in a formalised version [S](https://en.wikipedia.org/wiki/AcciMap_approach) [S](https://link.springer.com/article/10.1007/s10111-021-00669-w).

**STAMP.** Safety constraints, a hierarchical safety control structure and process models; each level constrains the level beneath it; the STPA handbook gives little guidance on building the structure itself [S](https://ul.com/sis/blog/introduction-to-stamp-stpa-and-cast) [S](https://acris.aalto.fi/ws/portalfiles/portal/45382519/1_s2.0_S0925753520303362_main.pdf).

**FRAM.** Used to compare work-as-imagined with work-as-done; variability between stakeholders drives the differences [S](https://www.york.ac.uk/media/assuring-autonomy/documents/Sujan%20RHC%20VI%20Chapter%2010%20pp101-106%202021%20(1).pdf) [S](https://publications.tno.nl/publication/34645043/LJbq2kcF/luijcks-2025-from.pdf).

## Explaining and teaching

**Minto.** Each point raises a *why* or *how*, answered by the points directly below; do not answer a question before it is raised; the dialogue goes on until no logical question remains; ideas in one group must be of the same kind and the same level of abstraction; groups are MECE [S](https://vnthomas1.blogspot.com/2021/12/the-minto-pyramid-principle-logic-in.html) [S](https://strategyu.co/structure-your-ideas-pyramid-principle-part-1/) [S](https://www.wikipedia.org/wiki/MECE).

**Reigeluth.** Elaboration theory: a wide-angle view of the major parts and relationships, zoom into one part, zoom back out to see how it fits; the first lesson *epitomizes* rather than summarizes; each elaboration ends with a summarizer and an expanded epitome; little empirical research [S](https://tecfa.unige.ch/themes/sa2/act-app-dos2-fic-elaborat.htm) [S](https://knowledgejump.com/hrd/learning/id/elaboration_theory.html) [S](https://core.ac.uk/download/357598756.pdf).

**Bret Victor.** *Up and Down the Ladder of Abstraction* (2011): stepping down shows one concrete run over time; stepping up abstracts over time and then over a parameter, showing all trajectories at once and then a summary plot against the parameter [S](https://nohn.bearblog.dev/victors-up-and-down-the-ladder-of-abstraction/) [K](http://worrydream.com/LadderOfAbstraction/).

**RST.** Mann and Thompson 1988: relations are mostly nucleus–satellite; the deletion test, satellites removed leaves a text that can still be understood, the reverse does not [S](https://en.wikipedia.org/wiki/Rhetorical_structure_theory) [S](https://www.ling.uni-potsdam.de/~stede/Papers/nucl08.pdf).

## Music and maps

**GTTM.** Four hierarchies (grouping, metre, time-span reduction, prolongational reduction), each governed by well-formedness rules (necessary conditions) and preference rules (which well-formed structure is heard); five grouping well-formedness rules, seven grouping preference rules, the last of which prefers the grouping that yields the more stable reduction; each time-span has a head, the structurally most important event, that represents it at the next level [S](https://en.wikipedia.org/wiki/Generative_theory_of_tonal_music) [S](https://archives.ismir.net/ismir2005/paper/000015.pdf).

**Schenker.** Foreground, middleground, background; each level prolongs the content of the one behind it; analysis removes events subordinate to larger patterns; contested as subjective [S](https://en.wikipedia.org/wiki/Structural_level) [S](https://sfcm.edu/sites/default/files/2024-04/allen_forte_on_schenker.pdf).

**Cartography.** Töpfer and Pillewizer (1966): n_f = n_a √(M_a / M_f); says how many objects to keep, not which; later generalised with a constant and an exponent per object type [S](https://www.spatial-effects.com/papers/jour/GDutton-CAGIS.pdf) [S](https://arxiv.org/pdf/1102.1561). Named generalization operators (McMaster and Shea 1992) [K].
