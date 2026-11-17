# Other Fields Keep Purpose and Parts Apart, Tie Each Level to the Question Above It, and Check the Links

**Question.** Is there a field with abstraction levels like those of software, where people build multi-level models to explain to others how a system works? If so, how do they build them? Business processes and mathematics were the first candidates. This survey looks at those two and at engineering, science, the military and safety, teaching, music and maps. For each field it asks what a level is, where a step ends, how levels are linked, how people read them and how agreement between levels is checked.

**Short answer.**

1. **Three fields are close to Mental, each in a different way.**
   - **Business process modelling** is close in subject matter: actors, cases and work steps from the value chain down to the work instruction.
   - **Mathematics** is close in the shape of its explanations: a claim at each node, proved by the level below, with some of it checked by machine (structured proofs, Lean blueprints).
   - **Hardware design** is close in technical shape: behaviour and structure are kept as separate hierarchies at matched levels, and tools translate between them.
2. **No field has one standard method, but the mature ones agree on six practices.**
   - (a) They keep a *purpose* hierarchy and a *part* hierarchy apart.
   - (b) Each level answers the *why* or *how* raised by its neighbour.
   - (c) A level is read with the level above it, which gives purpose and constraints, and the level below, which gives mechanism.
   - (d) Links between levels carry addresses or labels, and those are checked.
   - (e) Boundaries follow rules, and hard rules are kept separate from preferences.
   - (f) Names at the top come from a shared vocabulary, and the bottom levels are local.
   Mental's [multi-level design](../multi-level-models/report.md) already does (a), (b) and part of (e). It does (c), (d) and (f) only in part.
3. **Business processes: copy the rules at the links, not the level numbers.** Practice names five or six levels (L0–L5, APQC, SCOR, the ISO 9001 pyramid), but no two sources define the numbers the same way. Two things work well:
   - **Labels shared across levels.** In Silver's BPMN style, a subprocess names its end states and the parent's gateway tests exactly those names.
   - **Matching arrows.** IDEF0 requires the arrows of a box to reappear at the edge of its child diagram.
   Studies of process models also measure a cost of hierarchy: separate child diagrams split the reader's attention. This supports Mental's choice of mixed cuts over separate diagrams per level.
4. **Mathematics: the best precedent for an explanation that is also checked.** A Lamport proof is a step tree: each step states a claim, and its proof is the level below it. A reader stops at any depth. A Lean blueprint links each informal statement, written for people, to its formal counterpart and colours it by status. That is the relation Mental wants between an LLM-written name or charter and a computed, checked step class. Mathematics also shows a limit. Its knowledge is a dependency graph, and levels exist only inside one proof. Philosophers of science say the same of mechanisms. Levels are local, which fits Mental's step tree per run.
5. **Nine proposals follow (section 10).** Four can be tested on rq with the existing harness:
   - choosing between valid closing rules by a preference rule;
   - naming each step from its head effect;
   - an object-centric outcome level;
   - a test for levels with overlapping step lengths.
   Five are view or check changes to make while building:
   - show two levels up, as military orders do;
   - check boundary labels across levels;
   - add a status map in the style of a blueprint;
   - open on an epitome and zoom back out after descending;
   - add a view of one step class across many runs.

The sources were read in abstracts, excerpts and secondary summaries only. No field's results have been tested on program traces. Section 11 separates what is evidence from what is inference.

**Notes:** [sources, with evidence tags](notes/sources.md). **Builds on:** [multi-level models](../multi-level-models/report.md) (closing rules, charters, cuts, the rq experiment), [abstraction levels](../abstraction-levels/report.md) (how levels are related and checked in formal methods and tools).

---

## 1. Each source in one line

Sources already in the multi-level models report (Rasmussen, Cockburn, hierarchical task analysis, EventStorming, Domain Storytelling, APQC, ArchiMate, C4, event segmentation) are not repeated.

