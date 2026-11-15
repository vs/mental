"""Build the prompt that asks an LLM to propose an abstraction function, and
load the functions it proposes.

The prompt follows Mental's per-port slicing: whole files for the entity
modules, and only the queue and worker methods that the scenarios call.
"""

from __future__ import annotations

import ast
import json
import pathlib
import pickle
import re
from collections.abc import Callable
from typing import Any

WHOLE_FILES = ['rq/job.py', 'rq/registry.py', 'rq/executions.py', 'rq/intermediate_queue.py']
METHODS = {
    'rq/queue.py': ['enqueue', 'enqueue_call', 'enqueue_job', '_enqueue_job', 'enqueue_dependents', 'remove', 'lmove', 'dequeue_any'],
    'rq/worker/base.py': [
        'dequeue_job_and_maintain_ttl', 'prepare_execution', 'prepare_job_execution', 'perform_job',
        'handle_job_success', 'handle_job_failure', 'handle_job_retry', 'cleanup_execution',
    ],
    'rq/worker/worker_classes.py': ['execute_job'],
}
DOCS = ['docs/docs/jobs.md', 'docs/docs/job_registries.md', 'docs/docs/results.md']

TASK = '''You are helping build a runnable model of a software system at several levels of abstraction.
The lower level is the exact content of Redis, as written by the Python library rq. The upper
level is the business level: the state of the system as a user of rq thinks about it.

Write the abstraction function between the two levels: Python code that computes upper-level
state from a lower-level snapshot. Also write the business invariants that should hold at the
upper level, as predicates over the same snapshot.

A snapshot is a dict from Redis key to {"t": type, "v": value, "ttl": bool}:
- "t" is one of "string", "hash", "list", "set", "zset", "stream";
- "v" is a str for a string, a dict for a hash, a list for a list, a sorted list for a set,
  a sorted list of [member, score_class] pairs for a zset (score_class is "T" or "inf"),
  and a list of field dicts for a stream;
- clock values are replaced by "<T>", compressed blobs by "<blob>", tracebacks by "<exc:Name>";
- "ttl" says whether the key has an expiry.

Answer with exactly one ```python code block that defines:

    def abstract(snap: dict) -> dict:
        """Business-level state, as {variable name: JSON-serialisable value}."""

    def invariants(snap: dict) -> dict:
        """Business invariants, as {invariant name: bool}; True means it holds."""

Rules: use only the standard library; be deterministic; never raise on a snapshot from this
system (deleted jobs, half-finished operations and leftovers from races can all appear);
name variables and invariants in the words a user of rq would use; leave out implementation
detail that a user would not consider part of the state.'''


def _methods(source: str, names: list[str]) -> str:
    tree = ast.parse(source)
    lines = source.splitlines()
    out = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name in names:
            start = node.decorator_list[0].lineno if node.decorator_list else node.lineno
            out.append('\n'.join(lines[start - 1 : node.end_lineno]))
    return '\n\n'.join(out)


def build_prompt(rq_root: pathlib.Path, example_graph: pathlib.Path, with_docs: bool) -> str:
    parts = [TASK, '\n## Example snapshots (one sequential run: before, while the job runs, after)\n']
    with open(example_graph, 'rb') as fh:
        graph = pickle.load(fh)
    run = max(graph['runs'], key=lambda r: len(r['path']))
    picks = [run['path'][0], run['path'][len(run['path']) // 2], run['path'][-1]]
    for node_id in picks:
        snap = graph['snaps'][graph['nodes'][node_id]['snap_id']]
        parts.append('```json\n' + json.dumps(snap, indent=1, sort_keys=True) + '\n```')
    parts.append('\n## Source code of rq (the parts that touch job state)\n')
    for rel in WHOLE_FILES:
        parts.append(f'### {rel}\n```python\n' + (rq_root / rel).read_text() + '\n```')
    for rel, names in METHODS.items():
        parts.append(f'### {rel} (selected methods)\n```python\n' + _methods((rq_root / rel).read_text(), names) + '\n```')
    if with_docs:
        parts.append('\n## rq user documentation\n')
        for rel in DOCS:
            parts.append(f'### {rel}\n' + (rq_root / rel).read_text())
    return '\n\n'.join(parts)


CODE_BLOCK = re.compile(r'```python\n(.*?)```', re.S)


def load_answer(path: pathlib.Path) -> dict[str, Callable[[dict[str, Any]], Any]]:
    """Return the abstract and invariants functions defined in one stored answer."""
    text = path.read_text()
    if path.suffix == '.json':
        text = json.loads(text)['result']
    match = CODE_BLOCK.search(text)
    if match is None:
        raise ValueError(f'{path}: no python code block in the answer; re-run that sample')
    namespace: dict[str, Any] = {}
    exec(compile(match.group(1), str(path), 'exec'), namespace)
    return {'abstract': namespace['abstract'], 'invariants': namespace['invariants']}
