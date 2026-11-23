# Pyrefly against ty on real code

Research code behind the [experiment section of the report](../report.md#experiment-pyrefly-against-ty-on-five-projects). It runs Pyrefly's Pysa export and ty used as a Rust library on five projects, and compares their answers site by site.

## Layout

| Path | What it does |
|---|---|
| `tyharness/` | A Rust program on ty's crates (ruff commit `4228f4c`, the commit of ty 0.0.105). It walks ruff's syntax tree of each file and asks ty, per site, for the definition ("go to definition") and the inferred type. One record per call, attribute access (load or store), decorator and class base |
| `run.py` | Writes one Pyrefly and one ty configuration per project and runs both tools; records the time each takes |
| `compare.py` | Looks up Pyrefly's answer for every ty site by its `line:col-line:col` range, normalises names and classifies each site: `agree`, `overlap` (several targets, at least one shared), `disagree`, `ty only`, `pyrefly only`, `neither`, `no pyrefly record` |
| `mentalc.py` | Pairs the sites `mentalc build` writes with ty's answers recorded here ([below](#mentalc-build-against-these-answers)) |
| `neither.py` | For receivers neither tool resolved: is the base an unannotated parameter, a chain from one, a `self` attribute chain, or something else |
| `results/compare.json` | Counts per project and site kind, Pyrefly's unresolved reasons, timings |
| `results/neither.json` | Output of `neither.py` |
| `results/sites-<project>.jsonl.gz` | Every site with both answers |

## Corpus

| Project | Commit | Analysed | Lines (no tests) | Environment |
|---|---|---|---|---|
| report-jobs | this repository | `examples/report-jobs/src/app` | 189 | `requirements.lock`, Python 3.12 |
| [full-stack-fastapi-template](https://github.com/fastapi/full-stack-fastapi-template) | `77e550a` (2026-11-16) | `backend/app` | 1,513 | `uv.lock`, Python 3.14 |
| [mealie](https://github.com/mealie-recipes/mealie) | `04ba6b7` (2026-11-17) | `mealie` | 45,460 | `uv.lock`, Python 3.14 |
| [redash](https://github.com/getredash/redash) | `f561a53` (2026-11-10) | `redash` | 27,401 | `uv.lock`, Python 3.13 |
| [rq](https://github.com/rq/rq) | `5351724` (2026-11-13) | `rq` | 13,341 | `uv sync`, Python 3.13 |

Directories named `tests`, `test` or `testing` are left out of the counts.

## Reproduce

```sh
W=<work dir>
uv venv --python 3.12 $W/tools && uv pip install --python $W/tools/bin/python pyrefly==1.4.1
(cd tyharness && CARGO_TARGET_DIR=$W/target cargo build --release)
uv venv --python 3.12 $W/env-report-jobs && uv pip install --python $W/env-report-jobs/bin/python -r ../../../../examples/report-jobs/src/requirements.lock
mkdir -p $W/src && cd $W/src
git clone https://github.com/fastapi/full-stack-fastapi-template && git -C full-stack-fastapi-template checkout 77e550a
git clone https://github.com/mealie-recipes/mealie && git -C mealie checkout 04ba6b7
git clone https://github.com/getredash/redash && git -C redash checkout f561a53
git clone https://github.com/rq/rq && git -C rq checkout 5351724
(cd full-stack-fastapi-template && uv sync --frozen --python 3.14)
(cd mealie && uv sync --frozen --python 3.14)
(cd redash && uv sync --frozen --python 3.13)
(cd rq && uv sync --python 3.13)
cd -
python3 -I run.py $W
python3 -I compare.py $W results
python3 -I neither.py $W results
```

Installing the environments runs the projects' build backends (`uv sync`); the two tools themselves start no interpreter: Pyrefly gets the site-packages path and `skip-interpreter-query`, ty reads the environment's `pyvenv.cfg` and site-packages as files.

## What the comparison normalises

Each rule below removed a difference of spelling, not of answer; every rule was added after reading the sites it affects.

- **`self` and `cls`.** ty types them `Self@method`, a type variable whose bound (the enclosing class) is crate-private in ty's API; the script takes the enclosing class from the source.
- **Narrowed types.** ty writes `Recipe & ~AlwaysFalsy`; the harness asks ty's public "go to type definition" for the class.
- **Constructors.** For `ValueError(...)` Pyrefly names the class whose `__init__` runs (`BaseException`), ty names `ValueError`; accepted when Pyrefly's class is in the MRO of ty's, by Pyrefly's own definitions. Where Pyrefly's call graph names only `object.__init__` (seen for `FastAPI()`), its type of the callee name (`type[FastAPI]`) is used.
- **Same object, other name.** `cached_property` against `cached_property.__init__`; `NamedTuple` against `NamedTupleFallback`; `io` against `_io`; an enum member against its class; Pyrefly's module name `__unknown__` for files that are not importable modules (alembic migrations).
- **Unresolved on both sides.** `Unknown`, `Any`, `Never`, and a union with any of them; Pyrefly's `is_exhaustive: false`.
- **"Go to definition" on a variable.** For a call, ty may stop at the variable (`blue = make_colorizer(...)`) or an import; that names no callee, so the script falls back to ty's type of the callee.
- **Out of scope for one tool.** Built-in decorators (`property`, `classmethod`, `staticmethod`, `overload`, setters) are not listed by Pyrefly; expressions inside annotations are not exported by Pyrefly (`no pyrefly record`).

## `mentalc build` against these answers

On 2026-11-23, milestone 1's `mentalc build` was run on rq and full-stack-fastapi-template at the commits above, and its sites paired with ty's recorded answers by `mentalc.py` ([findings, section B](../../../findings.md#b-mentalc-build-on-outside-projects)). Neither project builds as committed, so each got the setup its diagnostics asked for:

```sh
M=<path to target/release/mentalc>
(cd rq && uv python pin 3.13 \
  && printf '\n[tool.uv]\nenvironments = ["python_version == '"'"'3.13'"'"'"]\n' >> pyproject.toml \
  && uv lock --exclude-newer 2026-11-18T00:00:00Z && uv sync --frozen && $M build)
(cd full-stack-fastapi-template && uv sync --frozen --python 3.14 && cd backend \
  && ln -s ../uv.lock uv.lock && ln -s ../.python-version .python-version && ln -s ../.venv .venv && $M build)
python3 -I mentalc.py <work dir>/src/rq results/sites-rq.jsonl.gz rq/ 10
python3 -I mentalc.py <work dir>/src/full-stack-fastapi-template/backend results/sites-fastapi-template.jsonl.gz app/ 10
```

rq has no `uv.lock` and no `.python-version`; its lock forks by Python version, which `mentalc` refuses, so `environments` keeps it to the 3.13 the experiment used, and `--exclude-newer` keeps it to packages published before the experiment. The template is a uv workspace whose lock, Python version and environment sit at the workspace root, above the project in `backend/`; the links put them where `mentalc` looks.

Experiment B1 ran it the same day on Mealie and Redash, as committed, with no setup beyond their environments ([findings](../../../findings.md#b-mentalc-build-on-outside-projects), F-B7 to F-B9). Both exit 1, for the gaps of F-B9, and both write their sites:

```sh
(cd mealie && uv sync --frozen --python 3.14 && $M build)
(cd redash && uv sync --frozen --python 3.13 && $M build)
python3 -I mentalc.py <work dir>/src/mealie results/sites-mealie.jsonl.gz mealie/ 10
python3 -I mentalc.py <work dir>/src/redash results/sites-redash.jsonl.gz redash/ 10
```

`mentalc` lists a decorator that is a call as a decorator site, not a callee, and the experiment truncated the text of a multi-line callee; `mentalc.py` counts both apart as unmatched. Three more spellings differ and are left as they are, so they show among the differing pairs: the experiment names a module `module:x` where `mentalc.py` prints `x`, a string's `.format` call carries the string in the experiment's text, and the script drops a trailing number from a name, so `get_object_or_404` finds no site.
