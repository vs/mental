"""Part P: the tables of the report, from results/python-<tool>.json.

Usage: python3 -I summarize_python.py <work-dir>

Prints Markdown tables (all five projects together) and writes them to
results/summary-python.md; reads the indexers' times from <work-dir>/out/timings-python.json.
"""

import collections
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
TOOLS = ("scip-python", "pyrefly")
KINDS = ("import", "callee", "decorator", "base-class", "write-receiver", "escape",
         "handler-type")
SAME = ("target same", "target same, attribute declared in a base",
        "target same object, private module",
        "target nested definition named without its enclosing function")
REASONS = ("no_type", "not_found", "ambiguous", "missing_module", "unreachable")


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "—"


def load(tool):
    data = json.loads((HERE / f"results/python-{tool}.json").read_text())
    per_kind = collections.defaultdict(collections.Counter)
    total = collections.Counter()
    for project in data.values():
        for kind, counts in project.items():
            per_kind[kind].update(counts)
            total.update(counts)
    return per_kind, total


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    out = []
    loaded = {tool: load(tool) for tool in TOOLS}

    out.append("### Coverage and agreement, per site kind\n")
    out.append("Sites with an ask point (ty's `dynamic` left out). *ty* and *index*: sites each "
               "answers with a target. *Same*: of the sites both answer, the same target, also "
               "when only its spelling differs (a private module such as `_io`, an attribute "
               "declared in a base class, a nested function named without its enclosing one).\n")
    out.append("| Kind | Sites | ty | scip-python | same as ty | Pyrefly | same as ty |")
    out.append("|---|---|---|---|---|---|---|")
    rows = [*KINDS, "all"]
    for kind in rows:
        cells = []
        for i, tool in enumerate(TOOLS):
            per_kind, total = loaded[tool]
            c = total if kind == "all" else per_kind[kind]
            sites = c["sites"] - c["no ask point (ty: dynamic)"]
            ty = c["ty resolved"] + c["ty external"]
            index = c["index global"]
            both = sum(v for k, v in c.items() if k.startswith("target "))
            same = sum(c[k] for k in SAME)
            if i == 0:
                cells += [f"{sites:,}", pct(ty, sites)]
            cells += [pct(index, sites), pct(same, both)]
        out.append(f"| {kind} | " + " | ".join(cells) + " |")

    out.append("\n### Where the two disagree\n")
    out.append("Sites both answer with a target, by how the index's target stands to ty's.\n")
    out.append("| Category | scip-python | Pyrefly |")
    out.append("|---|---|---|")
    categories = sorted({k for t in TOOLS for k in loaded[t][1] if k.startswith("target ")})
    for k in categories:
        out.append(f"| {k.removeprefix('target ')} | "
                   + " | ".join(f"{loaded[t][1][k]:,}" for t in TOOLS) + " |")

    out.append("\n### What the index has where ty is unresolved\n")
    out.append("| ty's reason | Sites | scip-python: symbol / local / nothing | "
               "Pyrefly: symbol / local / nothing |")
    out.append("|---|---|---|---|")
    for reason in REASONS:
        cells = []
        for i, tool in enumerate(TOOLS):
            _, total = loaded[tool]
            g = total[f"ty unresolved {reason}: index global"]
            loc = total[f"ty unresolved {reason}: index local"]
            none = (total[f"ty unresolved {reason}: index no occurrence"]
                    + total[f"ty unresolved {reason}: index ambiguous pair"])
            if i == 0:
                cells.append(f"{g + loc + none:,}")
            cells.append(f"{g:,} / {loc:,} / {none:,}")
        out.append(f"| `{reason}` | " + " | ".join(cells) + " |")

    out.append("\n### What the index lacks where ty answers\n")
    out.append("| Index answer | scip-python | Pyrefly |")
    out.append("|---|---|---|")
    for k in ("local", "no occurrence", "ambiguous pair"):
        out.append(f"| {k} | " + " | ".join(
            f"{loaded[t][1][f'ty answers: index {k}']:,}" for t in TOOLS) + " |")

    out.append("\n### Resolved or External, from the symbol\n")
    out.append("Of the sites both answer: whether the index's symbol puts the target in the "
               "repository exactly when ty does, judged by the symbol's package and by its "
               "module (a module of a file under the roots); and, for ty's external targets, "
               "whether the package is ty's pin.\n")
    out.append("| | scip-python | Pyrefly |")
    out.append("|---|---|---|")
    for k, label in (("outcome class right, by package", "by package"),
                     ("outcome class right, by module", "by module")):
        cells = []
        for t in TOOLS:
            total = loaded[t][1]
            both = sum(v for kk, v in total.items() if kk.startswith("target "))
            cells.append(pct(total[k], both))
        out.append(f"| {label} | " + " | ".join(cells) + " |")
    out.append("| external package is ty's pin | " + " | ".join(
        pct(loaded[t][1]["external package right"], loaded[t][1]["ty external"])
        for t in TOOLS) + " |")

    out.append("\n### Callees and escapes\n")
    out.append("| | scip-python | Pyrefly |")
    out.append("|---|---|---|")
    for k in ("callee: index symbol is a class (suffix #)", "callee: index symbol has a kind",
              "escape: decided from the symbol"):
        out.append(f"| {k} | " + " | ".join(f"{loaded[t][1][k]:,}" for t in TOOLS) + " |")
    for t in TOOLS:
        per_kind, _ = loaded[t]
        e = per_kind["escape"]
        both = sum(v for kk, v in e.items() if kk.startswith("target "))
        out.append(f"\n{t}: {both:,} escapes both answer; "
                   f"{e['escape: decided from the symbol']:,} are a repository `def` or `class` "
                   "by the symbol alone.")

    timings = json.loads((work / "out/timings-python.json").read_text())
    out.append("\n### Cost\n")
    out.append("| Project | scip-python s | MB | Pyrefly s | MB |")
    out.append("|---|---|---|---|---|")
    for name, t in timings.items():
        if name.startswith("seed-"):
            continue
        sp, pr = t["scip-python"], t["pyrefly"]
        out.append(f"| {name} | {sp['seconds']} | {sp['peak_mb']} | {pr['seconds']} | "
                   f"{pr['peak_mb']} |")

    text = "\n".join(out) + "\n"
    (HERE / "results/summary-python.md").write_text(text)
    print(text)


if __name__ == "__main__":
    main()
