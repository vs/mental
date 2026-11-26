"""Part P, seeded gaps: what `mentalc` and each index say where report-jobs is broken on purpose.

Usage: python3 -I seeds_python.py <work-dir>

Each gap is applied alone to a fresh copy of report-jobs in <work-dir>/seed/<gap>/. The copy
gets its environment with `uv sync --locked`; then `mentalc build` runs and both indexers run
as in run_python.py. Writes results/seeds-python.json: for every seeded line, ty's outcome of
each site on it and each index's occurrences there (symbol, diagnostics), the exit codes, and
how many occurrences of the whole index carry a diagnostic.
"""

import json
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import run_python  # noqa: E402
import scip_index  # noqa: E402

REPO = HERE.parents[3]
MENTALC = REPO / "target/release/mentalc"
API = "src/app/api.py"
COMMIT = "        s.commit()\n    try:"


def remove_package(proj):
    subprocess.run(["uv", "pip", "uninstall", "--python", ".venv/bin/python", "boto3"],
                   cwd=proj, check=True, capture_output=True, timeout=300)
    return [("src/app/tasks.py", "import boto3"), ("src/app/tasks.py", "boto3.client(")]


def edit_api(line):
    def apply(proj):
        path = proj / API
        text = path.read_text()
        assert COMMIT in text, "report-jobs changed: the seed's anchor is gone"
        path.write_text(text.replace(COMMIT, f"        s.commit()\n        {line}\n    try:", 1))
        return [(API, line)]
    return apply


SEEDS = {
    "dependency-removed": remove_package,
    "name-not-defined": edit_api("s.commit_all()"),
    "dynamic-call": edit_api('getattr(s, "commit")()'),
}


def line_of(proj, path, needle):
    for number, line in enumerate((proj / path).read_text().splitlines(), 1):
        if needle in line:
            return number
    raise AssertionError(f"{needle!r} not in {path}")


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    results = {}
    for gap, apply in SEEDS.items():
        proj = work / "seed" / gap / "report-jobs"
        if proj.exists():
            shutil.rmtree(proj)
        shutil.copytree(REPO / "examples/report-jobs", proj,
                        ignore=shutil.ignore_patterns(".venv", ".mental", "__pycache__"))
        subprocess.run(["uv", "sync", "--locked", "-q"], cwd=proj, check=True, timeout=600)
        seeded = apply(proj)
        build = subprocess.run([MENTALC, "build"], cwd=proj, capture_output=True, text=True,
                               timeout=600)
        diagnostics = [json.loads(l) for l in (proj / ".mental/diagnostics.jsonl").open()]
        sites = [json.loads(l) for l in (proj / ".mental/model/site.jsonl").open()]
        anchors = {a["id"]: a for a in map(json.loads, (proj / ".mental/anchors.jsonl").open())}
        name = f"seed-{gap}"
        runs = run_python.index(work, name, proj, ["src/app"])
        entry = {"mentalc exit": build.returncode,
                 "mentalc diagnostics": [(d["code"], d["message"][:160]) for d in diagnostics],
                 "index runs": runs, "lines": []}
        indexes = {tool: scip_index.load(work, work / "out" / tool / f"{name}.scip")
                   for tool in ("scip-python", "pyrefly")}
        for tool, index in indexes.items():
            entry[f"{tool}: occurrences with a diagnostic"] = sum(
                1 for _, occs in index.documents.values() for o in occs if o.diagnostics)
        for path, needle in seeded:
            number = line_of(proj, path, needle)
            text = (proj / path).read_text().splitlines()[number - 1]
            row = {"path": path, "line": number, "text": text.strip(),
                   "ty": [(s["kind"], s["id"].rsplit("/site:", 1)[1], s["outcome"])
                          for s in sites if anchors[s["id"]]["path"] == path
                          and anchors[s["id"]]["lines"][0] == number]}
            for tool, index in indexes.items():
                _, occs = index.documents.get(path, (None, []))
                row[tool] = [(text[o.start:o.end] if o.end_line == o.line else "", o.symbol,
                              o.diagnostics) for o in occs if o.line == number - 1]
            entry["lines"].append(row)
        results[gap] = entry
        print(gap, "mentalc exit", build.returncode, flush=True)
    out = HERE / "results/seeds-python.json"
    out.write_text(json.dumps(results, indent=1) + "\n")


if __name__ == "__main__":
    main()
