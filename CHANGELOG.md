# Changelog

All notable changes, newest first. Versions follow semantic versioning.

## Unreleased

- `model`: semantic IDs and slugs, anchors, diagnostics with their codes, and canonical JSONL that reads only what it writes.
- `model`: the `build_unit` and `pin` records, and the hand-written reference of report-jobs' build unit and 40 pins.
- `extract`: the build readers of `pyproject.toml`, `uv.lock`, `.python-version` and `pyvenv.cfg`, with the gaps `project-invalid`, `lock-invalid` and `lock-ambiguous`, and the pin origin `url`.
- `model`: the `site` record, and the hand-written reference of report-jobs' 133 reference sites.
- `extract`: `list_sites` parses each Python file with ruff's parser and lists its imports, decorators, callees, base classes, write receivers and handler types, with an ID and an anchor each; a file that does not parse gives a `syntax-error` per error.