| Field | Source | How it builds levels | Take for Mental |
|---|---|---|---|
| Business | Process architecture L0–L5 | Value chain → process group → process → activity → task or instruction; numbering differs by vendor | Level numbers are not a standard. The rules at the links carry the meaning. |
| Business | SCOR | Levels 1–3 shared across industry; level 4 left to each company | Standard words at the top, local words at the bottom. |
| Business | ISO 9001 pyramid | Policy, process, procedure, work instruction, record | "A hand-off between functions is a procedure; one task at one station is a work instruction" is Mental's stage/goal test. |
| Business | BPMN Method and Style | Child end events name end states; parent gateways test exactly those names | Name each step class's cases as end states and check that the labels match across levels. |
| Business | IDEF0 | 3–6 boxes per diagram; a box's arrows reappear at its child's edge; purpose and viewpoint stated | A boundary check for upper levels that are written by hand or documented. |
| Business | 7PMG | Guidelines backed by studies: few elements, verb-object labels, decompose large models | Supports verb-first names and small cuts. |
| Business | Modularization studies | Subprocesses help by abstraction and hurt by fragmentation | Hierarchy costs attention. Expand in place (a cut) rather than in a separate diagram. |
| Business | Capability vs process maps | *What* the business can do vs *how* it does it; mapped, not one hierarchy | Purpose and part (or capability) are separate axes, as in Rasmussen. |
| Business | Object-centric process mining | An event belongs to several objects; one case notion causes convergence and divergence | The precedent and the terms for Mental's overlapping outcomes. |
| Business | Fuzzy Miner | Keep significant activities, cluster correlated minor ones, drop the rest | A map metaphor for process logs. Close to Mental's support steps. |
| Mathematics | Lamport, Leron | Hierarchical proofs: each step a claim proved by its sub-steps; read to any depth | The step tree with a claim at every node. |
| Mathematics | Weber and Mejía-Ramos | Mathematicians read line by line, modularly, or on authority | Modular reading is the norm for experts, and authority stands in for checking. |
| Mathematics | Proof comprehension model | Understanding is tested on details and on high-level ideas and modular structure | A template for reader questions in a study with people. |
| Mathematics | Lean blueprints | Dependency graph of informal statements linked to formal ones, coloured by status | Human prose bound one-to-one to checked objects, with a status map. |
| Mathematics | Thurston | Many views of one idea, not one definition | Views are not levels. Mental needs both. |
| Mathematics | Tao | Readers go through pre-rigorous, rigorous and post-rigorous stages | A reader's home level moves as they learn. |
| Engineering | Gajski–Kuhn Y-chart | Behaviour, structure and geometry, each with system, algorithmic, register-transfer, logic and circuit levels | The cleanest statement of separate axes at matched levels. |
| Engineering | Arcadia / Capella | Operational analysis (no system), system need, logical, physical | The top is "pure": no system in it. |
| Engineering | Drawing sets | Whole to detail by scale; every callout carries a sheet address; a dead reference is a known error | Every link between levels is an address that can be checked. |
| Engineering | A Pattern Language | Each pattern names the larger patterns it completes and the smaller ones that complete it | Why and how links written into every element. |
| Engineering | Assembly instructions | Higher steps separated by new parts, finer ones by actions on one part; 35% faster, 50% fewer errors | Event segmentation used in practice, with a measured gain. |
| Science | Marr, Newell, Dennett | What and why / how as a procedure / how physically | Levels as kinds of question. The top has primacy. |
| Science | Craver, Wimsatt | Levels are local to a mechanism; levels are peaks of regularity and predictability | Levels are per subject and per run. Admit a level where the upper description predicts well. |
| Science | Simon | Near-decomposability: dense, fast interaction inside, sparse, slow between | Boundaries fall where interaction thins out, which is what Mental's wait rule finds. |
| Science | Hierarchy theory, Salthe | Focal level, with initiating conditions from below and boundary conditions from above | Read a level with both neighbours. Three levels on screen, not one. |
| Science | Noble and his critic | No privileged level overall; levels are privileged per question | Same as Mental's R1 and R5. |
| Command | Levels of war | Strategic, operational, tactical by purpose, not unit size | Strong outside support for "levels by purpose; containment is space zoom". |
| Command | Commander's intent | Each order restates the intent one and two levels up | Show the parent's and the grandparent's goal with every step. |
| Organisations | Jaques | Levels by time span; overlapping spans mean a level too many | An alternative merge test. |
| Safety | AcciMap | One causal diagram across levels, from operators up to government | Causes can sit *above* a symptom (a decision, a setting), not only below. |
| Safety | STAMP | Each controller constrains the one below and gets feedback | Control is a third relation between levels, beside purpose and part. |
| Safety | FRAM | Compare work as written with work as done | Already in Mental's processes around the software (5.9). |
| Teaching | Minto | Each point raises why or how, answered below; one level of abstraction per group | Mental's R1 and R12, in prose. |
| Teaching | Reigeluth | Epitome first, zoom in, zoom back out with an expanded epitome | Open on a simple whole and re-place the reader after each descent. |
| Teaching | Bret Victor | Up the ladder is abstracting over a variable: all runs at once | A third direction: one run ↔ the set of explored runs. |
| Teaching | RST | Nucleus and satellite; deleting satellites keeps the text coherent | Support steps are satellites. The deletion test is what makes them foldable. |
| Music | GTTM | Well-formedness rules plus preference rules; each group has a head | Separate hard closing rules from preferences. Name a group from its head. |
| Music | Schenker | Foreground, middleground, background; each prolongs the one behind | Reduction by removing subordinate events, read from the top. |
| Maps | Töpfer's radical law; generalization operators | How many objects survive a change of scale; named operations to coarsen | A quantity rule for grain, and a vocabulary of folding operations. |

