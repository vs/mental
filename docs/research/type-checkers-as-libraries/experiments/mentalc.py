"""Pair the sites `mentalc build` wrote with the ty answers this experiment recorded.

A pair is an experiment site and mentalc's site of the same file, first line, kind and
name as written (call arguments and subscripts dropped, as mentalc names a chained call).
Prints mentalc's rate over the analysed files, the experiment's, and the pairs by outcome,
with up to [examples] sites of each outcome other than "both, same".

usage: python3 -I mentalc.py <project dir with .mental/> <results/sites-<name>.jsonl.gz> <analysed dir/> [examples]
"""

import collections
import gzip
import json
import re
import sys

proj, exp_path, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
show = int(sys.argv[4]) if len(sys.argv) > 4 else 0
SKIP = re.compile(r"(^|/)(tests?|testing)/")
KIND = {"call": "callee", "write receiver": "write-receiver", "decorator": "decorator"}


def slug(text):
    """The name as mentalc writes it: call arguments and subscripts dropped."""
    out, depth = [], 0
    for c in text:
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth == 0:
            out.append(c)
    return re.sub(r"[^a-z0-9]+", "-", "".join(out).lower()).strip("-")


anchors = {}
for line in open(f"{proj}/.mental/anchors.jsonl"):
    a = json.loads(line)
    anchors[a["id"]] = a
sites = []
for line in open(f"{proj}/.mental/model/site.jsonl"):
    s = json.loads(line)
    a = anchors[s["id"]]
    if not a["path"].startswith(prefix) or SKIP.search(a["path"]):
        continue
    sites.append((s, a))


def target(outcome):
    for key in ("resolved", "external"):
        if key in outcome:
            o = outcome[key]
            return f"{o['module']}.{o['name']}" if o.get("name") else o["module"]
    return None


def reason(outcome):
    return outcome.get("unresolved", {}).get("reason")


bucket = collections.defaultdict(list)
for s, a in sites:
    bucket[(a["path"], a["lines"][0], s["kind"])].append(s)

# The rate of mentalc over every site in the analysed files, per kind.
rate = collections.Counter()
total = collections.Counter()
reasons = collections.defaultdict(collections.Counter)
for s, a in sites:
    total[s["kind"]] += 1
    if target(s["outcome"]):
        rate[s["kind"]] += 1
    else:
        reasons[s["kind"]][reason(s["outcome"])] += 1
print("mentalc over the analysed files:")
for k in sorted(total):
    print(f"  {k:15} {rate[k]:6}/{total[k]:<6} {100 * rate[k] / total[k]:5.1f}%  unresolved: {dict(reasons[k])}")

used = set()
pairs = collections.defaultdict(collections.Counter)
examples = collections.defaultdict(list)
exp_rate = collections.Counter()
exp_total = collections.Counter()
for line in gzip.open(exp_path, "rt"):
    r = json.loads(line)
    if r["kind"] not in KIND or r["outcome"] == "no pyrefly record":
        continue
    kind = KIND[r["kind"]]
    exp_total[kind] += 1
    ty = r["ty"]
    if ty:
        exp_rate[kind] += 1
    start = int(r["range"].split(":")[0])
    want = slug(r["text"] or "")
    match = None
    for s in bucket.get((r["file"], start, kind), []):
        name = re.sub(r"(-\d+)+$", "", s["id"])
        if s["id"] not in used and name.endswith("-" + want):
            match = s
            break
    if match is None:
        if "…" in (r["text"] or ""):
            pairs[kind]["unmatched: text truncated in the experiment"] += 1
            continue
        if kind == "callee" and any(
            re.sub(r"(-\d+)+$", "", s["id"]).endswith("-" + want)
            for s in bucket.get((r["file"], start, "decorator"), [])
        ):
            pairs[kind]["unmatched: a decorator call, a decorator site in mentalc"] += 1
            continue
        pairs[kind]["no mentalc site"] += 1
        examples[(kind, "no mentalc site")].append(f"{r['file']}:{start} {r['text']}")
        continue
    used.add(match["id"])
    got = target(match["outcome"])
    if ty and got:
        c = "both, same" if got in ty else "both, other target"
    elif ty:
        c = f"ty only (mentalc: {reason(match['outcome'])})"
    elif got:
        c = "mentalc only"
    else:
        c = f"neither (mentalc: {reason(match['outcome'])})"
    pairs[kind][c] += 1
    examples[(kind, c)].append(f"{r['file']}:{start} {r['text']!r}  exp={ty}  mentalc={got or reason(match['outcome'])}")

print("experiment ty over its sites:")
for k in sorted(exp_total):
    print(f"  {k:15} {exp_rate[k]:6}/{exp_total[k]:<6} {100 * exp_rate[k] / exp_total[k]:5.1f}%")
print("pairs:")
for k in sorted(pairs):
    print(f"  {k}:")
    for c, n in pairs[k].most_common():
        print(f"    {n:6}  {c}")
        if show and c != "both, same":
            for e in examples[(k, c)][:show]:
                print(f"              {e}")
