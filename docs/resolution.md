# Mental: reference resolution and dependencies

**Status: Proposed.** Nothing here is built.

This document gives the detail under [section 6 of the specification](specification.md#6-stage-2-from-code-to-facts): what `mentalc` does when it cannot tell what a name in the code refers to. That can be an import that leads nowhere, a call on a value of unknown type, a library with no pack, or a dependency with no exact version. Individual sites are resolved by a type checker; this document covers the inputs it needs, what counts as a gap, when a build fails and how an accepted gap enters the model.

Evidence is cited as **F-U numbers**, defined with their sources in [findings, section U](findings.md#u-unresolved-references-and-dependencies). The research behind them is in [`research/unresolved-references/`](research/unresolved-references/report.md).

Each section carries a status, as in the specification: **Proposed** (the working answer), **Experiment** (competing designs kept side by side, to be settled by measurement), **Later** (outlined, not version one), **Open** (not worked out).

## Contents

1. [The problem](#1-the-problem)
2. [Principles](#2-principles)
3. [Terms](#3-terms)
4. [Reference sites and outcomes](#4-reference-sites-and-outcomes)
5. [External references](#5-external-references)
6. [Where resolution evidence comes from](#6-where-resolution-evidence-comes-from)
7. [Packages, pins and packs](#7-packages-pins-and-packs)
8. [Gaps](#8-gaps)
9. [The gate: when a build fails](#9-the-gate-when-a-build-fails)
10. [Assumptions: how an accepted gap enters the model](#10-assumptions-how-an-accepted-gap-enters-the-model)
11. [The resolution report](#11-the-resolution-report)
12. [Execution and trust](#12-execution-and-trust)
13. [Experiments](#13-experiments)
14. [Other ecosystems](#14-other-ecosystems)
15. [Tests](#15-tests)
16. [Open questions](#16-open-questions)

---

## 1. The problem

**What goes wrong.** Almost every structural record of the model sits downstream of name resolution:

| Record | Exists only if |
|---|---|
| a provided port (entry point) | a decorator or registration call resolves to a pack slot |
| an effect | a call's receiver resolves to a type that a pack row describes |
| a state variable's write site | an attribute write's receiver resolves to a stored class |
| a call edge, and so a callee summary in a compile unit | the callee resolves to a function in the repository |
| a store | a class's base resolves to a pack type |

When one of these lookups fails, nothing fails loudly. The port, effect or write site is simply absent, the model looks cleaner than the code, and a search reports "none within these bounds" about a system that is missing a writer. A missing call edge can at least be counted. A missing write site hides exactly the races Mental exists to find.

**What the evidence says.**

- **Projects do not reliably build unattended.** Of 7,264 GitHub Java projects, 38% failed their default build in 2016 and 59% in a 2020 replication, with dependency resolution the largest cause (F-U1). So an extractor must not need a build.
- **A full build does not close the gap either.** Static call graphs built with everything available recover 70% of real call edges in Python and miss about 12% of reachable methods in Java (F-U3). Unresolved references are a property of static analysis, not of a missing environment.
- **The tools that need precision run the build tool.** IDEs and compiler-based indexers get their accuracy by executing build scripts, behind a trust prompt and with a human choosing the toolchain (F-U18). Running the build tool executes code the repository controls (F-U14).
- **The tools that avoid the build mark the hole and carry on.** They write a placeholder, a local symbol, an unknown type (F-U6). Only CodeQL measures how much it failed to resolve, and it only warns (F-U5). No surveyed tool fails a build on it, and none aimed at coding agents reports it at all.
- **Library source is not needed to name a library reference.** CodeQL's Python analysis stopped extracting dependencies in 2024, and the standard library later that year, and refers to library code by import path with declarative models. GitHub reports minimal effect on results (F-U4). Mental's packs already work this way.

**What follows for Mental.** The question is not how to resolve everything. It is how to make sure that nothing unresolved is silent, that the remedy is named, and that the build executes nothing to get there.

## 2. Principles

Later sections refer to these as R1 to R8.

### R1. Every reference site has exactly one outcome

Every site where the code names something gets one of three outcomes: `Resolved`, `External` or `Unresolved` with a reason. Nothing is dropped. This extends the conservation rule (X8) from calls, decorators and branches to every site kind in [section 4](#4-reference-sites-and-outcomes).

### R2. No gap is silent

**The closure property:** every unresolved site that could hide a port, an effect, a write site or a call edge is either reported as a gap or lies in code that no entry point can reach unless a reported gap intervenes. This is what "complete" means for a model: not that every reference resolved, but that **every reference is accounted for**.

### R3. `mentalc` executes nothing the repository controls

No build tool, package manager, interpreter or script. Resolution reads files as data. When only an executing tool can supply a missing input, `mentalc` names the command and stops; it does not run it ([section 12](#12-execution-and-trust)).

### R4. Evidence has a rank, and every fact says which rank it has

What is written in a committed file outranks what happens to be installed on this machine, which outranks a default. `mentalc` never fills a hole with the most likely answer.

### R5. Version classes, not version values

A dependency's exact version matters to the model only where it selects between packs. The model reasons about classes of versions, as it reasons about classes of input (P3). A version that slides inside one pack's range changes nothing.

### R6. Report root causes, not sites

Three hundred unresolved calls into one unknown package are one problem with one fix. Gaps are grouped by cause and ranked by what they put at risk.

### R7. A gap is an input problem first

The first answer to a gap is a missing input that someone can supply: a lockfile, an environment, a pack, an annotation, a checked-out submodule. Only when that is not possible may a gap be accepted, and then it becomes an explicit assumption record in the model, authored by a person or their agent in a committed file.

### R8. The LLM may propose a resolution; it may never dismiss a gap

An LLM-proposed target or type counts only after a deterministic check re-derives it from the code. An LLM never declares a reference unimportant, a package harmless or a decorator inert. When an LLM filters static findings it loses 6 to 46% of true ones (F-U24, F-R3).

## 3. Terms

These would enter the [glossary](glossary.md) on acceptance.

| Term | Definition |
|---|---|
| **Reference site** | One place in the code that names something whose identity the model depends on: an import, a call's callee, a decorator, a base class, the receiver of an attribute write. |
| **Outcome** | The result of resolving a site: `Resolved` (a definition in the repository), `External` (a named thing outside it) or `Unresolved` (with a reason). |
| **External symbol** | The identity of something outside the repository. In milestone 1, its package plus the module and qualified name where the type checker found the definition, such as `sqlalchemy.orm.session` and `Session.commit` ([section 5](#5-external-references)). |
| **Package** | One unit of outside code with a version: a distribution from a registry, a git repository at a commit, the language's standard library. |
| **Pin** | The version or commit a package is fixed to, together with how that is known: `declared`, `observed`, `ranged` or `unpinned`. |
| **Package class** | What Mental knows about a package's behavior: `modelled` (a pack describes it), `inert` (declared to have no effects and no entry points) or `unknown`. |
| **Import root** | A directory whose children are importable top-level modules. |
| **Gap** | An unresolved site, or a site in an `unknown` package, that could change the model. |
| **Root cause** | The one missing thing behind a group of gaps: a package, a module, an untyped binding, a dynamic construct. |
| **Acceptance** | An entry in an authored, committed file stating that a root cause is accepted as it is, with a reason. It produces assumption records. |
| **Opaque call** | The model's form of a call whose target is outside the model and has no pack: a call node wired to a stub, with an assumption attached. |
| **Resolution report** | The counts of sites by outcome and the list of root causes, for a whole repository and for each module. |

## 4. Reference sites and outcomes

**Status: Proposed.**

### 4.1 Site kinds

The extractor emits one fact per site. The **role** is what the site can add to the model, and so what an unresolved one can hide.

| Site kind | Example | Role: unresolved, it can hide |
|---|---|---|
| **import** | `from app.db import Session` | everything reached through the name |
| **decorator** | `@app.post("/reports")` | a provided port |
| **callee** | `s.commit()` | an effect, a call edge, a raised exception |
| **base class** | `class Report(Base)` | a store, inherited members, a framework slot |
| **write receiver** | `report.status = …` | a state variable's write site |
| **escape** | `register(handler)`: a repository function passed to an outside call, or a method overriding one from an outside base class | an entry point called from outside (an inverted edge) |
| **handler type** | `except ClientError:` | which exception paths a handler takes |
| **key** | a route path, a task name, an environment variable name | a wire, a config binding |

The first five are the dangerous ones. **Escapes** are easy to overlook: when repository code hands a function to a library, the library may call it later, and that call exists in no repository source. Under the separate compilation assumption a library can reach repository code only through what the repository passes to it or derives from it, so escapes are the complete list of places where an outside call into the repository can start (F-U7).

### 4.2 The three outcomes

| Outcome | Means | Carries |
|---|---|---|
| `Resolved(id)` | the target is a definition in the repository | the definition's ID |
| `External(symbol)` | the target is outside the repository and has an identity | the external symbol ([section 5](#5-external-references)) |
| `Unresolved(reason)` | the extractor cannot say what the target is | a reason from the closed list below, and the site's shape: the name as written, the number of arguments, the receiver expression |

`External` is a full answer about **identity**. Whether Mental knows what the external thing **does** is a separate question, answered by the package class ([section 7](#7-packages-pins-and-packs)). This split replaces the `no_pack` reason of X7: a call into a library without a pack is not an unknown target, it is a known target of unknown behavior.

### 4.3 Reasons

A closed list. Each reason belongs to one of three groups, and the group decides the remedy.

| Group | Reason | Means | Remedy |
|---|---|---|---|
| **Missing input** | `missing_module` | an import names no module in any import root, the standard library or any known package | supply the input: fix the roots, check out the submodule, add the pin or environment |
| | `unparsed` | the target lies in a file region the parser could not recover | fix the file, or set the Python version |
| **Outside the rules** | `no_type` | the receiver has no type that its own statement gives it: an unannotated parameter, an element of an untyped container | an annotation |
| | `not_found` | the module or class is known and does not define the name | usually a real defect or generated code; inspect |
| | `ambiguous` | several definitions can be meant: a conditional definition, a `try`/`except ImportError` pair whose modules both exist (one that is missing is a guarded import) | an acceptance that picks one, or both become candidates |
| | `not_converged` | the bounded fixpoint over module-level bindings ran out | raise the bound; report as a tool limit |
| **Dynamic by construction** | `dynamic_attr`, `dynamic_import`, `eval`, `wildcard_import` | the code computes the name at run time | an acceptance, which turns the site into an assumption |

Two rules apply to every reason:

- **One root, many followers.** An unresolved site whose only problem is an earlier unresolved site carries a `cause` naming that site. Reports count roots. In `x = unknown(); x.save(); x.close()` there is one root and two followers.
- **Candidates are never edges.** When the repository defines exactly the name being looked up in one or more places, the extractor records them as `candidates` on the unresolved fact. A candidate is input for a later, checked resolution step. It is never an edge, because name matching links unrelated same-named definitions (F-U6).

## 5. External references

**Status: Proposed.**

Milestone 1 identifies an external symbol by where the type checker found its definition: the package (a pin, or the standard library), the module and the qualified name, such as `sqlalchemy.orm.session` and `Session.commit` (owner choice, 2026-11-20). Building an access path from the import needs value tracking of Mental's own, which X1 and X9 rule out, so the form below waits for the packs, which are written in it. A library that moves a definition between its modules changes the symbol across versions.

### 5.1 Identity by access path (later)

An external symbol is a **package** plus an **access path** that starts at an import and uses the pack grammar of X11:

```text
sqlalchemy.orm            Member[sessionmaker].ReturnValue.ReturnValue     an instance made by sessionmaker(...)()
fastapi                   Member[FastAPI].ReturnValue.Member[post]         app.post, for app = FastAPI()
celery                    Member[Celery].ReturnValue.Member[task].Decorates
```

The path is what the code does to the imported name: take a member, call it, subclass it. It is built from repository source alone. No library file is opened, so an external reference has an identity whether or not the library is installed (F-U4).

**How paths grow.** Starting from an import, each step the code takes extends the path: attribute access adds `Member[name]`, a call adds `ReturnValue`, subclassing adds `Subclass`, instantiating adds `Instance`. A pack `type` row may rename a path to a type (`sqlalchemy.orm.Session`), after which rows for that type apply. A path that reaches a repository definition again, because a pack row says "returns an instance of argument 0" and argument 0 is a repository class, becomes `Resolved`.

**Where a path stops.** A path is an identity only while each step is one the extractor may take under X9. When a value's origin is lost (it comes from an untyped parameter, a container, an unresolved call), the site is `Unresolved(no_type)`, not a longer path.

### 5.2 The package part

The package is written as a [package URL](https://github.com/package-url/purl-spec) without its version: `pkg:pypi/sqlalchemy`. The version lives on the pin ([section 7.2](#72-pins)), so an external symbol's identity does not change when a version does (P9). The standard library is the synthetic package `pkg:generic/python-stdlib`, as scip-python does (F-U20). Package URLs are the one identifier that CycloneDX, SPDX and lockfile tools all embed (F-U20), which keeps the door open to linking models of several repositories later ([section 14](#14-other-ecosystems)).

### 5.3 From import name to package

`import yaml` names a module; the package is `PyYAML`. The mapping comes from evidence, in this order:

1. a pack that declares the import names it covers;
2. the observed environment: the installed distribution whose `RECORD` or `top_level.txt` provides the top-level name ([section 6.3](#63-rung-2-the-observed-environment));
3. no mapping. The import is `External` with package `unknown-distribution` and the top-level module name, and it raises a gap that asks for an environment or a pack.

Equality of names (`import requests` is distribution `requests`) is **not** evidence: it is a guess that happens to be right often.

## 6. Where resolution evidence comes from

**Status: Proposed.** Rungs 0 to 2 are what stage 1 of the build, the build model, reads.

Evidence is gathered in rungs. Each rung only reads files. A higher rung never overrules a lower one; it fills what the lower ones left open.

| Rung | Source | Yields | Evidence rank | Executes | Network |
|---|---|---|---|---|---|
| 0 | the repository snapshot | import roots, modules, Python version, standard-library module list | from source | no | no |
| 1 | committed lockfiles and manifests | pins | `declared` | no | no |
| 2 | an environment already on the machine | pins, import-name mapping | `observed` | no | no |
| 3 | packs | package classes, type, effect and slot rows | pack | no | no |
| — | anything that runs a build tool or package manager | a lockfile or environment that does not exist yet | — | **yes** | usually |

The last row is not a rung. `mentalc` never takes it. It names the command and fails ([section 12](#12-execution-and-trust)).

Individual sites are answered by a type checker set up from these inputs ([specification, section 6](specification.md#6-stage-2-from-code-to-facts)), not by a resolver of Mental's own.

### 6.1 Rung 0: the repository

**Import roots.** A module name is a path relative to an import root, so the roots decide what every import means. The roots are those of the two standard project layouts: `src/` when the project has one, else the project directory.

A directory with Python files is a package whether or not it has an `__init__.py`. An absolute import must resolve under exactly one root: under none it is a `missing_module` candidate (the other rungs may still claim it), under two it is `ambiguous`.

*In report-jobs,* the `app/` folder has no `__init__.py` and lives under `src/`, not the project directory. `src/` is the root by the standard layout. Without it every `from app.db import …` in the example is a missing module.

**Python version.** Taken from a `.python-version` file, else from the environment's `pyvenv.cfg`. `requires-python` is a range and is never used as the version. It selects the parser's target and the standard-library module list.

**Standard-library modules.** A table embedded in `mentalc`, generated from typeshed's `stdlib/VERSIONS` file, which lists each module with the Python versions that have it (F-U11).

**Submodules, subtrees, vendored code.** A git submodule's directory is part of the snapshot when it is checked out, and its code is repository code. When `.gitmodules` names a path whose directory is empty, every import into it fails for a nameable reason, and the gap says so with the fix (`git submodule update --init`). The pin of a submodule is in the superproject's tree; a subtree leaves its pin only in commit messages; vendored copies have none (F-U19). `mentalc` reads `.gitmodules` as text and does not run git: git's own manual says running it inside an untrusted repository is unsafe (F-U15).

### 6.2 Rung 1: declared pins

A pin is `declared` when a committed file states it exactly.

| File | Read as | Note |
|---|---|---|
| `pylock.toml`, `pylock.*.toml` | the specified format | the only tool-neutral, specified lock format (PEP 751, Final) |
| `uv.lock`, `poetry.lock`, `pdm.lock`, `Pipfile.lock` | each tool's own format, by its schema version | none has a published schema; uv states its format is not stable (F-U10) |
| `requirements*.txt`, `*.lock` in requirements syntax | only lines that pin: `name==version`, or a URL with a commit | the format allows ranges, includes and options; a line that does not pin yields a `ranged` or `unpinned` package |

Milestone 1 reads `uv.lock` only.

Rules:

- **An unknown schema version is an error** naming the file and the version, never a best-effort read (G11).
- **Git-sourced packages are pinned by commit.** Every lock format above records the resolved commit (F-U10), so a package from GitHub is as exact as one from a registry.
- **Several lockfiles that disagree** raise one gap naming both files.
- **A lock may hold several versions of one package** for different platforms. The pin is then the set, and the version-class rule of [section 7.3](#73-from-pin-to-pack) is applied to the whole set.
- **A package imported but absent from the declared pins** is `unpinned`, however sure its presence at run time. *Example:* code imports `botocore.exceptions`, and a requirements file pins `boto3` but not `botocore`: the file is a list of direct dependencies, not a full lock. A `uv.lock` pins every package, so this cannot happen in report-jobs.

How often this rung has anything to read depends on the ecosystem: 84 to 86% of popular Poetry and Pipenv projects commit their lockfile, and nearly every Go project, but under 1% of Gradle projects (F-U2).

### 6.3 Rung 2: the observed environment

An environment that already exists on the machine is evidence of what is installed. `mentalc` reads it; it never creates or changes one.

- **Where.** `.venv` in the project, which is where uv and PDM put it by default. Poetry and pipenv keep environments outside the project (F-U11); those are not read in milestone 1. `mentalc` does not search the machine.
- **What is read.** Per installed distribution: the name and version from `METADATA`, the top-level import names from `RECORD` (and `top_level.txt` where present, which is common but in no standard), and `direct_url.json`, which records the commit of a package installed from git (F-U11).
- **What is not done.** `.pth` files are not interpreted and no interpreter is started. An editable install that works through an import hook cannot be followed by any static tool (F-U11); its source is resolved through the import roots if it lies in the repository, and is a gap otherwise.
- **Rank.** A pin from the environment is `observed`. When a `declared` pin exists for the same package and differs, the declared one stands and a warning reports the drift.
- **Determinism.** The environment is an input of the build. What was read from it (names, versions, import names) is recorded with the build, so the same repository with the same recorded environment gives the same bytes. No timestamp is stored (G8).

### 6.4 Rung 3: packs

Packs supply everything about behavior. For resolution they add three declarations ([section 7.4](#74-what-a-pack-declares-for-resolution)): which import names a package provides, which versions the pack was written for, and whether the pack is closed.

## 7. Packages, pins and packs

**Status: Proposed.**

### 7.1 Package classes

Every package the repository references is in one class.

| Class | Means | A call into it becomes | An escape into it becomes |
|---|---|---|---|
| **modelled** | a pack describes it | the effect, slot or type the matching row gives; with no matching row in a **closed** pack, nothing | a port, if a slot row matches; nothing in a closed pack otherwise |
| **inert** | declared to have no effects on anything the model tracks and to call nothing back | nothing | nothing |
| **unknown** | neither | a gap | a gap |

The class is what turns thousands of sites into a short list. A repository uses tens of packages; each is classified once.

**Closed and open packs.** A pack is **closed** when its author states that its rows list every effect and every slot of the package, so anything else in the package does nothing the model tracks. A pack that is not closed covers only what its rows name, and a call that matches no row is a gap. Closedness is a claim, and it is recorded as one: records that rest on "no row matched in a closed pack" name the pack as their source.

**Inert packages** come from two places: a pack whose whole content is the statement that the package is inert (most of the standard library, pure computation libraries), and an acceptance in the project's own configuration ([section 10](#10-assumptions-how-an-accepted-gap-enters-the-model)).

### 7.2 Pins

Each referenced package gets one line in `.mental/deps.jsonl`:

```json
{"package":"pkg:pypi/sqlalchemy","pin":"declared","version":"2.0.58","source":"registry",
 "evidence":{"file":"uv.lock","quote":"version = \"2.0.58\""},
 "imports":["sqlalchemy"],"imports_from":"pack","class":"modelled","pack":"sqlalchemy@2"}
{"package":"pkg:pypi/botocore","pin":"declared","version":"1.44.6","source":"registry",
 "evidence":{"file":"uv.lock","quote":"version = \"1.44.6\""},
 "imports":["botocore"],"imports_from":"pack","class":"modelled","pack":"botocore@1"}
```

| `pin` | Means | Provenance of what follows from it |
|---|---|---|
| `declared` | an exact version or commit in a committed file | `attested` |
| `observed` | an exact version in the environment on this machine | `observed` |
| `ranged` | only a range is stated, such as `>=2.0,<3` | by the version-class rule below |
| `unpinned` | nothing states a version | by the version-class rule below |

`source` is `registry`, `git` (with the URL and commit), `url`, `path` or `stdlib`. The file is build output beside the model, like `packs.lock`. It is not a set of model records: removing it would not change what the interpreter computes (P10).

### 7.3 From pin to pack

A pack states the range of versions it was written for. Let *P* be what is known about the package's version: one version, a set of versions, a range, or everything.

| Situation | Result |
|---|---|
| *P* lies inside one pack's range | the pack applies; the pin's rank passes on to the records |
| *P* is a range or unknown, and every pack for the package behaves the same over *P* | the pack applies; nothing depends on the missing pin |
| *P* is a range or unknown and crosses a boundary between packs | a gap: the version decides behavior and is not known |
| *P* is exact and outside every pack's range | a gap: the pack was not written for this version |
| the package comes from `git`, `url` or `path` and the pack was written for the registry release | the pack applies with a warning that names the source: a fork may differ |

This is R5 at work. **A sliding version is a problem only when it slides across a pack boundary.** A Maven `SNAPSHOT`, a `latest` image tag or an unpinned requirement needs no resolving at all when one pack covers every version it can mean. When it does cross a boundary, the gap names the two packs and the missing pin, and the fix is the pin.

Exact pins are still recorded whenever they are known, for three later uses: showing the user what was assumed, generating packs from pinned artifacts, and linking repositories by identity.

### 7.4 What a pack declares for resolution

In addition to its rows and records (specification, section 7):

| Field | Meaning |
|---|---|
| `package` | the package URL it describes |
| `imports` | the top-level import names the package provides |
| `versions` | the version range the pack was written for |
| `class` | `modelled` or `inert` |
| `closed` | whether the rows list every effect and slot (modelled packs only) |

The standard library is a closed pack: rows for the modules that reach outside (files, sockets, subprocesses, the clock, the environment, threads), and everything else inert.

## 8. Gaps

**Status: Proposed.**

### 8.1 What is a gap

A **gap** is a site whose outcome could change the model and is not known:

- an `Unresolved` site of any reason;
- an `External` site whose package is `unknown`, or matches no row in an open pack;
- a package whose pin fails the rule of 7.3;
- a missing input the rungs asked for: an unknown lockfile version, an empty submodule directory, disagreeing lockfiles.

### 8.2 What a gap puts at risk

Each gap is named by its consequence, because the consequence is what the reader needs.

| Gap | Arises from | Severity |
|---|---|---|
| **possible entry point** | an unresolved or unknown-package decorator or escape | error |
| **possible effect** | an unresolved or unknown-package callee | error |
| **possible write** | an attribute write whose receiver is unresolved | error |
| **unknown base** | an unresolved or unknown-package base class | error |
| **missing module** | an import that nothing provides | error |
| **guarded import** | a missing module imported in a `try` whose handler names `ImportError` or `ModuleNotFoundError` | listed: the fallback runs (F-B4) |
| **version decides** | a pin that fails the rule of 7.3 | error |
| **imprecise handler** | an unresolved exception type in an `except` | warning |
| **unbound key** | a route, task name or environment name that is not a literal | handled by Link, as today: an assumption or a `heuristic` wire |

The exact diagnostic codes follow M13 and are fixed when the checks are built. Each diagnostic carries the reason, the root cause, the sites, the ports affected and the fix.

### 8.3 Which gaps count

A gap **counts** when it lies where it can reach the model:

- in a function reachable from an entry point along resolved call edges;
- at module level in a module imported, directly or indirectly, by such a function's module;
- on any decorator, base class or escape, wherever it is, since these create entry points and stores themselves.

A gap elsewhere, in code no entry point reaches, is recorded in the resolution report and raises no diagnostic. Test code and scripts are the usual case.

**Why this is sound (R2).** Suppose a function matters to the model and holds a gap that does not count. Then no entry point reaches it along resolved edges. If some entry point reaches it at all, the path must pass through a call that did not resolve, and that call lies in reachable code, so it counts and is reported. If the function is itself an entry point that was not recognized, its decorator or registration did not resolve, and those always count. So every gap that could matter is either reported or stands behind a reported one.

### 8.4 Root causes

Gaps are grouped under the one thing that would close them.

| Root cause | Closes | Typical fix |
|---|---|---|
| a **package** of class `unknown` | every site into it | a pack; or an acceptance as inert or opaque |
| a package whose **pin** is missing | its version-decides gap | a lockfile line, or an environment |
| a **module** nothing provides | every import of it and all followers | an import root, a submodule checkout, the generated file |
| an **untyped binding** (one parameter, one attribute) | every site that uses it | an annotation |
| a **dynamic construct** | that site | an acceptance |
| an **input file** the tool cannot read | everything that depended on it | the supported format |

A build's gap list is this table filled in, largest effect first: the number of counting sites and the ports each root cause can affect.

## 9. The gate: when a build fails

**Status: the mechanism is Proposed; the policy is an Experiment ([E1](#e1-the-gate-policy)).**

### 9.1 Where the gate sits

The gate runs after Extract and pack resolution and **before any LLM phase**. Its inputs are deterministic, so it costs no LLM call, and a build that will fail does not first spend paid answers.

```text
detect ─▶ extract ─▶ resolve packs ─▶ GATE ─▶ vocabulary ─▶ … ─▶ check
                                       │
                                       └─ counting gaps without an acceptance: fail, exit 1,
                                          one diagnostic per root cause, each with its fix
```

### 9.2 The recommended policy

1. **A missing input fails the build.** Missing modules, unreadable lockfiles, empty submodules, disagreeing pins. These have deterministic fixes, and no acceptance can stand in for them.
2. **A counting gap fails the build unless an acceptance covers its root cause.** With an acceptance, the gap becomes assumption records ([section 10](#10-assumptions-how-an-accepted-gap-enters-the-model)).
3. **A gap that does not count never fails the build.** It appears in the resolution report.
4. **An acceptance that matches nothing fails the build.** A stale acceptance would otherwise outlive the code it excused, the same rule as `#[expect]` in G18.
5. **The resolution rate is reported and never gates.** A rate hides where the holes are: one unresolved decorator matters more than a hundred unresolved calls in a script. CodeQL had to change its denominators and lower its threshold after false alarms (F-U5).

### 9.3 How this keeps "fail rather than degrade"

G11 forbids a model that is later read as complete when it is not. The gate keeps that rule and sharpens the word:

- A build succeeds only when every counting reference is resolved, external with known behavior, or covered by a visible assumption that a person or their agent wrote down. Nothing is left for another tool to fill.
- A build never substitutes a guess, a name match or a weaker mode. Sourcegraph's quiet fallback to text search and CodeQL's guessing of dependencies from package names (F-U5, F-U6) are the degraded paths Mental does not take.
- What cannot be known is not hidden behind success. It is in the model as assumption records, and every result that rests on them says so.

### 9.4 What the user and the agent see

Exit code 1, and one diagnostic per root cause:

```text
error: package pkg:pypi/stripe has no pack
  41 calls and 2 decorators in 3 modules; can affect ports:
    reports/api/port:post-charges, reports/worker/port:task-settle
  fix one of:
    - add a pack for stripe (mentalc pack new stripe)
    - accept it in mentalc.toml:   [[assume]] package = "stripe"  as = "opaque"  reason = "…"

error: module `billing_pb2` is imported and nothing provides it
  app/billing.py:3  import billing_pb2
  looks generated: the repository has billing.proto and ignores *_pb2.py
  fix: generate it, then build again:   python -m grpc_tools.protoc …      (mentalc does not run this)
```

Under the `agent` backend the same text is the result of `mental_task_next`, marked `isError: true` as every failed check is (C17). The agent can then supply the input with its own tools and call again. Because `task next` recomputes the cheap phases from the repository each time (C4), no state has to be carried between the attempts.

## 10. Assumptions: how an accepted gap enters the model

**Status: Proposed; the semantics of an opaque call is an Experiment ([E3](#e3-what-an-opaque-call-may-touch)).**

### 10.1 The acceptance

An acceptance is written in `mentalc.toml`, which is authored and committed like an experiment file. It names a root cause by code names, never by line numbers (P8).

```toml
[[assume]]
package = "tenacity"
as = "inert"                       # no effects, calls nothing back
reason = "retry decorators around functions that are already ports"

[[assume]]
package = "stripe"
as = "opaque"                      # each call reaches an outside system we do not model
reason = "payment provider; no pack yet"

[[assume]]
in = "app.billing.settle"          # the function that holds the site
callee = "handler(event)"          # the call as written
as = "opaque"
reason = "handler comes from a registry filled at import time"
```

| `as` | Allowed for | Produces |
|---|---|---|
| `inert` | a package; a decorator | an assumption that it has no effect on modelled state and starts nothing |
| `opaque` | a package; a single call | opaque call nodes (10.2) |
| `target` | an ambiguous or dynamic site | the named repository definition as the target, with provenance `assumed` |

Rules:

- The source of an acceptance is the committed file, so its records have source `user`. An agent may draft the entry; it is then a change to a file under version control that a person can review.
- **An LLM answer can never create an acceptance** (R8). There is no task role for it.
- A possible write cannot be accepted as inert in bulk: an unresolved write receiver is accepted per binding, by naming its type (`target`), because a wrong guess here removes a write site.

### 10.2 The opaque call

An opaque call is the model's honest form of "something happens here that we cannot see".

- **Structure.** A `call` node on a required port of an `External` entity for the package. Close wires that port to a stub, as it does for every port that leads outside (specification 10).
- **Outcomes.** The stub may return or raise. Both are choices of the environment, like any stub's answer.
- **The assumption.** An `assumption` record, provenance `assumed`, states what the build takes for granted about the call, with its alternative. The default takes the call to touch no state variable of the model.
- **In the step function.** The node fires as a call to a stub. The assumption joins the path condition of every run that executes the node, so the run knows what it rests on (P6: the record has a meaning in the step function).

### 10.3 Results name what they rest on

A result is only as strong as the assumptions under it, so each result lists them.

- A **witness** lists the assumptions on its path.
- A **clean search** reads "none within these bounds, under assumptions A1 to An", where the list is every resolution assumption reachable in the explored space.
- A **port** whose behavior contains no opaque call and no accepted gap is **closed**. This is derived from the records, not stored.
- The presentation rule "never hide an assumption" (specification 13) already covers all of it. A collapsed view may not hide an opaque call.

Flipping a resolution assumption is an edit like any other (specification 15.1): the run is repeated with the alternative.

## 11. The resolution report

**Status: Proposed.**

`mentalc resolve` runs the deterministic phases up to the gate and prints the report. It calls no LLM and writes no model. It is the first command to run on a new repository, and it is the same computation the gate uses.

```text
$ mentalc resolve
import roots   src/   (src layout)
python         3.12   (.python-version)
packages       12:  12 declared in uv.lock
               class: 10 modelled, 2 inert, 0 unknown

reference sites            all      counting
  resolved                  61         58
  external, modelled        74         74
  external, inert           19         17
  unresolved                 0          0
resolution rate           100%       100%

root causes: none
```

The numbers above are illustrative: the real ones are measured when the extractor first runs on report-jobs.

**Definitions.**

- **Sites** are counted once each, by the conservation rule. Followers are counted apart from roots.
- **Resolution rate** is the share of sites that are resolved, or external in a modelled or inert package. It is given for all sites and for counting sites. Accepted gaps are not counted as resolved; they are a line of their own.
- The same table is given per module with `--by module`, and as JSON with `--json`.

No other tool for coding agents reports this number (F-U6). CodeQL's two metrics, the share of calls with a known target and the share of expressions with a known type, are the nearest precedent (F-U5); mypy and Pyright have per-file measures of how much is untyped (F-U6).

## 12. Execution and trust

**Status: Proposed.**

### 12.1 The rule

`mentalc` never starts a build tool, a package manager, an interpreter or a script from the repository, and makes no network request to resolve dependencies. It starts no process at all.

### 12.2 Why the agent's permission prompt is not enough

It is tempting to let `mentalc` run `uv sync` or `gradle dependencies` and rely on the coding agent to ask the user. That does not work:

- A permission rule in Claude Code matches the command text the agent writes. A command that `mentalc` starts gets no prompt of its own (F-U17).
- A local MCP server, which is how `mentalc mcp` runs, is outside the sandbox and runs with the user's full access (F-U17).
- Headless sessions show no trust dialog at all (F-U17).
- The tools themselves execute repository code even for read-only questions. Any Gradle invocation evaluates the settings and build scripts. Maven loads the extensions named in `.mvn/extensions.xml`. Yarn runs the binary that `.yarnrc.yml` points to. Python resolvers build source distributions to read their metadata (F-U12, F-U14).
- An allow-list that lives in the repository, as the script allow-lists of npm, pnpm and Composer do, is the repository's choice and not the user's (F-U14).

So the default-deny must be inside `mentalc`.

### 12.3 What `mentalc` does instead

- **It names the command and stops.** A gap whose fix needs execution prints the command in its fix text. The agent or the user runs it, through the agent's own permission prompt, which then sees the real command. `mentalc` reads the result on the next build.
- **It reads tool output as data.** A lockfile, an environment, an exported `pylock.toml`: once it exists as a file, reading it executes nothing.
- **It does not run git.** `.gitmodules` is read as text ([section 6.1](#61-rung-0-the-repository)).

### 12.4 Enforcement

G4 already makes process and network calls a clippy error outside the listed edge files. This section adds a rule on content: `mentalc` starts no program, checked by a tidy test that finds no `Command::new` outside test code.

## 13. Experiments

**Status: Experiment.** Each keeps competing designs side by side until measurement on real repositories decides, as agreed for this phase of the project.

### E1. The gate policy

| | A. Strict (recommended) | B. Per port | C. Report only |
|---|---|---|---|
| **Behavior** | any counting gap without an acceptance fails the build | the build succeeds; a port whose closure has such gaps gets a generated opaque behavior and is marked not compiled | the build always succeeds; gaps are diagnostics |
| **For** | simplest reading of G11; nothing unaccounted | a first build on a large repository yields a usable model of the ports that are clean | never blocks |
| **Against** | a first build may ask for many acceptances, and bulk acceptance turns into wallpaper | a model with ports that are placeholders is close to the "model with gaps" G11 forbids | a degraded model read as complete: what Mental exists not to do. Kept only as the baseline other tools use |

**Decide by:** on each corpus repository, the number of root causes at first build, how many are closed by inputs (rungs 0 to 3) and how many need acceptances, and for B the share of ports that would be left opaque. If a first build asks for more acceptances than a person will read, A needs B as its first-run mode.

### E3. What an opaque call may touch

| | A. Inert by assumption (recommended) | B. Havoc |
|---|---|---|
| **Meaning** | the call touches no modelled state; the assumption is recorded and travels with results | the call may write any state variable its arguments can reach |
| **For** | small state spaces; honest because the assumption is named in every result | finds races that run through the unknown code |
| **Against** | a race through the unknown code is missed, though the result says under which assumption | false alarms; larger search |

**Decide by:** on corpus repositories with a pack removed on purpose, whether known findings survive under A (reported with the assumption) and how many false findings B adds. B can also be offered as the flipped alternative of the assumption, which costs nothing until someone flips it.

### E5. Lockfile formats

| | A. Own readers for each format | B. `pylock.toml` only |
|---|---|---|
| **How** | parse `uv.lock`, `poetry.lock`, `pdm.lock`, `Pipfile.lock` by schema version | read only the specified format; for other tools the gap names the export command (`uv export --format pylock.toml`) |
| **For** | works on a checkout with nothing run | one specified format; no chasing of formats that change without notice |
| **Against** | four unspecified formats; uv's is declared unstable (F-U10) | the user or agent must run an export first, and the exported file is then an input outside the commit |

**Decide by:** how often the formats of the corpus repositories change across a few months of history, and whether an exported file per build is acceptable in practice.

### The plan for the experiments

1. **Corpus.** report-jobs, plus three to five open-source repositories on the version-one stack (FastAPI, Celery, SQLAlchemy), chosen before measuring: one with a `src/` layout, one monorepo, one with generated code, one without a lockfile.
2. **Baseline.** Run `mentalc resolve` with the hand-written packs only. Record sites by outcome, roots by kind, counting and not.
3. **One change at a time.** Add the environment; then a second type checker. After each, record the roots closed and the time taken.
4. **Seed gaps.** Remove a pack, an import root, an annotation, a lockfile line. The gate must fire each time, with the right root cause.
5. **Decide.** Each experiment's criteria above. Results are recorded as findings; the losing designs stay in this document, marked as measured and not chosen.

Step 2 alone answers the question this document starts from: how large the problem is on real code.

## 14. Other ecosystems

**Status: Later.** Version one is Python. The principles and the gate do not depend on the language; the rungs do. This table records what the research verified, for when a second language is scheduled.

| Ecosystem | Declared pin (rung 1) | Observed (rung 2) | What only execution gives | Sliding forms |
|---|---|---|---|---|
| **Go** | `go.mod` gives the exact build list; pseudo-versions carry the commit | the module cache | nothing for resolution; note that a `go.mod` can make the `go` command download and run another toolchain | branch names are resolved once and written as pseudo-versions |
| **Rust, Cargo** | `Cargo.lock`: every package, git commits included | the registry source cache | code generated by build scripts and procedural macros | caret ranges, frozen by the lock |
| **npm, pnpm, Yarn, Bun** | each lockfile, commits of git dependencies included | `node_modules` | nothing for resolution | ranges and tags, frozen by the lock |
| **Maven** | none: Maven has no lockfile, and Maven 4 is not yet released | `~/.m2` holds what was resolved | the resolved graph: parents, imported BOMs and nearest-wins need a resolver | `SNAPSHOT`, ranges, `LATEST` |
| **Gradle** | opt-in lockfile with versions but no edges; committed by under 1% of projects | the modules cache | the graph: every Gradle call evaluates the build scripts | dynamic versions and changing modules, re-checked every 24 hours |
| **Container images** | a digest (`@sha256:…`) | — | — | every tag |
| **GitHub Actions** | `.github/workflows/actions.lock`, generally available since October 2026 | — | — | tags and branches; only 6% of workflows pin a commit |
| **Git submodules** | the commit in the superproject's tree | the checked-out directory | — | none |

Sources: F-U2, F-U14, F-U19.

Three things carry over unchanged:

- **Version classes (R5) matter most where pins are weakest.** Maven and Gradle have no committed pin in most repositories. A JVM pack that covers a whole major version makes most of those missing pins irrelevant, and the gate names the few that are not.
- **Compiled metadata is a type surface that needs no build.** Class files and reference assemblies in the local caches describe the public API of JVM and .NET libraries.
- **Generated code stays out of reach without execution.** CodeQL's no-build mode loses it too (F-U5). For Mental it is a missing-module gap with the generating command in its fix.

**Dependencies from GitHub and other git hosts.** These need no special handling. Where a lockfile exists it records the commit, so the pin is exact. The package URL carries the repository and commit. What changes is the pack: a pack written for the registry release is applied to a fork with a warning (7.3).

**Linking repositories.** When several repositories are compiled (specification, section 12), an external symbol in one and a definition in another meet by identity: the same package URL and access path. And a dependency whose source is available at its pinned commit can be compiled by `mentalc` into a pack, which is the long-term answer to unknown packages.

## 15. Tests

**Status: Proposed.** In the style of the guidelines: each rule with how it is checked.

| Rule | Checked by |
|---|---|
| R1: one outcome per site | the conservation oracle of X8, extended: a separate small visitor counts every site kind per file and compares with the emitted facts, on every fixture and on report-jobs |
| R2: no silent gap | **gap-or-same**, a property test: remove one piece of resolution evidence from a fixture (a pack row, an import, an annotation, a lockfile line, an import root). The resulting model is byte-identical, or the gate reports a gap. Any other result is a silent change |
| R2: the gate fires where it should | seeded gaps on report-jobs: each of the removals above, asserted by diagnostic code and root cause |
| R3: nothing is executed | the tidy test of 12.4; G4's clippy rule |
| R4: evidence rank | fixtures where a declared and an observed pin disagree: the declared one stands, with the drift warning |
| R5: version classes | fixtures for each row of the table in 7.3 |
| R6: root causes | a fixture with many sites behind one unknown package yields one diagnostic |
| R7: stale acceptance | a fixture whose acceptance matches nothing fails with that code |
| R8: no LLM acceptance | no task role produces an acceptance; a scripted answer that tries is rejected by the answer type |
| Determinism | the same repository and recorded environment give byte-identical `deps.jsonl` and report |
| Fixtures by category | new `extract` fixture categories beside those of X19: `roots` (namespace packages, `src/` layout, two roots), `external` (access paths), `gaps` (one per reason), `escapes` |

## 16. Open questions

- **How large is `no_type` in real application code?** Measured on five projects: 34% of the receivers neither checker resolves are an unannotated parameter or a chain from one ([type checkers as libraries](research/type-checkers-as-libraries/report.md)).
- **How many packages does a typical service use that would need a pack?** The class scheme works if the answer is tens. Classifying the standard library alone is real work, and its size is unmeasured.
- **Who may declare a package inert?** A pack author's statement and a user's acceptance are both claims. Whether a shipped list of inert packages needs evidence beyond review is undecided.
- **Closedness of hand-written packs.** A closed pack that misses an effect is a silent gap, the very thing this document forbids. A check of pack rows against the package's real surface would narrow it; nothing removes it.
- **Reachability and dynamic entry points.** The soundness argument of 8.3 rests on decorators, bases and escapes always counting. A framework that finds handlers by naming convention alone (a function named `handle_*` in a certain folder) creates entry points with no site at all. Such frameworks need a slot row keyed on names and paths, which the pack grammar does not have yet.
- **Conditional imports and platform markers.** `try: import ujson as json` followed by `except ImportError: import json` is two candidates today. Whether that should be a variation point instead is open.
- **Acceptances at scale.** If a first build of a large repository asks for fifty acceptances, the strict policy fails in practice, whatever its merits. E1 measures this.
- **The environment as an input.** Recording what was read from an environment keeps builds reproducible on one machine. Two developers with different environments and no lockfile get different models, each correctly labelled `observed`. Whether a build should refuse `observed` pins in automated runs is undecided.

---

## Sources

- [Findings, section U](findings.md#u-unresolved-references-and-dependencies): every F-U number, with its source.
- Research: the [survey report](research/unresolved-references/report.md), four [survey notes](research/unresolved-references/notes/) and five notes that re-read the primary sources (`verified_*.md`).
- Mental's own texts this builds on: specification sections 3, 4, 6, 7 and 8; `extract` guidelines X7 to X11; the cross-cutting rules G4, G8 and G11.
