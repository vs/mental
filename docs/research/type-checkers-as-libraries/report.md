# Python type checkers as a source of resolved references

**Question.** Which existing Python type checker can give Mental the most resolved references without LLM calls: for every place the code names something, what it points to and what type it has, and why when it fails? Can each one be used as a library, or what else does it export?

**Date:** 2026-11-17. The survey sections are read from primary sources (repositories, docs.rs, official docs); points not confirmed are marked *unverified*. Pyrefly and ty were then measured on five projects ([experiment](#experiment-pyrefly-against-ty-on-five-projects)).

## Per-product summary

- **Pyrefly** (Meta, Rust, MIT, stable 1.x): no library API, but `pyrefly check --report-pysa` dumps, per function, every call site with its resolved callees, receiver class and overrides, and an `unresolved` field with a reason; the richest batch export found.
- **ty** (Astral, Rust, MIT, beta 0.0.x): published on crates.io and usable as a Rust library on ruff's own syntax tree: the type of any expression, the definitions of names and attributes; no stable API.
- **Pyright** (Microsoft, TypeScript, MIT): the analysis package is private; the supported way in is the new type server (`pyright-typeserver`, Type Server Protocol) that returns structured types with their declarations, one node at a time.
- **basedpyright**: a Pyright fork with the same internals; no library API.
- **scip-python**: Pyright frozen at a 2023 commit plus a SCIP writer; definitions and references only, types as hover text.
- **mypy** (Python): usable as a Python library: `build.build` with `export_types` returns the type of every expression; the only checker with framework plugins (Django, Pydantic); `dmypy suggest` infers signatures from call sites.
- **Zuban** (Rust, AGPL): mypy-compatible checker and language server; no library API found.
- **Pyre**: superseded by Pyrefly. **pytype**: discontinued, Python 3.12 is its last version.

## What Mental needs from a resolver

For `with Session() as s: s.commit()` and `report.status = "done"`: the callee each call reaches (`Session.commit`), the type of each receiver (`s: Session`, `report: Report`), the definition behind every name, and a reason where the tool fails. A batch answer for the whole project is better than one query per position.

## Pyrefly

**Library use.** The workspace has crates `pyrefly_python`, `pyrefly_types`, `pyrefly_util`, `pyrefly` and others ([ARCHITECTURE.md](https://github.com/pyrefly-org/pyrefly/blob/main/ARCHITECTURE.md)), but no documented public API, and the `pyrefly` crate on crates.io is a placeholder (0.0.1). The repository is MIT-licensed Rust, so depending on it by git, or forking it, is possible but unsupported. Version policy: "any version may introduce new type errors and other breaking changes" ([README](https://github.com/pyrefly-org/pyrefly)). Latest release 1.4.1, 4 November 2026. Google adopted it as its Python type checker on 6 October 2026, replacing pytype ([Google blog](https://opensource.googleblog.com/2026/10/adopting-pyrefly-as-the-python-type-checker-at-google.html)); Meta's Pysa takes its type information from it ([Pysa](https://github.com/meta/Pysa)).

**`--report-pysa`** ([pysa.rs](https://github.com/pyrefly-org/pyrefly/blob/main/pyrefly/lib/report/pysa.rs), [call_graph.rs](https://github.com/pyrefly-org/pyrefly/blob/main/pyrefly/lib/report/pysa/call_graph.rs), [pysa_report.capnp](https://github.com/pyrefly-org/pyrefly/blob/main/pyrefly/lib/report/pysa/pysa_report.capnp)). Per module it writes `definitions/`, `type_of_expressions/` and `call_graphs/`, as Cap'n Proto (the default) or JSON with `format_version: 1`.

- **Call graph:** one per function, keyed by the location of each call expression. Each callee says whether it is a function or "all overrides of the given method", the receiver class, whether the receiver is implicit, class or static method, and the return type. Constructor calls list `__init__` and `__new__` targets; functions passed as arguments are listed as higher-order parameters.
- **Unresolved calls are marked:** `unresolved` is false or one of 14 reasons, such as `UnknownClassField` or `UnexpectedCalleeExpression`.
- **Types of expressions:** only names, the base of an attribute access (the receiver), and call arguments.
- **Definitions:** functions (signature, decorator callees, the base method they override), classes (bases, MRO, fields), globals.
- The format is undocumented and serves Pysa; no "unstable" marker on the flag. A third-party project pins Pyrefly 1.4.0 because the format may change ([FM-Agent #259](https://github.com/fmagent-project/FM-Agent/issues/259)), and reports that calls through variables, parameters or dict lookups are not linked.

**`--report-glean`** writes cross-references (each name's span to the declaration it points to, attribute writes included), declarations and caller-callee facts ([glean.rs](https://github.com/pyrefly-org/pyrefly/blob/main/pyrefly/lib/report/glean.rs)). **`--report-scip`** writes a SCIP index from the current checker: definitions, references, hover text, implementation relationships; types only in the hover text ([scip.mdx](https://github.com/pyrefly-org/pyrefly/blob/main/website/docs/scip.mdx)).

**Type server.** Pyrefly adds a batch request `pyrefly/typeFacts` (since 1.4.0): a list of ranges in one file, each answered with a class, instance, module, function or union, its qualified name, MRO, type arguments and members ([type_facts.rs](https://github.com/pyrefly-org/pyrefly/blob/main/pyrefly/lib/tsp/type_facts.rs)).

**Caveats.** With no configuration, Pyrefly uses a basic preset that skips the bodies of completely unannotated functions; `check-unannotated-defs = true` turns them back on ([configuration](https://pyrefly.org/en/docs/configuration/), from a summary of the page, *unverified* verbatim). The checker itself infers "types in most locations, apart from parameters to functions" ([ARCHITECTURE.md](https://github.com/pyrefly-org/pyrefly/blob/main/ARCHITECTURE.md)). `pyrefly infer` writes draft annotations and can be told to add parameter types; the official page says it is under active development and does not say it reads call sites ([autotype](https://pyrefly.org/en/docs/autotype/)); a third-party guide says it does ([pydevtools](https://pydevtools.com/handbook/how-to/how-to-add-type-annotations-with-pyrefly-infer/)).

## ty

**Library use.** `ty_python_semantic` 0.0.22 is published (docs.rs, 12 November 2026), marked "an internal component crate of Ruff" ([docs.rs](https://docs.rs/ty_python_semantic/latest/ty_python_semantic/)). The README: "ty does not yet have a stable API; breaking changes … may occur between any two versions", and "ty is currently in beta" ([ty](https://github.com/astral-sh/ty)). Latest ty release found: 0.0.90, 20 October 2026.

**Entry points** (all on ruff's syntax tree, the parser Mental's extractor already plans to use):

- Type of an expression: `HasType::inferred_type(&self, model: &SemanticModel) -> Option<Type>`, implemented for `Expr`, `ExprName`, `ExprAttribute`, `ExprCall` and more ([HasType](https://docs.rs/ty_python_semantic/latest/ty_python_semantic/trait.HasType.html)).
- Definitions: `definitions_for_name`, `definitions_for_attribute` ("inferring its receiver"), `SemanticModel::reaching_definitions` for a name, `Type::definition` for where a type is defined ([ide_support](https://docs.rs/ty_python_semantic/latest/ty_python_semantic/types/ide_support/fn.definitions_for_attribute.html)). No call-to-definition function; the route is the callee expression's type.
- Failures: `Db::check_file` returns the diagnostics.
- The `Type` enum has public variants, but most of their payloads are crate-private; one external user calls the lattice "almost entirely `pub(crate)`" ([smelt-py-types](https://docs.rs/smelt-py-types/latest/smelt_py_types/)).
- How to set up a project database (`ty_project`) is *unverified*: its docs.rs page did not load.

## Pyright and basedpyright

**Library use.** `pyright-internal`, which holds the type evaluator, is `"private": true` in its `package.json` and has no entry point ([package.json](https://github.com/microsoft/pyright/blob/main/packages/pyright-internal/package.json)). The CLI's JSON output has diagnostics only ([command line](https://github.com/microsoft/pyright/blob/main/docs/command-line.md)). basedpyright has no programmatic API either. Latest Pyright: 1.1.414.

**Type server.** Pyright 1.1.413 added a standalone Type Server Protocol server, published to npm as `pyright-typeserver` in 1.1.414 ([type-server.md](https://github.com/microsoft/pyright/blob/main/docs/type-server.md)). The protocol is JSON-RPC over the same transport as a language server, at version 0.4.1, and "while the protocol is in the `0.x` line, minor version changes may be breaking" ([TSP spec](https://github.com/microsoft/pylance-release/blob/main/docs/tsp/type-server-protocol.md)).

- Requests: computed, declared and expected type of a node (a file plus a range) or a declaration; resolve an import; search paths; snapshots.
- A returned type is structured: built-in (`unknown`, `any`, …), class or function with its **declaration** (file and range), union with members, module, overloads, synthesized types with stub text.
- No request returns "the declaration this name points to" directly (it comes with the type), and no batch request (Pyrefly adds one).
- Implementers: Pyright, Pyrefly (Pylance can use it), and a "ty-tsp" named in Pyright's source (*unverified*).

**scip-python** vendors Pyright at one commit and changes only its SCIP writer ([scip-python](https://github.com/sourcegraph/scip-python)).

## mypy

**Library use** (Python, internal API). `mypy.build.build` returns a `BuildResult` whose `types` maps every `Expression` to its `Type` when `options.export_types` is set; mypy's own tests use it, and a TODO in the source says `BuildResult` may go away ([build.py](https://github.com/python/mypy/blob/master/mypy/build.py)). Names and member accesses also carry the full name of what they refer to after semantic analysis (from mypy's node classes; not re-read here).

**Plugins.** mypy is the only checker with a plugin system, used by django-stubs, Pydantic and others to type what the framework creates at run time.

**Parameter types from call sites.** `dmypy suggest module.func` "uses call sites, return statements, and other heuristics" to propose a signature; `--json` output, `--no-any`, `--callsites` ([mypy daemon](https://mypy.readthedocs.io/en/latest/mypy_daemon.html)). Available since December 2019.

## Others

- **Zuban**, by the author of Jedi: Rust, AGPL, mypy-compatible mode and a Pyright-like mode, published as a CLI and language server ([zuban](https://github.com/zubanls/zuban)). No library API found; not examined further.
- **Pyre** is superseded by Pyrefly and its repository archived; Pysa now lives apart and uses Pyrefly.
- **pytype**: "Python 3.12 is the last Python version pytype will support" ([pytype](https://github.com/google/pytype)).
- **Jedi** (a Python library by design) was not examined in this round.

## Comparison

| | Way in | Every expression's type | What each name, attribute or call points to | Reason when it fails | Batch | Stability |
|---|---|---|---|---|---|---|
| **Pyrefly** | CLI export, or a git dependency on its crates | names, receivers, arguments | calls with callees, receiver class and overrides; cross-references | yes, 14 reasons on calls | yes | stable checker, undocumented format |
| **ty** | Rust crate | yes | names and attributes | diagnostics | in process | 0.0.x, no stable API |
| **Pyright** | type server process | yes, per node | through the type's declaration | `unknown` type only | no | protocol 0.x |
| **mypy** | Python library | yes | full names on name and member nodes | errors | in process | internal API |
| **SCIP** (scip-python, Pyrefly) | file | no, hover text | definitions and references | no | yes | format stable |

## Experiment: Pyrefly against ty on five projects

**Date:** 2026-11-17. **Code, corpus and how to reproduce:** [`experiments/`](experiments/README.md).

**What was run.** Pyrefly 1.4.1 through its Pysa export, and ty 0.0.105 used as a Rust library (a small program on its crates, `tyharness`), on report-jobs, full-stack-fastapi-template, Mealie, Redash and rq: about 88,000 lines without tests, each with its locked dependencies installed. Both tools got the environment as files; neither started an interpreter. Sites are the ones Mental's extractor would ask about: every call, every attribute access (its receiver, read or written), every decorator, every class's bases. A site is **resolved** when the tool names a function or class for a call or decorator, or a class or module for a receiver; `Unknown`, `Any` and `Never` are not resolved.

### Results

All five projects together:

| Site kind | Sites | ty resolves | Pyrefly resolves | Either | Both resolve, and agree | Real disagreements | Only ty | Only Pyrefly | Neither |
|---|---|---|---|---|---|---|---|---|---|
| call | 25,260 | 88.5% | 91.6% | 92.2% | 99.7% | 65 | 152 | 949 | 1,968 |
| receiver (read) | 31,312 | 85.8% | 90.9% | 91.0% | 99.8% | 66 | 25 | 1,606 | 2,825 |
| receiver of a write | 1,424 | 94.6% | 95.6% | 96.1% | 99.9% | 2 | 7 | 22 | 55 |
| decorator | 928 | 99.6% | 98.3% | 99.6% | 100% | 0 | 12 | 0 | 4 |
| class bases (per class) | 966 | 99.7% | 99.2% | 99.8% | 99.2% | 8 | 6 | 1 | 2 |

Per project, calls and read receivers resolved (ty / Pyrefly): report-jobs 93.5 / 90.9% and 91.8 / 87.6%; fastapi-template 100 / 99.3% and 100 / 100%; Mealie 95.7 / 98.1% and 92.9 / 98.5%; Redash 72.5 / 77.9% and 69.0 / 75.1%; rq 93.7 / 95.1% and 93.1 / 94.4%. Time per project: Pyrefly 0.7 to 7.1 s, ty 0.4 to 3.1 s, on the same machine. Expressions inside annotations (313 receivers, 67 calls) are not in Pyrefly's export and are left out of the percentages.

### What the numbers mean

- **Where both answer, they agree.** 99.7% of calls and 99.8% of receivers that both resolve resolve to the same thing, after the spelling differences listed in the experiment README. Two independent checkers agreeing is evidence, not proof, that the answer is right; no run of the code was compared.
- **Pyrefly resolves more, mostly because it infers return types.** A probe confirmed it: for `def get_logger(name): return logging.getLogger(name)`, Pyrefly gives `logging.Logger` and ty gives `Unknown`, even when the parameter is annotated. ty 0.0.105 infers no return type for a function without a return annotation. In Mealie this is `self.repos` (an unannotated property returning `AllRepositories`) and `self.t(...)`, which reaches `Translator.t` through an unannotated property; in Redash, loggers made by a helper function.
- **Pyrefly's return inference needs a configuration.** With no configuration file it uses the preset `basic`, which skips the bodies of unannotated functions; the runs here set `check-unannotated-defs = true`.
- **Pyrefly can lose the write sites Mental needs.** In report-jobs, Pyrefly types `report` as `Never` at five of the eight attribute writes, among them every `report.status = …`. The models use SQLAlchemy's older `Column(...)` style, so `report.status` is typed `Column`, whose `__bool__` is declared never to return; after `if report is None or report.status != X: return`, Pyrefly treats the rest of the function as unreachable. ty types `report` as `Report` there. `Never` at a receiver occurred 9 times in report-jobs, 16 in Mealie, 3 in Redash and 9 in rq. ty produced `Never` at a few sites of its own in rq (a loop in `worker_pool.py`, `job.py` line 518) where Pyrefly resolves.
- **Where they really differ, each is sometimes deeper.** For a call through a variable (`blue = make_colorizer(...)`), ty's "go to definition" stops at the variable while Pyrefly names the function called (`make_colorizer.inner`); for an overridden method, ty names the override actually called (`Form.validate`) while Pyrefly names the base method with "all overrides".
- **Neither tool resolves Celery's task methods or boto3 clients.** `generate_report.delay(...)`, `self.retry(...)` and `self.request` in a bound task, and `s3.put_object(...)` are unresolved by both, although Pyrefly bundles boto3 stubs. These stay with packs.
- **What neither resolves is measurable now.** Of the 2,880 receivers neither tool resolved, 987 (34%) are an unannotated parameter or an attribute chain from one: 803 of 2,380 in Redash, 134 of 231 in rq, 50 of 261 in Mealie. The rest are `self` attribute chains (428) and values from library calls, module globals and call results (1,465).

### What the library route costs

- **ty:** the type of any expression and "go to definition" are public and work directly on ruff's nodes, in process. But the bound of a type variable (`Self@m` → its class) and the parts of an intersection are crate-private; the harness has to ask the IDE's "go to type definition" or read the source instead. "Go to definition" on a call's callee can stop at a variable or a parameter.
- **Pyrefly:** a batch export with reasons for unresolved calls, but an undocumented format; constructors whose call graph names only `object.__init__` (`FastAPI()`); nothing for expressions inside annotations; no reason for an unresolved receiver.

## What follows

- **The richest output that exists today is Pyrefly's Pysa export:** resolved callees per call site, receiver classes, overrides, and a reason for every call it could not resolve, from a stable checker that Meta and Google run at scale. Measured on five projects, it resolves 91.6% of calls and 90.9% of receivers, against ty's 88.5% and 85.8%, mostly because it infers return types.
- **Its one dangerous behavior for Mental is `Never`:** on legacy SQLAlchemy models it marks the rest of a function unreachable and its writes disappear. Whoever adopts it needs a check that turns a `Never` receiver into a reported gap, or a second opinion from ty at those sites.
- **ty is the closest fit as an in-process library:** it works on ruff's syntax tree, so Mental's extractor and the checker see the same nodes, with no matching of positions, and it is about twice as fast. The price is an unstable 0.0.x API, crate-private type internals, and no return type inference in the version measured.
- **The two agree almost everywhere both answer,** so using both costs little in conflicts: their union resolves 92.2% of calls and 91.0% of receivers.
- Forking a current checker, if needed, means Pyrefly or ty, not scip-python: both are Rust and MIT, the language `mentalc` is written in.
- **Pyright's type server is the only supported, documented way into Pyright** and a protocol several checkers speak; it was not measured here.
- **mypy is the only route to framework plugins** and to signatures inferred from call sites (`dmypy suggest`); not measured here.
- **Not yet measured: correctness.** Agreement is not truth. The next check is against what the code actually does, for example call edges recorded while running the projects' tests.
