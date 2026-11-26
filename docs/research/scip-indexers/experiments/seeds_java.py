"""Part J, seeded gaps: what scip-java writes when a Java project is broken on purpose.

Usage: python3 -I seeds_java.py <work-dir>

Each gap is applied alone to a fresh copy of the project in <work-dir>/seed-java/<gap>/; then
`scip-java index --build-tool=gradle` runs with JDK 17, as for the corpus. Writes
results/seeds-java.json: the exit code, whether an index was written and how many main
documents it holds, the build's error lines, and the occurrences on each seeded line.
"""

import json
import os
import pathlib
import re
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python -I leaves the script's folder off the path

import scip_index  # noqa: E402

OWNER = "src/main/java/org/springframework/samples/petclinic/owner/OwnerController.java"
HAS_ERRORS = "\t\tif (result.hasErrors()) {\n\t\t\tredirectAttributes.addFlashAttribute(\"error\", \"There was an error in creating the owner.\");"
INIT_BINDER = "\t@InitBinder\n\tpublic void setAllowedFields("


def replace(path, old, new):
    text = path.read_text()
    assert old in text, f"{path}: the seed's anchor is gone"
    path.write_text(text.replace(old, new, 1))


def remove_dependency(proj):
    replace(proj / "build.gradle", "  implementation 'javax.cache:cache-api'\n", "")
    config = "src/main/java/org/springframework/samples/petclinic/system/CacheConfiguration.java"
    return [(config, "javax.cache")]


def undefined_name(proj):
    replace(proj / OWNER, HAS_ERRORS, HAS_ERRORS.replace(
        "\t\tif (result.hasErrors()) {", "\t\towner.noSuchMethod();\n\t\tif (result.hasErrors()) {"))
    return [(OWNER, "owner.noSuchMethod()")]


def reflective_call(proj):
    replace(proj / OWNER, INIT_BINDER,
            "\tObject reflect(Owner owner) throws Exception {\n"
            "\t\treturn Owner.class.getMethod(\"getId\").invoke(owner);\n\t}\n\n" + INIT_BINDER)
    return [(OWNER, ".invoke(owner)")]


def no_annotation_processor(proj):
    replace(proj / "build.gradle", "    annotationProcessor 'org.projectlombok:lombok'\n", "")
    return []


SEEDS = {
    "dependency-removed": ("spring-petclinic", remove_dependency),
    "name-not-defined": ("spring-petclinic", undefined_name),
    "dynamic-call": ("spring-petclinic", reflective_call),
    "annotation-processor-off": ("spring-boot-realworld-example-app", no_annotation_processor),
}


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    env = dict(os.environ, JAVA_HOME=subprocess.run(
        ["/usr/libexec/java_home", "-v", "17"], capture_output=True, text=True,
        check=True).stdout.strip())
    results = {}
    for gap, (project, apply) in SEEDS.items():
        proj = work / "seed-java" / gap / project
        if proj.exists():
            shutil.rmtree(proj)
        shutil.copytree(work / "src/java" / project, proj,
                        ignore=shutil.ignore_patterns("build", ".gradle", "target"))
        seeded = apply(proj)
        out = proj / "index.scip"
        run = subprocess.run([work / "bin/scip-java-v0.14.2", "index", "--build-tool=gradle",
                              "--output", out], cwd=proj, env=env, capture_output=True,
                             text=True, timeout=3600)
        log = run.stdout + run.stderr
        (work / "out/logs" / f"scip-java-seed-{gap}.log").write_text(log)
        entry = {"project": project, "exit": run.returncode, "index written": out.exists(),
                 # Paths relative to the project: the work directory is this machine's.
                 "errors": [l.replace(f"{proj}/", "").strip()[:200] for l in log.splitlines()
                            if re.search(r"error:|FAILED|What went wrong", l)][:12],
                 "lines": []}
        if out.exists():
            index = scip_index.load(work, out)
            entry["main documents"] = sum(1 for p in index.documents if "src/main/" in p)
            entry["occurrences with a diagnostic"] = sum(
                1 for _, occs in index.documents.values() for o in occs if o.diagnostics)
            for path, needle in seeded:
                lines = (proj / path).read_text().splitlines()
                number = next(i for i, l in enumerate(lines) if needle in l)
                _, occs = index.documents.get(path, (None, []))
                entry["lines"].append({
                    "path": path, "line": number + 1, "text": lines[number].strip(),
                    "document in index": path in index.documents,
                    "occurrences": [(lines[number][o.start:o.end], o.symbol, o.diagnostics)
                                    for o in occs if o.line == number]})
        results[gap] = entry
        print(gap, "exit", run.returncode, "index", out.exists(), flush=True)
    (HERE / "results/seeds-java.json").write_text(json.dumps(results, indent=1) + "\n")


if __name__ == "__main__":
    main()
