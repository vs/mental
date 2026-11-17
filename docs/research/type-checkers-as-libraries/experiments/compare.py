"""Compare what Pyrefly and ty resolve, site by site.

Usage: python -I compare.py <work-dir> <results-dir>

Sites come from tyharness (ruff's syntax tree); Pyrefly's answers are looked up by the same
`line:col-line:col` range in its Pysa export. Test directories are left out. Writes
<results-dir>/compare.json (counts per project and site kind) and
<results-dir>/sites-<project>.jsonl (every site with both answers), and prints a summary.
"""

import ast
import collections
import glob
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))  # -I leaves the script's folder out
from run import projects  # noqa: E402

SKIP_DIR = re.compile(r"/(tests?|testing)/")
BUILTIN_DECORATOR = re.compile(
    r"^((abc\.)?abstractmethod|property|classmethod|staticmethod|(typing\.)?overload|(typing\.)?override"
    r"|[\w.]+\.(setter|getter|deleter))$")


# ---------- module names from paths (ty's targets) ----------

def module_of(path, roots):
    if path.startswith("vendored:"):
        rel = path.split(":", 1)[1]
        for prefix in ("stdlib/", "stubs/"):
            if rel.startswith(prefix):
                rel = rel[len(prefix):]
        if rel.startswith("ty_extensions/"):
            return "ty_extensions"
    else:
        rel = None
        for r in roots:
            if path.startswith(r + "/"):
                rel = path[len(r) + 1:]
                break
        if rel is None:
            m = re.search(r"/lib/python3\.\d+/(.*)$", path)  # the standard library of the interpreter
            rel = m.group(1) if m else path
    rel = re.sub(r"\.pyi?$", "", rel)
    parts = [p for p in rel.split("/") if p]
    if parts and parts[-1] == "__init__":
        parts.pop()
    if parts and parts[0].endswith("-stubs"):
        parts[0] = parts[0][: -len("-stubs")]
    return ".".join(parts)


def ty_qual(t, roots):
    return f"{module_of(t['path'], roots)}.{t['name']}" if t.get("name") else module_of(t["path"], roots)


# ---------- Pyrefly export ----------

class Pyrefly:
    def __init__(self, out):
        self.out = pathlib.Path(out)
        self.defs = {}  # module name -> definitions json (lazy)
        self.by_path = {}  # source path -> (call graph json, types json, definitions json)
        for f in glob.glob(str(self.out / "call_graphs/*.json")):
            cg = json.load(open(f))
            sp = cg["source_path"]
            path = sp.get("FileSystem") or sp.get("Memory") or str(sp)
            stem = pathlib.Path(f).name
            self.by_path[path] = stem

    def load(self, kind, stem):
        p = self.out / kind / stem
        return json.load(open(p)) if p.exists() else {}

    def module_defs(self, module, mid=None):
        """Definitions of one module; by id when known, since files that are not importable
        modules (alembic migrations) all share the module name `__unknown__`."""
        key = (module, mid)
        if key not in self.defs:
            pat = f"{glob.escape(module)}:{mid if mid is not None else '*'}.json"
            hits = glob.glob(str(self.out / "definitions" / pat))
            self.defs[key] = json.load(open(hits[0])) if hits else {}
        return self.defs[key]

    def class_chain(self, module, class_id, mid=None):
        d = self.module_defs(module, mid).get("class_definitions", {}).get(str(class_id))
        if not d:
            return [f"<class {class_id}>"]
        return self.parent_chain(module, d["parent"], mid) + [d["name"]]

    def func_chain(self, module, function_id, mid=None):
        d = self.module_defs(module, mid).get("function_definitions", {}).get(function_id)
        if not d:
            return [f"<{function_id}>"]
        return self.parent_chain(module, d["parent"], mid) + [d["name"]]

    def parent_chain(self, module, parent, mid=None):
        if parent == "TopLevel" or not isinstance(parent, dict):
            return []
        if "Class" in parent:
            return self.class_chain(module, parent["Class"]["class_id"], mid)
        if "Function" in parent:
            fn = parent["Function"]
            return self.func_chain(module, fn.get("function_id") or f"F:{fn['func_def_index']}", mid)
        return []

    def func_qual(self, ref):
        chain = self.func_chain(ref["module_name"], ref["function_id"], ref.get("module_id"))
        if chain[-1].startswith("<"):
            chain = [ref.get("function_name", chain[-1])]
        return ".".join([ref["module_name"]] + chain)

    def class_qual(self, ref):
        chain = self.class_chain(ref["module_name"], ref["class_id"], ref.get("module_id"))
        if chain[-1].startswith("<"):
            chain = [ref.get("class_name", chain[-1])]
        return ".".join([ref["module_name"]] + chain)

    def mro(self, module, name):
        """Qualified names of a class and its ancestors, from Pyrefly's own definitions."""
        for cd in self.module_defs(module).get("class_definitions", {}).values():
            if ".".join(self.class_chain(module, cd["class_id"])) == name:
                anc = cd.get("mro", {}).get("Resolved", [])
                return [f"{module}.{name}"] + [self.class_qual(a) for a in anc]
        return [f"{module}.{name}"]


