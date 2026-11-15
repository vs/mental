# Rendering on the web: what "extremely performant" needs

Notes for [the report](../report.md). Numbers are quoted with their source and its weakness. "Not re-checked" marks things written from memory.

## How big is the scene, really

A model of one repository has hundreds of entities, perhaps a few thousand at the deepest zoom. A trace has hundreds to tens of thousands of steps, of which a few dozen are on screen at any moment. That is small.

For comparison, PixiJS v8 reports 200,000 ordinary sprites or 1,000,000 particles at 60 frames per second on a MacBook Pro M3, with the movement logic, not the drawing, as the bottleneck ([ParticleContainer](https://pixijs.com/blog/particlecontainer-v8)); and 100,000 moving sprites at about 24 ms of CPU and 2 ms of GPU time per frame ([v8 launch](https://pixijs.com/blog/pixi-v8-launches)). These are the vendor's own figures from one machine.

So raw drawing speed is not the problem; any GPU path has two orders of magnitude to spare. "Extremely performant" for Mental means four other things:

1. **No missed frame, ever**: no garbage collection pauses, no layout and no text shaping inside a frame.
2. **Instant seek**: dragging the time slider is as smooth as playing.
3. **Fast start**: small download, first picture quickly.
4. **Scale by not drawing**: closed entities and off-screen things cost nothing.

## Browser GPU support in late 2026

- Chrome has had WebGPU on desktop since version 113, and on Android since 121.
- Safari added it in Safari 26 (autumn 2025) on macOS, iOS, iPadOS and visionOS.
- Firefox shipped it on Windows in version 141 (July 2025). Apple Silicon Macs followed in 145 or 147; sources disagree. Linux and Android were still missing by default, with Android targeted for late 2026.

Source: [web.dev](https://web.dev/blog/webgpu-supported-major-browsers) plus secondary reports. Consequence: the specification's "WebGPU with a WebGL2 fallback" is still right. The fallback cannot be dropped yet.

## What large canvas applications do

**Figma.** The renderer is C++ compiled to WebAssembly. It moved from WebGL to WebGPU behind an existing interface layer between high-level rendering code and the graphics API, and kept a fallback ([Figma blog](https://www.figma.com/blog/figma-rendering-powered-by-webgpu/)). Lesson: put an interface between "what to draw" and "the graphics API" from the first day.

**Perfetto UI.** A TypeScript application with the trace engine compiled to WebAssembly and run in a worker, so the interface stays responsive while queries run. It draws only visible tracks, fetches only what maps to pixels at the current zoom, and a recent change moved slice and counter tracks to WebGL. Browser memory (about 2 GB is typical) is the limit, so a native engine over a WebSocket can take over for large traces ([UI docs](https://perfetto.dev/docs/visualization/perfetto-ui), [large traces](https://perfetto.dev/docs/visualization/large-traces)). Lesson: this is the same shape as `mentaldbg serve` plus the viewer. Query at the resolution of the screen.

**Rerun.** A Rust viewer on `wgpu` with its own renderer; the same code runs natively and in the browser, with WebGPU when available and a reduced WebGL layer otherwise ([architecture](https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md)). Older source comments note the 4 GB ceiling of 32-bit WebAssembly memory. Lesson: one Rust renderer for web and desktop is proven, at the price of writing the renderer.

## Candidate rendering stacks

| Stack | For | Against |
|---|---|---|
| **PixiJS v8** (TypeScript; WebGPU and WebGL) | mature 2D sprite renderer; both back ends; batching, atlases and bitmap text built in; far more throughput than needed | TypeScript, so the draw loop is outside the Rust core; the particle API is marked experimental |
| **Own renderer on `wgpu`** (Rust, compiled to WebAssembly) | one renderer for the browser and for Mental Desktop; no boundary between core and drawing; full control of shaders | must be written and maintained; larger download; text is unsolved work |
| **Three.js `WebGPURenderer`** | true 3D if a town ever needs it; one shader language for both back ends | forum reports of it running slower than the WebGL renderer with many draw calls (350 against 140 frames per second at 5,000 cubes in one demo), so instancing is mandatory ([forum](https://discourse.threejs.org/t/why-webgpurenderer-performance-significantly-lower-than-webglrenderer/77629)); anecdotal |
| **Bevy** | full engine with an entity system; targets WebGPU through `wgpu` ([Bevy](https://bevy.org/news/bevy-webgpu/)) | a whole engine's download for a diagram; no measured size found |
| **Vello** | high-quality vector 2D on GPU compute | the web is a secondary target; glyph caching still open ([Vello](https://docs.rs/crate/vello/latest)) |

An isometric town does not need 3D. It is 2D sprites drawn back to front on a tile grid, which is how AI Town (PixiJS) and Pixel Agents (Canvas 2D) do it.

## Techniques that matter for this workload

**The picture is a function of time.** Upload each movement once as a segment: start position, end position, start time, end time, easing, sprite. Send the playhead as a single uniform each frame and let the vertex shader interpolate. Nothing is re-uploaded while playing, and seeking costs the same as playing. deck.gl's `TripsLayer` exposes exactly this shape, with a `currentTime` property over timestamps given per vertex ([TripsLayer](https://deck.gl/docs/api-reference/geo-layers/trips-layer)); that it is implemented as a shader uniform is from memory and not re-checked. deck.gl's own guidance is that re-creating layers or data every frame is what costs.

**Instancing and one atlas.** All entities of a skin come from one texture atlas and are drawn as instanced quads: a handful of draw calls for the whole scene.

**Text.** Text is the usual real bottleneck in diagram tools. Use a signed-distance-field atlas (MSDF) so one atlas serves every zoom; draw glyphs as instances. Label layout is CPU work proportional to the number of labels, so cap labels per view and cache the layout. No benchmark of thousands of MSDF labels on WebGPU was found.

**Level of detail from the zoom axes.** Mental's rule that interiors are drawn only when approached is also the performance plan: a closed entity is one sprite and one label.

**Layout off the frame.** Layout per container, computed once in the WebAssembly core (in a worker), cached by container and version. The frame loop only reads positions.

**Workers.** Run the core (trace store, script resolver, layout) in a worker and send typed arrays to the main thread. Rendering in a worker through `OffscreenCanvas` is possible, but Firefox had open defects for WebGPU in workers ([Mozilla bug](https://bugzilla.mozilla.org/show_bug.cgi?id=1818042)); current state not confirmed. Shared memory between threads needs cross-origin isolation headers, which complicates embedding the viewer in other products; transferring buffers avoids that.

**Seek cache.** Keep configuration snapshots at intervals, as game replays do (Box2D: a key frame every 16 steps within a memory budget, [Box2D](https://box2d.org/posts/2026/06/replay/)), so the state panel follows the slider without re-running.

**Camera paths.** For moving between two views, van Wijk and Nuij's "smooth and efficient zooming and panning" gives the path that looks smooth and its recommended duration; D3 ships it as `interpolateZoom` ([D3](https://d3js.org/d3-interpolate/zoom)).

## Open measurements

Nothing above was measured on a Mental trace. The first prototype should record: 99th-percentile frame time while playing and while scrubbing, time to first picture, download size, and memory, on the report-jobs race trace and on a synthetic trace 100 times larger.
