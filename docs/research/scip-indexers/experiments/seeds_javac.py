"""Part J, run 2: what the javac tool answers on the seeded gaps of seeds_java.py.

Usage: python3 -I seeds_javac.py <work-dir>

Each gap is applied alone to a fresh copy of the project in <work-dir>/seed-javac/<gap>/, as
in seeds_java.py; then run_javac.py runs twice: with javac's default policy, and with
`--should-stop=ifError=FLOW`. Writes results/seeds-javac.json: per gap and policy, the tool's
exit, javac's errors, the sites answered against the unbroken project's run, split into files
with an error and files without one, and every site on the seeded lines.
"""

import collections
import json
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import pair_javac  # noqa: E402
import seeds_java  # noqa: E402

ANSWERED = ("global", "local", "type")


def per_file(proj, out_dir):
    """Per main source file: sites, and sites answered."""
    at, diagnostics = pair_javac.load(out_dir)
    files = collections.defaultdict(collections.Counter)
    for path, _, _, _, _, answer in pair_javac.answers(proj, at):
        files[path]["sites"] += 1
        files[path]["answered"] += answer in ANSWERED
    return files, diagnostics


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    results = {}
    for gap, (project, apply) in seeds_java.SEEDS.items():
        proj = work / "seed-javac" / gap / project
        if proj.exists():
            shutil.rmtree(proj)
        shutil.copytree(work / "src/java" / project, proj,
                        ignore=shutil.ignore_patterns("build", ".gradle", "target"))
        seeded = apply(proj)
        baseline, _ = per_file(work / "src/java" / project, work / "out/javac" / project)
        entry = {"project": project}
        for policy in ("default", "flow"):
            out_dir = work / "out/javac-seed" / gap / policy
            if out_dir.exists():
                shutil.rmtree(out_dir)
            subprocess.run([sys.executable, "-I", HERE / "run_javac.py", work, proj, out_dir]
                           + (["flow"] if policy == "flow" else []), check=True, timeout=3600)
            run = json.loads((out_dir / "run.json").read_text())
            files, diagnostics = per_file(proj, out_dir)
            errors = [d for d in diagnostics if d["kind"] == "ERROR"]
            broken = {d["path"] for d in errors}
            split = collections.defaultdict(collections.Counter)
            for path, c in files.items():
                group = "files with an error" if path in broken else "files without one"
                split[group]["sites"] += c["sites"]
                split[group]["answered"] += c["answered"]
                split[group]["answered before the gap"] += baseline[path]["answered"]
            at, _ = pair_javac.load(out_dir)
            lines = []
            for path, needle in seeded:
                text = (proj / path).read_text().splitlines()
                number = next(i for i, l in enumerate(text) if needle in l)
                lines.append({
                    "path": path, "line": number + 1, "text": text[number].strip(),
                    "sites": [{"kind": s.kind, "form": s.form, "name": s.name,
                               "javac": a, "symbol": r and r.get("symbol", r.get("callee_symbol"))}
                              for p, _, s, _, r, a in pair_javac.answers(proj, at)
                              if p == path and s.line == number],
                    "errors": [f"{d['code']}: {d['message']}" for d in errors
                               if d["path"] == path and d["line"] == number + 1]})
            entry[policy] = {
                "gradle exit": run["gradle exit"],
                "tool exit": {n: p["exit"] for n, p in run["projects"].items()},
                "errors": len(errors),
                "error codes": dict(collections.Counter(d["code"] for d in errors)),
                "first errors": [f"{d['path']}:{d['line']}: {d['message']}" for d in errors[:5]],
                "sites": {g: dict(c) for g, c in sorted(split.items())},
                "seeded lines": lines,
            }
            print(gap, policy, entry[policy]["errors"], dict(split), flush=True)
        results[gap] = entry
    (HERE / "results/seeds-javac.json").write_text(json.dumps(results, indent=1) + "\n")


if __name__ == "__main__":
    main()