def pyrefly_call(entry, pf):
    """(targets as qualified names, constructor classes, unresolved reason or None)."""
    call = entry.get("Call") or entry.get("AttributeAccess") or {}
    if "callees" in call:  # AttributeAccess wraps its call callees
        call = call["callees"]
    targets, ctor = [], []
    for t in call.get("call_targets", []):
        tgt = t["target"]
        if "Function" in tgt:
            targets.append(pf.func_qual(tgt["Function"]))
        elif "Overrides" in tgt:
            targets.append(pf.func_qual(tgt["Overrides"]))
        else:
            targets.append(next(iter(tgt)))
    for key in ("init_targets", "new_targets"):
        for t in call.get(key, []):
            tgt = t["target"].get("Function") or t["target"].get("Overrides")
            if tgt:
                q = pf.func_qual(tgt).rsplit(".", 1)[0]
                if q not in ("builtins.object", "builtins.type"):
                    ctor.append(q)
    unresolved = call.get("unresolved")
    reason = unresolved["True"] if isinstance(unresolved, dict) and "True" in unresolved else None
    return targets, sorted(set(ctor)), reason


# ---------- matching ----------

ALIASES = {"NamedTupleFallback": "NamedTuple", "TypedDictFallback": "TypedDict", "NoneType": "None"}


def norm(q):
    """Same definition, different spelling: a class versus its constructor, special forms."""
    q = re.sub(r"\.(__init__|__new__|__call__)$", "", q)
    last = q.rsplit(".", 1)[-1]
    return ALIASES.get(last, last) if last in ALIASES else q


def tail(q, n=2):
    return ".".join(norm(q).split(".")[-n:])


def same(a, b):
    """Two qualified names name the same definition, allowing for re-exporting modules (io vs _io)."""
    if a == b or tail(a) == tail(b) or norm(a) == norm(b):
        return True
    if a.startswith("__unknown__.") or b.startswith("__unknown__."):
        return a.rsplit(".", 1)[-1] == b.rsplit(".", 1)[-1]  # Pyrefly's name for a non-importable file
    return a.startswith(b + ".") or b.startswith(a + ".")  # an enum member and its class


def classify(ty_set, py_set):
    if not ty_set and not py_set:
        return "neither"
    if ty_set and not py_set:
        return "ty only"
    if py_set and not ty_set:
        return "pyrefly only"
    if any(same(a, b) for a in ty_set for b in py_set):
        return "agree" if len(ty_set) == len(py_set) or set(map(tail, ty_set)) == set(map(tail, py_set)) else "overlap"
    return "disagree"


_CLASSES = {}


def enclosing_class(path, line, module):
    """The innermost class around a line: what ty's `Self@method` is bound to.

    ty knows this bound but keeps it crate-private (`TypeVarInstance::upper_bound` is pub(crate)),
    so a library user has to find the class from the source, as done here.
    """
    if path not in _CLASSES:
        spans = []

        def walk(body, prefix):
            for n in body:
                if isinstance(n, ast.ClassDef):
                    q = prefix + [n.name]
                    spans.append((n.lineno, n.end_lineno, ".".join(q)))
                    walk(n.body, q)
                elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    walk(n.body, prefix + [n.name])
        walk(ast.parse(open(path).read()).body, [])
        _CLASSES[path] = spans
    inner = [s for s in _CLASSES[path] if s[0] <= line <= s[1]]
    return f"{module}.{max(inner)[2]}" if inner else None


def type_classes_ty(t, roots, site_path=None, site_line=None):
    if not t or t.get("display") in (None, "Never") or t.get("unknown"):
        return []
    if (t["display"].startswith("Self@") or t["display"].startswith("type[Self@")) and site_path:
        c = enclosing_class(site_path, site_line, module_of(site_path, roots))
        return [c] if c else []
    out = []
    for p in t.get("parts", []):
        if "path" in p:
            q = ty_qual(p, roots)
            if q.startswith("ty_extensions") or q in ("typing.Any", "typing_extensions.Any"):
                return []  # Unknown or Any, alone or in a union: not resolved
            out.append(q)
        elif p.get("display", "").startswith("<module"):
            out.append("module:" + p["display"].split("'")[1])
        elif p.get("display") in ("Any", "Never", "Unknown"):
            return []
        elif p.get("display") == "None":
            out.append("None")
    return out


