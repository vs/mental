# Rendering Scripts: Compile Once, Play With One Number

**Question.** A view is a stage, a script and a skin over one trace ([trace visualization](../trace-visualization/report.md)). How should the viewer be built so that playing a script is extremely fast: no dropped frame at 60 or 120 Hz, scrubbing as smooth as playing, a quick start, and room to grow a hundred times?

**Short answer.**

1. **Treat the script like a program and compile it.** Trace + stage + script are compiled, off the frame loop, into a **timeline**: flat arrays of *interval instances*. Each instance says "this thing looks like this, from here to there, between time t0 and time t1".
2. **Play with one number.** The GPU holds the timeline. Each frame the viewer sends the playhead `t` and the camera, and the vertex shader decides what is visible and where. Nothing is re-uploaded, nothing is allocated, and no code runs per object. Scrubbing, stepping back and jumping cost exactly what playing costs.
3. **Everything that varies in time is an interval**, not only movement: highlights, dimming, open and closed entities, captions. So there is one mechanism, and seeking never needs to replay events.
4. **Cut the timeline into time chunks** so that a frame draws only the instances that can be alive at `t`. Cost per frame then depends on what is on screen, not on how long the trace is.
5. **Positions are local to their container**, with the container-to-screen transforms computed in 64-bit on the CPU for the few visible containers. This is what makes deep zoom (service to statement) free of jitter, and it matches "layout computed per container".
6. **A skin is data, never code**: an atlas and a style table read by fixed shaders. Changing skin compiles nothing and rebuilds nothing.
7. **The viewer is a thin player.** The compiler of timelines lives in the shared Rust core, which runs natively inside `mentaldbg serve` and as WebAssembly in a worker. The main thread has a small TypeScript player and a handful of shaders.

With this design, a frame costs the main thread a few hundred bytes of uniform upload and about ten draw calls. All of it is inference from the sources below and from arithmetic; none of it has been measured on a Mental trace. The first report leaned toward PixiJS; this analysis shifts the recommendation (see "Which stack").

---

## In brief: what each source contributes

| Source | What it is | What it says | Take for Mental |
|---|---|---|---|
| deck.gl `TripsLayer` | animated trips on a map | timestamps are per-instance attributes; `currentTime` is a uniform; the shader discards what is outside the window (read in the source) | the core trick, in production |
| Figma, WebGPU move (2025) | C++/WebAssembly canvas renderer | interface layer in front of the graphics API; naive per-draw uniform uploads regress, so batch them into one buffer; cache bind groups; fall back to WebGL *during* a session | one uniform upload per frame; a stateless renderer makes fallback trivial |
| Toji, WebGPU best practices | guidance from a WebGPU author | `writeBuffer` is the preferred upload path, also from WebAssembly; render bundles keep commands fixed while buffer contents change | fits "commands never change, only `t` does" |
| Perfetto UI | trace viewer | engine in a worker, or a native engine over a WebSocket; draw only what maps to pixels | compile natively in `mentaldbg`, play in the browser |
| Godot, AGI on precision | large-world rendering | 32-bit floats jitter far from the origin; subtract the camera on the CPU in 64-bit | per-container coordinates |
| tldraw, Lucid | canvas editors | spatial index for culling and picking; Lucid reports over 1,000 times faster spatial search with an R-tree | index per container for picking |
| Quadratic | spreadsheet on WebGL | treat the grid "like a game"; GPU text instead of HTML | all labels on the GPU |
| Chrome: WebGPU compatibility mode | WebGPU on older GPUs | shipped in Chrome 146 on OpenGL ES 3.1 and Direct3D 11 | helps Chrome only; WebGL2 fallback still needed |
| MDN: parallel shader compile | WebGL extension | not available everywhere; draw calls can stall on an uncompiled shader | compile every shader at load; never during play |
| Rust and WebAssembly book; V8 | size and loading | 240 KB to 37 KB with LTO, `opt-level=z` and `wasm-opt`; compiled-code caching only through the streaming APIs | keep the core small; load it by streaming |
| Chrome: Long Animation Frames, timestamp queries | measuring tools | LoAF in Chrome 123+, not Firefox or Safari; GPU pass timing is an optional feature | measurement plan |
| WebCodecs | video encoding in the browser | Chrome, Edge, Firefox 130+ on desktop, Safari 26 | video export from the same frame function |
| PixiJS v8 | 2D renderer | 200,000 sprites at 60 fps (vendor figure); custom shaders are drawn outside its batcher | the naive path is already fast enough at today's scale |

