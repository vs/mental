# Mental: specification

## Value proposition

Mental compiles a repository into a model of the system that you can run: play a scenario and get a trace, zoom in and out between levels, search for races, and change something to see what changes. A coding agent shares the same view.

## Target users

Software developers first: people who change concurrent or distributed systems and need a runnable picture of them, and the coding agents working beside them. Later, managers and other roles who need the big picture of what the systems do, with the option to go down into technical detail.

## Requirements

| ID | Requirement | Acceptance check |
|---|---|---|
| R1 | mentalc reads a repository's build files into a build model, with no LLM. | Given examples/report-jobs, when mentalc build runs, then it writes one build unit with its pins, three artifacts, five launches and twelve configuration inputs, and reports its three gaps, each with a fix. |
| R2 | Every reference site in the code gets an outcome from a type checker, and a gate stops the build on gaps that matter. | Given report-jobs, when sites are resolved with ty, then every site has an outcome and the write receivers resolve to Report; and given a copy with a seeded missing module, then the gate fails and names the fix. |
| R3 | mentalc extracts facts with packs and static tools only. | Given report-jobs, when extraction runs, then the facts list 6 entry points (2 routes, 2 tasks, 1 schedule, 1 command) and 2 state variables, list every branch as an obligation, and report WEBHOOK_URL as read and unset, with no LLM call. |
| R4 | The mechanical model predicts findings with no LLM and no interpreter. | Given report-jobs, when the mechanical model and its static checks are built, then the four known findings are predicted. |
| R5 | An LLM names and summarizes every port, and every answer is checked before it counts. | Given report-jobs and recorded answers, when mentalc build runs with the replay backend, then every port has a named summary that passes coverage, guard algebra and summary agreement; and given the k-th call fails, for every k, then the build either succeeds after a repair or exits with a clear error and writes no partial model. |
| R6 | mentalvm runs the model and searches all orderings within stated bounds. | Given the report-jobs model, when search runs, then it confirms the duplicate claim with a replayable witness; and given the claim fixed in the code, then a rebuild and a fresh search within the same bounds report no violation. |
| R7 | A run can be read at several levels. | Given the duplicate-claim witness, when levels are built, then the race shows at the goal level as delivered twice and its cause at the move level, and the pair check passes. |
| R8 | A person and an agent debug one run together. | Given the witness, when a person steps it in the terminal debugger and an agent through the MCP tools, then both see the same step at each point. |
| R9 | Everything runs locally with the user's own coding agent as the LLM, and output is deterministic. | Given report-jobs, when mentalc build runs twice with the agent backend and no API key, then both runs succeed and write byte-identical model files; and given the api backend with no key, then it fails with an error naming the variable. |

## Not settled yet

These parts of the specification stay open until you settle them. Until then their principles are off.

- Performance: `/alpha:update performance`
- Auth: `/alpha:update auth`

## Responsibilities

| Component | Responsibility | Calls |
|---|---|---|
| model | Owns the record format: IDs, records, guards, the canonical JSONL text, the well-formedness checks and the diagnostic type that every other crate reads and writes. | - |
| extract | Turns build files, Python source and compose JSON into the build model and facts (sites and outcomes from ty, call graph, obligations, effects, state variables, entry points), with no I/O and no LLM. | model |
| mentalc | Owns the build: reads the repository at its one I/O edge, runs the six stages, asks LLM backends for judgment, checks every answer and writes .mental/ with its cache and ledger. | model, extract |

## Contracts

Interfaces between components are defined once, as other, in `model/src/id.rs`, `model/src/record.rs`, `model/src/record/**`, `model/src/anchor.rs`, `model/src/guard.rs`, `model/src/diagnostic.rs`, `model/src/jsonl.rs`. Changes there are reviewed as risky.

## Errors

Every error a user sees says what went wrong and how to fix it.

| Component | Format | Meaning |
|---|---|---|
| model | exceptions | Typed exceptions whose message says how to fix the problem. |
| extract | exceptions | Typed exceptions whose message says how to fix the problem. |
| mentalc | message-and-exit-code | A message on stderr and a non-zero exit code. |

Categories:

- validation
- not-found
- unavailable
- check-failed
- internal

## Auth

Not settled yet: `/alpha:update auth` settles it.
