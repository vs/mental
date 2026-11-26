# Part J, run 2: javac's answers read by hand

Every site where javac and scip-java name different targets was read, and a sample of the sites
only javac answers: 8 per category, drawn at random (fixed seed 4) from
`examples-javac.jsonl.gz`, each read against its source line.

## Where the two differ

| Sites | What | Judged |
|---|---|---|
| 2 | `wanted.toString()` on `DescribedInvocation`, an interface that redeclares `toString()`: javac names `java/lang/Object#toString()`, scip-java the interface's declaration | both right: two declarations of the method that runs; scip-java's is the one written in the repository |
| 1 | `Strictness.valueOf(…)`, an enum's implicit method: javac says own code, scip-java has no definition of it in the index, so the definition rule says not | javac right |
| 5 | Lombok-generated getters (`userData.getEmail()`), where scip-java puts two symbols on the call | javac right: one symbol, `UserData#getEmail()`, the receiver's |
| 6 | `javax.servlet` classes in spring-boot-realworld-example-app: javac reads them from `tomcat-embed-core`, scip-java names `jakarta.servlet-api` | javac right: `tomcat-embed-core` is the only jar on `compileJava`'s classpath holding them; scip-java's dependency list spans every configuration |
| 60 | `new Foo() { … }`, an anonymous class: javac names the anonymous class's constructor (a local symbol), scip-java the base type | both right; the base type is one more step in javac's API |
| 4 | Jackson annotations on fields of a Lombok `@Data` class (`@JsonIgnore private String articleId;`) | neither answers; with Lombok off, javac does |

## Only javac answers

| Category | In all | Read | Right |
|---|---|---|---|
| a static import's member (`import static …Reporter.notAnException;`) | 264 | 8 | 8 |
| a name handed to a call, with the call's target and the name's type | 3,118 locals and parameters, 489 fields | 8 | 8 |
| a lambda, by its functional interface (`field -> …` as `Predicate`) | 106 | 8 | 8 |

A static import's member is the tool's own lookup: javac keeps no element on the import, so the
tool lists the static members of the qualifier's class with that name. 47 of the 264 name
several overloads; the record gives the first and the count.

An argument's type is its static type: `mock` declared as `Object` is an `Object`, so the
escape rule sees no repository type there.

## Not checked

Overload choice was not compared beyond the name, in either tool.
