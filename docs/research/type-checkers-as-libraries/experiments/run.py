"""Run Pyrefly (Pysa export) and ty (through tyharness) on every corpus project.

Usage: python -I run.py <work-dir>

<work-dir> holds the checkouts (src/<name>), environments, the Pyrefly install (tools/) and the
tyharness build (target/release/tyharness); output goes to <work-dir>/out/{pyrefly,ty}/<name>.
"""

import json
import pathlib
import subprocess
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]

# name: (project dir, environment dir, python version, analysed dirs relative to the project dir)
def projects(work):
    src = work / "src"
    return {
        "report-jobs": (REPO / "examples/report-jobs/src", work / "env-report-jobs", "3.12", ["app"]),
        "fastapi-template": (src / "full-stack-fastapi-template/backend", src / "full-stack-fastapi-template/.venv", "3.14", ["app"]),
        "mealie": (src / "mealie", src / "mealie/.venv", "3.14", ["mealie"]),
        "redash": (src / "redash", src / "redash/.venv", "3.13", ["redash"]),
        "rq": (src / "rq", src / "rq/.venv", "3.13", ["rq"]),
    }


def site_packages(env):
    return next(env.glob("lib/python3.*/site-packages"))


def run(cmd, cwd):
    t = time.monotonic()
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    ms = int((time.monotonic() - t) * 1000)
    if p.returncode not in (0, 1):  # 1: type errors found
        sys.exit(f"failed ({p.returncode}): {' '.join(map(str, cmd))}\n{p.stderr[-3000:]}")
    return ms, p.stderr


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    only = sys.argv[2:]
    cfg = work / "cfg"
    cfg.mkdir(exist_ok=True)
    timings = {}
    for name, (proj, env, py, dirs) in projects(work).items():
        if only and name not in only:
            continue
        sp = site_packages(env)
        # Pyrefly: one explicit config, so no project config is migrated; no interpreter is started.
        pcfg = cfg / f"{name}.pyrefly.toml"
        pcfg.write_text(
            f'project-includes = {json.dumps([str(proj / d) for d in dirs])}\n'
            f'search-path = ["{proj}"]\n'
            f'site-package-path = ["{sp}"]\n'
            f'python-version = "{py}"\n'
            "skip-interpreter-query = true\n"
            "check-unannotated-defs = true\n"
        )
        pout = work / "out/pyrefly" / name
        pout.parent.mkdir(parents=True, exist_ok=True)
        pms, perr = run([work / "tools/bin/pyrefly", "check", "-c", pcfg, "--report-pysa", pout,
                         "--report-pysa-format", "json", "--output-format", "json"], proj)
        # ty: the environment is read as files (pyvenv.cfg, site-packages); nothing is executed.
        tcfg = cfg / f"{name}.ty.toml"
        tcfg.write_text(f'[environment]\npython = "{env}"\npython-version = "{py}"\nroot = ["{proj}"]\n')
        tout = work / "out/ty" / f"{name}.json"
        tout.parent.mkdir(parents=True, exist_ok=True)
        tms, terr = run([work / "target/release/tyharness", tcfg, proj, tout, *dirs], proj)
        timings[name] = {"pyrefly_ms": pms, "ty_ms": tms}
        print(name, timings[name], perr.strip().splitlines()[-1:] if perr.strip() else "")
    (work / "out/timings.json").write_text(json.dumps(timings, indent=1))


if __name__ == "__main__":
    main()
