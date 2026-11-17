"""Why neither tool resolved a receiver: classify by where the base name comes from.

Usage: python -I neither.py <work-dir> <results-dir>
"""
import ast
import collections
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from run import projects  # noqa: E402


def origins(path):
    """line -> {name: origin} for unannotated parameters, annotated ones, and other names."""
    tree = ast.parse(open(path).read())
    spans = []
    for fn in ast.walk(tree):
        if isinstance(fn, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            a = fn.args
            params = a.posonlyargs + a.args + a.kwonlyargs + [x for x in (a.vararg, a.kwarg) if x]
            un = {p.arg for p in params if p.annotation is None and p.arg not in ("self", "cls")}
            spans.append((fn.lineno, fn.end_lineno, un))
    return spans


def main():
    work, results = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    out = {}
    for name, (proj, *_rest) in projects(work).items():
        cache, c = {}, collections.Counter()
        for line in open(results / f"sites-{name}.jsonl"):
            r = json.loads(line)
            if r["kind"] not in ("receiver", "write receiver") or r["outcome"] != "neither":
                continue
            path = str(proj / r["file"])
            if path not in cache:
                cache[path] = origins(path)
            base = r["text"].split(".")[0]
            ln = int(r["range"].split(":")[0])
            inner = [s for s in cache[path] if s[0] <= ln <= s[1]]
            if base in ("self", "cls"):
                c["self/cls attribute chain"] += 1
            elif inner and base in max(inner)[2] and "(" not in r["text"].split(".")[0]:
                c["unannotated parameter" if r["text"].count(".") == 1 else "chain from an unannotated parameter"] += 1
            else:
                c["other (library return, module global, call result)"] += 1
        total = sum(c.values())
        out[name] = {"neither": total, **dict(c.most_common())}
        print(name, total, dict(c.most_common()))
    (results / "neither.json").write_text(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