---

## 1. What "fast" has to mean here

**The budget.** A frame has 16.7 ms at 60 Hz and 8.3 ms at 120 Hz. The browser needs part of it. The targets proposed:

| Measure | Target |
|---|---|
| main-thread work per frame | under 1 ms at the 99th percentile |
| GPU time per frame | under 2 ms on an integrated GPU |
| allocations per frame | zero |
| frame cost while scrubbing | the same as while playing |
| script or subject change to first new frame | under 50 ms for a 10,000-step trace |
| first picture after opening the viewer | under 1 s with a warm cache |
| idle | no frames at all when nothing changes |

**The scale.** One repository: hundreds of entities, a few thousand at full depth. One trace: hundreds to tens of thousands of steps. A step becomes a few interval instances. So a whole timeline is 10,000 to 500,000 instances. At about 48 bytes each, that is 0.5 to 24 MB. This fits in GPU memory whole, which is what makes the design below possible.

**An honest baseline.** At today's scale, the naive design (a scene graph, and code that moves each object every frame) would also reach 60 frames per second. The design here is chosen for what the naive one cannot give: scrubbing with no replay, no garbage collection, headroom at 120 Hz, low battery use, and a hundred times more room. The experiment plan measures both so the gain is known, not assumed.

---

## 2. The timeline: what the script compiles to

### One mechanism: the interval instance

Everything on screen is an instance of one of a few kinds (shape, line, glyph, token). Every instance has an interval `[t0, t1)` in script time. Outside it, the vertex shader collapses the instance to nothing, so it costs no pixels. Inside it, the shader computes position and appearance from `t`.

| What the viewer sees | As intervals |
|---|---|
| a message travels from A to B | one token instance: from, to, `t0`, `t1`, easing |
| an entity is highlighted for three steps | one shape instance with the highlight style over that interval; the normal one covers the rest |
| a store's value changes | a label instance per value, each over the interval it holds |
| an entity opens during the script | interior instances whose interval starts at the opening |
| a caption | glyph instances over the caption's interval |
| things that never change | instances with the interval "always" |

This is deck.gl's `TripsLayer` generalized. There, timestamps are per-instance attributes, the playhead is a uniform, and the shader discards fragments outside the window; this was checked in its [source](https://github.com/visgl/deck.gl/blob/master/modules/geo-layers/src/trips-layer/trips-layer.ts).

Why state changes are intervals and not events: an event ("at step 40, turn red") makes the picture depend on history, so seeking backwards needs a replay from a snapshot. An interval ("red from 40 to 43") makes the picture depend only on `t`.

### A sketch of one instance

About 48 bytes. The fields, not a final layout:

| Field | Meaning |
|---|---|
| `frame` | which container's coordinate system (section 4) |
| `style` | row in the skin's style table |
| `sprite` | cell in the skin's atlas, if any |
| `a`, `b` | start and end position, or the control points of a curve |
| `size` | width and height |
| `t0`, `t1` | interval, relative to the chunk's start |
| `ease`, `fade` | how to move inside the interval and how to appear and disappear |
| `lod` | the range of on-screen size in which it is visible |

### Time chunks

The vertex shader would run for every instance, alive or not. For 500,000 instances that is still affordable, but it grows with the trace. So the timeline is cut into chunks of script time (for example one chunk per beat group, sized to a few thousand instances). An interval that crosses a chunk boundary is split. "Always" instances live in a separate static chunk. A frame draws the static chunk and the one chunk containing `t`.

Consequences:

- Frame cost depends on what is alive now, not on trace length.
- Times are stored relative to the chunk start, so 32-bit floats never lose precision on long timelines.
- A change that affects only the future (flipping a fork at step 300) leaves earlier chunks, and their GPU buffers, untouched.
- A long compile can deliver its first chunks while the rest is still being produced.

### Level of detail in the shader

Each instance carries the range of on-screen sizes in which it should show. The shader multiplies its size by the camera's scale and fades or collapses it. So semantic zoom during free camera movement needs no CPU work at all: a label that would be 3 pixels high simply is not drawn.