def type_classes_py(entry, pf):
    if not entry or entry.get("string") in (None, "Unknown", "Any", "typing.Any", "Never", "typing.Never"):
        return []
    s = entry["string"]
    if s.startswith("super["):
        return ["builtins.super"]
    if "class_names" in entry and not entry["class_names"].get("is_exhaustive", True):
        return []  # Pyrefly marks a type with members it cannot name (Unknown, Any) as not exhaustive
    if s.startswith("Module["):
        return ["module:" + m for m in re.findall(r"Module\[([\w.]+)\]", s)]
    cls = entry.get("class_names", {}).get("classes", [])
    return [pf.class_qual(c["class"]) for c in cls] or [re.sub(r"\[.*\]$", "", s)]


def compare_project(name, proj, env, work, results):
    roots = [str(next(env.glob("lib/python3.*/site-packages"))), str(proj)]
    ty = json.load(open(work / "out/ty" / f"{name}.json"))
    pf = Pyrefly(work / "out/pyrefly" / name)
    counts = collections.defaultdict(collections.Counter)
    reasons = collections.Counter()
    rows = []
    for f in ty["files"]:
        path = f["path"]
        if SKIP_DIR.search(path[len(str(proj)):]):
            continue
        stem = pf.by_path.get(path)
        cg = pf.load("call_graphs", stem) if stem else {}
        tp = pf.load("type_of_expressions", stem) if stem else {}
        df = pf.load("definitions", stem) if stem else {}
        calls, types = {}, {}
        for g in cg.get("call_graphs", {}).values():
            for k, v in g.items():
                if "|" not in k:
                    calls[k] = v
        for fn in tp.get("functions", {}).values():
            table = fn["type_table"]
            for loc, i in fn["locations"].items():
                types[loc] = table[i]
        deco = {}
        for fd in df.get("function_definitions", {}).values():
            for loc, callees in (fd.get("decorator_callees") or {}).items():
                deco[loc.split("-")[0]] = [pf.func_qual(c.get("Function") or c.get("Overrides")) for c in callees
                                           if c.get("Function") or c.get("Overrides")]
        bases = {}
        for cd in df.get("class_definitions", {}).values():
            bases[cd["name_location"]] = [pf.class_qual(b) for b in cd.get("bases", [])]

        class_sites = collections.defaultdict(list)
        for s in f["sites"]:
            kind = s["kind"]
            row = {"file": path[len(str(proj)) + 1:], "kind": kind, "range": s["range"], "text": s.get("text")}
            if kind == "call":
                # A call resolves to a function or a class; "go to definition" may stop at a
                # variable (`blue = make_colorizer(...)`) or an import, which names no callee.
                real = [t for t in s["targets"] if t.get("def") in ("function", "class")]
                ty_t = [ty_qual(t, roots) for t in real]
                ty_var_only = bool(s["targets"]) and not real
                if not ty_t and not s["callee_type"].get("unknown"):
                    ty_t = type_classes_ty(s["callee_type"], roots, path, int(s["range"].split(":")[0]))  # a callable instance, cls
                e = calls.get(s["range"])
                py_t, ctor, reason = pyrefly_call(e, pf) if e else ([], [], "no entry")
                for t in real:  # Pyrefly names the class whose __init__ runs: accept any ancestor
                    if t["def"] == "class" and ctor:
                        anc = pf.mro(module_of(t["path"], roots), t["name"])
                        if any(same(c, x) for c in ctor for x in anc):
                            ctor = [ty_qual(t, roots)]
                if e and not py_t and not ctor and "\n" not in (s.get("text") or "\n"):
                    # The call graph names only object.__init__ (seen for FastAPI()): take the class
                    # from Pyrefly's own type of the callee name, `type[C]`.
                    l, c0 = s["range"].split("-")[0].split(":")
                    callee = types.get(f"{l}:{c0}-{l}:{int(c0) + len(s['text'])}")
                    if callee and callee.get("string", "").startswith("type["):
                        ctor = type_classes_py(callee, pf)
                        for t in real:
                            if t["def"] == "class" and ctor:
                                anc = pf.mro(module_of(t["path"], roots), t["name"])
                                if any(same(c, x) for c in ctor for x in anc):
                                    ctor = [ty_qual(t, roots)]
                py_all = py_t + ctor
                row["ty_variable_only"] = ty_var_only
                c = classify(ty_t, py_all) if e else "no pyrefly record"
                if reason and not py_all:
                    reasons[reason] += 1
                row.update(ty=ty_t, pyrefly=py_all, pyrefly_unresolved=reason, ty_type=s["callee_type"].get("display"))
            elif kind == "attr" and re.search(r"\.(setter|getter|deleter)$", s.get("text") or ""):
                continue  # the receiver of a built-in property decorator
            elif kind == "attr":
                kind = "write receiver" if s["ctx"] == "store" else "receiver"
                ty_c = type_classes_ty(s["base_type"], roots, path, int(s["base_range"].split(":")[0]))
                py_c = type_classes_py(types.get(s["base_range"]), pf)
                c = classify(ty_c, py_c) if s["base_range"] in types else "no pyrefly record"
                if (types.get(s["base_range"]) or {}).get("string") in ("Never", "typing.Never"):
                    reasons[f"Never at a {kind}" + (" (ty: a class)" if ty_c else "")] += 1
                row.update(kind=kind, ty=ty_c, pyrefly=py_c, ty_type=s["base_type"].get("display"),
                           pyrefly_type=(types.get(s["base_range"]) or {}).get("string"),
                           ty_field=[ty_qual(t, roots) for t in s["targets"]])
            elif kind == "decorator" and BUILTIN_DECORATOR.match(s.get("text") or ""):
                continue  # Pyrefly treats these specially and never lists them as decorator callees
            elif kind == "decorator":
                ty_t = [ty_qual(t, roots) for t in s["targets"] if t.get("def") in ("function", "class")]
                py_t = deco.get(s["range"].split("-")[0], [])
                c = classify(ty_t, py_t)
                row.update(ty=ty_t, pyrefly=py_t)
            elif kind == "base":
                class_sites[s["class_range"]].append(s)
                continue
            counts[kind][c] += 1
            row["outcome"] = c
            rows.append(row)
        # Bases are compared per class: Pyrefly lists only the bases it resolved.
        for crange, sites in class_sites.items():
            ty_t = []
            for s in sites:
                cls = [t for t in s["targets"] if t.get("def") == "class"]
                if cls:
                    ty_t.append(ty_qual(cls[0], roots))
                else:
                    tc = type_classes_ty(s.get("type"), roots)
                    if tc:
                        ty_t.append(tc[0])
            py_t = bases.get(crange, [])
            n = len(sites)
            ty_ok, py_ok = len(ty_t) == n, len(py_t) == n
            c = {(True, True): "agree" if all(any(same(a, b) for b in py_t) for a in ty_t) else "disagree",
                 (True, False): "ty only", (False, True): "pyrefly only", (False, False): "neither"}[(ty_ok, py_ok)]
            counts["class bases"][c] += 1
            rows.append({"file": path[len(str(proj)) + 1:], "kind": "class bases", "range": crange,
                         "text": ", ".join(s["text"] for s in sites), "ty": ty_t, "pyrefly": py_t, "outcome": c})
    with open(results / f"sites-{name}.jsonl", "w") as out:
        for r in rows:
            out.write(json.dumps(r) + "\n")
    return {k: dict(v) for k, v in counts.items()}, dict(reasons.most_common())


