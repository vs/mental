# Part P: disagreements read by hand

Ten sites were drawn at random (fixed seed) from each disagreement category of
`examples-<tool>.jsonl.gz`, read against the source line, and judged:

- **a**: the same object, named differently;
- **b**: the index names the declaration (a variable, an alias, a parameter, the function under a
  decorator), and ty names what runs (a wrapper's `__call__`, a callable's class);
- **c**: the index is wrong;
- **d**: ty is wrong;
- **e**: both can be meant, because the receiver is a union.

This is a sample, not every disagreement: the categories read hold 1,118 sites for scip-python
and 373 for Pyrefly (`summary-python.md`). The samples were drawn before the last pairing fix
(the longest name a site's slug ends with), which moved one site into these categories.

## Pyrefly 1.5.0

| Category | Read | a | b | c | d | e | Seen |
|---|---|---|---|---|---|---|---|
| other | 10 | 1 | 8 | 1 | 0 | 0 | `get_app_settings()` under `@lru_cache`: ty `_lru_cache_wrapper.__call__`, index the function; `jwt.encode`: ty `PyJWT.encode`, index the module variable `encode`; `from text_unidecode import os`: index names `mealie.routes.spa` (wrong) |
| same name, other module | 10 | 0 | 6 | 0 | 0 | 4 | `self.t(...)`: index the class attribute `t`, ty `Translator.t`; `connection.sadd(...)` on `Redis \| Pipeline`: ty names `Redis`'s method, the index `Pipeline`'s |
| index names a variable or attribute | 10 | 0 | 10 | 0 | 0 | 0 | `Session()`, `exit()`, `rmtree()`, `IOError(...)`: the index stops at the name's declaration |
| ty names a member of the index's answer | 10 | 0 | 10 | 0 | 0 | 0 | a parameter called (`on_progress(...)`, `cls(...)`): the index names the parameter, ty the callable it is annotated with |
| nested definition named without its enclosing function | 10 | 10 | 0 | 0 | 0 | 0 | `db.Model` as a base: ty `SQLAlchemy.Model` set in `__init__`, the index `SQLAlchemy#Model.` |

## scip-python 0.6.8

| Category | Read | a | b | c | d | e | Seen |
|---|---|---|---|---|---|---|---|
| other | 10 | 3 | 5 | 2 | 0 | 0 | `os.path.abspath(...)` and `os.path.isfile(...)`: index `genericpath/exists()` (wrong); `TYPE_CHECKING` and `flask.request` named `TYPE_CHECKING.TYPE_CHECKING.`, `request.request.` |
| same name, other module | 10 | 4 | 1 | 5 | 0 | 0 | `….filter(...)` on SQLAlchemy queries: index `builtins/filter#` (wrong); `os.environ.get`: index `typing/Mapping#get()`, the method's declaring base |
| module named without its package | 10 | 10 | 0 | 0 | 0 | 0 | `from alembic import op`: index `op/__init__:`; `from . import cleaner`: index `cleaner/__init__:`. The object is right, but the symbol names a top-level module that does not exist |
| nested definition named without its enclosing function | 10 | 10 | 0 | 0 | 0 | 0 | a function defined inside a method is named as a method of the class: `RepositoryHousehold#model_count()` for `RepositoryHousehold.statistics.model_count` |
| same object, private module | 10 | 10 | 0 | 0 | 0 | 0 | `io/BytesIO#` for `_io.BytesIO`, `contextvars/ContextVar#` for `_contextvars.ContextVar` |

## What it says

- **Pyrefly's index is nearly always right where it differs from ty**: 1 wrong answer in 50. Most
  differences (34 of 50) are the index naming the declaration where ty names what runs, which is
  the missing expression type of the plan, not an error.
- **scip-python's index has wrong targets**: 7 in 50, where its 2023 Pyright resolves an
  attribute of a typed receiver to an unrelated builtin (`filter`, `genericpath.exists`). And
  three naming defects that break identity: module symbols without their package, nested
  functions merged into their class, and doubled names.
- **No case where ty was wrong** turned up in the samples.