## 2. What was asked of each field

The multi-level models report settled Mental's own terms: a **level** has a charter (readers, question, what a step is), a **closing rule** says where a step ends, a **step class** is a kind of upper step, **support steps** are off the causal chain, a **cut** mixes levels, and a reader has a **home level**. For each field we looked for five things:

| | Mental's term | What we looked for |
|---|---|---|
| 1 | level, charter | What makes something a level, and who it is for |
| 2 | closing rule | Where one step or element ends |
| 3 | step map, why and how links | How an element is tied to the levels above and below |
| 4 | cut, home level | How people actually read the levels |
| 5 | checks | How agreement between levels is established |

## 3. Business processes

Business process modelling is the field with the most explicit levels of work above software, and the one where most organisations already have names. It is the closest match for Mental's outcome, stage and goal levels.

### 3.1 What practice calls a level

| Source | Levels | Defined by |
|---|---|---|
| Common process architecture | L0 value chain, L1 process category, L2 process group, L3 process, L4 activity, L5 task or work instruction | position in a tree; definitions differ by vendor |
| APQC | category, process group, process, activity, task | a reference catalogue; mutually exclusive and exhaustive; order does not mean execution order |
| SCOR | 1 scope (plan, source, make, deliver, return, enable), 2 configuration, 3 process elements, 4 company-specific | industry-neutral down to level 3; level 4 deliberately outside the standard |
| ISO 9001 pyramid | policy, process, procedure, work instruction, record | what a document describes: a hand-off between functions (procedure) or one task at one station (work instruction) |
| Capability maps | capability, sub-capability | *what* the business is able to do, named by nouns; mapped to processes, not part of their tree |

Three observations.

- **Level numbers carry little meaning.** "L3" means different things in different tools. The definitions that do carry meaning are the ones that say what a step is. The best of them is the ISO practitioners' test: if a document names a hand-off between two functions it is a procedure, and if it names one task at one station it is a work instruction. That is Mental's distinction between a *stage* (the subject changes hands) and a *goal* (one actor gets one thing done), arrived at independently.
- **Standard at the top, local at the bottom.** SCOR stops at level 3 on purpose, because level 4 cannot be standardised. Consultants advise taking L1–L3 names from APQC and writing the lower levels locally. Mental's skeleton is general at every level, and its project-specific levels come from the project itself. For the upper levels, the report's section 5.9 already says to reuse an organisation's framework names where it has one. This survey adds that this is where practice expects them.
- **What and how are kept apart.** Business architects keep capability maps (what) and process maps (how) as separate models that are mapped, not nested. This is Rasmussen's means-ends versus part-whole distinction again, from a field that does not cite him.

### 3.2 Rules at the links between levels

Two methods put their rigour into the link between a level and its children rather than into the level numbers.

**BPMN Method and Style** (Silver). A process is modelled top-down: one top-level diagram, with collapsed subprocesses that each have a child diagram. Each subprocess with more than one outcome has one end event per *end state*, labelled with that state ("report approved", "report rejected"). In the parent, the gateway right after the subprocess has gates with exactly those labels. The matching labels are what make the hierarchy traceable from the top. Silver also insists that one process diagram is about one kind of instance, an order or a claim; a batch of orders is a different process [K].

For Mental, the end state is the upper-level *case* of a step class. The multi-level models report says "a step class can have several possible expansions … these are its cases at the upper level". Silver's rule turns this into a check: the upper level may branch only on the named end states of its children, and every end state must be handled.

**IDEF0** (FIPS 183). Every model opens with a context diagram, A-0: one box, its arrows, and a stated *purpose* and *viewpoint*. Every child diagram has three to six boxes. The arrows into and out of a box (inputs, controls, outputs, mechanisms: ICOM) must reappear at the boundary of its child diagram, so nothing enters or leaves a function without being accounted for one level down.

For Mental, the purpose and viewpoint are the charter's question and readers. The ICOM rule is a boundary check. For levels Mental computes, it holds by construction: an upper step's footprint is the union of its children's. It matters for levels that are written or documented: a runbook stage, a stage grouped by the LLM, a charter's claim about what a level hides. There, "what enters and leaves this upper step" can be compared mechanically with the footprint of its expansion. The name-truth check of the earlier report is the same idea applied to a name.

### 3.3 What the comprehension research found

