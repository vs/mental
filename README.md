# Mental

Mental compiles a repository into a **model of the system that you can run**. You play a scenario and get a trace of what happens, zoom into any step or out to the level above, search for races, and change something (add a load balancer, flip an assumption) to see what changes. A coding agent shares the same view.

**Status:** design only; nothing is built yet. **Current focus:** building the model from one repository.

```
source code, infrastructure files, docs ─► mentalc ─► MODEL ─► mentalvm ─► TRACES ─► mentaldbg ─► viewer / agent
```

![Three tools and what connects them. mentalc compiles a repository and packs into a model, asking an LLM backend for judgment. mentalvm runs the model and produces traces and findings. mentaldbg holds the debug session and serves the viewer, the coding agent and the terminal. Products named Mental wrap all three tools and the viewer.](docs/images/components.png)

## The three tools

| Tool | Job | In | Out |
|---|---|---|---|
| `mentalc` | the compiler | a repository, models of its libraries (packs), answers from an LLM | a complete model |
| `mentalvm` | the interpreter | a model, inputs, a schedule | traces and findings |
| `mentaldbg` | the debugger | a model and its runs | a shared session for the viewer, coding agents and the terminal |

The LLM is a setting of the compiler: by default your own coding agent answers the compiler's questions, and an API or a local model can be plugged in instead. Every answer is checked against the code before it counts as fact.

Products built on the tools are called Mental: Mental for Claude Code, Mental for Codex, Mental Desktop.

## Documentation

| Document | What it is | Read it when |
|---|---|---|
| [Specification](docs/specification.md) | The first design, built up from first principles: the model, the levels, each tool, version one and what comes later | you want the whole design |
| [Glossary](docs/glossary.md) | Every term with one precise definition and examples, including what a *port* is and why it is called that | a word is unclear |
| [Findings](docs/findings.md) | Facts from our two hand-made trials, the research and checks of outside facts | you want the evidence behind a decision |
| [`examples/report-jobs`](examples/report-jobs/) | The sample system used in every example, and the first target | you want to see the code being modeled |

Research reports:
- [Static analysis and LLM system models](docs/research/static-analysis-llm-system-models/report.md): who should do what: extractors, the LLM or checkers.
- [Model interpreter semantics](docs/research/model-interpreter-semantics/report.md): how to run a model and find races.
- [Abstraction levels](docs/research/abstraction-levels/report.md): how other fields relate levels of description and check them.
- [Multi-level models](docs/research/multi-level-models/report.md): which levels a person needs, how steps of one level map to the level above, and what the LLM is asked; from research on how people perceive events and levels, with an experiment on rq comparing three ways to get the levels, and a [step-by-step walkthrough](docs/research/multi-level-models/ladder-walkthrough.md) of one run.
- [Ladder strategies](docs/research/multi-level-models/ladder-strategies.md): four ways of building the level ladder, their options, and the experiments that compared them on rq and report-jobs.
- [Levels in other domains](docs/research/levels-in-other-domains/report.md): how business processes, mathematics, hardware, science, military command, teaching, music and maps build multi-level models, what they agree on, and nine proposals for Mental.
- [Verification and proof](docs/research/verification-and-proof/report.md): what TLA+, Ivy, Kani, SLAM, Specula and others can prove, how they tie a result to real code, and when a result about Mental's model may be called a proof.
- [Type checkers as libraries](docs/research/type-checkers-as-libraries/report.md): which Python type checker can give the most resolved references without LLM calls, and how to use each one as a library, with an experiment comparing Pyrefly and ty on five projects.
- [Type checkers and builds](docs/research/type-checkers-and-builds/report.md): how Pyright, mypy, ty, Pyrefly, rust-analyzer, gopls, TypeScript, clangd and JVM tools learn a project's units, roots and environments from its build, and what that means for the build model.

## Where to start in the specification

| Question | Section |
|---|---|
| What is the idea underneath? | [2. First principles](docs/specification.md#2-first-principles) |
| What is a model made of? | [4. The model](docs/specification.md#4-the-model) |
| How does one system look at different heights? | [5. Levels and refinement](docs/specification.md#5-levels-and-refinement) |
| How does source code become a model? | [6. `mentalc`](docs/specification.md#6-mentalc-building-the-model) |
| How does `mentalc` learn how a project is built and deployed? | [6.10 The build model first](docs/specification.md#610-the-build-model-first) |
| How are libraries modeled? | [7. Packs](docs/specification.md#7-packs) |
| How is the model run, and how are bugs found? | [8. `mentalvm`](docs/specification.md#8-mentalvm-running-the-model) |
| Can Mental prove there are no bugs? | [8.11 Properties, verdicts and proof](docs/specification.md#811-properties-verdicts-and-proof) |
| How do I change something and see what happens? | [8.8 What-if](docs/specification.md#88-what-if-edits-variants-and-experiments) |
| What do people and agents see? | [9. `mentaldbg`](docs/specification.md#9-mentaldbg-debugging-and-showing) |
| What is built first? | [11. Version one](docs/specification.md#11-version-one-scope-and-build-order) |
| What is undecided? | [13. Open questions](docs/specification.md#13-open-questions) |
