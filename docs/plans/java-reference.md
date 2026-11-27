# Plan: a Java twin of report-jobs and its reference model

**Status:** built, 2026-11-27. The example, its hand-written reference model and both checks exist. The site rules are in the [specification](../specification.md#67-java-sites) (owner choice, 2026-11-27). The pin rules of section 3 are **Proposed**: they come into the specification with the plan that reads a Maven project. Nothing here chooses how `mentalc` would read Java. The choice between an index and a thin tool on javac stays open ([R4](../research/scip-indexers/report.md)).

## 1. Goal

Python and Java each have one small system with a hand-written reference model, kept true by tests:

| | Python | Java |
|---|---|---|
| Example | [`examples/report-jobs`](../../examples/report-jobs/) | [`examples/report-jobs-java`](../../examples/report-jobs-java/) |
| Reference | `model/`: 1 build unit, 40 pins, 140 sites | `model/`: 1 build unit, 130 pins, 251 sites |
| Checked against | the example's files, and `mentalc build` | the example's files, and javac and Maven |

The Java example is the same system as report-jobs, with the same four bugs, so each milestone's "done when" has a Java counterpart, and the two references can be compared fact by fact.

## 2. Decisions

| # | Question | Decision | Why |
|---|---|---|---|
| J1 | Which system? | A twin of report-jobs: Spring Boot 4.2 (web, JPA), RabbitMQ for the queue, the AWS SDK's S3 client, a picocli command; one jar with the profiles `api`, `worker`, `beat` and `cli` (owner choice, 2026-11-27) | every acceptance fact of report-jobs carries over |
| J2 | Which build? | Maven, with `spring-boot-starter-parent` and the AWS SDK's BOM, as a Spring Boot project is usually set up; Java 25 (owner choice, 2026-11-27) | the common case. Maven has no lockfile, so exact versions come only from Maven's own resolution (F-U19) |
| J3 | What keeps the reference true before `mentalc` reads Java? | The invariant tests both references share, and a check against javac and Maven, slow and local like report-jobs' environment tests (owner choice, 2026-11-27) | a wrong outcome is caught now, not when a Java build exists |
| J4 | Who writes the reference? | A person, as for report-jobs: sites, names and targets by hand. The group counters, the quotes taken from the cited lines and the sorting are mechanical. The pins are Maven's resolution, written out | the javac check is only independent if the reference does not come from javac |
| J5 | Which records? | Those of milestone 1: the build unit, its pins and its sites, with their anchors. The build unit has only `id`, `dir` and `roots`: its Java version and its Maven inputs get fields when a plan specifies how Maven is read | the reference grows with the milestones (T6), and a field nothing specifies yet would be a guess |
| J6 | What does the RabbitMQ twin change? | Celery's `task_acks_late` on Redis becomes a listener that acknowledges after the method returns, which is also at-least-once. A retry is sent again at once, without Celery's 30 s delay. `requeue_stuck` runs in the `beat` process rather than on a worker. No bug depends on any of these | Spring has no Celery; RabbitMQ is what its listener acknowledges |

## 3. Pins

A build unit's pins are the artifacts Maven resolves for it, in every scope, one per `group:artifact`, at the version Maven picks:

- `package` is `group:artifact`, such as `org.springframework.data:spring-data-commons`;
- the ID's slug comes from that, such as `pin:org-springframework-data-spring-data-commons`;
- the origin is the registry Maven Central, `https://repo.maven.apache.org/maven2`;
- the anchor is the `<artifactId>` line in `pom.xml` of the direct dependency the artifact comes through in Maven's tree (`mvn dependency:tree`). For a direct dependency, that is its own line.

Plugins are not pins: they build the project, and no reference points to them.

## 4. Sites

The site rules are in the specification, [6.7](../specification.md#67-java-sites) (owner choice, 2026-11-27): which sites a Java file has, their scopes and names, and the target each is asked for. The checker is `{"name": "javac", "version": "25"}`, the JDK's feature version.

None of report-jobs-java's sites is unresolved. Python's sites are unresolved for reasons such as `no_type`; Java's checker types everything that compiles. Which reasons code that does not compile gets ([F-I7](../findings.md#i-scip-indexes-experiment-r4)) is for the plan that reads Java.

## 5. Checks

| Test | Runs | Checks |
|---|---|---|
| `model/tests/reference.rs` | always, in CI | both references: canonical JSONL, one anchor per record, every quote at its lines; for report-jobs, a pin per locked package |
| `extract/tests/java_reference.rs` | when `RUN_SLOW_TESTS` is set, after `mvn dependency:go-offline` in the example, with a JDK 25 first on `PATH` | every site, outcome and anchor equals what `extract/tests/javac/ReferenceSites.java` gets from javac's trees and elements; the pins equal Maven's resolved artifacts and versions, each anchored to the dependency it comes through |

The javac tool is written from the rules of the specification's 6.7 and uses only javac's public API: the Compiler Tree API, `javax.lang.model`, and the file manager to find a class's jar. Run on the hand-written reference, it agreed on all 251 sites and their anchors the first time. A changed target or pin version makes the check fail.

## 6. Not done

- How `mentalc` reads a Maven project: the build model of a project with no lockfile, and whether Maven runs or its local repository is read as files.
- The records after milestone 1 (facts, the mechanical model), for both references. Each milestone writes them for both.
- On-demand imports, static imports, anonymous and local classes (specification 6.7). The example has none, and the javac tool stops on an on-demand import rather than guess.
- CI runs neither example's slow checks: alpha's generated jobs can't install uv's environment, a JDK or Maven's cache yet (CONTRIBUTING.md, Exceptions).