- **7PMG** (Mendling, Reijers, van der Aalst 2010) condenses studies of errors and comprehension into seven guidelines. Two bear on Mental: label activities verb-object, and decompose a model above roughly 50 elements [K]. The first agrees with the verb-first rule of the `name-step` task.
- **Modularization has a cost.** Zugal, Pinggera, Weber, Mendling and Reijers name two opposing effects of putting parts of a model into subprocesses: *abstraction* helps (information hiding, recognising patterns), and *fragmentation* hurts (attention split between diagrams). A 2024 study confirmed both at coarse and fine grain. Turetken and colleagues measured higher mental effort from fragmentation. Reviews say the results do not yet add up to validated guidelines.

This is the most direct outside evidence for a design choice Mental has already made. The trial viewer switched the whole view from one level to another (F-Z9), which is fragmentation in its pure form. A cut opens one step in place and keeps its context on the same screen, so it keeps the abstraction and avoids most of the fragmentation. The cost cannot be removed entirely. A reader who descends far still loses the overview, which is what section 8's "re-place" proposal addresses.

### 3.4 Cases that are not a tree

Mental's rq experiment found that 4.4% of goals change two jobs and so belong to two outcomes: "the step map above the goal is a relation, not a partition". Process mining met exactly this problem and named it. In an object-centric event log (OCEL), an event relates to several typed objects. Forcing one case notion causes *convergence*, where one event is copied into several cases, and *divergence*, where independent repetitions inside a case look related. The field's answer is to keep events linked to objects and derive each object's view when it is needed, never to pick one case type.

For Mental, the outcome level should be object-centric in this sense. Every goal lists the subjects it changes. An outcome is the per-subject projection, and a goal under two subjects is shown under both, marked as shared. Convergence and divergence are useful diagnostics for a candidate subject: if choosing *job* as the subject makes many goals appear under several outcomes, the subject may be wrong or a second one is missing.

### 3.5 What business processes do not give

The levels describe *work as imagined*, and the models are rarely checked against what happens. Process mining is the exception, and it works bottom-up from logs. It has no equivalent of Mental's interpreter that explores runs. The field also gives little help with the lower levels: below the work instruction there is nothing, and that is where Mental's activation, move and effect levels live.

## 4. Mathematics

### 4.1 Structured proofs

Lamport (1993, and again in 2011) proposes writing a proof as a hierarchy of numbered steps. Each step states a claim, and its proof is either a short justification or a list of lower-level steps that ends by deriving the claim. "The high-level structure provides the general outline; readers can look at as much or as little of the lower-level detail as they want." He imagines a reading tool that shows one level and hides the ones below. Leron (1983) proposed a similar hierarchy for communicating proofs. TLAPS checks proofs in this form step by step [K].

The correspondence with Mental's step tree is close:

| Structured proof | Mental |
|---|---|
| a step states a claim | a step class has a name and a "so that" sentence |
| its proof is the list of sub-steps | its expansion is the list of lower step classes |
| the step is checked from its sub-steps | the upper step is checked against its expansion (pair check) |
| a reader expands only the steps they doubt | a cut opens only the steps with a finding |
| a leaf is a justification by a known fact | a leaf is an effect fired by an interpreter rule |

Lamport's own experience adds a warning. In 1993 he wrote that structured proofs did not seem to avoid errors better than ordinary ones; in 2011 he wrote that structure makes avoiding errors *possible* and hard work makes it *probable*. Structure gives the reader a place to look. It does not do the checking.

### 4.2 How mathematicians read

Weber and Mejía-Ramos interviewed nine mathematicians, and a later survey of 118 mostly agreed with them. They read proofs in three ways: line by line, *modularly* (grasping the high-level structure and the role of each part), or by relying on the authority of colleagues who checked it. All nine read mainly for ideas useful in their own work, not to check correctness.

The proof comprehension model of Mejía-Ramos and colleagues (2012) turns this into tests. One group of dimensions is about details: the meaning of terms, the logical status of statements, the justification of each claim. The other is holistic: summarising the high-level ideas, identifying the modular structure, transferring the method, illustrating with an example [K for the grouping]. It produced the first validated tests of proof comprehension.

For Mental, two points.

- **Authority is a reading strategy.** Experts accept a part they did not read because someone trusted has checked it. This is what Mental's provenance and verdicts are for: a reader skips a step marked as checked, the way a mathematician skips a lemma from a refereed paper.
- **A ready template for the study with people.** The questions of the earlier E3 experiment were all of the "what happened" kind. The holistic dimensions add question types that were missing: summarise this run in one sentence; which part is responsible for X; would this step still work if Y changed.

### 4.3 Lean blueprints

A blueprint, after Massot's template, is used in large formalisation projects such as Tao's PFR. It is a document in ordinary mathematical prose in which every definition and lemma is a node of a dependency graph. Each node is linked to its formal counterpart in Lean, and the graph is coloured by status:

- stated;
- ready to formalise;
- statement formalised;
- proof formalised;
- in the newer versions, also a darker shade when the node's whole dependency cone is formalised.