Which entities are *open* is view state, so it is not in the timeline: it is one flag per entity in a small table the shader reads. Opening an entity by hand writes a few bytes.

### Two clocks

The shader gets two times. **Script time** `t` is the playhead. **Transition time** `u` is real time, used for things the viewer causes: an entity opening, a change of stage (each entity gets a from-place and a to-place), a skin cross-fade. Keeping them apart means a paused script can still animate a zoom.

### What stays on the CPU each frame

1. Compute `t` from the frame timestamp given by the browser: `t = t_start + (now − wall_start) × rate`. Nothing is integrated, so there is no drift and no fixed-step loop.
2. Evaluate the camera: the script's camera track (a few keyframes of intents resolved to center and scale, interpolated with the smooth zoom-and-pan curve, [D3](https://d3js.org/d3-interpolate/zoom)), or the viewer's own camera if they took it.
3. Compute the transforms of the visible containers (section 4).
4. Write one uniform block: `t`, `u`, camera, container transforms.
5. Issue the draw calls: about four kinds × two chunks.

That is the whole frame. There is no per-object code.

---

## 3. The GPU side

**Few pipelines, fixed shaders.** Shapes (rounded rectangles and borders computed by distance in the fragment shader, so they are sharp at any zoom), lines and curves (curve evaluated in the vertex shader), glyphs, and tokens or sprites. Aggregate flow, where a view shows a *rate* and not individual messages, is drawn as particles whose position is a function of edge, phase and `t`, again with no CPU work.

