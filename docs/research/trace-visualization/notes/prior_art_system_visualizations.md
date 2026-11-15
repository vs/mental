# Prior art: how running systems have been shown

Notes for [the report](../report.md). Each entry says what the tool shows, how it separates data from presentation, and what Mental can take. "Not re-checked" marks things written from memory with no source fetched in this study.

## Tools that already split the recording from the presentation

**Proof Animation (Wolverine Software, 1990s).** A discrete-event simulation animator with two inputs: a *layout file* (the static geometry) and a *trace file* (a time-ordered list of commands such as CREATE, DESTROY, PLACE, MOVE, SET COLOR). It normally runs after the simulation has finished, so the simulation and the animation never compete for the machine. Any language that can write text can produce a trace, and one layout serves many scenarios ([WSC 2000](https://www.informs-sim.org/wsc00papers/028.PDF), [WSC 1996](https://www.informs-sim.org/wsc96papers/064.pdf), [WSC 1998](https://www.informs-sim.org/wsc98papers/032.PDF)).

- Take: the two-file split is thirty years old and works. Post-processing is the right default.
- Do not take: Proof's trace file is already a list of *drawing* commands. The simulation has to know how it will be drawn. Mental's trace must stay a list of facts; the drawing commands are a later, derived product.

**Rerun.** A recording holds the logged data; a *blueprint* decides how it is displayed. They are only loosely linked (by application ID), and either can be swapped. The blueprint is itself stored as time-series data in a separate store with its own timeline, so the viewer's state can be scrubbed and undone like data. Since version 0.17, styling can be set as defaults and overrides in the blueprint, so it stays out of the recording ([Blueprints](https://rerun.io/docs/concepts/blueprint), [Defaults and overrides](https://rerun.io/blog/blueprint-overrides)). The viewer is Rust on `wgpu`, using WebGPU in the browser when available and a more limited WebGL layer otherwise ([architecture](https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md)).

- Take: this is the closest living precedent for "one trace, many views". Blueprint = Mental's view state plus skin. Storing the view description as data with its own timeline is exactly what a *script* is (view state over time).
- Gap: a Rerun blueprint is a static arrangement of panels. It has no camera that moves by itself and no pacing. Nothing in Rerun decides what to look at.

**IcePanel Flows.** A flow is a named sequence of steps laid over an existing architecture diagram. The viewer steps through it and the diagram shifts focus. Several flows share one diagram and stay in sync when the diagram changes ([docs](https://docs.icepanel.io/features/flows), [comparison with sequence diagrams](https://icepanel.io/blog/comparison-icepanel-flows-vs-uml-sequence-diagrams)).

- Take: a flow refers to connections of the diagram, not to coordinates. That is why it survives layout changes. A script must refer to IDs for the same reason.
- Gap: flows are written by hand and can drift from the code.

## Tools that animate a model's run

**Runway (Ongaro, Salesforce, 2016).** One model drives specification, model checking, simulation and visualization. Each model ships with a hand-written JavaScript *view file*; together they give an interactive visualization in the browser ([introduction](https://medium.com/salesforce-open-source/dc0d9578e248), [runway-browser](https://github.com/salesforce/runway-browser), [Raft model](https://www.github.com/ongardie/runway-model-raft)).

- Take: closest in spirit to Mental. The lesson is the cost: every model needs its own view code. Mental cannot ask for that, so views must be generic over the record kinds.

**TLA+ animation module and Spectacle.** The user writes a *view expression*: a state expression that turns the spec's variables into SVG elements. One SVG frame is produced per state of the trace. Spectacle runs a TLA+ interpreter in the browser and evaluates arbitrary expressions at each state ([animation module](https://github.com/will62794/tlaplus_animation/), [interactive specs](https://will62794.github.io/formal-methods/specification/2024/12/12/interactive-formal-specs.html)).

- Take: "a view is a pure function from state to picture" is the right contract. It makes stepping backwards and seeking free.
- Gap: frame per state, no motion between states, no camera, hand-written per spec.

**Oddity (Woos et al., 2018).** A graphical step-through debugger for distributed systems. The user chooses which message is delivered next and which node fails. Time travel lets several *branching* executions be explored in one session. It works on models and on implementations, and students found it useful for exploratory testing ([paper](https://arxiv.org/pdf/1806.05300)).

- Take: this is Mental's fork requirement, working. Branches are first-class in the picture, not a separate mode.

**ShiViz.** Draws a time-space diagram of a distributed execution from logs with vector clocks. The authors argue that a single total order is misleading because it hides which events were concurrent. The user can transform the graph to drop what is irrelevant, search for patterns of events, and put two executions side by side. Evaluated with 109 students and two developers ([TOSEM 2020](https://www.cs.ubc.ca/~bestchai/papers/tosem20-shiviz.pdf)).

- Take: the time-space diagram is the one form that shows a partial order honestly. Any animated view needs it as a companion (finding F-Z7 says the same).

**Cisco Packet Tracer, simulation mode.** Time stops. Each packet is an envelope on the topology. A Capture/Forward button advances to the next event; an event list with filters sits beside the map. Clicking an envelope opens it and shows, per OSI layer, what the current device does with it on the way in and on the way out ([help](https://tutorials.ptnetacad.net/help/default/intro.htm)).

- Take: this is "follow the request" plus Mental's *layers* and *data* zoom axes, proven on generations of networking students. Event-driven time ("next event", not "next second") is the right clock.

**Process-mining animation (Disco, Apromore, processanimateR).** Each case is a token that travels over the process map according to recorded times, with a timeline scrubber. processanimateR draws tokens as SVG with SMIL animation ([processanimateR](https://github.com/bupaverse/processanimateR), [Apromore](https://documentation.apromore.org/conformancechecking/animatelogsprocessmap.html)).

- Take: many tokens on a fixed map is a third view, between "follow one request" and "town". Good for seeing where things pile up.
- Gap: no performance figures were found. SVG with one element per token will not scale.

## Tools that show a live system as a place

**Vizceral (Netflix).** A WebGL view of traffic as particles flowing between nodes, at three levels (global, region, service) with click to drill down. The README says Netflix no longer uses it and it is not maintained; the last push was in 2023 ([repository](https://github.com/Netflix/vizceral)).

- Take: particle flow gives a feeling for volume and for "something is wrong over there". It does not explain *why*. Its abandonment is a warning that the gestalt view is not enough to be useful day to day.

**Software cities (ExplorViz, DynaCity).** Packages become districts and classes become buildings; traces are drawn over them. In a controlled experiment against the trace tool EXTRAVIS, ExplorViz users needed 28% less time on the larger system and were 39% and 61% more correct on the two system sizes ([ICPC 2015 preprint](https://oceanrep.geomar.de/id/eprint/28324/1/Comparing_Trace_Visualizations_ICPC2015_preprint.pdf)). DynaCity, which draws dynamic dependencies as arcs over a city, gave 11.7% more correct answers and 5.8% less time than a traditional trace view, and was preferred ([Dashuber and Philippsen](https://cris.fau.de/publications/277227816)).

- Take: this is the only measured evidence that a town helps with traces. It does help, most on larger systems.
- Caveat: these cities are *structure* with traces laid on top. Nothing walks. The experiments do not test animated inhabitants.

**SimCity (2013) GlassBox.** The simulation is made of agents (water, power, workers) that physically travel the roads, and the stated rule was "what you see is what we sim". Agents are kept very simple so that tens of thousands can run ([GDC 2012 coverage](https://gamedeveloper.com/design/gdc-2012-breaking-down-em-simcity-em-s-glassbox-engine), [Willmott's slides](https://www.andrewwillmott.com/talks/inside-glassbox)).

- Take: the rule is the right one for a town view of a model. Every moving thing on screen must be a step or a message of the trace. Nothing moves for decoration unless it is marked as decoration.

**Pixel Agents and AI Town.** Pixel Agents shows coding agents as pixel-art characters in an office by reading their transcripts; it draws on Canvas 2D ([repository](https://github.com/pablodelucca/pixel-agents)). AI Town re-implements the Stanford "Smallville" sandbox, itself described as reminiscent of The Sims, and renders with PixiJS ([overview](https://the-decoder.com/ai-town-lets-you-build-your-own-gpt-based-ai-civilization/)).

- Take: a Sims-like view of software activity is cheap to build with 2D sprites, and people enjoy watching it. Press coverage of Pixel Agents stresses that it is watched for pleasure, not for answers.

**Isometric diagram tools (Isoflow, FossFLOW, Cloudcraft).** Static isometric architecture diagrams placed on a tile grid; Cloudcraft builds the diagram from a live cloud account ([FossFLOW](https://ostechnix.com/fossflow-create-isometric-diagrams/)). No animated request path was found in any of them.

- Take: the isometric look is established for infrastructure, and a tile grid makes layout simple. Nobody has put a model's trace on it.

## What nobody does

No surveyed tool combines all of: a trace that is a partial order of facts, several generic views of the same trace, a camera that moves by rule, and forks that can be taken. Runway and TLA+ need hand-written views. Rerun and Proof have the split but no script. Packet Tracer has the follow view but one fixed presentation. The software cities have the town but nothing moves in it.
