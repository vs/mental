"""Part J: the tables of the report, from results/java.json and results/seeds-java.json.

Usage: python3 -I summarize_java.py

Prints Markdown tables and writes them to results/summary-java.md.
"""

import collections
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "—"


def main():
    data = json.loads((HERE / "results/java.json").read_text())
    per_form = collections.defaultdict(collections.Counter)
    for project in data.values():
        for key, counts in project["kinds"].items():
            per_form[key].update(counts)
    out = ["### Coverage, per site kind and form\n",
           "All three projects, main sources only. *Global*: a symbol that is not file-local.\n",
           "| Kind: form | Sites | global | local | no occurrence | several symbols |",
           "|---|---|---|---|---|---|"]
    total = collections.Counter()
    for key in sorted(per_form):
        c = per_form[key]
        total.update(c)
        out.append(f"| {key} | {c['sites']:,} | {pct(c['index global'], c['sites'])} | "
                   f"{c['index local']:,} | {c['index no occurrence']:,} | "
                   f"{c['index several symbols']:,} |")
    out.append(f"| all | {total['sites']:,} | {pct(total['index global'], total['sites'])} | "
               f"{total['index local']:,} | {total['index no occurrence']:,} | "
               f"{total['index several symbols']:,} |")

    out += ["\n### Own code or a dependency\n",
            "Global answers whose target is the repository's own code: by the symbol's package "
            "(`. .`) and by a definition of the symbol in the index.\n",
            "| Project | Global answers | own, by package | own, by definition | disagree |",
            "|---|---|---|---|---|"]
    for name, project in data.items():
        t = collections.Counter()
        for counts in project["kinds"].values():
            t.update(counts)
        out.append(f"| {name} | {t['index global']:,} | {t['origin repository']:,} | "
                   f"{t['defined in the index']:,} | {t['package and definition disagree']:,} |")

    callee = per_form["callee: method"] + per_form["callee: new"]
    ctor = per_form["callee: new"]
    override = per_form["escape: override"]
    argument = per_form["escape: argument"]
    bare = per_form["write-receiver: bare"]
    out += ["\n### What the symbols tell\n", "| | Count |", "|---|---|",
            f"| `new` sites naming a constructor | {ctor['names a constructor']:,} of "
            f"{ctor['index global']:,} (the rest: anonymous classes, named by their type) |",
            f"| callees with symbol information (a kind) | {callee['has a kind']:,} of "
            f"{callee['index global']:,}: only the repository's own symbols have it |",
            f"| method declarations that override outside code | "
            f"{override['overrides outside code']:,} of {override['index global']:,} |",
            f"| … that override repository code only | "
            f"{override['overrides repository code only']:,} |",
            f"| arguments that are a local or parameter (need a type) | "
            f"{argument['argument: a local or parameter (needs its type)']:,} of "
            f"{argument['sites']:,} |",
            f"| arguments that are a field (need its type) | "
            f"{argument['argument: a field (needs its type)']:,} |",
            f"| bare assignments the index shows are a field | {bare['a field']:,} of "
            f"{bare['sites']:,} (the rest: local variables) |",
            f"| lambdas handed to a call | {per_form['escape: lambda']['sites']:,}, "
            "never named |",
            f"| occurrences with a diagnostic, in all three indexes | "
            f"{sum(p['occurrences with a diagnostic'] for p in data.values()):,} |"]

    seeds_path = HERE / "results/seeds-java.json"
    if seeds_path.exists():
        seeds = json.loads(seeds_path.read_text())
        out += ["\n### Seeded gaps\n",
                "| Gap | Project | exit | index written | main documents | on the seeded line |",
                "|---|---|---|---|---|---|"]
        for gap, e in seeds.items():
            lines = "; ".join(
                f"line {l['line']}: " + (", ".join(f"`{t}` → `{s.split(' ')[-1]}`"
                                                    for t, s, _ in l["occurrences"])
                                         if l["occurrences"] else "nothing")
                for l in e["lines"]) or "—"
            out.append(f"| {gap} | {e['project']} | {e['exit']} | {e['index written']} | "
                       f"{e.get('main documents', '—')} | {lines} |")

    text = "\n".join(out) + "\n"
    (HERE / "results/summary-java.md").write_text(text)
    print(text)


if __name__ == "__main__":
    main()
