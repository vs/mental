"""Part J, run 2: pair the walker's Java sites with the javac tool's answers and scip-java's.

Usage: python3 -I pair_javac.py <work-dir>

Reads <work-dir>/out/javac/<project>/*.jsonl (run_javac.py) and, for the same sites,
<work-dir>/out/scip-java/<project>.scip. Main sources only, as in pair_java.py. Writes
results/javac.json (counts per project, site kind and form, and the tool's cost) and
results/examples-javac.jsonl.gz (every site where the two differ, for the hand check).
seeds_javac.py pairs its broken copies with `pair` and no index.
"""

import collections
import gzip
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import pair_java  # noqa: E402
import scip_index  # noqa: E402
import walk_java  # noqa: E402

OWN = ("source", "repository")
# The record that answers a site, by kind and form; the rest are answered by the name's record.
RECORD = {("import", "static"): "static-import", ("callee", "new"): "new",
          ("escape", "method-ref"): "method-ref", ("escape", "lambda"): "lambda",
          ("escape", "argument"): "arg", ("escape", "override"): "method"}
NAME_RECORDS = ("identifier", "select")
GRADLE_JAR = re.compile(r"/files-2\.1/([^/]+)/([^/]+)/([^/]+)/[^/]+/[^/]+\.jar$")


def load(out_dir):
    """The tool's records by (path, 0-based line, UTF-16 column), its diagnostics, its runs."""
    at, diagnostics = collections.defaultdict(list), []
    for f in sorted(out_dir.glob("*.jsonl")):
        for line in f.open():
            r = json.loads(line)
            if r["t"] == "diag":
                diagnostics.append(r)
            elif r["t"] != "done":
                at[(r["path"], r["line"] - 1, r["col"])].append(r)
    return at, diagnostics


def coordinates(origin):
    """`group:artifact` for a jar in Gradle's cache; the origin itself otherwise."""
    m = GRADLE_JAR.search(origin or "")
    return f"{m.group(1)}:{m.group(2)}" if m else origin


def place(origin):
    """Where a target is defined, without this machine's paths."""
    if origin in OWN or origin in ("jdk", "package", None):
        return origin
    return "jar" if GRADLE_JAR.search(origin) else "other"


def last_name(symbol):
    return re.split(r"[/#.]", symbol.rstrip("/#.").removesuffix("()"))[-1]


def javac_answer(records, site):
    """The site's record, and its answer: global, local, error, no element or no record."""
    want = RECORD.get((site.kind, site.form))
    found = [r for r in records if (r["t"] if r["t"] != "ref" else r["tree"]) == want
             or (want is None and r["t"] == "ref" and r["tree"] in NAME_RECORDS)]
    key = "callee_symbol" if want == "arg" else "symbol"
    if want == "lambda":
        return (found[0], "type" if found[0]["type"] not in (None, "<error>") else "error") \
            if found else (None, "no record")
    symbols = {r.get(key) for r in found}
    if len(symbols) > 1:
        # Lombok gives the code it generates the position of its annotation.
        name = "<init>" if site.form in ("new", "this", "super") else site.name.split("::")[-1]
        found = [r for r in found if r.get(key) and last_name(r[key]) == name] or found
        symbols = {r.get(key) for r in found}
    if not found:
        return None, "no record"
    if len(symbols) > 1:
        return found[0], "several symbols"
    r, symbol = found[0], found[0].get(key)
    if want == "arg":
        if symbol in (None, "<error>") or r["type"] in (None, "<error>"):
            return r, "error"
        return r, "global"
    if symbol is None:
        return r, "no element"
    return r, "error" if symbol == "<error>" else "local" if symbol == "local" else "global"


def scip_symbol(symbol):
    """A SCIP symbol in the tool's form: descriptors only, without overload numbers."""
    sym = scip_index.parse_symbol(symbol)
    return "".join(n + ("()." if s == "(" else s) for n, s in sym.descriptors if s not in "pt")


def scip_coordinates(symbol):
    sym = scip_index.parse_symbol(symbol)
    return sym.package.removeprefix("maven/").replace("/", ":")


def main_sources(proj):
    return sorted(str(p.relative_to(proj)) for p in proj.rglob("src/main/**/*.java")
                  if "/build/" not in str(p.relative_to(proj)))


def answers(proj, at):
    """Each site of the main sources with its line, UTF-16 column, record and javac's answer."""
    for path in main_sources(proj):
        lines = (proj / path).read_bytes().split(b"\n")
        for site in walk_java.sites(b"\n".join(lines)):
            col = pair_java.utf16(lines[site.line], site.column)
            r, answer = javac_answer(at.get((path, site.line, col), []), site)
            yield path, lines, site, col, r, answer