ORDER = ["agree", "overlap", "disagree", "ty only", "pyrefly only", "neither", "no pyrefly record"]
KINDS = ["call", "receiver", "write receiver", "decorator", "class bases"]


def main():
    work = pathlib.Path(sys.argv[1]).resolve()
    results = pathlib.Path(sys.argv[2]).resolve()
    results.mkdir(parents=True, exist_ok=True)
    timings = json.load(open(work / "out/timings.json"))
    summary = {}
    for name, (proj, env, py, dirs) in projects(work).items():
        counts, reasons = compare_project(name, proj, env, work, results)
        summary[name] = {"counts": counts, "pyrefly_unresolved_reasons": reasons, "timings": timings.get(name)}
        print(f"\n== {name}  {timings.get(name)}")
        print(f"{'kind':16}" + "".join(f"{o:>14}" for o in ORDER) + f"{'ty %':>8}{'pyrefly %':>11}")
        for k in KINDS:
            c = counts.get(k, {})
            n = sum(v for o, v in c.items() if o != "no pyrefly record")
            if not n:
                continue
            ty_res = sum(c.get(o, 0) for o in ("agree", "overlap", "disagree", "ty only"))
            py_res = sum(c.get(o, 0) for o in ("agree", "overlap", "disagree", "pyrefly only"))
            print(f"{k:16}" + "".join(f"{c.get(o, 0):>14}" for o in ORDER)
                  + f"{100 * ty_res / n:>7.1f}%{100 * py_res / n:>10.1f}%")
        print("pyrefly unresolved reasons (calls it left without target):", reasons)
    (results / "compare.json").write_text(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main()
