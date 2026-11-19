# Glossary

The words Mental uses, with one precise meaning each. The [specification](specification.md) uses them exactly as defined here.

Most examples come from the sample repository in [`examples/report-jobs`](../examples/report-jobs/). In it, an API accepts report requests, stores a `PENDING` row in Postgres and queues a Celery task through Redis. Workers take tasks, build a CSV, upload it to S3 and mark the row `DONE`.

**Contents:** [the whole idea](#the-whole-idea) · [structure](#structure) · [ports in depth](#ports-in-depth) · [state](#state) · [behavior](#behavior) · [levels](#levels) · [how a record is known](#how-a-record-is-known) · [building](#building) · [running](#running) · [showing](#showing) · [tools and products](#tools-and-products) · [words with several meanings](#words-with-several-meanings)

## The whole idea

| Term | Definition |
|---|---|
| **System** | The software being studied, together with the people and outside services it talks to. |
| **Model** | A description of the system that a program can run. It is a set of records, compiled from a repository. It says what exists (entities, ports, wires), what can be remembered (stores, state variables) and what can happen (behaviors). The model is *not* the code and holds no real values: it holds named conditions and classes of input. |
| **Record** | One item of the model, one line in a file: an entity, a port, a predicate and so on. Every record has an ID, an anchor and a provenance. |
| **Run** | One execution of the model by the interpreter. Four things identify it: the model version, the binding, the schedule and the edits. Given those four, a run always produces the same trace. |
| **Trace** | The output of a run: the ordered list of steps that happened, with what caused each one. It is what you watch and step through. |
| **Scenario** | A situation in plain language that someone wants to see, such as "the first upload fails, the retry succeeds". A scenario is turned into a binding; it is not a record of the model. |
| **Finding** | Something the tools report as wrong or unknown: a race, a write the lifecycle forbids, input nothing handles, a gap in the model. |

## Structure

**Entity.** Anything that has a boundary and can take part in an interaction. An entity has ports on its boundary and, optionally, an interior made of smaller entities.

- *Examples:* the `api` service, a worker process, the Postgres server, the `reports` table, the user, the function `create_report`, a lock.
- *Test:* if you can ask "what goes in and out of it?", it is an entity.

**Boundary and interior.** The boundary is the set of an entity's ports. The interior is what is inside: child entities and the wires between them. A *closed* entity shows only its boundary. An *open* entity shows its interior too. Opening an entity must explain every one of its ports: each port is connected to something inside, or the gap is reported.

**Type and instance.** A type is a kind of entity described once, such as "Celery worker". An instance is one concrete copy that exists in a run. Version one runs each entity as one instance, with the entity's ID. Runs act on instances.

**Port.** A named point on an entity's boundary through which one kind of interaction passes. See [Ports in depth](#ports-in-depth).

**Wire.** A connection between two ports. Two kinds:

- An **assembly wire** joins a required port of one entity to a provided port of another entity beside it. *Example:* the API's `db` port to Postgres's `sql` port.
- A **delegation wire** joins a port on an entity's boundary to a port of something inside it. *Example:* host port 8000 to the container's port 8000, and that to the route `POST /reports`.

A wire also carries a **channel**: how messages travel on it. A channel is ordered or unordered, may lose messages, may deliver one twice, and may have a size limit. *Example:* the wire from Redis to the workers is at-least-once (it may deliver a task twice) because the code sets `task_acks_late = True`.

**Message.** One unit of interaction passing through a port and along a wire: an HTTP request, a SQL statement, a task, a function call. A message has a schema (its fields) and may be nested in layers (HTTP inside TLS inside TCP).

**Adapter.** A change of a message's form on a wire that keeps no state and maps one message to one message, such as Docker's port mapping. Anything that keeps state, such as TLS termination, is an entity.

**Boundary entity.** An entity that sits on another entity's edge and stands in front of what is inside: a load balancer, gateway or proxy. It sits on the smallest entity that contains everything it fronts.

**Facet.** One of two containment trees over the same instances. The *logical* facet says what a thing is part of (system → service → component). The *physical* facet says where it runs (host → container → process). **Allocation** links them: "the `api` service runs as the process in container `api`".

**Actor.** An entity outside the code that starts interactions: a user, an operator, a clock. An actor's behavior is not compiled from code; it is a list of what the actor *may* do.

**External.** An entity outside the repository that the code calls: S3, a payment API. It is modeled by a pack when one exists, otherwise by a stub.

## Ports in depth

**Definition.** A port is a named point on an entity's boundary through which exactly one kind of interaction passes. A port has:

- an **owner**: the entity it belongs to;
- a **direction**: *provided* (others start interactions here; the entity answers) or *required* (the entity starts interactions here; someone else answers);
- a **protocol**: the rules of the interaction, such as an HTTP route, SQL, a Celery task, a function call;
- a **message schema**: what the messages contain.

Everything that enters or leaves an entity passes through one of its ports. There is no other way in or out.

**Why the model needs ports.** Without ports, "the worker talks to the database" is a line between two boxes, and nothing says where that line attaches, what flows through it or what replaces the database when it is missing. Ports give each interaction a fixed attachment point with a name. That makes four things possible:

1. **Behavior has a home.** What an entity does is described per provided port: "when a message arrives *here*, this happens".
2. **Wiring can be checked.** A required port must be wired to a provided port with a matching protocol, or it is reported as open.
3. **Parts can be swapped.** Replace Postgres with a stub, or put a load balancer in front of a service: the ports stay, only the wires change.
4. **Zoom keeps its bearings.** A port keeps its identity when its owner is opened or closed, so a message can be followed from outside to inside.

**Why this word.** The term comes from component models in software and systems engineering: UML 2 and SysML call the interaction points of a component *ports*, hardware modules have ports, and "ports and adapters" (hexagonal architecture) uses the word the same way. It is *not* a network port number. A TCP port such as 5432 is one detail of one protocol; in Mental it would be an attribute of a port, not the port.

**Rules of thumb for what counts as a port.**

- A **provided port** per entry point: one per HTTP route, per task a worker consumes, per CLI command, per scheduled job, per public function of a library.
- A **required port** per distinct thing the code calls outside itself: one per database, per queue-and-task, per bucket, per external API.

**Examples.**

![Ports on a service. A client reaches the service through its provided port POST /reports. The service reaches the outside only through four required ports: a database port wired to the SQL port of Postgres, a storage port wired to the object API of an S3 bucket, a queue port wired to a Redis queue, and a repository port wired to the files of a Git repository.](images/ports.svg)

| What the code does | Required port (on the caller) | Provided port (on the other side) | Wire | Message |
|---|---|---|---|---|
| **Database access.** `s.query(Report)…`, `s.commit()` | `reports/api/port:db`, protocol SQL | `postgres/port:sql` on the Postgres entity. Inside Postgres, the tables `reports` and `orders` are stores | found from the SQLAlchemy pack rows; carries a transaction | one SQL statement: a read or a write of named columns |
| **Object store access.** `s3.put_object(Bucket, Key, Body)` | `reports/worker/port:storage`, protocol S3 API | `s3/port:objects` on the external S3 entity; the bucket is a store | found from the bucket name; HTTPS underneath | one `PutObject` with a key and a body; the answer is success or `ClientError` |
| **Queue access.** `generate_report.delay(id)` | `reports/api/port:enqueue-generate-report` | on the worker: the provided port `task:generate_report`. Redis sits between them as an entity with a queue | matched by task name; the channel may duplicate | one task message with the report ID |
| **Git repository access.** A tool runs `update-ref` or reads `.git/objects` | `tool/port:repo`, protocol "Git repository files" | on the repository entity: `repo/port:refs` and `repo/port:objects`. Refs and objects are stores | file system calls on a directory | "read ref `main`", "write object", "lock, then rename `main.lock`" |
| **An incoming HTTP request** | on the client: a required port for the call | `reports/api/port:post-reports`, protocol HTTP route | a delegation chain: host port → container port → server → route | `POST /reports` with a JSON body and an `Idempotency-Key` header |
| **A function call** | the call site | the function's parameters and return value form its provided port | the call | arguments in, a return value or an exception out |

In the Git example the repository is an entity even though it is "just a directory". It has a boundary (the files tools agree to touch), state (refs, objects) and rules (a ref changes through a lock file). Modeling access as a port is what lets two tools writing the same ref show up as a race.

**Related terms.**

- **Open port:** a required port that nothing in the model is wired to. Closing the model gives it a stub.
- **Exposure:** whether a provided port can be reached from outside the system (`external`) or only from inside (`internal`). It is decided from infrastructure files and framework rules, never guessed.
- **Slot:** a place in a framework's model where the framework calls your code. FastAPI owns the socket and the router; your handler fills the slot for `POST /reports`. Slots are how provided ports are found.

## State

**Store.** An entity that holds data which outlives a single interaction: a table, a queue, a bucket, a file, a cache.

**State variable.** The part of a store that behavior depends on. *Example:* `reports.status`, one value per row, read to decide whether to work and written in four handlers. A column that nothing branches on is data, not a state variable.

**Lifecycle machine.** The legal changes of one state variable: its values, the allowed moves between them, and the node that performs each move. It is derived from the write sites and the tests the code makes before them, so it states what the code relies on. *Example:* `PENDING → RUNNING → DONE`, `RUNNING → FAILED`, `RUNNING → PENDING` on retry. A write that is not a legal move is an **illegal write**.

**Mechanism.** A recurring arrangement that keeps a promise for other code: a lock, a connection pool, a retry policy. A *known* mechanism comes from a pack (Celery's retry policy). An *improvised* one is built by hand in the code and shows up as a move. *Example:* the worker reads `status`, checks for `PENDING` and writes `RUNNING`; the column is being used as a lock, called a **claim**, and the move's verdict says whether the claim is safe.

**Invariant.** A statement about state that must hold in every reachable state, written in the words of one level. *Example:* "a report is delivered at most once".

## Behavior

**Behavior.** What one provided port does when a message arrives. Every provided port has exactly one behavior. It exists at two levels of detail:

- **Summary:** a list of cases.
- **Full:** a graph of nodes joined by guarded edges, close to the statements of the code.

The summary must agree with the full graph; a check enforces it.

**Node.** One action inside a behavior: read a state variable, write one, send a message, reply, branch, loop, take a lock.

**Case.** One guarded path through a behavior: a condition (`when`), an ordered list of nodes and an outcome. *Example:* "the report is pending and the upload succeeds → write `RUNNING`, upload, write `DONE`, acknowledge".

**Predicate.** A named yes-or-no fact about one specific thing, tied to a test in the code. *Example:* `range-too-large` is `(req.end - req.start).days > 366` at `api.py:22`. A predicate is *about* a message field, a stored value, a config value or the answer of a port. Guards, input classes and assumptions are all formulas over predicates. A predicate that tests a state variable for one value (`status-is-pending`) says so with `equals`, so the tools know that two such predicates on one variable are never both true.

**Guard.** The condition on a case or an edge, written as a formula over predicates with `!`, `&`, `|` and parentheses. People read `!range-too-large & key-seen`; the stored form uses full IDs: `!reports/api/pred:range-too-large & reports/api/pred:key-seen`. A case with no guard is the *otherwise* case.

**Decides.** The predicates whose value becomes known at a node. *Example:* `upload-ok` is decided at the upload node, not before. The interpreter never uses a predicate's value before the node that decides it.

**Characteristic and input class.** A characteristic is one way the input of a port can vary, such as "the date range". Its input classes are the groups of values that behave differently: "at most 366 days", "longer than 366 days", "malformed". The model reasons about classes, never about individual values.

**Obligation.** A branch, an exception path or an outside effect that the extractors found in a handler's code. A compiled behavior must mention every obligation; otherwise the coverage check fails. Obligations are how the tools know an LLM-written summary left nothing out.

**Effect.** An action that reaches outside the function: a database read or write, a send, an HTTP call, a file access, a clock read.

**Evidence.** What the code does to make a group of nodes atomic: a transaction, a lock, a conditional update (`UPDATE … WHERE status = 'PENDING'`). Atomicity is computed from evidence, never declared.

## Levels

**Level.** One reader's way of describing the system, with its own steps and its own words. One step at a level stands for several steps at the level below. Going up answers *why*; going down answers *how*. The kinds of level, from the top: *outcome*, *goal*, *activation*, *move*, *effect* (one read, write or send); *stage* and *exchange* are admitted only where they add something. Opening a service to see its components is not a change of level: it is space zoom.

**Closing rule.** The rule that says when one step of a level ends, stated over things the model already has. *Example:* a goal closes when the actor's request is answered and the subject is at rest; an activation closes when it must wait.

**Move.** A group of nodes that makes one change to shared state; it closes at the write. *Example:* in `generate_report`, "read the row, test `PENDING`, write `RUNNING`" is one move, the claim. A move is identified by its function and the order of its closing site in it.

**Verdict of a move.** *Atomic:* its reads and write can be reordered into one block nothing else can enter. *Guarded:* the write is conditional on what was read, inside the store (`UPDATE … WHERE status = 'PENDING'`). *Non-atomic:* it reads, decides in the program, then writes unconditionally. Non-atomic moves are where races live.

**Subject.** A thing with identity and a lifecycle that the system works on: a report, a job. Outcomes and goals are stated about subjects.

**Step class.** A kind of step at one level, identified by code names, so that any run can be shown at any level. *Example:* the goal "request a report" is the actor *client*, the port `POST /reports` and the change "a `PENDING` report exists".

**Step map.** The record of which lower step classes make up each upper step class. *Example:* the activation "generate the report" is made of the moves "claim", "upload" and "mark done".

**Support step.** A lower step that nothing on the way to the outcome reads, such as a worker's heartbeat. Support steps are folded behind a count, never deleted.

**Charter.** What a level is for: its readers, its question, what one step is, what it hides, and its neighbours above (*why*) and below (*how*).

**Cut.** What a view shows of one run: a set of steps that covers the run once, high everywhere and opened where something is wrong. "The goal level" is the cut that takes every goal.

**Ladder and ladder strategy.** The ladder is the levels of one model with their charters, step classes and step map. A ladder strategy is a way of building it. Version one builds one strategy, `closing-rules`; the others were measured and not chosen ([ladder strategies](research/multi-level-models/ladder-strategies.md)).

**Vocabulary.** The words that may be used at one level. *Example:* at the goal level, "report", "requested", "delivered"; never "Redis".

**Abstraction barrier.** The rule that a name or a step at one level uses only that level's vocabulary. A violation is a finding.

**Abstraction function.** The rule that computes upper-level state from lower-level state. In version one a subject's upper state is its lifecycle state variable.

**Term.** A word of the project's own language, with a definition and links to the code and docs that use it. All terms together are the project's glossary, from which every level takes its names. *Example:* `report` is "a CSV of orders for a date range", anchored to the `Report` class and the route `POST /reports`.

**Assumed level.** A level beneath the effect that the repository does not contain: the inside of a library, a runtime, the operating system. The model does not discover it. A pack stands for it and states where it leaks. *Example:* "read a file" stands for system calls and page-cache lookups; a cache miss changes only the timing, so the step holds.

## How a record is known

**Anchor.** The link from a record to the code that justifies it: file, line range and the quoted text. A record without an anchor cannot be shown as fact.

**Provenance.** How a record is known. Five values:

| Value | Meaning | Example |
|---|---|---|
| `inferred` | derived from the source at this commit | the route `POST /reports` |
| `attested` | supported by declarative evidence other than source | an image digest pinned in a manifest |
| `assumed` | a default standing in for something unknown | "S3 answers within the timeout" |
| `observed` | seen in a real execution | a log line matched to a step |
| `hypothetical` | a design idea that is in no code | "add a load balancer in front of the API" |

**Grounded.** A record is grounded when it is anchored to evidence about the real system: source, build files, lockfiles. The model is a simulation of the running system, and these are the observations that keep it close to it. Where evidence is missing, the model has a gap and says so.

**Source.** Who produced the record: an extractor, a pack, a person, or an LLM (with a link to the exact call in the call ledger).

**Checks.** The list of checks a record has passed.

**Semantic ID.** A record's identity, written as a path of what the thing is: `reports/worker/beh:generate-report`. IDs are built from names in the code (symbols, routes, task names, tables), never from positions and never by the LLM, so they stay the same across rebuilds. Each ID has exactly one text form, and stored records always use the full ID; short forms such as `beh:generate-report` appear only on screen.

**Assumption.** A record stating that a predicate was given a value by default, with the reason and the alternatives. Flipping an assumption re-runs with the other value.

## Building

**Stage.** One of the six parts of a build, in order: build model, facts, mechanical model, LLM, link-load-close, levels.

**Extractor.** A deterministic program that reads build files, code or infrastructure files and produces facts: the build model, reference sites, the call graph, branch points, effects, entry points. Extractors never guess.

**Reference site.** A place where the code names something: an import, a call, a decorator, a base class, the receiver of an attribute write, a function handed to outside code. Every site gets exactly one outcome.

**Outcome of a site.** *Resolved:* the target is a definition in the repository. *External:* the target is outside the repository and has an identity, such as `sqlalchemy.orm.Session.commit`. *Unresolved:* the type checker cannot say, with a reason from a short closed list.

**Gap.** Something unknown that could change the model: an unresolved site, a call into a library with no pack, a missing input such as an environment. A gap *counts* when it lies in code an entry point can reach, or is a decorator, base class or escape.

**Gate.** The point after the facts and before the first LLM call where a build with counting gaps fails, one diagnostic per root cause, each with its fix. An *acceptance* in `mentalc.toml` turns a gap into a visible assumption.

**Mechanical model.** The model as it stands after stage 3, from facts alone: entities, ports, full behaviors, predicates with the code's own expressions, machines, moves with verdicts. It is complete and hard to read. The interpreter runs it; the LLM only names and summarizes over it.

**Pack.** A ready-made model of a library, framework, runtime or outside system. Its behaviors use the same record format as the model of your code; what the extractor reads from it (types, effects, slots, raised exceptions) is written as rows in CodeQL's models-as-data style. A pack says what `session.commit()` does, how Celery retries, how Postgres behaves. Packs are data and never execute.

**Compile unit.** The piece of work given to the LLM for a summary: one provided port, with its full behavior, its obligations, pack summaries of what it calls and the relevant glossary terms.

**Task and answer.** A task is one request to the LLM written by `mentalc`: its role, its prompt, its inputs and the exact shape of the expected answer. An answer is the records that come back. An answer becomes part of the model only after it passes the checks.

**LLM backend.** Whatever turns a task into an answer: the user's coding agent, an API, or *replay*, which answers only from stored answers and is what tests use. The backend is a setting of `mentalc`.

**Call ledger.** The log of every LLM call made during builds: role, model, prompt template with its version and hash, the task and answer hashes, checks passed. Each LLM-written record points to its call.

**Repair.** A follow-up task sent when an answer fails a check, naming exactly what failed.

**Link, load, close.** *Link* connects required ports to provided ports and fills slots. *Load* creates one instance per entity. *Close* adds actors, drivers and stubs so the model can run on its own.

**Build model.** A small model of how a project is built, made before anything else (like Maven's `pom.xml`): its build units, each with its source roots, Python version, pins and environment. *Example:* report-jobs has one build unit, named after its project, with a pin for each package in `uv.lock` other than the project itself. It is stage 1 of every build and is made by plain readers of build files.

**Build unit.** One part of a project that is built on its own: a directory with a `pyproject.toml`. Each of its packages at an exact version is a **pin**. Not to be confused with a *component*, which is a box inside a service.

**Build-tool model.** *Later.* A pack for the build layer: a model of one build tool at one version (uv, pip), with its concepts and the rules that turn build files into a build model. Written by an LLM from the tool's documentation, and accepted only after it agrees with the real tool's own output on a corpus.

**Deployment space.** *Later.* The ways the same build can be deployed: which services run, how many instances of each, which configuration values are set. A *deployment configuration* picks one of each, and `mentalvm` can run any configuration.

**Driver and stub.** A driver is a generated actor that can send any input class to a port open to the outside. A stub is a generated stand-in that answers a required port leading outside the model with any of its declared outcomes.

**Diagnostic.** One reported item from a build: a failed check, a gap, an assumption made, a finding. Each has a stable code, such as `environment-missing`, and the code fixes whether it is an error or a warning.

**Reference model.** The hand-written records of an example system, in `examples/<system>/model/`, that a build is compared against. It is a specification, never rewritten by a tool, and it grows by milestone: each milestone writes, before its code, the records of the kinds it is the first to produce. *Example:* milestone 1 writes report-jobs' build unit, pins and sites.

## Running

**Interpreter.** The program that runs the model one step at a time (`mentalvm`). It never reads source code.

**Configuration.** The complete state of a run at one moment: which instances exist, the value of every state variable, held locks, the contents of every channel, every activation and where it stands, timers, and the conditions assumed so far.

**Step.** One thing happening in a run: one rule of the interpreter fires once. A step has a kind (deliver, receive, branch, read, write, send, reply, timeout and so on) and becomes one entry in the trace. A *node* is part of a behavior; a *step* is an event in a run.

**Activation.** One execution of one behavior, started by one message. It runs until it must wait. Several activations exist at once; in a picture each is a **lane**.

**Choice.** A point where more than one thing could happen next: which activation moves, whether a message is duplicated or lost, what an outside service answers, which class an unbound input belongs to.

**Scheduler and schedule.** The scheduler makes every choice. The schedule is the recorded list of the choices made. Replaying a schedule reproduces the run.

**Binding.** What a run fixes before it starts: the input classes, the values of predicates the scenario pins down, and the selected variants.

**Edit.** A change made to the model or the state for one run only, such as flipping an assumption, setting a state variable or adding a hypothetical entity. Edits are how "what if" questions are asked.

**Strategy.** How the choices of a run are made. *Concrete:* everything is bound, one trace. *Bounded-exhaustive:* every schedule within stated bounds is tried. *Replay:* a stored schedule decides. Symbolic and sampled runs are Later.

**Bounds.** The limits of an exhaustive search: number of requests, activations of a port at once, duplicated messages, preemptions, depth. Results always state their bounds.

**Fork.** A choice shown as a branching point in a trace; the side not taken can be taken later by flipping it.

**Witness.** A shortest run found by search that reaches a bad state, with its schedule, so it can be replayed. *Example:* the run in which two activations of `generate_report` both read `PENDING` before either writes `RUNNING`.

**Stuck outcome.** Why a run cannot continue: a **model gap** (a message nothing handles), a **deadlock**, an **illegal write**, or a **zero-time cycle**.

**Bound exceeded.** The result when one of the tool's own hard caps is hit (search depth, minimizer rounds, a fixpoint). It says nothing about the model and is never reported as "none within these bounds".

**Footprint.** What a node or step reads and writes.

**Atomicity.** Whether a group of steps can be treated as one. Computed from footprints and evidence, never declared. For a move it is the move's verdict, *atomic*, *guarded* or *non-atomic* (see *Verdict of a move* above and [spec 8.1](specification.md#81-moves-and-verdicts)); in a non-atomic one, another activation can get in between.

**Race.** Two activations touch the same state, at least one writes, and no evidence orders them. A non-atomic check-then-act is the commonest form.

**Macro step.** One step that stands for several smaller ones: a whole activation, a summary case, a collapsed segment. A macro step must agree with its expansion.

**Replay verdict.** The result of replaying a stored schedule on a changed model, per choice: *matched*, *infeasible* (the choice no longer exists) or *diverged*.

**Property.** *Later.* A statement about every run that a search or a prover checks, written in one level's words. *Safety* says nothing bad happens ("at most one worker holds a report"); *liveness* says something good eventually happens ("every requested report ends `DONE` or `FAILED`"). Some properties follow from the model's form and need nobody to write them: no deadlock, no illegal write, no message nothing handles. The others are `property` records, proposed by the LLM or written by a person, and accepted by a person. An *invariant* is a safety property about state.

**Fairness.** *Later.* An assumption that the scheduler does not starve a step forever, needed to check liveness. *Weak:* a step that stays enabled eventually happens. *Strong:* a step that is enabled again and again eventually happens. *Example:* without weak fairness on the workers, "every report ends" fails because a worker may simply never run.

**Over- and under-approximation.** *Later.* The direction in which a record may differ from the code. A record that *over-approximates* allows everything the code can do, and possibly more: a predicate left as a free choice where the code links it to another. One that *under-approximates* may allow less: two code steps fused into one, so that nothing can run between them. A "holds" result carries over to the code only if every record it rests on over-approximates or is exact; a violation carries over only if its steps are possible in the code.

**Verdict of a property.** *Later.* *Violation*, with a witness that is *model-only* or *reproduced* in the code; or *holds*, with an *extent* (*bounded*, with the bounds, or *unbounded*, with the prover) and a *basis* (*sound* if every record it rests on over-approximates or is exact, otherwise *unsound*, naming the records). Only *holds, unbounded, sound* is called a **proof**, and always with its assumptions.

**Inductive invariant.** *Later.* A property that holds in the initial state and that every step preserves. It is how a property is proved for runs of any length and any number of instances, which no bounded search can do. *Example:* "for each report, at most one activation is between claim and mark", once the claim is a compare-and-set.

**Cutoff.** *Later.* A theorem that checking a small instance of a system settles every size ("two processes suffice"). It holds only for the class of systems the theorem covers.

## Showing

**Session.** One loaded model with its runs, breakpoints and focus, held by `mentaldbg`. The viewer, the coding agent and the terminal all attach to the same session.

**Focus.** What is currently selected in a session: an entity, a step, a field. The focus is what the agent receives as context.

**View state.** Everything about how a run is shown and nothing about what it computes: zoom, collapsed parts, pins, the active lens. View state never changes a model or a run.

**Zoom axes.** Four independent ways to see more detail. *Space:* open an entity. *Time:* expand a macro step. *Layers:* open a message's nesting. *Data:* go deeper into one message's content.

**Collapse.** Hiding a group of steps or entities behind one item. Collapse never deletes anything and may never hide an error, an assumption or a non-atomic group.

**Lens.** A rule that ranks what matters in a view, such as "fields that decide a branch first".

**Skin.** How entities and events look. Layout does not belong to the skin, so changing skins moves nothing.

**Breakpoint and watchpoint.** A breakpoint stops a run when a condition over state becomes true. A watchpoint stops it when a state variable changes.

## Tools and products

| Term | Definition |
|---|---|
| **`mentalc`** | The compiler. Reads a repository and packs, writes a complete model. Asks an LLM backend for judgment and checks every answer. Pronounced "mental-see". |
| **`mentalvm`** | The interpreter. Reads a model, produces runs: traces and findings. |
| **`mentaldbg`** | The debugger. Holds sessions and serves the viewer, coding agents (MCP), IDEs (DAP) and the terminal. |
| **Viewer** | The graphical front end of `mentaldbg`: a web application that draws the model and plays traces. |
| **Mental** | The product family built on the three tools: *Mental for Claude Code*, *Mental for Codex*, *Mental Desktop*. |

## Words with several meanings

| Word | Meaning in Mental | Not this |
|---|---|---|
| **Port** | an interaction point on an entity's boundary | a TCP port number |
| **Level** | one reader's way of describing the system: outcome, goal, move | a message layer; a box opened inside another (that is space zoom) |
| **Layer** | one nesting of a message: HTTP in TLS in TCP | a level |
| **Step** | one event in a run | a node (a part of a behavior) |
| **Model** | the compiled description of the system | an LLM (always called "LLM" or "LLM model ID") |
| **Trace** | the output of a run in the interpreter | a distributed-tracing span; a real execution is called an *observed trace* |
| **Task** | a request from `mentalc` to an LLM | a Celery task (always "Celery task") |
| **Check** | a rule that an answer or the model must pass | a test in the repository |
| **Stage** | one of the six parts of a build | the *stage* kind of level (a subject moving to its next stage) |