**A skin is data.** A skin is an atlas plus a style table (colors, border widths, corner radii, sprite cells, fonts). Instances refer to style rows. Switching skin swaps two textures. No shader is compiled, no buffer rebuilt, and a cross-fade between skins is a blend of two table lookups. This also rules out a whole class of stalls: on WebGL a first draw with a new shader can block, and the extension that would avoid it is not available in every browser ([MDN](https://developer.mozilla.org/docs/Web/API/KHR_parallel_shader_compile)). So all shaders are compiled at load, before the first frame, and never during play.

**One upload per frame.** Figma found that the obvious port, one buffer upload per uniform change, regresses performance, and fixed it by collecting all uniform data for a frame into one buffer with offsets, and by caching bind groups ([Figma](https://www.figma.com/blog/figma-rendering-powered-by-webgpu/)). Here there is one block per frame by construction. The recommended upload call is `writeBuffer` ([Toji](https://toji.dev/webgpu-best-practices/buffer-uploads)).

**Render bundles fit, but matter little.** A bundle records commands once; buffer contents may change between executions ([Toji](https://toji.dev/webgpu-best-practices/render-bundles)). The draw commands here are identical every frame, so they can be recorded per chunk. With about ten draw calls the saving is small. Use them if measurement shows command encoding in the profile.

**Text.** Labels are glyph instances from a distance-field atlas shipped with the skin, laid out once at compile time in the Rust core. One atlas serves every zoom, so zooming never re-rasterizes text. Identifiers are mostly ASCII; other characters need a fallback path (rasterize the glyph on demand into a spare atlas page). HTML is never positioned over the canvas per frame. Quadratic reached the same conclusion for a spreadsheet: treat it like a game and draw text on the GPU ([Quadratic](https://www.quadratichq.com/blog/building-a-high-performance-spreadsheet-renderer-why-we-chose-webgl-over-html)). No benchmark for thousands of distance-field labels on WebGPU was found; this is the first thing to measure.

**Isometric towns.** Static tiles are sorted once at compile time and are one draw call. Tokens change depth as they move, so depth is computed in the shader from the position and tested with the depth buffer; pixel-art sprites have hard edges, so a simple alpha test is enough.

**Pixels, not objects, are the GPU cost.** On a 4K screen at high pixel density the limit is how many times each pixel is painted. Keep full-screen layers to two or three.

**The fallback.** WebGPU is absent by default in Firefox on Linux and Android. Chrome's compatibility mode (Chrome 146, on OpenGL ES 3.1 and Direct3D 11, [intent to ship](https://groups.google.com/a/chromium.org/g/blink-dev/c/N3RlLGCOTJ4)) widens Chrome only. So the design uses only what WebGL2 also has: instancing, uniform blocks, array textures, integer texture lookups. No storage buffers, no compute shaders. Tables (styles, container transforms, open flags) are small data textures on both back ends.

**Falling back in mid-session.** Figma could not predict which devices would fail and ended up switching from WebGPU to WebGL during a session when anything went wrong. Because the player here keeps no state except the timeline and `t`, a switch (or a lost device) is: create the other back end, upload the same arrays, draw frame `t`.

---

## 4. Precision: deep zoom without jitter

Zooming from a service to a statement can change scale by a factor of a million. A 32-bit float has about seven significant digits. With one global coordinate system, the positions of small things deep inside would be rounded and would visibly shake. This is the known large-world problem; the known fix is to keep precise positions on the CPU and send the GPU only small numbers relative to something near the camera ([Godot](https://godotengine.org/article/emulating-double-precision-gpu-render-large-worlds), [AGI](https://help.agi.com/STKComponents/html/BlogPrecisionsPrecisions.htm)).

For Mental the natural unit is the container:

- Every instance's position is in the coordinates of its container, as 32-bit floats. Inside one container the numbers are small.
- Each container has a transform to its parent. Each frame, the player multiplies these down from the camera in 64-bit for the containers that are visible (dozens), and uploads the results as a small table.
- An instance carries its container's index and the shader applies that row.

Benefits beyond precision: re-laying out one container changes one buffer range; culling is per container (a container off screen is skipped whole); and picking uses one small spatial index per container. tldraw and Lucid both rely on such an index, with Lucid reporting more than a thousand times faster search ([tldraw](https://tldraw.dev/sdk-features/culling), [Lucid](https://lucid.co/techblog/2015/05/19/big-content-in-a-little-canvas)).

A message that crosses a container boundary has its two ends in different containers. The shader transforms each end with its own row and interpolates between the results.

---

## 5. Where the work happens

```text
 mentaldbg serve (native)  ─┐                         main thread (TypeScript)
   or                       ├─ timeline chunks ─────►  player: clock, camera,
 worker (Rust → WebAssembly)┘  (binary, transferable)   one uniform block, draws
        ▲                                                     │
        └────────── view-state changes, subject, flips ───────┘
```

**The timeline is a wire format.** The compiler from (trace, stage, script) to timeline chunks is part of the shared Rust core. In every product `mentaldbg serve` is already running locally, natively, and the viewer is its client. So the server can compile and send chunks over the WebSocket, as Perfetto does with its native engine ([Perfetto](https://perfetto.dev/docs/visualization/large-traces)). The same compiler as WebAssembly in a worker serves a viewer with no server (a shared static page). The viewer does not care which one answered.

**No WebAssembly on the main thread.** Chunks arrive as binary buffers, transferred without a copy. This avoids three things: a call into WebAssembly per frame; the trap that typed-array views over WebAssembly memory become invalid when the memory grows ([discussion](https://github.com/rustwasm/wasm-bindgen/discussions/3802)); and the need for shared memory, which requires cross-origin isolation headers and would complicate embedding the viewer in other products.

**Compile cost and incremental work.** Compiling is linear in the number of steps. What triggers it, and what is reused:

| Change | Work |
|---|---|
| play, pause, scrub, jump, speed | none |
| move the camera by hand | none |
| change skin | swap two textures |
| open or close an entity by hand | a few bytes in the flag table; interior chunk for that container if not yet loaded |
| change the script (new subject, another director) | recompile the timeline; layout is reused |
| change the stage | recompile; play a transition on the second clock |
| flip a fork or apply an edit | new run in the interpreter; chunks before the change are kept |

While a new timeline compiles, the old one keeps playing; the swap happens between two frames.

**The state panel.** Showing the full configuration at `t` (the values of stores, the queues) is the debugger's job, not the picture's. The core keeps a snapshot every N steps and replays at most N from there, as game replays do (Box2D: every 16 steps, [Box2D](https://box2d.org/posts/2026/06/replay/)). The panel updates a moment after the slider; the picture never waits for it.

**No garbage.** The frame loop allocates nothing: one preallocated array for the uniform block, no objects, no closures. Small collections are short, but a large one can cost a frame, and the known cure is not to allocate ([Construct](https://www.construct.net/en/blogs/construct-official-blog-1/write-low-garbage-real-time-761)).

**No frames when idle.** When paused with no transition running, the player requests no animation frames. A canvas application that keeps its loop running uses the CPU and the battery for nothing.

---

## 5a. Live debugging: the same pipeline, growing at the end

When a scenario is run under the debugger, the trace is not finished: it grows one step at a time, goes back, and forks. The pipeline stays the same. It becomes incremental. Nothing here is measured; the cost figures are arithmetic.

**The representation is not the slow part.** One step compiles to a handful of interval instances, a few hundred bytes. Producing and uploading them is far below one frame. What takes time in "step → picture" is the interpreter's step and the round trip to it. A second, direct drawing path for debugging would save nothing measurable and would give two renderers to keep in agreement.

**What changes is the director, not the renderer.** A director working on a finished trace can look ahead. A live director cannot. So each director has two modes:

| | Replay (finished trace) | Live (debugging) |
|---|---|---|
| Sees | the whole trace | the trace so far |
| Idiom directors (follow, watch, race) | work | work unchanged: they are state machines that react to each step |
| Scored director | picks the best shots overall | reacts to the score of the step that just happened |
| Time map | may collapse dull stretches, hold on what matters later | one beat per step; no collapsing of what has not happened |
| Camera | can be in place before something happens | arrives when it happens |
| Authored by the agent | yes | no; the agent can change the *subject* through the shared focus |

The subject of the live script is the session's focus (P11): what the person or the agent selected is what the camera follows.

**How the timeline grows.**

- Sealed chunks are immutable, as before. One small **live chunk** at the end holds the intervals that are still open ("highlighted until further notice"). Each step closes some, opens some, and rewrites this small chunk. When it is full it is sealed.
- **Step:** append, then move the playhead one beat forward with a short animation.
- **Step back:** move the playhead back. Nothing is recompiled, because the earlier chunks are still there.
- **Flip a fork, set a value:** the trace diverges at that step. Chunks before it are kept; chunks after it are replaced as the new run produces them. Two branches can share their common chunks, which is what a side-by-side comparison needs.
- **Continue to a breakpoint:** thousands of steps may arrive in a moment. They are compiled in batches and the playhead does not try to animate each one: it runs fast or cuts to the stop.

**The debugger never waits for the animation.** The new state is correct on the next frame; the motion that leads to it is decoration on top and is cut short by the next command. Target: from command to a correct picture within one frame after the interpreter answers.

**The stage does not depend on the run.** Layout comes from the model, which exists before the run starts. Stepping never moves anything. Only instances created during the run (a new worker, a new connection) need a place, and each container reserves room for them.

**After the session, direct it again.** The trace that debugging produced is now finished, so a replay director can make a better film of it: skip the dull part, hold on the race, add the agent's captions. Scripts and timelines are cheap and derived; only the run file is kept.

## 6. Starting fast

- **A small player.** A few shaders and a draw loop are tens of kilobytes. No engine is downloaded.
- **The first picture needs no WebAssembly** when a server is present: connect, receive the static chunk and the first time chunk, draw.
- **The core, when used in the browser, is streamed and small.** Compile it while it downloads with the streaming API, which is also the only path with compiled-code caching in Chrome ([V8](https://v8.dev/blog/wasm-code-caching), [web.dev](https://web.dev/articles/loading-wasm)). Size tactics: link-time optimization, optimizing for size, `wasm-opt`, aborting on panic. The textbook example goes from 240 KB to 37 KB ([Rust and WebAssembly book](https://rustwasm.github.io/book/game-of-life/code-size.html)); real savings must be measured.
- **Shaders compiled at load, asynchronously,** before the first frame is requested.
- **Skins load on demand;** the default skin needs no atlas beyond the font, because its shapes are computed.

---

## 7. Which stack

The first report suggested starting with PixiJS. With this design the answer changes, because the design does not use what a scene-graph library offers: nothing is moved from code, and all four kinds are custom shaders, which PixiJS draws outside its batcher (according to third-party documentation of its custom rendering). PixiJS would serve only as a layer over the two graphics APIs.

| Option | What it is | For | Against |
|---|---|---|---|
| **A. Thin own player** (TypeScript, WebGPU + WebGL2) | four pipelines written twice, or once in WGSL and translated at build time | smallest download; nothing between the design and the GPU; full control of the fallback | two back ends to keep in step; all text code is ours |
| **B. PixiJS v8 as the layer** | its `Geometry` with instancing and a shader pair per back end | both back ends and context handling done; quick start | a dependency used for a tenth of what it is; its particle API is marked experimental |
| **C. `wgpu` in Rust** | player in the core, compiled to WebAssembly | one code base with a native desktop window; one shader language | WebAssembly on the main thread again; the WebGL path ships a shader translator in the download ([features](https://lib.rs/crates/wgpu/features)); no size figure was found |
| **D. Naive baseline** | PixiJS sprites moved from code each frame | simplest | no free seeking; garbage; cost grows with objects |

Recommendation: build **A**, and build **D** once as the yardstick. Keep the timeline format independent of both, so that **C** stays possible if Mental Desktop needs a native window instead of a web view.

---

## 8. Measuring

Because the picture is a function of `t`, tests are simple and exact.

| What | How |
|---|---|
| correctness | draw fixed `t` values and compare with stored images; draw the same `t` after seeking from different places and require identical output |
| frame time | timestamps of consecutive frames; report the 99th percentile, playing and scrubbing |
| long frames | the Long Animation Frames API (Chrome 123 and later; not in Firefox or Safari, [Chrome](https://developer.chrome.com/docs/web-platform/long-animation-frames)) |
| GPU time | timestamp queries on the render pass, an optional feature, coarsened in Chrome unless a developer flag is set ([Chrome](https://developer.chrome.com/blog/new-in-webgpu-121)) |
| allocations | heap sampling over a few hundred frames in an automated browser run; require a flat line |
| scale | a generator of synthetic timelines at 1, 10 and 100 times the report-jobs trace |
| start | time from navigation to first frame, cold and warm |
| size | bytes of player and of core, compressed |

A reference low-end machine must be chosen (for example a laptop with an integrated GPU from about 2019); none is named yet.

**Video export** falls out of the same function: step `t` by a fixed amount, draw, and hand each frame to the browser's encoder, which is available in Chrome, Edge, Firefox 130 and later on desktop, and Safari 26 ([caniuse](https://caniuse.com/webcodecs)). It can run faster than real time.

---

## 9. Changes to the experiment plan

Step 1 of the [experiment plan](../trace-visualization/report.md#experiment-plan) becomes more specific:

| Step | Build | Decides |
|---|---|---|
| 1a | the timeline format and its compiler for one stage (map) and one idiom (follow), on the hand-written report-jobs trace | whether everything a script needs can be expressed as interval instances |
| 1b | player A on WebGPU, then on WebGL2 | the numbers of section 1; the cost of keeping two back ends |
| 1c | baseline D on the same trace | how much the design really gains at 1, 10 and 100 times scale |
| 1d | labels: distance-field glyph instances at 1,000 and 10,000 labels | whether text meets the budget; the largest unknown |
| 1e | deep zoom over five levels with per-container coordinates | that there is no jitter |
| 1f | live mode: step, step back, flip and continue-to-breakpoint through the terminal debugger while the viewer follows | that the incremental path meets "correct picture on the next frame" |

Steps 2 to 5 are unchanged; step 5 (a `wgpu` player) is only worth doing if a native window is wanted.

---

## What is evidence and what is inference

**Evidence:** the playhead-as-uniform technique (deck.gl source); batching uniform uploads, bind-group caching and mid-session fallback (Figma); upload and bundle guidance (Toji); precision loss and the camera-relative fix (Godot, AGI); spatial indexes (tldraw, Lucid); browser support facts (Chrome, MDN, caniuse); loading and size guidance (V8, web.dev, the Rust and WebAssembly book).

**Inference:** that every time-varying aspect of a script fits the interval form; the chunking scheme; level of detail in the shader; per-container frames as the precision fix for nested zoom; the two clocks; the split between a compiling core and a thin player; all budgets and byte counts; that option A is little code.

**Unknown until measured:** label performance; vertex cost of large chunks on integrated GPUs; fill cost of a town on high-density screens; compile time of a timeline; the real size of the core as WebAssembly; behavior of WebGPU inside the web views of the products that embed the viewer.

**Not covered:** the WebGPU specification's list of compatibility-mode restrictions; asynchronous pipeline creation (the page meant to cover it could not be fetched); how other players (Rive, Lottie) structure their runtimes.
