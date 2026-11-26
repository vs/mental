"""Part J: pair the walker's sites with the occurrences of each project's scip-java index.

Usage: python3 -I pair_java.py <work-dir> [project ...]

Reads <work-dir>/src/java/<project> and <work-dir>/out/scip-java/<project>.scip. Only main
sources are counted: a path with `src/main/` in it, as test code is left out in part P. A site
is paired with the occurrences that start at its last name (line and UTF-16 column, scip-java
being a JVM indexer). Writes results/java.json (counts per project, site kind and form) and
results/examples-java.jsonl.gz (every site with its answer, for the hand check).
"""

import collections
import gzip
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import scip_index  # noqa: E402
import walk_java  # noqa: E402

PROJECTS = ("spring-petclinic", "mockito", "spring-boot-realworld-example-app")


def utf16(line: bytes, byte_col: int) -> int:
    return len(line[:byte_col].decode(errors="replace").encode("utf-16-le")) // 2


def origin(sym):
    """Where a global symbol is defined: the repository, the JDK, or a Maven package."""
    if sym.package == ".":
        return "repository"
    if sym.package == "jdk":
        return "jdk"
    return "maven"


def written_name(sym):
    """The name a symbol ends with, as written in source: a constructor is its class."""
    names = [n for n, s in sym.descriptors if s not in "pt"]
    if names and names[-1] == "<init>":
        return names[-2] if len(names) > 1 else ""
    return names[-1] if names else ""


def pair(work, project):
    root = work / "src/java" / project
    index = scip_index.load(work, work / "out/scip-java" / f"{project}.scip")
    counts = collections.defaultdict(collections.Counter)
    examples = []
    defined = {o.symbol for _, occs in index.documents.values() for o in occs
               if o.roles & scip_index.ROLE_DEFINITION}
    for path, (encoding, occurrences) in sorted(index.documents.items()):
        if "src/main/" not in path or not path.endswith(".java"):
            continue
        source = (root / path).read_bytes()
        lines = source.split(b"\n")
        at = collections.defaultdict(list)
        for occ in occurrences:
            at[(occ.line, occ.start)].append(occ)
        for site in walk_java.sites(source):
            key = f"{site.kind}: {site.form}"
            c = counts[key]
            c["sites"] += 1
            occs = at.get((site.line, utf16(lines[site.line], site.column)), [])
            # `Foo::new` has two occurrences at its start: the type, and the reference to the
            # constructor over the whole `Foo::new`; the longer one is the site's.
            longest = max(((o.end_line, o.end) for o in occs), default=None)
            occs = [o for o in occs if (o.end_line, o.end) == longest]
            symbols = sorted({o.symbol for o in occs if o.symbol})
            parsed = [scip_index.parse_symbol(s) for s in symbols]
            if not symbols:
                answer = "no occurrence"
            elif len(symbols) > 1:
                answer = "several symbols"
            elif parsed[0] is None:
                answer = "local"
            else:
                answer = "global"
            c[f"index {answer}"] += 1
            record = {"project": project, "path": path, "line": site.line + 1,
                      "kind": site.kind, "form": site.form, "name": site.name,
                      "answer": answer, "symbols": symbols,
                      "text": lines[site.line].decode(errors="replace").strip()[:160]}
            if answer == "global":
                sym = parsed[0]
                c[f"origin {origin(sym)}"] += 1
                # The repository's own code, by the package alone and by a definition in the
                # index: a module of a multi-module build names its siblings by coordinates.
                own = symbols[0] in defined
                c["defined in the index" if own else "not defined in the index"] += 1
                if own != (origin(sym) == "repository"):
                    c["package and definition disagree"] += 1
                name = written_name(sym)
                if site.form in ("this", "super"):  # a constructor, named by its class
                    c["name matches" if "`<init>`" in symbols[0] else "name differs"] += 1
                elif site.form == "method-ref" and site.name.endswith("::new"):
                    c["name matches" if "`<init>`" in symbols[0] else "name differs"] += 1
                else:
                    c["name matches" if name == site.name else "name differs"] += 1
                record["origin"] = origin(sym)
                info = index.symbols.get(symbols[0])
                c["has symbol information" if info else "no symbol information"] += 1
                if info and info.kind:
                    c["has a kind"] += 1
                if site.kind == "callee":
                    c["names a constructor" if "`<init>`" in symbols[0] else
                      "names a method" if sym.last_suffix == "(" else "names something else"] += 1
                if site.kind == "write-receiver":
                    field = sym.last_suffix == "." and any(s == "#" for _, s in sym.descriptors)
                    c["a field" if field else "not a field"] += 1
                if site.kind == "escape" and site.form == "argument":
                    c["argument: a method or class" if sym.last_suffix in "#(" else
                      "argument: a field (needs its type)"] += 1
                if site.kind == "escape" and site.form == "override":
                    outside = [r for r in (info.relationships if info else ())
                               if r.is_implementation and scip_index.parse_symbol(r.symbol)
                               and origin(scip_index.parse_symbol(r.symbol)) != "repository"]
                    inside = [r for r in (info.relationships if info else ())
                              if r.is_implementation and scip_index.parse_symbol(r.symbol)
                              and origin(scip_index.parse_symbol(r.symbol)) == "repository"]
                    c["overrides outside code" if outside else
                      "overrides repository code only" if inside else "overrides nothing"] += 1
                    record["overrides"] = [r.symbol for r in info.relationships] if info else []
            elif answer == "local" and site.kind == "escape" and site.form == "argument":
                c["argument: a local or parameter (needs its type)"] += 1
            elif answer == "local" and site.kind == "write-receiver":
                c["a local variable (no write site)"] += 1
            diagnostics = [d for o in occs for d in o.diagnostics]
            if diagnostics:
                c["index diagnostic"] += 1
            examples.append(record)
    return counts, examples, index


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    only = sys.argv[2:]
    results, all_examples = {}, []
    for project in PROJECTS:
        if only and project not in only:
            continue
        if not (work / "out/scip-java" / f"{project}.scip").exists():
            print(project, "has no index; skipped")
            continue
        counts, examples, index = pair(work, project)
        results[project] = {
            "tool": index.tool,
            "documents": sum(1 for p in index.documents if "src/main/" in p),
            "occurrences with a diagnostic": sum(
                1 for _, occs in index.documents.values() for o in occs if o.diagnostics),
            "kinds": {k: dict(sorted(c.items())) for k, c in sorted(counts.items())},
        }
        all_examples += examples
        total = collections.Counter()
        for c in counts.values():
            total.update(c)
        print(project, {k: total[k] for k in ("sites", "index global", "index local",
                                              "index no occurrence", "index several symbols",
                                              "name differs")})
    (HERE / "results/java.json").write_text(json.dumps(results, indent=1) + "\n")
    with gzip.open(HERE / "results/examples-java.jsonl.gz", "wt") as f:
        for e in all_examples:
            f.write(json.dumps(e) + "\n")


if __name__ == "__main__":
    main()
