# One Trace, Many Films: Stage, Script and Skin

**Question.** A run produces one trace. How should it be turned into something to watch, so that the same trace can be seen in very different ways (the camera on one request; the whole system as a town), how can those visual scripts be generated, and how can they be drawn in a browser without ever dropping a frame?

**Short answer.**

1. The split the question asks for is real and has precedent. Between the trace and the pixels there are **three** independent things, not one: the **stage** (where things are), the **script** (what to look at, when, and how fast) and the **skin** (how things look). A "Sims town" is a stage. "Camera on the request" is a script. Neither is a skin.
2. A script should be **data that refers to IDs and intents, never to coordinates**. Then any script plays on any stage in any skin.
3. Scripts can be generated three ways, all of them local and cheap: by fixed **idioms**, by an **interest score** computed from model facts, and by the **coding agent** for a specific question. They should be tried side by side.
4. The scene is small, so drawing speed is not the hard part. Performance comes from making **the picture a pure function of (trace, stage, script, time)**, uploading movements to the GPU once, and keeping layout and text out of the frame loop.
5. Research on animation is sobering: watching does not produce understanding; stepping, asking and comparing do. Every moving view needs a still companion, and a town can be *sensed* but not *followed*.

Everything under "Proposal" is inference from the evidence in the four notes. Nothing has been built or measured.

**Notes:** [prior art](notes/prior_art_system_visualizations.md) · [generating a script](notes/script_generation.md) · [rendering performance](notes/web_rendering_performance.md) · [perception and clarity](notes/perception_and_clarity.md)

---

## In brief: what each product or study contributes

| Product or study | What it is | How it handles the question | Take for Mental |
|---|---|---|---|
| Proof Animation | simulation animator, 1990s | layout file + trace file, played after the run | the split works; but keep drawing commands out of the trace |
| Rerun | viewer for recorded sensor data | recording and *blueprint* are separate stores; blueprint is data with its own timeline | closest precedent for "one trace, many views"; has no moving camera |
| IcePanel Flows | steps laid over an architecture diagram | flows refer to connections, not coordinates | refer to IDs so scripts survive layout changes |
| Runway | model + simulation + view in the browser | a hand-written view file per model | per-model view code does not scale; views must be generic |
| TLA+ animation, Spectacle | SVG picture per state | view = pure function of state | right contract; no motion, no camera |
| Oddity | graphical debugger for distributed systems | branching executions in one session | forks belong in the picture |
| ShiViz | time-space diagram from logs | shows the partial order, not one line | the honest still companion for any animation |
| Cisco Packet Tracer | network simulator for teaching | envelope on a map, next-event stepping, open it per layer | "follow the request" plus the layers axis, proven |
| Disco, Apromore | process-mining animation | one token per case on a fixed map | a third view: many tokens, fixed map |
| Vizceral | Netflix traffic particles | flow as particles, three levels | good for a feeling of volume; abandoned; not enough alone |
| ExplorViz, DynaCity | software cities with traces | structure as a city, traces on top | measured: faster and more correct on larger systems; nothing moves |
| SimCity GlassBox | game simulation engine | "what you see is what we sim" | every moving thing must be a fact of the trace |
| Pixel Agents, AI Town | Sims-like views of agents | 2D sprites (Canvas 2D, PixiJS) | a town is cheap to draw and pleasant to watch |
| Virtual Cinematographer | automatic camera, 1996 | film idioms as small state machines + camera modules | idiom = script; camera module = stage |
| Cinemachine | Unity camera system | virtual cameras with targets and priorities, blended | shots as intents over named things |
| Esports auto-observers | automatic spectator cameras | score or rank what is interesting | the town needs an interest score; Mental can look ahead |
| Segel and Heer | study of 58 narrative visualizations | author-driven versus reader-driven; "martini glass" | a script must be leavable and resumable |
| Gemini; Remotion | animation grammar; video from code | staged steps as data; frame = function of time | script as data; picture as a function of time |
| TheoremExplainAgent, Code2Video | LLMs writing animations | story is fine, layout goes wrong | let the LLM choose *what*, never *where* |
| Tversky; Robertson; Hundhausen | studies of animation | animation alone does not help analysis; interaction does | still companion; stepping over watching |
| Multiple object tracking | attention research | about four tracked objects | follow = one subject; a town is sensed, not followed |
| PixiJS v8 | 2D GPU renderer | 200,000 sprites at 60 fps on an M3 (vendor figure) | 100 times more than needed |
| Figma, Perfetto, Rerun | large canvas applications | WebAssembly core, GPU drawing, interface in front of the graphics API | same shape as `mentaldbg` + viewer |
| WebGPU (2026) | browser GPU API | Chrome, Safari, Firefox on Windows and Apple Silicon; not Linux or Android Firefox | keep the WebGL2 fallback |