This is the closest precedent found for what Mental needs between people and the model:

| Blueprint | Mental |
|---|---|
| informal statement written for people | LLM-written name, "so that" sentence, charter |
| linked one-to-one to a formal declaration | bound to a computed step class |
| status per node, coloured | provenance (`inferred`, `attested`, `assumed`) and check verdicts per step class |
| graph reduced and shaded by cone status | a map of which step classes, and everything below them, pass their checks |

Proposal N6 in section 10 takes this over.

### 4.4 What mathematics does not give

Mathematical knowledge is a dependency graph, not a ladder. A theorem rests on lemmas and definitions, and those are shared between many theorems. Levels exist only inside one proof and are local to it. Thurston's essay makes a related point: the derivative is understood through many views (infinitesimal, symbolic, geometric, as a rate, as an approximation …), and none of them is a level of another. Tao's stages (pre-rigorous, rigorous, post-rigorous) are stages of the *reader*, not of the object.

For Mental this confirms two things the design already assumes. Levels belong to a step tree per run, generalised into step classes, not to the whole system at once. And views (stage, script, skin) are a separate dimension from levels.

## 5. Engineering and design

**The Y-chart.** Gajski and Kuhn (1983) drew hardware design as three axes: behaviour, structure and geometry. Each has levels at the same heights:

| Level | Behaviour | Structure |
|---|---|---|
| System | systems | processors |
| Algorithmic | algorithms | ALUs, RAM |
| Register transfer | register transfers | registers |
| Logic | Boolean equations | gates, flip-flops |
| Circuit | transfer functions | transistors |

Synthesis moves *across* axes at one level (behaviour to structure), and refinement moves *inward* along an axis. This is the clearest statement in any field that "what it does" and "what it is made of" are two hierarchies with matching levels, not one ladder. It is Mental's two axes, purpose (time zoom) and part (space zoom), with a third axis, geometry, that corresponds to Mental's physical facet.

**Arcadia** (Thales, the method behind Capella). Its first perspective, operational analysis, describes what users must do and contains no system at all. Then come system need, logical architecture and physical architecture. Domain Storytelling's *pure* stories and Mental's rule R10 (levels above the software use no software words) are the same idea.

**Drawing sets.** A set of architectural drawings goes from the whole building at a small scale to details at a large one. Every callout bubble carries two numbers, the detail and the sheet it is drawn on, so a callout is an address. A callout that points to nothing is a *dead reference*, one of the most common errors in a set, and checking for it is routine.

For Mental: every "how" link from a step must resolve to an expansion, and every "why" link to a parent. Coverage (A6) already says this. Drawing practice shows it is the first thing checked, not the last.

**A Pattern Language** (Alexander et al., 1977). The 253 patterns run from regions to construction details. Each one names the larger patterns it "helps to complete" and the smaller ones by which "it is itself completed", and the sequence is read from larger to smaller. Every element carries its own why (above) and how (below). This is the charter's `why` and `how` written into each element rather than only into each level.

**Assembly instructions.** Agrawala and colleagues (SIGGRAPH 2003) found that people conceive assembly hierarchically. A new part or a significant component starts a higher-level step, and different actions on the same part separate the finer ones; they cite Zacks and Tversky's event segmentation. Instructions generated on these principles cut assembly time by about 35% and errors by about 50% in a follow-up study. This is the strongest measured gain found for cutting steps where perception cuts them. It is evidence for R3 from a practical task, though from assembly, not from program traces.

## 6. Science: what makes something a level

Several fields have argued about what a level *is*. Their answers are useful as admission tests.

| Answer | Source | As a test for Mental |
|---|---|---|
| A level is a kind of question: what and why, how as a procedure, how physically | Marr; Newell; Dennett | A1: a level states its question. Marr's primacy of the top matches R6: a finding is shown at the highest level that can name it. |
| Levels exist relative to one mechanism, not globally | Craver; Wimsatt | Levels are per subject and per run, generalised to step classes. There is no universal ladder for all systems. |
| A level is where regularity and predictability peak | Wimsatt | A5: an admitted level must predict its own next steps (the *det* measure). A level between two peaks is a valley and should be merged. |
| Boundaries fall where interaction thins: dense and fast inside, sparse and slow between | Simon's near-decomposability | The activation rule ("closes when it must wait") is a near-decomposability boundary: inside an activation there is no interleaving; between activations there is. |
| A focal level is explained by initiating conditions below and boundary conditions above | Salthe; hierarchy theory | A level is understood with both neighbours visible. One level alone cannot be read (proposal N1). |
| No privileged level overall, only per question | Noble, and the reply to him | R1 and R5: home levels per role, not one true level. |

