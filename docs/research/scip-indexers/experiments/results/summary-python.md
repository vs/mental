### Coverage and agreement, per site kind

Sites with an ask point (ty's `dynamic` left out). *ty* and *index*: sites each answers with a target. *Same*: of the sites both answer, the same target, also when only its spelling differs (a private module such as `_io`, an attribute declared in a base class, a nested function named without its enclosing one).

| Kind | Sites | ty | scip-python | same as ty | Pyrefly | same as ty |
|---|---|---|---|---|---|---|
| import | 7,941 | 97.7% | 98.0% | 95.8% | 98.3% | 99.9% |
| callee | 24,465 | 88.2% | 87.3% | 96.6% | 92.1% | 98.5% |
| decorator | 1,902 | 99.8% | 99.0% | 99.3% | 99.1% | 99.3% |
| base-class | 1,075 | 97.4% | 100.0% | 100.0% | 100.0% | 100.0% |
| write-receiver | 1,423 | 93.1% | 94.7% | 98.1% | 94.8% | 99.2% |
| escape | 1,869 | 71.4% | 79.9% | 100.0% | 78.7% | 100.0% |
| handler-type | 787 | 97.8% | 94.8% | 98.9% | 95.9% | 99.9% |
| all | 39,462 | 90.5% | 90.4% | 96.9% | 93.4% | 99.0% |

### Where the two disagree

Sites both answer with a target, by how the index's target stands to ty's.

| Category | scip-python | Pyrefly |
|---|---|---|
| index names a variable or attribute | 57 | 69 |
| index names no class | 3 | 2 |
| module named without its package | 155 | 0 |
| nested definition named without its enclosing function | 104 | 23 |
| other | 427 | 197 |
| same | 33,053 | 35,146 |
| same name, other module | 392 | 60 |
| same object, private module | 40 | 0 |
| same, attribute declared in a base | 57 | 38 |
| ty names a member of the index's answer | 22 | 24 |

### What the index has where ty is unresolved

| ty's reason | Sites | scip-python: symbol / local / nothing | Pyrefly: symbol / local / nothing |
|---|---|---|---|
| `no_type` | 2,743 | 916 / 43 / 1,784 | 867 / 37 / 1,839 |
| `not_found` | 777 | 328 / 140 / 309 | 319 / 140 / 318 |
| `ambiguous` | 99 | 94 / 2 / 3 | 96 / 2 / 1 |
| `missing_module` | 111 | 5 / 106 / 0 | 0 / 111 / 0 |
| `unreachable` | 27 | 27 / 0 / 0 | 21 / 0 / 6 |

### What the index lacks where ty answers

| Index answer | scip-python | Pyrefly |
|---|---|---|
| local | 1,217 | 57 |
| no occurrence | 102 | 69 |
| ambiguous pair | 76 | 20 |

### Resolved or External, from the symbol

Of the sites both answer: whether the index's symbol puts the target in the repository exactly when ty does, judged by the symbol's package and by its module (a module of a file under the roots); and, for ty's external targets, whether the package is ty's pin.

| | scip-python | Pyrefly |
|---|---|---|
| by package | 70.2% | 99.7% |
| by module | 99.3% | 99.7% |
| external package is ty's pin | 45.5% | 99.0% |

### Callees and escapes

| | scip-python | Pyrefly |
|---|---|---|
| callee: index symbol is a class (suffix #) | 5,062 | 4,974 |
| callee: index symbol has a kind | 0 | 0 |
| escape: decided from the symbol | 1,327 | 1,326 |

scip-python: 1,327 escapes both answer; 1,327 are a repository `def` or `class` by the symbol alone.

pyrefly: 1,326 escapes both answer; 1,326 are a repository `def` or `class` by the symbol alone.

### Cost

| Project | scip-python s | MB | Pyrefly s | MB |
|---|---|---|---|---|
| report-jobs | 1.52 | 465 | 0.24 | 235 |
| fastapi-template | 1.97 | 463 | 0.22 | 252 |
| mealie | 15.34 | 1379 | 0.8 | 624 |
| redash | 7.6 | 908 | 0.36 | 450 |
| rq | 3.66 | 422 | 0.17 | 221 |
