"""Part P: index each Python corpus project with scip-python and with Pyrefly's SCIP report.

Usage: python3 -I run_python.py <work-dir> [project ...]

<work-dir> holds the checkouts (src/<name>), the tools (node/, pyrefly15/) and gets the indexes
in out/<tool>/<name>.scip, the configurations in cfg/, and the time and peak memory of every
run in out/timings-python.json. Each tool is set up with the roots, Python version and
environment the build unit gave ty (`.mental/model/build_unit.jsonl`), and starts no interpreter
of the project: scip-python gets an environment file made from the installed RECORD files, so
it does not call pip, and Pyrefly gets the site-packages path with `skip-interpreter-query`.
"""

import csv
import json
import pathlib
import re
import subprocess
import sys

PROJECT_NAME = "project"
PROJECT_VERSION = "0"


def projects(work):
    """name: (project dir, analysed dirs). Tests are left out, as in the type-checker corpus."""
    src = work / "src"
    return {
        "report-jobs": (src / "report-jobs", ["src/app"]),
        "fastapi-template": (src / "full-stack-fastapi-template/backend", ["app"]),
        "mealie": (src / "mealie", ["mealie"]),
        "redash": (src / "redash", ["redash"]),
        "rq": (src / "rq", ["rq"]),
    }


def build_unit(proj):
    lines = (proj / ".mental/model/build_unit.jsonl").read_text().splitlines()
    assert len(lines) == 1, f"{proj}: one build unit expected"
    return json.loads(lines[0])


def environment_file(site_packages, out):
    """scip-python's environment: one entry per installed distribution, its files from RECORD."""
    packages = []
    for dist in sorted(site_packages.glob("*.dist-info")):
        meta = (dist / "METADATA").read_text(errors="replace")
        name = re.search(r"^Name: (.+)$", meta, re.M).group(1).strip()
        version = re.search(r"^Version: (.+)$", meta, re.M).group(1).strip()
        record = dist / "RECORD"
        files = [row[0] for row in csv.reader(record.open())] if record.exists() else []
        packages.append({"name": name, "version": version, "files": files})
    out.write_text(json.dumps(packages))


def timed(cmd, cwd, log):
    """Runs cmd under /usr/bin/time -l; returns exit code, seconds and peak memory in MB."""
    p = subprocess.run(["/usr/bin/time", "-l", *map(str, cmd)], cwd=cwd, capture_output=True,
                       text=True, timeout=3600)
    log.write_text(p.stdout + "\n--- stderr ---\n" + p.stderr)
    seconds = float(re.search(r"([\d.]+) real", p.stderr).group(1))
    rss = int(re.search(r"(\d+)\s+maximum resident set size", p.stderr).group(1))
    return p.returncode, seconds, round(rss / 2**20)


def index(work, name, proj, dirs):
    """Indexes one project with both tools; returns their exit codes, times and peak memory."""
    cfg, out = work / "cfg", work / "out"
    for d in (cfg, out / "scip-python", out / "pyrefly", out / "logs"):
        d.mkdir(parents=True, exist_ok=True)
    proj = proj.resolve()  # Pyrefly resolves symbolic links; its search path must match
    unit = build_unit(proj)
    env = (proj / unit["environment"]).resolve()
    site_packages = next(env.glob("lib/python3.*/site-packages"))
    python = unit["python"]["version"]
    roots = [str((proj / root).resolve()) for root in unit["roots"]]

    envfile = cfg / f"{name}.environment.json"
    environment_file(site_packages, envfile)
    # scip-python reads pyrightconfig.json from its working directory, the project.
    (proj / "pyrightconfig.json").write_text(json.dumps({
        "include": dirs,
        "exclude": ["**/tests", "**/test", "**/testing", "**/.*", "**/__pycache__"],
        "extraPaths": roots,
        "venvPath": str(env.parent),
        "venv": env.name,
        "pythonVersion": python,
    }, indent=1))
    sp_out = out / "scip-python" / f"{name}.scip"
    sp = timed([work / "node/node_modules/.bin/scip-python", "index", "--quiet",
                "--project-name", PROJECT_NAME, "--project-version", PROJECT_VERSION,
                "--environment", envfile, "--output", sp_out], proj,
               out / "logs" / f"scip-python-{name}.log")

    pcfg = cfg / f"{name}.pyrefly.toml"
    pcfg.write_text(
        f"project-includes = {json.dumps([str(proj / d) for d in dirs])}\n"
        'project-excludes = ["**/tests/**", "**/test/**", "**/testing/**"]\n'
        f"search-path = {json.dumps(roots)}\n"
        f'site-package-path = ["{site_packages}"]\n'
        f'python-version = "{python}"\n'
        "skip-interpreter-query = true\n"
        "check-unannotated-defs = true\n"
    )
    py_out = out / "pyrefly" / f"{name}.scip"
    pr = timed([work / "pyrefly15/bin/pyrefly", "check", "-c", pcfg, "--output-format",
                "omit-errors", "--report-scip", py_out, "--relative-to", proj,
                "--report-scip-project-name", PROJECT_NAME,
                "--report-scip-project-version", PROJECT_VERSION], proj,
               out / "logs" / f"pyrefly-{name}.log")
    for tool, path in (("scip-python", sp_out), ("pyrefly", py_out)):
        assert path.exists(), f"{tool} wrote no index for {name}; see out/logs"
    return {
        "scip-python": {"exit": sp[0], "seconds": sp[1], "peak_mb": sp[2]},
        "pyrefly": {"exit": pr[0], "seconds": pr[1], "peak_mb": pr[2]},
    }


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    only = sys.argv[2:]
    timings_path = work / "out" / "timings-python.json"
    timings = json.loads(timings_path.read_text()) if timings_path.exists() else {}
    for name, (proj, dirs) in projects(work).items():
        if only and name not in only:
            continue
        timings[name] = index(work, name, proj, dirs)
        print(name, timings[name], flush=True)
    timings_path.write_text(json.dumps(timings, indent=1))


if __name__ == "__main__":
    main()