Physics adds one idea that no other field states so sharply [K]. In a coarse-grained or effective theory, what happens below the cutoff enters the upper level only through a few parameters. For Mental, the upper level should depend on the lower one only through the abstraction function's observables. The barrier check enforces this on vocabulary. It could also be applied to state: an upper step's behaviour should be predictable from upper state alone, which again is the *det* measure.

## 7. Command, organisations and safety

**Levels of war are defined by purpose.** US doctrine holds that whether an action is strategic, operational or tactical depends on the nature of its objective, not on the echelon, the size of the unit or its equipment. The Marine Corps calls defining the operational level by echelon "erroneous". A squad can act at the strategic level. This is the best outside support found for R2: levels move along purpose, and containment (the size of the box) is space zoom. It comes from a field that has argued about the question for decades.

**Commander's intent, two levels up.** An order restates the intent of the next higher commander, and leaders are expected to understand intent two levels up, so a subordinate can act sensibly when the plan breaks. Auftragstaktik trained officers to work two levels above their post. A 2025 Canadian article argues the separate intent paragraph is redundant because the rest of the order already contains it. That critique concerns where intent is written down, not whether subordinates need it.

For Mental, a step shown alone loses its purpose. Showing the parent's goal sentence (the why link) is one level. Doctrine says two: the parent says what this step is for, and the grandparent says what the parent is for. Only with both can a reader judge whether a different way of reaching the parent would have been fine. This matters most for the coding agent, which acts on a step and should know what must not break above it.

**Jaques: levels by time span.** In Requisite Organization each managerial level has its own band of *time span*: the longest task a role is given. When two levels' bands overlap, there is a level too many. The theory is contested, but the test transfers. For Mental, measure each level's step lengths in scheduler steps or wall time. If two adjacent levels' distributions overlap heavily, one restates the other. This is a second form of the revised merge test (in the earlier report, four in five upper steps having exactly one lower step) and can be compared with it on rq.

**AcciMap: causes can be above.** Rasmussen's AcciMap draws an accident as one causal diagram across levels: equipment and operators at the bottom, then management, company, regulators and government. Causes are ordered by their distance from the outcome. Mental's E4 looked only *down* from a symptom for its cause. In a software system a cause can also sit higher than the symptom: a requirement that never said what should happen on retry, a deployment setting, a decision recorded in an issue. An explanation of a finding should allow a causal chain that crosses levels in both directions.

**STAMP: control as a third relation.** Leveson models a system as a hierarchy of controllers: each constrains the level below and receives feedback from it. This is neither purpose nor containment. A supervisor that restarts workers, a scheduler that requeues jobs and an operator who changes a setting are controllers. Mental's model has no explicit place for this relation. It is noted here, not proposed.

## 8. Explaining and teaching; music and maps

### 8.1 How to tell a multi-level story

**Minto's pyramid.** Each point raises a *why* or *how* in the reader's mind, and the points directly below answer it. Do not answer a question before it is raised. Keep going until no logical question remains. Ideas in one group share a single level of abstraction and are mutually exclusive and collectively exhaustive. This is the why and how rule (R1, R12) as a writing rule. It also gives a test for the `narrate` task: every sentence in a narrated cut should answer the question raised by the line above it.

**Reigeluth's elaboration theory.** Teach like a zoom lens. Start with an *epitome*: a simple but complete whole that shows the major parts and their relations. Reigeluth stresses that it epitomises; it does not summarise or abstract. Then zoom into one part, and zoom back out to show where it fits, with an expanded epitome. Empirical support is thin.

For Mental, the home level is the epitome. What the current design lacks is the *zoom back out*: after a descent to the cause, the view should return to the opening level with the visited path marked, the expanded epitome.

**Bret Victor's ladder of abstraction.** Going *down* shows one concrete run. Going *up* abstracts over time and then over a parameter: all trajectories at once, then a summary plot against the parameter. Understanding comes from moving between the two. This is a direction Mental's design does not name. Purpose and part both stay within one run. Mental already explores thousands of runs, and the step class is the object that exists across them. A view of one step class over all explored runs would show which cases occurred, how often, and under which interleavings. It is Victor's "up" applied to traces.

**Rhetorical Structure Theory.** A text is a tree of relations, most of them between a *nucleus* and a *satellite*. Delete the satellites and the text still makes sense; delete the nuclei and it does not. This is the property that makes Mental's support steps safe to fold: they are satellites of the causal chain. Mental computes them from footprints, which is a mechanical version of the deletion test.

### 8.2 Music: rules for hearing groups

**GTTM** (Lerdahl and Jackendoff 1983) is the most rigorous account found of how people group a stream of events into a hierarchy. It has two kinds of rule:

