# Experiment plan: SCIP indexers as the checker behind `extract`

**Status: Experiment, run 2026-11-26** ([report](report.md)). **Date:** 2026-11-25. Experiment **R4** of the [specification](../../specification.md#152-experiments) (owner choice, 2026-11-26).

## Question

When Mental adds a language after Python, can a SCIP indexer be the checker behind `extract`? `extract` would keep its own syntax walk for the site list and take each site's target from the index. What does that lose compared with what Mental needs: an outcome per site, a reason when the site is unresolved, the class behind a callee, and the escapes?

Python stays on ty: scip-python bundles Pyright 1.1.301 from March 2023 ([type checkers as libraries](../type-checkers-as-libraries/report.md#per-product-summary)). Python is still in the experiment, as the calibration: it is the one language where Mental already has full answers to compare against.

## What SCIP holds

A SCIP index ([scip.proto](https://github.com/scip-code/scip/blob/main/scip.proto)) has one document per file. Each document lists **occurrences**: a range, a symbol, and role bits (definition, import, read, write). Each symbol has **symbol information**: its kind, its relationships (such as "implements" or "overrides"), and its documentation and signature as text. A symbol outside the repository names its package: manager, name and version (F-U20). Each indexer runs on the language's own compiler or checker: scip-java is a javac plugin, scip-typescript runs on `tsc`, and rust-analyzer writes SCIP itself.

From earlier reading, without measuring, three things Mental needs seem to be missing ([unresolved references](../unresolved-references/report.md#scip-names-external-symbols-kythe-records-the-compiler), and the SCIP design of [an earlier resolution draft](https://github.com/vs/mental/blob/0d3e89b/docs/resolution.md#66-indexes-scip-and-other-tools-answers)):

1. **No unresolved outcome.** When a name doesn't resolve, the index simply has no occurrence for it, or scip-python writes a file-local symbol (F-U6). A reason has nowhere to go, unless an indexer fills the occurrence's optional `diagnostics` field.
2. **No expression types.** The type of a receiver or an argument exists only as hover text, never as a structured field.
3. **No site kinds.** The role bits don't separate a callee from a base class, a decorator, an exception handler type or an escape, and an indexer may leave them unset.

This experiment measures how much each of these costs on real code, and what is left once they are taken away.

## Design

Two parts, both reading an index that `mentalc` didn't produce. The indexer runs outside `mentalc`, because it runs the project's build.

### Part P: Python, calibration

On the five projects of the type-checker experiment, at the same commits and with the same environments ([corpus](../type-checkers-as-libraries/experiments/README.md#corpus)):

1. Run `mentalc build`, which uses ty, as experiment B1 did ([findings, section B](../../findings.md#b-mentalc-build-on-outside-projects)).
2. Make two indexes: one with scip-python, at its latest release on the day of the run, pinned and recorded in the experiment's README, and one with `pyrefly check --report-scip` from Pyrefly 1.4.1. The second is a current checker, so a gap that shows only in scip-python comes from its outdated Pyright, not from SCIP.
3. Pair each site of `site.jsonl` with an occurrence: same file, an occurrence starting on the site's anchor lines, on the last name the site is written with (`commit` for `s.commit()`). When that name appears twice on those lines, the pair is ambiguous and counted apart.
4. Use ty's outcome as the reference. Normalize names with the rules of the earlier experiment's [comparison](../type-checkers-as-libraries/experiments/README.md#what-the-comparison-normalises), plus one more: a SCIP descriptor written as a module and a qualified name.

Both sides of this part are built and run, so its results can become findings.

### Part J: Java

1. **Corpus.** Three Java projects, chosen and pinned before measuring:
   - a small Spring Boot service with JPA, which is the closest analogue of the version-one stack. Candidate: [spring-petclinic](https://github.com/spring-projects/spring-petclinic), which builds with Maven and with Gradle;
   - a Gradle project with several modules;
   - a project that relies on an annotation processor such as Lombok, so some code exists only after generation.
2. **Indexes.** Run `scip-java index` on each project with the project's own build. This runs the build scripts, so it happens in the experiment's work directory: `mentalc` executes nothing the repository controls ([resolution, R3](../../resolution.md#r3-mentalc-executes-nothing-the-repository-controls)).
3. **Site list.** A walker in research code, on tree-sitter's Java grammar, lists sites with the site kinds of the specification ([6.1](../../specification.md#61-parse-and-list-sites)):

   | Site kind | Java |
   |---|---|
   | import | each `import`, static and on-demand imports included |
   | decorator | each annotation, such as `@GetMapping("/owners")` |
   | callee | each method call, `new`, and `this(…)` or `super(…)` |
   | base class | each type after `extends` or `implements` |
   | write receiver | each field assigned to; a bare name on the left can be a field or a local variable, and only the index says which |
   | escape | a method reference or lambda handed to a call; an object or method of the repository handed to outside code; a method that overrides an outside one |
   | handler type | each type in a `catch`, one per element of `A \| B` |

4. **Pairing.** As in part P, on the last name of each site, with positions converted from the document's stated encoding.
5. **Correctness.** javac is the reference for code that compiles, so a wrong answer can only come from the indexer or from the pairing. A sample of 30 paired sites per site kind and project is checked by hand in an IDE.

### Seeded gaps (both parts)

Each gap is applied alone to a copy of the project, and we record what the indexer writes. Possible results: no index; an index without the affected files; occurrences silently missing; or a diagnostic.

| Gap | Python | Java |
|---|---|---|
| a dependency removed | uninstall one package from the environment | remove one dependency from the build file |
| a name that doesn't exist | call a method no class defines | the same; the project no longer compiles |
| generated code missing | — | turn off the annotation processor |
| a dynamic call | `getattr(obj, name)()` | a reflective `Method.invoke` |

This tells us whether "no occurrence" can ever be turned into a reason, or whether every reason needs another source.

## What is measured

For each project and site kind:

| # | Measure | Parts |
|---|---|---|
| 1 | **Coverage:** the share of sites with a paired occurrence on a symbol that isn't file-local | P, J |
| 2 | **Agreement:** where ty and the index both answer, whether they give the same target; every disagreement read by hand | P |
| 3 | **Unresolved:** for each site ty leaves unresolved, by reason, what the index has: nothing, a local symbol, a symbol, or a diagnostic | P |
| 4 | **Outcome from the symbol:** whether `Resolved` and `External` can be told apart from the symbol's package alone, and whether the package maps to a pin; for Java, its Maven coordinates | P, J |
| 5 | **Class behind a callee:** whether the symbol's kind tells a class or constructor apart, as Mental needs for a constructor call | P, J |
| 6 | **Escapes:** how many of `mentalc`'s escapes (part P) or the walker's escape candidates (part J) can be decided from index data alone. Overrides should come from relationships, so the count is of how many need an argument's type, which only hover text holds | P, J |
| 7 | **Seeded gaps:** what the indexer does with each one (above) | P, J |
| 8 | **Correctness:** wrong answers in the hand-checked sample | J |
| 9 | **Cost:** time and peak memory of the indexer, what has to be installed, and what it runs | P, J |

## Designs under test

These are kept side by side, as for every experiment in this phase:

| | A. Index only | B. Thin tool on the compiler | C. Index, plus the compiler for the rest |
|---|---|---|---|
| **How** | `mentalc` reads a SCIP index; targets come from occurrences, and kinds and escapes from the walk | a small program for each language on the compiler's own API (javac's tree and type API for Java) writes Mental's records directly, as ty does in process for Python | the index answers targets; a small query on the compiler answers only what the index lacks: reasons, argument types |
| **For** | one reader for every language, maintained indexers | every outcome, reason and type, in Mental's own form | the least new code that still leaves no silent gap |
| **Against** | no reasons; the unresolved sites would be gaps with no cause named | one tool to write and maintain for each language | two sources for one site, which can disagree |

**Decide by:**

- **A is enough for a language** if:
  - measure 1 is near 100% for the code that compiles;
  - measure 7 shows that every seeded gap either stops the indexer with a message that names it, or leaves a diagnostic on the site;
  - measure 6 needs hover text for only a few escape candidates.
- **C** if the targets are right but reasons or argument types are missing.
- **B** if the index's targets themselves fall short: low coverage, or wrong answers in the sample.

Part P also decides one thing for Python: whether the Pyrefly index adds answers ty lacks. That adds evidence to the second-checker experiment (R2) without changing which checker Python uses.

This plan compares only what is run. Design B for Java isn't built, so no result here says how B would score. When the measures point to B, B is the next thing to build and measure.

## Needs and steps

1. **Step 0.** Confirm in the current `scip.proto` the fields this plan relies on: the symbol kind, relationships, occurrence diagnostics and the position encoding. Confirm in the scip-java documentation which builds it supports and which Java versions.
2. **Part P.** Needs milestone 1, which is done, and the environments of experiment B1. Code: one indexing script and one pairing script, beside the earlier experiment's `compare.py`.
3. **Part J.** Needs a JDK, Maven and Gradle in the work directory, scip-java, and tree-sitter's Java grammar for the walker. Each tool is pinned at its version, with the reason recorded in the experiment's README.
4. **Seeded gaps,** then the hand checks.
5. **Write-up:** results in `experiments/results/`, a report in this folder, measured facts as findings, and the chosen design recorded with the decision skill. The designs not chosen stay in this plan, marked as measured and not chosen.

Not in this run: TypeScript, Go, Kotlin and Rust. The pairing script and the seeded gaps carry over to their indexers unchanged, so each is a later run of the same plan.
