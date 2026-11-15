# How a visual script can be generated

Notes for [the report](../report.md). The question: given a trace, what decides where the camera is, what is shown, and how fast time runs? Four bodies of work answer it.

## 1. Automatic cinematography: idioms as small state machines

**The Virtual Cinematographer (He, Cohen, Salesin, SIGGRAPH 1996).** Film knowledge is encoded as *idioms*: small, hierarchically organized finite state machines, one per kind of scene (three actors talking, one actor crossing a room). Idioms choose shots and cuts. A separate set of *camera modules*, shared by all idioms, does the geometry of placing the camera for a shot. It runs in real time as the action unfolds ([paper](https://doi.org/10.1145/3596711.3596786)). A later review calls it the seminal work and names its limit: each new variation needs a new hand-made idiom ([AAAI 2014 workshop](https://cdn.aaai.org/ocs/ws/ws1218/8800-38051-1-PB.pdf)).

**Camera control surveys (Christie, Olivier, Normand, 2008).** The field splits the problem into three parts: viewpoint computation, motion planning and editing (when to cut). Approaches run from constraint solving to optimization; the hard recurring problems are occlusion and how expressive a declarative shot language can be ([survey](https://people.irisa.fr/Marc.Christie/Publications/2008/CON08.html)).

**Cinemachine (Unity).** The production version of the same idea. Many *virtual cameras* exist at once, each with a follow target, a look-at target and a priority. A "brain" makes the highest-priority active one live and blends between them. A *target group* frames several weighted objects. A *state-driven camera* picks a child camera from the state of a state machine. Manager cameras nest ([docs](https://docs.unity3d.com/Packages/com.unity.cinemachine@2.3/manual/CinemachineManagerCameras.html)).

What this gives Mental:

- The split between **idiom** (what to show, when to cut) and **camera module** (how to frame it) is the split between a script and a stage. A script says "frame the worker and the database"; the stage knows where they are.
- Shots are stated as **intents over named things** ("follow X", "frame the group {A, B}"), never as coordinates.
- Mental's camera problem is far easier than film's. The world is a 2D or isometric map, so there is no occlusion and no line-of-action rule. A handful of idioms covers it.

## 2. Automatic observers in esports: scoring what is interesting

Games with many simultaneous events need an automatic camera for spectators. Two families exist. Event-based observers use hand-defined in-game events and need a lot of domain knowledge; learned observers rank candidate views from data of human observers, for example with a learning-to-rank model or an object detector ([automatic observer, ESWA 2023](https://cilab.gist.ac.kr/hp/wp-content/uploads/publications/international_journal/2023/ESWA_AO.pdf), [learning-to-rank camera control](https://ouci.dntb.gov.ua/works/4Yg3a5P9)). Related work predicts kills a few seconds ahead so the camera can be there in time.

What this gives Mental:

- The **town** view has the same problem: much happens at once and one screen cannot show it all. An interest score per step, and a camera that goes to the highest score, is the known answer.
- Mental has a large advantage over games: the trace is finished before playback, so the director can look ahead. And the model already says what matters, so no learning is needed. Facts that can feed a score: the step is part of a finding or a witness; it belongs to a non-atomic group; it is a fork; it is an error or an assumption; it writes a store; it crosses an entity boundary at the current zoom; it is on the causal chain of the selected step.

## 3. Narrative visualization: who drives, the author or the reader

Segel and Heer studied 58 narrative visualizations and found seven genres (among them annotated chart, comic strip, slide show and video) and three structures for mixing a told story with free exploration. The *martini glass* starts with a narrow, author-driven path and then opens into reader-driven exploration; the *interactive slideshow* lets the reader explore inside each step; the *drill-down story* shows everything and lets the reader pick ([paper](https://homes.cs.washington.edu/~jheer/files/narrative.pdf)).

What this gives Mental:

- A script is the author-driven part. It must be possible to leave it at any moment (grab the camera, click an entity) and to come back ("resume"). This is the martini glass.
- "Follow the request" is a slideshow-like structure (one step at a time, look around inside each). "Town" is a drill-down structure. They are different structures, not different skins, which supports the user's point.

## 4. Declarative animation and LLM-written animation

**Grammars.** Gemini describes an animated transition between two charts as a set of *steps*, each controlling one component and its timing, and can recommend transitions; 9 of 11 published designs could be reproduced exactly ([Gemini](https://idl.uw.edu/papers/gemini)). The lesson is that staged animation fits in a small declarative language.

**Programmatic video tools.** Remotion renders markup as a function of the frame number; Motion Canvas uses generator functions that advance virtual time on one canvas ([comparison](https://www.remotion.dev/docs/compare/motion-canvas)). Theatre.js keeps keyframes on named timelines and exports the project as JSON ([package](https://npmjs.org/package/theatre)). Remotion's model is the one to copy: *the picture is a pure function of time*. It gives seeking, backwards stepping and video export with no extra work.

**LLM-written animation.** TheoremExplainAgent has a planner write a story and a coder write Manim scripts; with o3-mini it reaches a 93.8% success rate, yet the videos "frequently exhibited minor visual layout inaccuracies": misaligned text, overlapping shapes, inconsistent sizes, more so on harder topics ([paper](https://arxiv.org/abs/2502.19400)). Code2Video adds a critic agent whose job is to repair spatial layout ([paper](https://arxiv.org/pdf/2510.01174)).

What this gives Mental:

- An LLM is good at deciding *what the story is* and bad at *where things go*. So an LLM-written script should contain only choices (which steps matter, what to call them, what to compare) and never geometry. Geometry comes from the stage.
- A script should be data, not code. Data can be checked against the trace, shown as a diff, stored beside the run, and written by an agent over MCP with no sandbox.

## 5. Replays in games: how to seek

Two ways to store a replay: inputs plus a seed (tiny, needs a deterministic simulation) or state snapshots (larger, no determinism needed). Real systems combine them: inputs everywhere and a snapshot every few seconds so that seeking does not re-run from the start. Box2D's replay keeps a key frame every 16 steps by default, within a memory budget ([Box2D replay](https://box2d.org/posts/2026/06/replay/)).

What this gives Mental: the run file is already the "inputs plus seed" form, and the interpreter is deterministic. For instant scrubbing the viewer needs configuration snapshots at intervals and the steps in between. This is a cache, never a stored format.

## The resulting design space

Three generators of a script, to be compared (see the report's experiment plan):

| Generator | How | Cost | Risk |
|---|---|---|---|
| **Idiom** | a fixed rule per view kind: follow this message; frame these two lanes | none, instant, deterministic | rigid; a new kind of story needs a new idiom |
| **Scored** | an interest score per step from model facts; the camera and the clock follow the score | none, instant, deterministic | weights need tuning; may jump around |
| **Authored** | the coding agent writes a script (choices and wording only) for one question | LLM tokens; seconds | may tell a wrong story; must be validated and labeled |