- **Well-formedness rules** say what a valid grouping is: groups nest, they do not overlap, they cover the piece.
- **Preference rules** say which of the valid groupings a listener hears: boundaries at gaps and changes, groups of similar length, parallel passages grouped alike. The last preference rule favours the grouping that gives the more stable reduction.

Each group has a **head**, its most important event, which stands for it at the next level up.

Two lessons for Mental.

- **Separate the two kinds of rule.** Mental's closing rules are written as if each level had one right cut. The rq experiment found a real ambiguity at the move level: close at every write, or at the end of an atomic group. Both are well-formed. GTTM's answer is a preference rule chosen by a global criterion. The analogue of "the more stable reduction" is the grouping that makes the upper level more deterministic (the *det* measure) or that matches step classes across versions better (E6). This can be tested (proposal N3).
- **Name a group from its head.** In each step class, one effect changes the level's observable or closes the step. That effect is the head. Giving the head explicitly to the `name-step` task, and checking the name against it, may raise the low agreement of move names (22% identical) without letting the LLM regroup anything (proposal N4).

**Schenkerian analysis** reads a piece at foreground, middleground and background. Each level *prolongs* the one behind it, and analysis removes events subordinate to larger ones. It is contested as subjective. Its lesson is the same as GTTM's from the other side: without stated rules, reductions differ between analysts. That is what Mental's free-form LLM ladder did (design C).

### 8.3 Maps

Cartographers change scale with named operations (selection, simplification, aggregation, displacement, exaggeration) [K]. Töpfer's radical law gives how many objects survive: the count falls with the square root of the scale ratio. It explicitly says *how many*, not *which*. This parallels the split in Mental: a count or band can say whether a level is too crowded (A3), and the closing rules and degree of interest decide which steps appear. The Fuzzy Miner carried the map metaphor into process mining. It keeps significant activities, aggregates correlated minor ones into clusters and drops the rest. These correspond to Mental's open steps, folded support steps with a count, and closed steps.

## 9. What recurs

Practices found in at least three unrelated fields, set against Mental's current design.

| # | Practice | Fields | Mental today | Gap |
|---|---|---|---|---|
| **C1** | Keep a purpose hierarchy and a part hierarchy apart, at matched levels | hardware (Y-chart), business (capability vs process), military (purpose not size), cognitive science (Marr), process control (Rasmussen) | R2: levels on the purpose axis, containment as space zoom | none; strong outside support |
| **C2** | Each level answers the why or how raised by its neighbour | Minto, Lamport, Alexander, Cockburn, commander's intent | R1, R12, the charter's `why` and `how` | none |
| **C3** | Read a level with the level above (purpose) and below (mechanism) on view | Salthe, commander's intent (two up), Alexander, Reigeluth | the why link shows the parent | grandparent goal; zoom back out (N1, N8) |
| **C4** | Links between levels are labels or addresses, and they are checked | BPMN end states, IDEF0 arrows, drawing callouts, Lean blueprint links | coverage (A6), pair check | label consistency for end states; boundary check for written levels (N2) |
| **C5** | Boundaries follow rules; hard rules and preferences are separate | GTTM, event segmentation, assembly instructions, Simon | closing rules, one per level kind | no preference rules; the move-level ambiguity is unresolved (N3) |
| **C6** | A group is represented by its most important member | GTTM head, RST nucleus, Lamport step claim | the LLM names a step from its children | no explicit head (N4) |
| **C7** | Cases or subjects can overlap; do not force one case notion | object-centric process mining, BPMN instance rule | outcomes overlap in 4.4% of goals, noted | no model of shared goals (N5) |
| **C8** | Status is shown per element, across the hierarchy | Lean blueprint, drawing revisions, process maturity | provenance and verdicts per record | no map view (N6) |
| **C9** | A level is admitted where the upper description works on its own | Wimsatt, Simon, physics (effective theories), Jaques (separate time spans) | A5 (*det*), revised A3 | span-overlap test not tried (N7) |
| **C10** | A shared vocabulary at the top, local words at the bottom | SCOR, APQC practice, Arcadia | 5.9 suggests reusing framework names | none for now |
| **C11** | Hierarchy has a cost in split attention | process model studies, Reigeluth | cuts, not level switching | evidence only, no change |
| **C12** | Across instances is its own direction | Bret Victor, process mining (maps over many cases) | step classes exist across runs; views show one run | no run-set view (N9) |

## 10. Proposals for Mental

Following the experimenting phase, each proposal is an alternative to test, not a decision. "Runnable" means the existing rq harness and data can test it.

