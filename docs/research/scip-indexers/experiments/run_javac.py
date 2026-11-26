"""Part J, run 2: run the javac tool (javac/JavacQuery.java) on each Java project.

Usage: python3 -I run_javac.py <work-dir> <project-dir> <out-dir> [flow]

Reads what each Gradle project's compileJava would hand javac (javac/compile-args.gradle, Build
Scans off, offline), writes one argfile per Gradle project, and runs the tool on it with JDK 17
under /usr/bin/time. `flow` adds `--should-stop=ifError=FLOW`, so that javac attributes every
file even after an error. Writes <out-dir>/<gradle project>.jsonl and <out-dir>/run.json (the
Gradle step's time and exit, and per Gradle project the tool's exit, time and peak memory).
"""

import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
TOOL = HERE / "javac/JavacQuery.java"
INIT = HERE / "javac/compile-args.gradle"
# Error Prone runs as a javac plugin in mockito's build: a check of the code, not part of what
# the names mean, and it needs its own --add-exports; it is left out.
# Its plugin also sets javac's error policy, which this run sets itself.
DROP = re.compile(r"^(-Xplugin:ErrorProne|-XDcompilePolicy=|-XDshould|--should-stop|-Werror)")


def jdk17():
    return subprocess.run(["/usr/libexec/java_home", "-v", "17"], capture_output=True,
                          text=True, check=True).stdout.strip()


def compile_args(proj, out, env):
    args_dir = out / "compile-args"
    if args_dir.exists():
        shutil.rmtree(args_dir)
    started = time.monotonic()
    run = subprocess.run(
        [proj / "gradlew", "--no-daemon", "--offline", "--no-scan", "--no-configuration-cache",
         "-q", "-I", INIT, f"-Pmental.out={args_dir}", "mentalCompileArgs"],
        cwd=proj, env=env, capture_output=True, text=True, timeout=1800)
    (out / "gradle.log").write_text(run.stdout + run.stderr)
    return run.returncode, round(time.monotonic() - started, 1), sorted(args_dir.glob("*.json"))


def argfile(spec, scratch, flow):
    """javac's options and the main source files, one per line."""
    processors = [p for p in spec["processorpath"] if "error_prone" not in p]
    # A module (module-info.java) reads its dependencies from the module path, as Gradle hands
    # them to javac.
    modular = any(s.endswith("/module-info.java") for s in spec["sources"])
    lines = ["--module-path" if modular else "-classpath", os.pathsep.join(spec["classpath"]),
             "-d", str(scratch / "classes"), "-s", str(scratch / "generated"),
             "-implicit:none", "-encoding", spec["encoding"] or "UTF-8",
             "-Xmaxerrs", "1000000"]  # javac reports 100 errors by default
    lines += ["-processorpath", os.pathsep.join(processors)] if processors else ["-proc:none"]
    if spec["release"]:
        lines += ["--release", str(spec["release"])]
    else:
        lines += ["-source", spec["source"], "-target", spec["target"]]
    lines += [a for a in spec["args"] if not DROP.match(a)]
    if flow:
        lines.append("--should-stop=ifError=FLOW")
    return lines + [s for s in spec["sources"] if s.endswith(".java")]


def main():
    work, proj, out = (pathlib.Path(a).resolve() for a in sys.argv[1:4])
    flow = sys.argv[4:] == ["flow"]
    out.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, JAVA_HOME=jdk17())
    java = pathlib.Path(env["JAVA_HOME"]) / "bin/java"
    code, seconds, specs = compile_args(proj, out, env)
    report = {"gradle exit": code, "gradle seconds": seconds, "projects": {}}
    for spec_path in specs:
        spec = json.loads(spec_path.read_text())
        sources = [s for s in spec["sources"] if s.endswith(".java")]
        if not sources:
            continue
        name = spec_path.stem
        scratch = work / "tmp/javac" / proj.name / name
        for d in ("classes", "generated"):
            (scratch / d).mkdir(parents=True, exist_ok=True)
        args = out / f"{name}.args"
        args.write_text("\n".join(argfile(spec, scratch, flow)) + "\n")
        result = out / f"{name}.jsonl"
        run = subprocess.run(
            ["/usr/bin/time", "-l", java,
             "--add-exports", "jdk.compiler/com.sun.tools.javac.code=ALL-UNNAMED",
             TOOL, proj, args, result],
            env=env, capture_output=True, text=True, timeout=3600)
        peak = re.search(r"(\d+)\s+maximum resident set size", run.stderr)
        real = re.search(r"([\d.]+) real", run.stderr)
        report["projects"][name] = {
            "sources": len(sources), "exit": run.returncode,
            "seconds": float(real.group(1)) if real else None,
            "peak MB": round(int(peak.group(1)) / 2**20) if peak else None,
            "stderr": [l.replace(str(proj) + "/", "") for l in run.stderr.splitlines()
                       if "maximum resident" not in l][:5] if run.returncode else []}
        print(proj.name, name, report["projects"][name]["exit"],
              report["projects"][name]["seconds"], flush=True)
    (out / "run.json").write_text(json.dumps(report, indent=1) + "\n")


if __name__ == "__main__":
    main()
