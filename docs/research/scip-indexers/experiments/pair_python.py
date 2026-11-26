"""Part P: pair the sites `mentalc build` wrote with the occurrences of a SCIP index.

Usage: python3 -I pair_python.py <work-dir> <tool> [project ...]

<tool> is scip-python or pyrefly (out/<tool>/<name>.scip). Writes results/python-<tool>.json
(counts per project and site kind for the measures of the plan) and
results/examples-<tool>.jsonl.gz (every site whose answers differ, or that only one side answers,
with its line, for reading by hand).

A site is paired with the occurrences that start where `extract` asks for its kind
(ask_points.py), on the site's anchor lines, at a name the site's slug ends with, with or without
its counter. When the paired occurrences carry more than one symbol, the pair is ambiguous and
counted apart.
"""

import collections
import gzip
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import ask_points  # noqa: E402
import run_python  # noqa: E402
import scip_index  # noqa: E402

SKIP = re.compile(r"(^|/)(tests?|testing)/")
NAME = re.compile(r"[\w.]+")
OWN = run_python.PROJECT_NAME


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


def site_slug(site_id):
    return site_id.rsplit("/site:", 1)[1]


def repository_modules(proj, roots):
    """The module of every .py file under the roots, as mentalc names it."""
    out = set()
    for root in roots:
        base = (proj / root).resolve()
        for f in base.rglob("*.py"):
            rel = f.relative_to(base).with_suffix("")
            parts = list(rel.parts)
            if parts[-1] == "__init__":
                parts.pop()
            if parts and not any(p.startswith(".") for p in parts):
                out.add(".".join(parts))
    return out


def ty_target(outcome):
    """(class, package, module, name) of mentalc's outcome."""
    cls, body = next(iter(outcome.items()))
    if cls == "unresolved":
        return cls, None, None, body["reason"]
    package = None
    if cls == "external":
        p = body["package"]
        package = "stdlib" if p == "stdlib" else slug(p["pin"].rsplit("/pin:", 1)[1])
    return cls, package, body["module"], body.get("name", "")


def index_target(sym):
    """(package, module, name) of a global SCIP symbol; name is '' for a module."""
    package = "stdlib" if sym.package == "python-stdlib" else slug(sym.package)
    if sym.package == OWN:
        package = OWN
    descriptors = [d for d in sym.descriptors if d[1] not in "pt"]
    if descriptors and descriptors[-1] == ("__init__", ":"):
        descriptors = descriptors[:-1]
    module = ".".join(n for n, s in descriptors if s == "/")
    name = ".".join(n for n, s in descriptors if s != "/")
    return package, module, name


def compare(kind, ty, occ_symbol, index):
    """How the index's answer stands to ty's, for a site ty resolved or made external."""
    _, _, t_module, t_name = ty
    sym = scip_index.parse_symbol(occ_symbol)
    _, module, name = index_target(sym)
    if kind == "write-receiver":
        # ty answers the class written to; the index, the attribute: compare its owner.
        owner_symbol = occ_symbol.rsplit("#", 1)[0] + "#" if "#" in occ_symbol else None
        if owner_symbol:
            _, module, name = index_target(scip_index.parse_symbol(owner_symbol))
            if (module, name) == (t_module, t_name):
                return "same"
            classes = [k for k in index.symbols if k.endswith("#")
                       and index_target(scip_index.parse_symbol(k))[1:] == (t_module, t_name)]
            if any(owner_symbol in scip_index.ancestors(index, k) for k in classes):
                return "same, attribute declared in a base"
            return "other"
        return "index names no class"
    t_name = t_name or ""
    if (module, name) == (t_module, t_name) or (not name and t_name == module):
        return "same"
    public = lambda m: ".".join(part.lstrip("_") for part in m.split("."))  # noqa: E731
    if name == t_name and public(module) == public(t_module):
        return "same object, private module"
    if not name and not t_name and t_module.endswith("." + module):
        return "module named without its package"
    t_parts, parts = t_name.split("."), name.split(".")
    if (module == t_module and len(t_parts) > len(parts) > 0 and t_parts[0] == parts[0]
            and t_parts[-1] == parts[-1]):
        return "nested definition named without its enclosing function"
    if name and parts[-1] == t_parts[-1] and module != t_module:
        return "same name, other module"
    if sym.last_suffix == "." and not t_name.endswith(parts[-1]):
        return "index names a variable or attribute"
    if t_name.startswith(name + ".") and module == t_module:
        return "ty names a member of the index's answer"
    return "other"


