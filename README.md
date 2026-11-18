# Mental

Mental compiles a repository into a **model of the system that you can run**. You play a scenario and get a trace of what happens, zoom into any step or out to the level above, search for races, and change something (add a load balancer, flip an assumption) to see what changes. A coding agent shares the same view.

**Status:** design only; nothing is built yet. **Current focus:** building the model from one repository.

```text
source code, infrastructure files, docs ─► mentalc ─► MODEL ─► mentalvm ─► TRACES ─► mentaldbg ─► viewer / agent
```

![Three tools and what connects them. mentalc compiles a repository and packs into a model, asking an LLM backend for judgment. mentalvm runs the model and produces traces and findings. mentaldbg holds the debug session and serves the viewer, the coding agent and the terminal. Products named Mental wrap all three tools and the viewer.](docs/images/components.svg)

## The three tools

| Tool | Job | In | Out |
|---|---|---|---|
| `mentalc` | the compiler | a repository, models of its libraries (packs), answers from an LLM | a complete model |
| `mentalvm` | the interpreter | a model, inputs, a schedule | traces and findings |
| `mentaldbg` | the debugger | a model and its runs | a shared session for the viewer, coding agents and the terminal |

The LLM is a setting of the compiler: by default your own coding agent answers the compiler's questions, and an API or a local model can be plugged in instead. Every answer is checked against the code before it counts as fact.

Products built on the tools are called Mental: Mental for Claude Code, Mental for Codex, Mental Desktop.

## Setup, test and run

### model (`model/`)

```sh
cd model
cargo fetch
cargo test
# run: add the command once the entry point exists
```

### extract (`extract/`)

```sh
cd extract
cargo fetch
cargo test
# run: add the command once the entry point exists
```

### mentalc (`mentalc/`)

```sh
cd mentalc
cargo fetch
cargo test
# run: add the command once the entry point exists
```

See [CONTRIBUTING.md](CONTRIBUTING.md) before your first commit.

## Documentation

| Document | What it is | Read it when |
|---|---|---|
| [Specification](docs/specification.md) | The design: the six stages of a build, the model, the levels, each tool, the milestones and what comes later | you want the whole design |
| [Glossary](docs/glossary.md) | Every term with one precise definition and examples, including what a *port* is and why it is called that | a word is unclear |
| [Findings](docs/findings.md) | Facts from our two hand-made trials, the research and checks of outside facts | you want the evidence behind a decision |
| [Resolution](docs/resolution.md) | Detail under the specification's section 6: packages and pins, gaps, the gate, acceptances, the trust rule | you want to know what happens when a reference cannot be resolved |
| [Guidelines](docs/guidelines/README.md) | Implementation rules per crate, and the testing playbook | you are about to write code |
| [Milestone 1 plan](docs/plans/milestone-1-build-model.md) | The first implementation step: the build model | you want to know what is built first |
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
| What is the idea underneath? | [2. Principles](docs/specification.md#2-principles) |
| What does a build do, stage by stage? | [4. The build at a glance](docs/specification.md#4-the-build-at-a-glance) |
| How does `mentalc` learn how a project is built? | [5. The build model](docs/specification.md#5-stage-1-the-build-model) |
| How are references in the code resolved? | [6. From code to facts](docs/specification.md#6-stage-2-from-code-to-facts) |
| What is a model made of? | [7. The model](docs/specification.md#7-the-model) |
| What is known without an LLM? | [8. The mechanical model](docs/specification.md#8-stage-3-the-mechanical-model) |
| What does the LLM do? | [9. What the LLM adds](docs/specification.md#9-stage-4-what-the-llm-adds) |
| How does one system look at different heights? | [11. Levels](docs/specification.md#11-stage-6-levels) |
| How is the model run, and how are bugs found? | [12. `mentalvm`](docs/specification.md#12-mentalvm-running-the-model) |
| What do people and agents see? | [13. `mentaldbg` and the viewer](docs/specification.md#13-mentaldbg-and-the-viewer) |
| What is built first? | [14. Version one](docs/specification.md#14-version-one-scope-and-milestones) and the [milestone 1 plan](docs/plans/milestone-1-build-model.md) |
| What is undecided? | [15. Later, experiments and open questions](docs/specification.md#15-later-experiments-and-open-questions) |