---

## The evidence in four paragraphs

**The split exists and is old.** Proof Animation separated a layout file from a trace file in the 1990s. Rerun separates a recording from a blueprint and stores the blueprint as data with its own timeline. IcePanel lays several flows over one diagram by referring to its connections. What none of them has is a camera that moves by rule: their view descriptions are static arrangements. And where views of a *model's run* exist (Runway, TLA+), they are written by hand for each model. Mental needs views that are generic over its record kinds. ([Prior art](notes/prior_art_system_visualizations.md))

**Both views the question names have working ancestors.** "Follow the request" is Cisco Packet Tracer's simulation mode: an envelope moves across the map one event at a time, and it can be opened to see what each device does to it at each layer. The town is the software-city line of research, which is the only place where a benefit was measured: 28% less time on a larger system and 39 to 61% more correct answers against a classic trace tool. But in those cities nothing walks; the trace is drawn on top of a still city. A town with inhabitants is new ground, and the game that tried hardest to make a visible simulation, SimCity, states the rule that makes it honest: what you see is what is simulated. ([Prior art](notes/prior_art_system_visualizations.md))

**Automatic camera direction is a solved problem for a harder case than ours.** Film-style camera control has been automatic since 1996 using *idioms*, small state machines that choose shots, over shared *camera modules* that do the geometry. Game engines ship this as virtual cameras with targets and priorities. Esports adds a score of what is interesting when too much happens at once. Mental's case is easier in every way: a flat map with no occlusion, a finished trace that allows looking ahead, and a model that already says which steps matter. LLMs can write animations too, but the studies agree on where they fail: layout. ([Generating a script](notes/script_generation.md))

**Motion is not understanding.** Reviews of animation studies find no general benefit over good still pictures for explaining complex systems; animations are too fast to take in. For analysis, small multiples beat animation in speed and accuracy, though people enjoy animation more. In algorithm teaching, what the learner *does* matters more than what is shown. Animation does help for transitions between views, where it keeps track of which thing is which. And people can follow only about four moving things. ([Perception and clarity](notes/perception_and_clarity.md))

---

## Proposal

### Three things between the trace and the pixels

The specification has four layers: model, run, view state, skin. The question shows that "view state" and "skin" are not enough to describe a town or a camera on a request. The proposal is to say what each of the following is, keeping the placement test (removing any of them changes nothing the interpreter computes).