def pair(project, proj, out_dir, index=None):
    """Counts per kind and form, and the sites where javac and the index differ."""
    at, diagnostics = load(out_dir)
    occurrences = pair_java.occurrences_at(index) if index else {}
    defined = ({o.symbol for _, occs in index.documents.values() for o in occs
                if o.roles & scip_index.ROLE_DEFINITION} if index else set())
    errors = collections.Counter((d["path"], d["line"]) for d in diagnostics
                                 if d["kind"] == "ERROR")
    counts, examples = collections.defaultdict(collections.Counter), []
    for path, lines, site, col, r, answer in answers(proj, at):
        c = counts[f"{site.kind}: {site.form}"]
        c["sites"] += 1
        c[f"javac {answer}"] += 1
        if answer in ("error", "no record", "no element"):
            c[f"javac {answer}, error on the line" if errors[(path, site.line + 1)]
              else f"javac {answer}, no error on the line"] += 1
        record = {"project": project, "path": path, "line": site.line + 1, "column": col,
                  "kind": site.kind, "form": site.form, "name": site.name,
                  "javac": answer,
                  "text": lines[site.line].decode(errors="replace").strip()[:160]}
        if r is not None and answer in ("global", "local"):
            own_key = "callee_origin" if site.form == "argument" else "origin"
            record["javac symbol"] = r.get("callee_symbol" if site.form == "argument"
                                           else "symbol")
            o = r.get(own_key)
            record["javac origin"] = coordinates(o) if place(o) == "jar" else place(o)
            c[f"javac origin {place(r.get(own_key))}"] += 1
        if site.form in ("argument", "lambda") and r is not None:
            record["javac type"] = r["type"]
        if site.form == "argument" and answer == "global":
            outside = r["callee_origin"] not in OWN
            own_type = r.get("type_origin") in OWN
            c["argument: decided"] += 1
            c["argument: escapes (own type to outside code)"] += outside and own_type
        if site.form == "lambda" and answer == "type":
            c["lambda: functional interface known"] += 1
        if site.form == "override" and answer == "global":
            targets = [place(o["origin"]) for o in r["overrides"]]
            record["javac overrides"] = (
                "overrides outside code" if any(t not in OWN for t in targets) else
                "overrides repository code only" if targets else "overrides nothing")
            c[record["javac overrides"]] += 1
        if site.form == "static" and answer == "global":
            c["static import: several members"] += r["members"] > 1
        if index is None or path not in occurrences:
            continue
        _, symbols, scip = pair_java.index_answer(occurrences[path], site.line, col)
        record["scip"] = scip
        record["scip symbols"] = symbols
        c[f"scip {scip}, javac {answer}"] += 1
        if scip == "global" and answer == "global" and site.form != "argument":
            same = scip_symbol(symbols[0]) == record["javac symbol"]
            c["target same" if same else "target differs"] += 1
            own_scip = symbols[0] in defined
            own_javac = r["origin"] in OWN
            c["own code: same" if own_scip == own_javac else "own code: differs"] += 1
            if own_scip != own_javac:
                examples.append({**record, "class": "own code differs"})
            if not own_javac and place(r["origin"]) == "jar":
                jar_same = coordinates(r["origin"]) == scip_coordinates(symbols[0])
                c["jar: same coordinates" if jar_same else "jar: other coordinates"] += 1
                if not jar_same:
                    examples.append({**record, "class": "other coordinates",
                                     "scip coordinates": scip_coordinates(symbols[0])})
            if not same:
                examples.append({**record, "class": "target differs"})
            if site.form == "override":
                record["scip overrides"] = pair_java.overrides(index, symbols[0], defined)
                agree = record["scip overrides"] == record["javac overrides"]
                c["overrides: same" if agree else "overrides: differ"] += 1
                if not agree:
                    examples.append({**record, "class": "overrides differ"})
        elif (scip == "global") != (answer in ("global", "type")):
            examples.append({**record, "class": f"scip {scip}, javac {answer}"})
    return counts, examples, diagnostics


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    results, all_examples = {}, []
    for project in pair_java.PROJECTS:
        proj = work / "src/java" / project
        out_dir = work / "out/javac" / project
        index = scip_index.load(work, work / "out/scip-java" / f"{project}.scip")
        counts, examples, diagnostics = pair(project, proj, out_dir, index)
        run = json.loads((out_dir / "run.json").read_text())
        results[project] = {
            "gradle seconds": run["gradle seconds"],
            "tool": {name: {k: v for k, v in p.items() if k != "stderr"}
                     for name, p in run["projects"].items()},
            "errors": sum(d["kind"] == "ERROR" for d in diagnostics),
            "kinds": {k: dict(sorted(c.items())) for k, c in sorted(counts.items())},
        }
        all_examples += examples
        total = collections.Counter()
        for c in counts.values():
            total.update(c)
        print(project, {k: total[k] for k in ("sites", "javac global", "javac local",
                                              "javac error", "javac no record",
                                              "target same", "target differs")})
    (HERE / "results/javac.json").write_text(json.dumps(results, indent=1) + "\n")
    with gzip.open(HERE / "results/examples-javac.jsonl.gz", "wt") as f:
        for e in all_examples:
            f.write(json.dumps(e) + "\n")


if __name__ == "__main__":
    main()
