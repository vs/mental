# SCIP indexes as the checker behind `extract`: experiment R4

**Question.** When Mental adds a language after Python, can a SCIP index be the checker behind `extract`, with the site walk staying Mental's? What does that lose against what Mental needs: an outcome per site, a reason when it is unresolved, the class behind a callee, and the escapes? ([plan](plan.md))

**Date:** 2026-11-26. Everything below was run; the scripts, tool versions and corpus are in [`experiments/`](experiments/README.md), the raw counts in [`experiments/results/`](experiments/results/). Measured facts are in the [findings, section I](../../findings.md#i-scip-indexes-experiment-r4).

## Per-product summary

- **Pyrefly 1.5.0, `--report-scip`** (Python): answers 93.4% of `mentalc`'s sites and agrees with ty on 99.0% of those both answer; 1 wrong answer in 50 disagreements read. No reasons, no symbol kinds, no roles.
- **scip-python 0.6.8** (Python, Pyright of March 2023): answers 90.4% and agrees on 96.9%; 7 wrong answers in 50 read, and three naming defects. It names the right package for fewer than half of the external targets, crediting the rest to the project.
- **scip-java 0.14.2** (Java, a javac plugin): names 99.8–100% of single and on-demand imports, calls, constructors, annotations, base types and `catch` types, all right in a sample of 63. It never names a static import's member, writes no index at all when the build fails, and cannot say what a local variable handed to a call is.

## Part P: Python, against ty

Five projects (report-jobs, full-stack-fastapi-template, Mealie, Redash, rq) at the commits of the type-checker experiment, tests left out: 39,462 sites with an ask point. Each site from `mentalc build` was paired with the occurrence at the column where `extract` asks ty, found with Python's own parser, since the site records keep whole lines.

| Kind | Sites | ty answers | scip-python answers | same as ty | Pyrefly answers | same as ty |
|---|---|---|---|---|---|---|
| import | 7,941 | 97.7% | 98.0% | 95.8% | 98.3% | 99.9% |
| callee | 24,465 | 88.2% | 87.3% | 96.6% | 92.1% | 98.5% |
| decorator | 1,902 | 99.8% | 99.0% | 99.3% | 99.1% | 99.3% |
| base class | 1,075 | 97.4% | 100.0% | 100.0% | 100.0% | 100.0% |
| write receiver | 1,423 | 93.1% | 94.7% | 98.1% | 94.8% | 99.2% |
| escape | 1,869 | 71.4% | 79.9% | 100.0% | 78.7% | 100.0% |
| handler type | 787 | 97.8% | 94.8% | 98.9% | 95.9% | 99.9% |
| **all** | **39,462** | **90.5%** | **90.4%** | **96.9%** | **93.4%** | **99.0%** |

*Same* counts a target spelled differently but meaning the same object: a private module (`_io.BytesIO`), an attribute declared in a base class, or a nested function named without its enclosing one.

**Where they differ.** Fifty disagreements per index were read by hand ([hand check](experiments/results/hand-check-python.md)):

- **Pyrefly** was wrong once. In 34 of 50 it names the declaration where ty names what runs. Examples: the function under `@lru_cache` against `_lru_cache_wrapper.__call__`, a variable `Session` against `sessionmaker.__call__`, a parameter against the callable it is annotated with. That is the missing expression type the plan expected, not an error.
- **scip-python** was wrong 7 times. Its old Pyright sends `….filter()` on SQLAlchemy queries to `builtins.filter`, and `os.path.isfile` to `genericpath.exists`. It also has three naming defects that break identity:
  - 155 module symbols lose their package: `from alembic import op` becomes the top-level module `op`;
  - 104 nested functions are named as methods of their class;
  - some symbols are doubled, as in `TYPE_CHECKING.TYPE_CHECKING.`.

**Where ty has no answer.** Both indexes answer sites ty cannot:

- **No type:** 867 (Pyrefly) and 916 (scip-python) of 2,743 `no_type` sites, such as `logger.info(…)` and `Model.query.filter(…)`. This is return-type inference, the same advantage Pyrefly showed in the type-checker experiment.
- **`not_found`:** 319 (Pyrefly) and 328 (scip-python) of 777. For Pyrefly, 229 of its 319 are in Mealie's alembic migrations, where the index resolves inside a file that ty cannot name as a module (F-B9).

Where ty *has* an answer, scip-python leaves 1,217 sites without one. It writes builtins such as `isinstance`, `len` and `str.format` as file-local symbols. Pyrefly leaves 57.

**No reasons.** On a copy of report-jobs broken on purpose, neither index tells why it has no answer:

| Gap | `mentalc` (ty) | scip-python and Pyrefly |
|---|---|---|
| `boto3` uninstalled | exit 1, `module-missing`, "run `uv sync`, or add the package … with `uv add`" | `import boto3` gets a file-local symbol; scip-python exits 0 |
| `s.commit_all()`, no such method | `not_found` | no occurrence |
| `getattr(s, "commit")()` | `dynamic` | nothing for the outer call |

Neither index fills an occurrence's `diagnostics` field anywhere, on any project. Neither sets a symbol's kind. scip-python marks every reference, writes included, as a read; Pyrefly sets no roles at all. A class is still recognizable by its symbol's suffix (`#`).

**Own code or a dependency.** Pyrefly's package tells `Resolved` from `External` correctly on 99.7% of sites, and names ty's pin for 99.0% of external targets. scip-python credits a symbol defined in a dependency's submodule, such as `fastapi.applications.FastAPI`, to the project: by package it is right on 70.2%, by the module's file on 99.3%, and it names ty's pin for 45.5% of external targets.

**Escapes.** Of `mentalc`'s 1,869 escapes:

- 1,334 are repository functions or classes by ty's answer. Each index's symbol alone decides the ones it answers: 1,326 (Pyrefly) and 1,327 (scip-python).
- The other 535 are escapes because their type is unknown or callable. No index can say that, since the type is only hover text.

**Cost** (analysed folders only, tests left out):

| Project | scip-python | Pyrefly |
|---|---|---|
| report-jobs | 1.5 s, 465 MB | 0.24 s, 235 MB |
| fastapi-template | 2.0 s, 463 MB | 0.22 s, 252 MB |
| mealie | 15.3 s, 1,379 MB | 0.8 s, 624 MB |
| redash | 7.6 s, 908 MB | 0.36 s, 450 MB |
| rq | 3.7 s, 422 MB | 0.17 s, 221 MB |

`mentalc build` ran on whole trees, tests included, in 0.45 s to 4.3 s.

## Part J: Java, scip-java

Three projects, chosen and pinned before measuring:

- **spring-petclinic** (`500158f`): a small Spring Boot service with JPA;
- **mockito** (`32938e6e7`): Gradle with several modules;
- **spring-boot-realworld-example-app** (`ee17e31`): built with Lombok.

A tree-sitter walk listed 21,965 sites in the main sources, with the plan's mapping of site kinds to Java.

| Kind: form | Sites | named by a global symbol | unanswered |
|---|---|---|---|
| import: single | 3,217 | 100.0% | — |
| import: on-demand | 10 | 100.0% | — |
| import: static | 264 | **0.0%** | the class is named, the imported member never |
| callee: method | 7,483 | 99.9% | 6 local, 5 with two symbols (Lombok) |
| callee: `new` | 1,229 | 100.0% | — |
| callee: `this(…)`, `super(…)` | 104 | 100.0% | — |
| decorator: annotation | 1,680 | 99.8% | 4 |
| base class | 460 | 100.0% | — |
| handler type | 209 | 100.0% | — |
| write receiver: `x.f = …` | 455 | 99.6% | 2 |
| write receiver: `f = …` | 451 | 32.2% | 306 are local variables, which the index tells apart |
| escape: method reference | 70 | 100.0% | — |
| escape: method declaration | 2,620 | 97.4% | 66 local, all in mockito; a sample reads as methods of anonymous classes (`evaluate()`, `answer()`), whose overrides were not counted |
| escape: argument | 3,607 | 13.6% | 3,118 locals and parameters |
| escape: lambda | 106 | 0.0% | no name to ask |

**Right.** Every global answer names what the source names. The symbol's last name equals the written name, and a constructor counts as its class's name. All 63 answers sampled by hand were right ([hand check](experiments/results/hand-check-java.md)), nested classes and inherited fields included.

**What the index tells beyond Python:**

- **Overrides.** A method declaration carries its overrides as relationships: 188 of 2,552 methods override outside code, so they are escapes. That decides the other kind of escape with no type at all, once two habits are allowed for:
  - **Implementations, flagged like overrides.** scip-java also relates a method to the methods that implement it, with the same flags: `ArgumentMatcher#matches()` lists its 37 implementations. A target in a subclass of the method's own class is one of those.
  - **Own code, from definitions.** In mockito, own code is told by a definition in the index, not by the package.

  Read without either, as this report first did, the count is 1,140.
- **Kinds.** The repository's own symbols carry a kind; dependency symbols carry none (4,621 of 8,701 callees have one).

**What it lacks:**

- **Argument types.** Of 3,607 names handed to a call, 3,607 need a type to decide whether they escape: 3,118 locals or parameters and 489 fields. The index gives the type only as hover text.
- **Static imports.** No member of a static import is named.
- **Generated code.** Lombok's generated getters land on a source token, so 5 calls carry two symbols.
- **Own code by package.** In a build with several modules, the package cannot tell own code from a dependency. mockito names its own classes by Maven coordinates (`maven/org.mockito/mockito-core 5.24.1-SNAPSHOT`): 11 targets are own by package, 8,669 by a definition in the index. The definition rule agrees with the package on petclinic (225 of 225) and nearly on realworld (2 apart).

**All or nothing.** Any compile error means no index:

| Gap | scip-java | Where the reason is |
|---|---|---|
| `javax.cache:cache-api` removed from `build.gradle` | exit 1, no index | javac in the build log: `CacheConfiguration.java:24: error: package javax.cache.configuration does not exist` (8 errors) |
| `owner.noSuchMethod()` | exit 1, no index | `OwnerController.java:79: error: cannot find symbol` |
| Lombok's annotation processor removed | exit 1, no index | 100 `cannot find symbol` errors |
| `Owner.class.getMethod("getId").invoke(owner)` | exit 0, index | the call is `java/lang/reflect/Method#invoke()`, like any other |

The reasons exist, in javac's words with file and line, but only in the build's output, never in an index. And one missing dependency removes the whole project's answers, where ty marks one import.

**Setup and cost.**

- **Maven route fails on macOS.** scip-java's Maven route fails there: its generated `javac` wrapper expands an empty bash array under `set -u`, which bash 3.2, the system bash, rejects. The Gradle route worked, so petclinic was indexed through its Gradle build.
- **First runs are slow.** They took 78 s (petclinic), 81 s (realworld) and 198 s (mockito), dependency downloads included.
- **The indexer runs the project's build, with whatever it does.** mockito's build published a Gradle Build Scan to an outside service during indexing.

## Decision by the plan's rules

- **Java: design A (index only) is not enough.**
  - Code that compiles is fully covered, but measure 6 needs a type for 3,607 argument escapes, not a few.
  - Three of the four seeded gaps stop the indexer with javac's message naming them, which the plan's rule accepts. But the message is only in the build log, the stop takes the whole project's answers with it, and the reflective call passes silently as `Method#invoke()`. No gap leaves a diagnostic on a site.
  - Static imports are a hole of their own.
- **Not design B either.** The index's targets are right, so nothing pushes toward B on correctness.
- **That points to design C:** the index for targets, plus a query on the compiler for argument types, static import members, and code that does not compile.
- **But C's compiler part is the same tool as B.** Since that part has to handle builds that fail, B and C converge. What the index adds over B is that it is maintained and gets each build tool's classpath right. What B adds is answers on code that does not compile.
- **Not measured:** whether javac's own API (the tree and type API a compiler plugin sees) still answers on code with errors. That is the next run: a thin javac tool on the same three projects and the same seeded gaps, measured against these indexes.
- **Python: no change.** ty stays. Pyrefly's index answers 867 `no_type` sites with targets that read right, which is evidence for experiment R2 (a second checker) through a documented format, though Pyrefly marks it experimental. scip-python is not a candidate: wrong targets, broken identities and an outdated checker.

## Deviations from the plan

- **Pyrefly 1.5.0, not 1.4.1.** 1.4.1 has no `--report-scip`; 1.5.0, released 2026-11-19, is the first release with it.
- **petclinic built with Gradle, not Maven.** The Maven route fails on macOS (above).
- **A smaller Java hand check.** 63 sites, 3 per project and kind, not 30 per project and kind. A mechanical name check covers every answer.
- **Not every Python disagreement read.** 50 per index were read, not all of them.
- **Pairing by ask point.** The pairing finds each site's column with Python's own parser, because the site records keep whole lines. When a site's name is the end of a longer name on the same line (`op.add_column(sa.Column(…))`), the longest matching name is the site's.
- **Java seeded gaps on two projects.** petclinic and realworld (Lombok), not all three.

## Not checked

- Overload choice in Java beyond the name.
- Answers in test code.
- Kotlin.
- Warm-cache indexing times.
- Whether a newer scip-python fixes its package attribution.
- Whether the empty-array failure is already reported upstream.
