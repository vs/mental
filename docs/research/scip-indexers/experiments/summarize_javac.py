"""Part J, run 2: the tables of the report, from results/javac.json and results/seeds-javac.json.

Usage: python3 -I summarize_javac.py

Prints Markdown tables and writes them to results/summary-javac.md.
"""

import collections
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "—"


def main():
    data = json.loads((HERE / "results/javac.json").read_text())
    scip = json.loads((HERE / "results/java.json").read_text())
    per_form, scip_form = collections.defaultdict(collections.Counter), \
        collections.defaultdict(collections.Counter)
    for project in data.values():
        for key, counts in project["kinds"].items():
            per_form[key].update(counts)
    for project in scip.values():
        for key, counts in project["kinds"].items():
            scip_form[key].update(counts)
    out = ["### Coverage, per site kind and form\n",
           "All three projects, main sources only. *javac answers*: an element that is not "
           "local, or for a lambda its functional interface; *same target*: of the sites both "
           "name with a global symbol.\n",
           "| Kind: form | Sites | scip-java global | javac answers | javac local | "
           "javac none | same target |",
           "|---|---|---|---|---|---|---|"]
    total, scip_total = collections.Counter(), collections.Counter()
    for key in sorted(per_form):
        c, s = per_form[key], scip_form[key]
        total.update(c)
        scip_total.update(s)
        answered = c["javac global"] + c["javac type"]
        none = c["sites"] - answered - c["javac local"]
        both = c["target same"] + c["target differs"]
        out.append(f"| {key} | {c['sites']:,} | {pct(s['index global'], s['sites'])} | "
                   f"{pct(answered, c['sites'])} | {c['javac local']:,} | {none:,} | "
                   f"{pct(c['target same'], both)} |")
    answered = total["javac global"] + total["javac type"]
    both = total["target same"] + total["target differs"]
    out.append(f"| all | {total['sites']:,} | "
               f"{pct(scip_total['index global'], scip_total['sites'])} | "
               f"{pct(answered, total['sites'])} | {total['javac local']:,} | "
               f"{total['sites'] - answered - total['javac local']:,} | "
               f"{pct(total['target same'], both)} ({total['target differs']} differ) |")

    argument, override = per_form["escape: argument"], per_form["escape: override"]
    out += ["\n### Beyond the targets\n", "| | Count |", "|---|---|",
            f"| own code or not, javac against scip-java's definition rule | "
            f"{total['own code: same']:,} same, {total['own code: differs']:,} differ |",
            f"| jar targets: same Maven coordinates | {total['jar: same coordinates']:,} of "
            f"{total['jar: same coordinates'] + total['jar: other coordinates']:,} |",
            f"| arguments with a callee and a type | {argument['argument: decided']:,} of "
            f"{argument['sites']:,} |",
            f"| … that escape (a repository type handed to outside code) | "
            f"{argument['argument: escapes (own type to outside code)']:,} |",
            f"| methods that override outside code, javac | "
            f"{override['overrides outside code']:,} of {override['javac global']:,} |",
            f"| … same classification as scip-java's relationships | "
            f"{override['overrides: same']:,} of "
            f"{override['overrides: same'] + override['overrides: differ']:,} |",
            f"| static imports with several members (overloads) | "
            f"{per_form['import: static']['static import: several members']:,} of "
            f"{per_form['import: static']['sites']:,} |",
            f"| javac errors on the unbroken projects | "
            f"{sum(p['errors'] for p in data.values())} |"]

    out += ["\n### Cost\n",
            "Warm Gradle cache; the Gradle step runs only what `compileJava` depends on.\n",
            "| Project | Gradle step | Module | Sources | javac tool | Peak memory |",
            "|---|---|---|---|---|---|"]
    for name, project in data.items():
        for module, run in project["tool"].items():
            out.append(f"| {name} | {project['gradle seconds']} s | {module} | "
                       f"{run['sources']} | {run['seconds']} s | {run['peak MB']} MB |")

    seeds = json.loads((HERE / "results/seeds-javac.json").read_text())
    out += ["\n### Seeded gaps\n",
            "*Answered*: sites with an answer, against the same files before the gap.\n",
            "| Gap | Policy | tool exit | errors | files without an error: answered | "
            "files with one: answered | on the seeded line |",
            "|---|---|---|---|---|---|---|"]
    for gap, e in seeds.items():
        for policy in ("default", "flow"):
            p = e[policy]
            clean = p["sites"].get("files without one", {})
            broken = p["sites"].get("files with an error", {})
            lines = "; ".join(
                ", ".join(f"`{s['name']}` {s['javac']}" for s in l["sites"])
                + (f" ({'; '.join(l['errors'])})" if l["errors"] else "")
                for l in p["seeded lines"]) or "—"
            out.append(
                f"| {gap} | {policy} | {set(p['tool exit'].values())} | {p['errors']} | "
                f"{clean.get('answered', 0):,} of {clean.get('answered before the gap', 0):,} | "
                + (f"{broken['answered']:,} of {broken['answered before the gap']:,}"
                   if broken else "—") + f" | {lines} |")

    text = "\n".join(out) + "\n"
    (HERE / "results/summary-javac.md").write_text(text)
    print(text)


if __name__ == "__main__":
    main()
