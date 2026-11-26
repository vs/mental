### Coverage, per site kind and form

All three projects, main sources only. *Global*: a symbol that is not file-local.

| Kind: form | Sites | global | local | no occurrence | several symbols |
|---|---|---|---|---|---|
| base-class: extends | 116 | 100.0% | 0 | 0 | 0 |
| base-class: implements | 344 | 100.0% | 0 | 0 | 0 |
| callee: method | 7,483 | 99.9% | 6 | 0 | 5 |
| callee: new | 1,229 | 100.0% | 0 | 0 | 0 |
| callee: super | 76 | 100.0% | 0 | 0 | 0 |
| callee: this | 28 | 100.0% | 0 | 0 | 0 |
| decorator: annotation | 1,680 | 99.8% | 0 | 4 | 0 |
| escape: argument | 3,607 | 13.6% | 3,118 | 0 | 0 |
| escape: lambda | 106 | 0.0% | 85 | 21 | 0 |
| escape: method-ref | 70 | 100.0% | 0 | 0 | 0 |
| escape: override | 2,620 | 97.4% | 66 | 2 | 0 |
| handler-type: catch | 209 | 100.0% | 0 | 0 | 0 |
| import: on-demand | 10 | 100.0% | 0 | 0 | 0 |
| import: single | 3,217 | 100.0% | 0 | 0 | 0 |
| import: static | 264 | 0.0% | 0 | 264 | 0 |
| write-receiver: bare | 451 | 32.2% | 306 | 0 | 0 |
| write-receiver: field | 455 | 99.6% | 2 | 0 | 0 |
| all | 21,965 | 82.3% | 3,583 | 291 | 5 |

### Own code or a dependency

Global answers whose target is the repository's own code: by the symbol's package (`. .`) and by a definition of the symbol in the index.

| Project | Global answers | own, by package | own, by definition | disagree |
|---|---|---|---|---|
| spring-petclinic | 703 | 225 | 225 | 0 |
| mockito | 14,402 | 11 | 8,669 | 8,680 |
| spring-boot-realworld-example-app | 2,981 | 1,340 | 1,338 | 2 |

### What the symbols tell

| | Count |
|---|---|
| `new` sites naming a constructor | 1,169 of 1,229 (the rest: anonymous classes, named by their type) |
| callees with symbol information (a kind) | 4,621 of 8,701: only the repository's own symbols have it |
| method declarations that override outside code | 188 of 2,552 |
| … that override repository code only | 625 |
| arguments that are a local or parameter (need a type) | 3,118 of 3,607 |
| arguments that are a field (need its type) | 489 |
| bare assignments the index shows are a field | 145 of 451 (the rest: local variables) |
| lambdas handed to a call | 106, never named |
| occurrences with a diagnostic, in all three indexes | 0 |

### Seeded gaps

| Gap | Project | exit | index written | main documents | on the seeded line |
|---|---|---|---|---|---|
| dependency-removed | spring-petclinic | 1 | False | — | — |
| name-not-defined | spring-petclinic | 1 | False | — | — |
| dynamic-call | spring-petclinic | 0 | True | 30 | line 60: `Owner` → `org/springframework/samples/petclinic/owner/Owner#`, `class` → `org/springframework/samples/petclinic/owner/Owner#class.`, `getMethod` → `java/lang/Class#getMethod().`, `invoke` → `java/lang/reflect/Method#invoke().`, `owner` → `1` |
| annotation-processor-off | spring-boot-realworld-example-app | 1 | False | — | — |