| Thing | Answers | Examples | Changes when |
|---|---|---|---|
| **Stage** | where is everything? | map (node-link), lanes (time-space), town (isometric districts), journey (the request's path as a strip) | the viewer wants a different *kind of space* |
| **Script** | what do we look at, when, and how fast? | follow this message; frame the two racing workers; ambient with interest-driven camera | the viewer asks a different *question* |
| **Skin** | how does each thing look? | blueprint lines; pixel-art town; plain boxes | the viewer has a different *taste* |

A **view** is one choice of each. The two views in the question come out as:

| View | Stage | Script | Skin |
|---|---|---|---|
| Camera on the request | map, or journey | follow (subject = one message and what it causes) | any |
| Sims town | town | ambient (interest score), with click-to-follow | pixel-art or isometric |

Two consequences:

- The specification says layout does not belong to the skin, "so changing skins moves nothing". That stays true. Layout belongs to the **stage**. Changing the stage does move things, and must be animated so that each entity can be tracked from the old place to the new.
- **View state** (zoom on four axes, collapsed parts, pins, focus) remains what it is: a value at one moment. A script is *view state over time, plus pacing*. Playing a script writes the session's view state; when the viewer grabs the camera, the script pauses and the view state is theirs.

**Naming (decided).** This thing is the **script**, made by a **director**. Earlier drafts used *script* for the output of a run; that is now always the **trace**, and the old use is retired.

### What a script contains

A script never contains coordinates, sprites or colors. It contains, in order of importance:

1. **Subject**: what the film is about. A message (and its causal descendants), an entity, a state variable, a finding, a field of a payload, or nothing (ambient).
2. **Selection**: which steps are shown fully, which are summarized, which are skipped. Given as rules over the trace (on the subject's causal chain; kind is write; inside entity X), not as a list of positions.
3. **Time map**: how the partial order of steps becomes seconds of playback. Steps have no real durations, so this is entirely a choice: equal time per step; longer for forks and findings; compress a run of uneventful steps into one beat; show independent steps of two lanes together or one after the other.
4. **Shots**: for each stretch, a camera *intent* ("follow the subject", "frame the group {worker[1], worker[2], db}", "wide") and the zoom on each of the four axes.
5. **Annotations**: captions and callouts, each attached to a step ID. Anything written by an LLM is labeled as such.

A sketch, to show the level, not a format proposal:

```json
{
  "script": "follow",
  "subject": { "message": "<full ID of the POST /reports request>" },
  "select":  [ { "where": "causal-descendant-of-subject", "show": "full" },
               { "where": "otherwise", "show": "dim" } ],
  "time":    { "base": "one beat per step", "hold": ["fork", "finding", "non-atomic"], "collapse": "runs of forwarding" },
  "shots":   [ { "while": "subject crosses a boundary", "camera": "follow", "zoom": { "layers": "open" } },
               { "when": "step in finding", "camera": { "frame": "entities-in-finding" }, "hold": true } ]
}
```

Because it is data over IDs, a script can be validated against the trace (every ID exists; no selected step contradicts the presentation rules), stored beside the run file, diffed, and written by an agent through MCP. It is small for the same reason the run file is small.

**The engine still enforces the presentation rules.** A script may dim or summarize, but it may not skip an error, an assumption or a non-atomic group that lies on what it shows, and it may not present a hypothetical as fact. A script that tries is rejected, in line with failing rather than degrading.

### Three ways to generate a script (Experiment)

These compete; the plan below compares them.

| | **A. Idiom** | **B. Scored** | **C. Authored** |
|---|---|---|---|
| What | one fixed rule set per kind of film | an interest score per step drives camera and clock | the coding agent writes a script for one question |
| Input | trace + subject | trace + weights | trace + question in words |
| Precedent | Virtual Cinematographer, Cinemachine | esports observers | TheoremExplainAgent, with layout taken away from the LLM |
| Cost | none; instant; deterministic | none; instant; deterministic | tokens through the user's agent; seconds |
| Best for | follow, race, why | town and other ambient views; long traces | "explain to a new teammate what happens on submit" |
| Risk | rigid | jumpy camera; weights to tune | tells a wrong or partial story |

Starting idioms worth writing: **follow** (a message), **watch** (an entity or a state variable), **race** (two lanes side by side up to the conflicting steps, then the flip), **why** (walk the causal chain backwards from a step), **diff** (two runs, in step until they diverge).

Facts that can feed the score in B, all already in the model or trace: part of a finding or witness; member of a non-atomic group; fork; error; assumption; store write; boundary crossing at the current zoom; on the causal chain of the selected step. Constraints borrowed from film practice: a minimum shot length, no cut while the subject is moving, at most about four things moving at once.

For C, the agent gets the trace through the existing session tools and returns a script. It chooses subject, selection, emphasis and wording. It cannot place anything, and the result is validated before it plays. This fits the local-first rule: the user's own agent does the work.

### A catalog of views to try

| View | Stage | Script | Good for | Weak at |
|---|---|---|---|---|
| **Follow** | map or journey | follow | "what happens when someone presses submit" | anything concurrent with the subject |
| **Town** | town | ambient, scored | a feeling for the whole; where it is busy or stuck; first contact with a system | exact reasoning; anything needing order |
| **Tokens** | map | ambient, all messages as tokens | queues, pile-ups, fan-out | detail of any one request |
| **Lanes** | time-space diagram | none, or why | the partial order; races | spatial sense of the system |
| **Race** | two lanes or split screen | race | one finding, shown twice with the fork flipped | everything else |
| **Storyboard** | a strip of still frames | cuts only | reading without motion; sharing in a pull request; reduced-motion users | the feel of flow |
| **Field** | map | follow a field | where a piece of data goes and changes form | control flow |

Lanes and Storyboard are the still companions the perception research calls for. They are not optional extras.

### The pipeline

```text
trace ──┐
model ──┼─► stage ──► placed scene (entity → place, per zoom)        cached per container
        │
        └─► director ──► script (data) ──► resolver ──► timeline of segments
                                                             │   (thing, from, to, t0, t1, easing)
                                                             ▼
                                              picture(t) = draw(scene, timeline, skin, t)
```

The contract to hold on to: **the picture is a pure function of time**. Nothing in the renderer accumulates. From it follow, with no further work: dragging the slider, stepping backwards, starting anywhere, exporting a video frame by frame, and testing the viewer by comparing frames at fixed times.

### Rendering

The measured facts say the scene is two orders of magnitude below what a 2D GPU renderer handles. So the design goal is not throughput but steadiness.

- **Core in WebAssembly, in a worker** (as already decided): trace store, stage layout, script resolver. It hands the main thread typed arrays. This is the Perfetto and Figma shape.
- **Movements uploaded once.** Each segment of the timeline goes to the GPU with its start and end times; each frame sends only the playhead. Playing and scrubbing cost the same and upload nothing.
- **Instanced quads from one atlas per skin; distance-field text** with a cap on labels per view and cached label layout. Text, not sprites, is the likely real cost.
- **Zoom is the level-of-detail system.** A closed entity is one sprite.
- **A draw-list interface in front of the graphics API**, so the renderer can be replaced. Figma's move to WebGPU went through exactly such a layer.
- **A town is 2D.** Isometric sprites drawn back to front on a tile grid. No 3D engine is needed for it.
- **WebGPU first, WebGL2 fallback kept.** Firefox on Linux and Android still lacks WebGPU by default.

Two stacks to compare (Experiment): **PixiJS v8** driven by the WebAssembly core (fastest to a result; both back ends built in), and **an own renderer on `wgpu`** (one renderer shared with Mental Desktop, as Rerun does; more work and a larger download). Starting with PixiJS behind the draw-list interface keeps the second option open. Details and sources are in the [rendering note](notes/web_rendering_performance.md). A later, deeper analysis of the implementation, [script rendering](../script-rendering/report.md), changes this recommendation: it favors a thin own player over PixiJS.

### Rules for clarity

1. **One subject, or say there is none.** A view either follows something or is explicitly ambient.
2. **About four things move at once.** More concurrency than that is slowed, staged or split.
3. **Time is event time.** The default clock is "next step", as in Packet Tracer. Durations on screen are the script's choice and are never presented as latency.
4. **Every channel is bound or decorative.** For each stage and skin, each visual channel (position, distance, size, speed, color, sound) is declared as bound to a model fact or as decoration, and the viewer can show the list. A town must not let a long walk mean "slow".
5. **Everything that moves is in the trace.** No extras walking around for atmosphere unless marked as decoration and switchable off.
6. **Motion for transitions, stillness for study.** Changing zoom, stage or shot is animated and staged. At a fork, a finding or a pause, the picture holds.
7. **The script can always be left and resumed.** Grabbing the camera pauses it; one action returns.
8. **Every view has a still companion and respects reduced motion.**

---

## Where this touches the specification

The layers and the name were accepted and are now in the specification (P10) and the glossary. The other rows are still open.

| Specification says | Proposal | Status |
|---|---|---|
| four layers: model, run, view state, skin (P10) | add **stage** and **script** between view state and skin; view state stays a value at one moment | decided |
| "Layout does not belong to the skin" (glossary, Skin) | unchanged; layout belongs to the stage | compatible |
| "Runs dim or hide entities they do not involve; they never move them" (9.4) | true within a stage; the **journey** stage lays entities out along a path, so it is a different stage, entered by an animated transition | to decide; journey may be dropped if it confuses |
| the three presentation rules (9.4) | also bind scripts: a script that hides an error on what it shows is rejected | extension |
| performance targets (9.4) | add: seek as smooth as play; no per-frame upload; frame as a function of time | extension |
| *script* was an old word for the trace (glossary) | *script* now means the visual script; the trace is always the trace | decided |

Glossary entries added: **Stage**, **Script**, **Director**, **View**. Still to add when their design settles: **Shot**, **Time map**.

---

## Experiment plan

All on the duplicate-claim race of `examples/report-jobs`, whose trace already exists by hand at seven levels (findings Z).

| Step | Build | Measure | Decides |
|---|---|---|---|
| 1 | the pipeline with one stage (map), one idiom (follow), boxes for a skin; PixiJS behind a draw-list interface | 99th-percentile frame time playing and scrubbing; time to first picture; download size; the same on a synthetic trace 100 times larger | whether "picture as a function of time" holds up; baseline numbers |
| 2 | a second stage (lanes) and the race idiom, same trace | can the stage be swapped under an unchanged script, and the script under an unchanged stage? | whether stage and script are truly independent |
| 3 | the town stage with a pixel skin, and the scored director | does a person shown only the town notice *that* something is wrong and *where*? does click-to-follow then explain it? | whether the town earns its place, and as what |
| 4 | scripts from A, B and C for the same question ("why were two reports generated?") | time to a correct answer for a person new to the example; errors in C's story; tokens spent by C | which generators to keep, and for what |
| 5 | the same renderer contract on `wgpu` | the numbers of step 1; work needed | which stack |

Steps 1 and 2 are small and test the central claim. Step 3 is the one that answers the user's wish for a town. The viewer is step 8 of the specification's build order, after the model and interpreter; steps 1 and 2 can run earlier on the hand-written trace.

---

## What is evidence and what is inference

**Evidence (cited in the notes):** the recording/presentation split in Proof, Rerun and IcePanel; software-city experiments; Packet Tracer's follow mode; idiom-based camera control; the animation and attention studies; LLM layout errors; renderer and browser figures (vendor or secondary sources, one machine each).

**Inference:** that stage, script and skin are independent in practice; the content of a script; the three generators and their fit; the interest facts; the clarity rules as a set; that a town with inhabitants helps (the measured cities were still); that PixiJS is enough.

**Not found or not verified:** any tool that plays a model's trace on a town; performance of text-heavy scenes on WebGPU; the current state of WebGPU in workers in Firefox; the exact Firefox version for Apple Silicon; the detailed results of the animated-edge studies. Several well-known systems (Stateright Explorer, P's trace viewers, Kiali, AnyLogic, Jhala and Young's planning-based camera work) were not covered and may add precedent.
