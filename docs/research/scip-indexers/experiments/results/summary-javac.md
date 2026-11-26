### Coverage, per site kind and form

All three projects, main sources only. *javac answers*: an element that is not local, or for a lambda its functional interface; *same target*: of the sites both name with a global symbol.

| Kind: form | Sites | scip-java global | javac answers | javac local | javac none | same target |
|---|---|---|---|---|---|---|
| base-class: extends | 116 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| base-class: implements | 344 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| callee: method | 7,483 | 99.9% | 99.9% | 6 | 0 | 100.0% |
| callee: new | 1,229 | 100.0% | 95.1% | 60 | 0 | 100.0% |
| callee: super | 76 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| callee: this | 28 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| decorator: annotation | 1,680 | 99.8% | 99.8% | 0 | 4 | 100.0% |
| escape: argument | 3,607 | 13.6% | 100.0% | 0 | 0 | — |
| escape: lambda | 106 | 0.0% | 100.0% | 0 | 0 | — |
| escape: method-ref | 70 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| escape: override | 2,620 | 97.4% | 97.5% | 66 | 0 | 100.0% |
| handler-type: catch | 209 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| import: on-demand | 10 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| import: single | 3,217 | 100.0% | 100.0% | 0 | 0 | 100.0% |
| import: static | 264 | 0.0% | 100.0% | 0 | 0 | — |
| write-receiver: bare | 451 | 32.2% | 32.2% | 306 | 0 | 100.0% |
| write-receiver: field | 455 | 99.6% | 99.6% | 2 | 0 | 100.0% |
| all | 21,965 | 82.3% | 98.0% | 440 | 4 | 100.0% (2 differ) |

### Beyond the targets

| | Count |
|---|---|
| own code or not, javac against scip-java's definition rule | 17,534 same, 3 differ |
| jar targets: same Maven coordinates | 2,277 of 2,283 |
| arguments with a callee and a type | 3,607 of 3,607 |
| … that escape (a repository type handed to outside code) | 116 |
| methods that override outside code, javac | 188 of 2,554 |
| … same classification as scip-java's relationships | 2,552 of 2,552 |
| static imports with several members (overloads) | 47 of 264 |
| javac errors on the unbroken projects | 0 |

### Cost

Warm Gradle cache; the Gradle step runs only what `compileJava` depends on.

| Project | Gradle step | Module | Sources | javac tool | Peak memory |
|---|---|---|---|---|---|
| spring-petclinic | 3.3 s | root | 30 | 0.71 s | 262 MB |
| mockito | 6.3 s | mockito-core | 486 | 1.6 s | 542 MB |
| mockito | 6.3 s | mockito-extensions-mockito-errorprone | 5 | 0.64 s | 204 MB |
| mockito | 6.3 s | mockito-extensions-mockito-junit-jupiter | 6 | 0.57 s | 210 MB |
| spring-boot-realworld-example-app | 3.0 s | root | 115 | 1.52 s | 505 MB |

### Seeded gaps

*Answered*: sites with an answer, against the same files before the gap.

| Gap | Policy | tool exit | errors | files without an error: answered | files with one: answered | on the seeded line |
|---|---|---|---|---|---|---|
| dependency-removed | default | {0} | 4 | 771 of 771 | 10 of 14 | `MutableConfiguration` error (compiler.err.doesnt.exist: package javax.cache.configuration does not exist) |
| dependency-removed | flow | {0} | 4 | 771 of 771 | 10 of 14 | `MutableConfiguration` error (compiler.err.doesnt.exist: package javax.cache.configuration does not exist) |
| name-not-defined | default | {0} | 1 | 665 of 665 | 120 of 120 | `noSuchMethod` error (compiler.err.cant.resolve.location.args: cannot find symbol) |
| name-not-defined | flow | {0} | 1 | 665 of 665 | 120 of 120 | `noSuchMethod` error (compiler.err.cant.resolve.location.args: cannot find symbol) |
| dynamic-call | default | {0} | 0 | 789 of 785 | — | `invoke` global, `getMethod` global, `owner` global |
| dynamic-call | flow | {0} | 0 | 789 of 785 | — | `invoke` global, `getMethod` global, `owner` global |
| annotation-processor-off | default | {0} | 213 | 856 of 852 | 2,165 of 2,562 | — |
| annotation-processor-off | flow | {0} | 213 | 856 of 852 | 2,172 of 2,562 | — |
