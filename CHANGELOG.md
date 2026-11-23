# Changelog

All notable changes, newest first. Versions follow semantic versioning.

## Unreleased

- `model`: semantic IDs and slugs, anchors, diagnostics with their codes, and canonical JSONL that reads only what it writes.
- `model`: the `build_unit` and `pin` records, and the hand-written reference of report-jobs' build unit and 40 pins.
- `extract`: the build readers of `pyproject.toml`, `uv.lock`, `.python-version` and `pyvenv.cfg`, with the gaps `project-invalid`, `lock-invalid` and `lock-ambiguous`, and the pin origin `url`.
- `mentalc build`: writes the build model of the project in the current directory to `.mental/`, prints its diagnostics as text or, with `--json`, as JSON lines, and exits 0, 1 or 2.
- `mentalc build`: when the build model has no error, resolves every reference site of the project's Python files with ty and writes `site.jsonl`, their anchors and diagnostics, and `resolution.json`; its last line gives the totals.
- `model`: the `site` record, and the hand-written reference of report-jobs' 133 reference sites.
- `extract`: `extract_sites` parses each Python file with ruff's parser, lists its imports, decorators, callees, base classes, write receivers, handler types and escapes, and resolves each with ty to `resolved`, `external` or `unresolved` with a reason, with an ID and an anchor each; a file that does not parse gives a `syntax-error` per error, and an import of a module that is nowhere a `module-missing`.
- `model`: the outcome and checker of each site, and report-jobs' 140 sites with their outcomes.
- `extract`: `self` and `cls` in a method resolve to the method's class, at write receivers and calls of `cls`; they were `not_found`.
