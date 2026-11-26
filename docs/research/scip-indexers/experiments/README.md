# Experiment R4: SCIP indexes against `mentalc`'s sites

Research code behind the [report](../report.md) of experiment R4 and its [plan](../plan.md). Part P pairs the sites
`mentalc build` writes on five Python projects with two SCIP indexes of the same code. Part J
pairs a Java site walk with scip-java's index on three Java projects. Both parts also break a
project on purpose and record what each tool writes.

## Layout

| Path | What it does |
|---|---|
| `run_python.py` | Indexes each Python project with scip-python and with Pyrefly's SCIP report, set up from the build unit `mentalc` wrote (roots, Python version, environment); records time and peak memory |
| `scip_index.py` | Reads a SCIP index (protobuf, generated from the pinned `scip.proto`) and parses SCIP symbols |
| `ask_points.py` | Where `extract` asks the checker for each site kind, found with Python's `ast`: the site records keep whole lines, so this gives the column |
| `pair_python.py` | Pairs each `mentalc` site with the occurrences at its ask point and compares the index's target with ty's |
| `summarize_python.py` | The tables of part P (`results/summary-python.md`) |
| `seeds_python.py` | Part P's seeded gaps on copies of report-jobs |
| `walk_java.py` | Part J's site walk, on tree-sitter's Java grammar |
| `pair_java.py` | Pairs the Java sites with scip-java's occurrences |
| `seeds_java.py` | Part J's seeded gaps on copies of spring-petclinic and spring-boot-realworld-example-app |
| `results/` | What the scripts write, and the hand checks |

## Tools

| Tool | Version | Why this one |
|---|---|---|
| scip-python | 0.6.8 (npm `@sourcegraph/scip-python`) | the latest release on 2026-11-25 |
| Pyrefly | 1.5.0 | the plan named 1.4.1, which has no `--report-scip`; 1.5.0 (2026-11-19) is the first release with it |
| scip-java | 0.14.2 (GitHub release, SHA-256 checked) | the latest release on 2026-11-25 |
| `scip.proto` | [scip-code/scip](https://github.com/scip-code/scip) at `8fe5037` (2026-11-21) | the schema the index reader is generated from |
| protobuf, grpcio-tools | 7.36.2, 1.85.4 | to read the indexes and to generate `scip_pb2.py` |
| tree-sitter, tree-sitter-java | 0.27.0, 0.23.5 | the Java site walk |
| `mentalc` | this repository's `main` at `3ca28cd` | the sites and ty's answers |

## Corpus

Python: the five projects and commits of the [type-checker experiment](../../type-checkers-as-libraries/experiments/README.md#corpus), set up as experiment B1 did ([findings, section B](../../../findings.md#b-mentalc-build-on-outside-projects)). Only the analysed folders count, without tests.

Java, chosen and pinned on 2026-11-25 before any Java measurement:

| Project | Commit | Build | Main source files | Why |
|---|---|---|---|---|
| [spring-petclinic](https://github.com/spring-projects/spring-petclinic) | `500158f` | Gradle 9.5.1 (also Maven) | 30 | a small Spring Boot service with JPA |
| [mockito](https://github.com/mockito/mockito) | `32938e6e7` | Gradle 9.8.2, several modules | 497 | a Gradle project with several modules |
| [spring-boot-realworld-example-app](https://github.com/gothinkster/spring-boot-realworld-example-app) | `ee17e31` | Gradle 7.4 | 93 | relies on Lombok, an annotation processor |

## Reproduce

```sh
W=<work dir>
git clone https://github.com/scip-code/scip $W/src/scip && git -C $W/src/scip checkout 8fe5037
uv venv --python 3.14 $W/py
uv pip install --python $W/py/bin/python \
  protobuf==7.36.2 grpcio-tools==1.85.4 tree-sitter==0.27.0 tree-sitter-java==0.23.5
$W/py/bin/python -m grpc_tools.protoc -I $W/src/scip --python_out=$W/gen $W/src/scip/scip.proto
uv venv --python 3.12 $W/pyrefly15 && uv pip install --python $W/pyrefly15/bin/python pyrefly==1.5.0
(mkdir -p $W/node && cd $W/node && npm init -y && npm install @sourcegraph/scip-python@0.6.8)
curl -L -o $W/bin/scip-java-v0.14.2 \
  https://github.com/sourcegraph/scip-java/releases/download/v0.14.2/scip-java-v0.14.2
chmod +x $W/bin/scip-java-v0.14.2
# Python corpus in $W/src/{report-jobs,full-stack-fastapi-template,mealie,redash,rq}, each
# with its environment and `mentalc build` run as in the type-checker experiment's README.
NODE_OPTIONS=--max-old-space-size=8192 $W/py/bin/python -I run_python.py $W
$W/py/bin/python -I pair_python.py $W scip-python && $W/py/bin/python -I pair_python.py $W pyrefly
$W/py/bin/python -I summarize_python.py $W
$W/py/bin/python -I seeds_python.py $W
# Java corpus in $W/src/java/<project> at the commits above.
for p in spring-petclinic spring-boot-realworld-example-app mockito; do
  (cd $W/src/java/$p && JAVA_HOME=$(/usr/libexec/java_home -v 17) \
    $W/bin/scip-java-v0.14.2 index --build-tool=gradle --output $W/out/scip-java/$p.scip)
done
$W/py/bin/python -I pair_java.py $W
$W/py/bin/python -I seeds_java.py $W
```

The indexers run the projects' own builds (scip-java runs Gradle and its build scripts) in the
work directory, never inside `mentalc`. scip-python gets an environment file made from the
installed `RECORD` files, so it does not call pip; Pyrefly gets the site-packages path with
`skip-interpreter-query`.