| # | Proposal | From | Checkable as | Runnable now |
|---|---|---|---|---|
| **N1** | Every focused step carries the goal sentences of its parent and grandparent; the agent's focus includes both | commander's intent; Salthe | the focus record has both; reader study: can the reader say what must not change | view change; no test yet |
| **N2** | Name each step class's cases as end states; an upper level may branch only on its children's end states, and every end state is handled. For written or documented levels, compare what the level says enters and leaves a step with its expansion's footprint | BPMN Method and Style; IDEF0 | label check over step classes; boundary check on attested and assumed steps | partly: end-state labels on rq's move and activation classes |
| **N3** | Split closing rules into well-formedness (nesting, coverage, one cut per run) and preference. Where several valid cuts exist, prefer the one that makes the upper level more deterministic or more stable across versions | GTTM; Wimsatt | on rq, compare *close at every write* and *close at the end of an atomic group* by *det*, E6 stability and stand-in reader lines | yes |
| **N4** | Each step class has a head: the effect that changes the level's observable or closes it. `name-step` gets the head, and name truth is checked against it | GTTM head; RST nucleus | naming agreement for moves against the 22% and 0.51 of section 8.4 of the earlier report; name-truth failures | yes |
| **N5** | Outcomes are object-centric: a goal lists every subject it changes and appears under each, marked shared; report convergence and divergence per candidate subject | OCEL | the 4.4% shared goals shown under both jobs; divergence as a check on subject choice | yes |
| **N6** | A status map: the graph of step classes, coloured by which checks pass, with a darker shade where everything below also passes | Lean blueprints | a view over existing verdicts | view change |
| **N7** | A second merge test: merge adjacent levels whose step lengths overlap heavily | Jaques; Simon (separate time scales) | on rq, compare with the revised A3 (four in five with one lower step): do they merge the same pairs? | yes |
| **N8** | Open on the home level as an epitome; after any descent, offer *zoom back out*, which returns to the opening cut with the visited path marked | Reigeluth | view behaviour; reader study | view change |
| **N9** | A run-set view of one step class: its cases across all explored runs, how often, and under which interleavings | Bret Victor | view over existing explored runs | view change; data exists |

**Not proposed, noted.** Causes above the symptom (AcciMap) need levels above the software that carry decisions and settings, which rq does not have. Control as a third relation between levels (STAMP) would need a new kind of record.

**Suggested order.** N3, N4, N5 and N7 together on rq, because they reuse `mlexp/levels.py` and the naming sheet and cost one more set of LLM calls for N4. N1, N6, N8 and N9 belong to the viewer and wait for it. N2 needs documented processes (E5 of the earlier report) to matter, and rq has none.

## 11. What is evidence and what is inference

**Evidence, from others, as seen this session.** The level schemes and rules of each field (SCOR, ISO practice, BPMN style, IDEF0, Y-chart, Arcadia, drawing sets, A Pattern Language, GTTM, doctrine on levels of war and intent, Jaques, AcciMap, STAMP, Lean blueprints, Lamport). These are descriptions of practice, seen in summaries, and are reliable as such. Measured results are fewer:

- the assembly instruction study (about 35% less time, 50% fewer errors);
- the process modularization studies (abstraction helps, fragmentation hurts, results mixed);
- the studies of how mathematicians read proofs (nine interviews, a survey of 118);
- the proof comprehension tests.

**Background knowledge, not re-verified.** The specific 7PMG guideline wording, Silver's one-instance rule, the local and holistic grouping of the proof comprehension model, cartographic generalization operators and the physics of coarse-graining. They are marked [K] in the notes.

**Inference.** Everything in sections 9 and 10. That a practice recurs in several fields is evidence that it suits people. It is not evidence that it suits program traces or that Mental's version of it works.

**Weak points.**

- **Summaries only.** No full text was read this session. Several sources disagree in details (the number of AcciMap levels, the number of Jaques's strata, SCOR's level count), which the notes record.
- **Transfer.** None of the measured results were on software, let alone traces. The assembly study is the closest, because it measures exactly the rule Mental uses (cut where the object changes), but on physical assembly.
- **Selection.** Fields were chosen for having multi-level models, so the survey cannot show that levels are the wrong tool. The modularization studies are the one source that measures a cost.
- **The business levels describe intent, not runs.** The closest field in subject matter checks its models least, so its rules are about notation and consistency, not about agreement with behaviour.
- **Mathematics is a graph.** The structured proof is a close analogue of one run's step tree. The blueprint is a close analogue of the step-class graph. Neither says anything about interleaving, which is where Mental's hardest levels (activation, move) are.

## 12. Sources

All outside sources, with links and evidence tags, are in [notes/sources.md](notes/sources.md). Mental's own: the [multi-level models report](../multi-level-models/report.md) and its [walkthrough](../multi-level-models/ladder-walkthrough.md), the [abstraction levels report](../abstraction-levels/report.md), the [specification](../../specification.md), the [glossary](../../glossary.md) and the [findings](../../findings.md) (section Z).