def pair(work, tool, name, proj, dirs):
    unit = run_python.build_unit(proj)
    sites = [json.loads(l) for l in (proj / ".mental/model/site.jsonl").open()]
    anchors = {}
    for line in (proj / ".mental/anchors.jsonl").open():
        a = json.loads(line)
        anchors[a["id"]] = a
    index = scip_index.load(work, work / "out" / tool / f"{name}.scip")
    lines = scip_index.by_line(index)
    modules = repository_modules(proj, unit["roots"])
    texts = {}

    counts = collections.defaultdict(collections.Counter)
    examples = []
    for site in sites:
        a = anchors[site["id"]]
        path = a["path"]
        if not any(path.startswith(d + "/") for d in dirs) or SKIP.search(path):
            continue
        kind = site["kind"]
        ty = ty_target(site["outcome"])
        c = counts[kind]
        c["sites"] += 1
        c[f"ty {ty[0]}"] += 1
        if ty[0] == "unresolved" and ty[3] == "dynamic":
            c["no ask point (ty: dynamic)"] += 1
            continue
        if path not in texts:
            source = (proj / path).read_text()
            texts[path] = (source.splitlines(), ask_points.ask_points(source))
        text, points = texts[path]
        encoding, by = lines.get(path, ("none", {}))
        s = site_slug(site["id"])
        bare = re.sub(r"-\d+$", "", s)  # mentalc appends one counter
        first, last = a["lines"]
        candidates = []
        for (ln, cp), kinds in points.items():
            if kind not in kinds or not first - 1 <= ln < last:
                continue
            w = slug(NAME.match(text[ln], cp).group(0))
            if s.endswith("-" + w) or bare.endswith("-" + w):
                candidates.append((len(w), ln, cp))
        # `op.add_column(sa.Column(...))`: the site ends in `add-column`, which also ends in
        # `column`; the longest name the slug ends with is the site's own.
        longest = max((n for n, _, _ in candidates), default=0)
        asked, matched = 0, []
        for n, ln, cp in candidates:
            if n != longest:
                continue
            asked += 1
            col = ask_points.to_encoding(text[ln], cp, encoding)
            matched += [o for o in by.get(ln, ()) if o.start == col and o.symbol]
        if not asked:
            c["pairing: no ask point found"] += 1
            examples.append({"project": name, "path": path, "line": first, "kind": kind,
                             "site": s, "ty": site["outcome"], "index": [],
                             "text": text[first - 1].strip()[:160],
                             "class": "pairing: no ask point found"})
            continue
        symbols = {o.symbol for o in matched}
        diagnostics = [d for o in matched for d in o.diagnostics]
        if diagnostics:
            c["index diagnostic"] += 1
        if not matched:
            answer = "no occurrence"
        elif len(symbols) > 1:
            answer = "ambiguous pair"
        else:
            only = next(iter(symbols))
            answer = "global" if scip_index.parse_symbol(only) else "local"
        c[f"index {answer}"] += 1
        record = {"project": name, "path": path, "line": first, "kind": kind,
                  "site": s, "ty": site["outcome"], "index": sorted(symbols),
                  "text": text[first - 1].strip()[:160]}

        if ty[0] == "unresolved":
            c[f"ty unresolved {ty[3]}: index {answer}"] += 1
            if answer == "global":
                examples.append({**record, "class": f"ty unresolved {ty[3]}, index global"})
            continue
        if answer != "global":
            c[f"ty answers: index {answer}"] += 1
            examples.append({**record, "class": f"ty answers, index {answer}"})
            continue
        symbol = next(iter(symbols))
        sym = scip_index.parse_symbol(symbol)
        package, module, _ = index_target(sym)
        # Resolved or External from the symbol's package alone, and from its module.
        own_by_package = package == OWN
        own_by_module = module in modules
        ty_own = ty[0] == "resolved"
        c["outcome class right, by package"] += own_by_package == ty_own
        c["outcome class right, by module"] += own_by_module == ty_own
        if ty[0] == "external" and not own_by_package:
            c["external package right"] += package == ty[1]
        verdict = compare(kind, ty, symbol, index)
        c[f"target {verdict}"] += 1
        if verdict != "same":
            examples.append({**record, "class": f"target {verdict}"})
        if kind == "callee":
            c["callee: index symbol is a class (suffix #)"] += sym.last_suffix == "#"
            info = index.symbols.get(symbol)
            c["callee: index symbol has a kind"] += bool(info and info.kind)
        if kind == "escape":
            decided = own_by_module and sym.last_suffix in "#("
            c["escape: decided from the symbol"] += decided
    return counts, examples


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    tool = sys.argv[2]
    only = sys.argv[3:]
    out = HERE / "results"
    out.mkdir(exist_ok=True)
    results, all_examples = {}, []
    for name, (proj, dirs) in run_python.projects(work).items():
        if only and name not in only:
            continue
        counts, examples = pair(work, tool, name, proj, dirs)
        results[name] = {kind: dict(sorted(c.items())) for kind, c in sorted(counts.items())}
        all_examples += examples
        total = collections.Counter()
        for c in counts.values():
            total.update(c)
        print(name, {k: total[k] for k in ("sites", "index global", "index local",
                                           "index no occurrence", "index ambiguous pair",
                                           "target same")})
    (out / f"python-{tool}.json").write_text(json.dumps(results, indent=1) + "\n")
    with gzip.open(out / f"examples-{tool}.jsonl.gz", "wt") as f:
        for e in all_examples:
            f.write(json.dumps(e) + "\n")


if __name__ == "__main__":
    main()
