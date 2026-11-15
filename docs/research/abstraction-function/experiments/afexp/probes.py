"""Behavioural abstraction: states are equal when no probe can tell them apart.

This is the Nerode congruence that automata learning (L*) approximates with a
finite set of distinguishing experiments. A probe is a sequence of up to two
rq operations, run on a restored copy of the snapshot; its observation is each
operation's return value or exception and the job bodies it ran. The abstract
state is the vector of observations over all probes. It needs no names and no
knowledge of the key layout, only the ability to run the code.
"""

from __future__ import annotations

import itertools
import json
import multiprocessing
import pathlib
import pickle
from collections.abc import Callable
from typing import Any

import fakeredis

from afexp import harness


def probe_ops() -> dict[str, Callable[[Any], Any]]:
    from rq import Queue, SimpleWorker
    from rq.job import Job
    from rq.registry import FailedJobRegistry, StartedJobRegistry
    from rq.timeouts import BaseDeathPenalty

    class NoDeath(BaseDeathPenalty):
        def setup_death_penalty(self) -> None:
            pass

        def cancel_death_penalty(self) -> None:
            pass

    def work(conn: Any) -> str:
        worker = SimpleWorker([Queue('default', connection=conn)], connection=conn, name='wp')
        worker.death_penalty_class = NoDeath
        worker.last_cleaned_at = harness.rq_now()
        found = worker.dequeue_job_and_maintain_ttl(None)
        if found is None:
            return 'idle'
        worker.execute_job(*found)
        return 'worked'

    def on_job(action: str, job_id: str) -> Callable[[Any], Any]:
        def run(conn: Any) -> Any:
            job = Job.fetch(job_id, connection=conn)
            getattr(job, action)()
            return action

        return run

    def requeue(conn: Any) -> Any:
        return FailedJobRegistry('default', connection=conn).requeue('j1').id

    def cleanup(conn: Any) -> Any:
        import datetime

        far = (harness.FROZEN_NOW + datetime.timedelta(days=30)).timestamp()
        StartedJobRegistry('default', connection=conn).cleanup(timestamp=far)
        return 'cleaned'

    return {
        'work': work,
        'cancel-j1': on_job('cancel', 'j1'),
        'delete-j1': on_job('delete', 'j1'),
        'cancel-j2': on_job('cancel', 'j2'),
        'requeue-j1': requeue,
        'cleanup': cleanup,
    }


def probe_sequences(names: list[str], depth: int) -> list[tuple[str, ...]]:
    out: list[tuple[str, ...]] = []
    for length in range(1, depth + 1):
        out.extend(itertools.product(names, repeat=length))
    return out


_OPS: dict[str, Callable[[Any], Any]] = {}


def signature(raw: dict[str, Any], depth: int) -> list[Any]:
    import afjobs

    if not _OPS:
        _OPS.update(probe_ops())
    out = []
    for sequence in probe_sequences(sorted(_OPS), depth):
        conn = harness.restore(fakeredis.FakeServer(), raw)
        afjobs.EFFECTS.clear()
        observed = []
        for name in sequence:
            try:
                observed.append(harness.normalize_value(_OPS[name](conn)))
            except Exception as exc:
                observed.append(f'exc:{type(exc).__name__}')
        out.append([list(sequence), observed, dict(sorted(afjobs.EFFECTS.items()))])
    afjobs.EFFECTS.clear()
    return out


def _work(item: tuple[str, str, dict[str, Any], int]) -> tuple[str, str, str]:
    scenario, snap_id, raw, depth = item
    return scenario, snap_id, harness.digest(signature(raw, depth))


def probe_dir(graphs: pathlib.Path, depth: int) -> None:
    import rq

    version = str(getattr(rq, '__version__', '?'))
    items: dict[tuple[str, str], tuple[str, str, dict[str, Any], int]] = {}
    for path in sorted(graphs.glob('*.b[02].pkl')):
        with open(path, 'rb') as fh:
            graph = pickle.load(fh)
        for snap_id, raw in graph['raw'].items():
            items.setdefault((graph['scenario'], snap_id), (graph['scenario'], snap_id, raw, depth))
    values: dict[str, dict[str, str]] = {}
    with multiprocessing.get_context('fork').Pool(8) as pool:
        for scenario, snap_id, sig in pool.imap_unordered(_work, list(items.values()), chunksize=4):
            values.setdefault(scenario, {})[snap_id] = sig
    target = graphs / f'behavioral-d{depth}@{version}.json'
    target.write_text(json.dumps({'probe_rq': version, 'depth': depth, 'probes': len(probe_sequences(sorted(probe_ops()), depth)), 'values': values}))
    print(f'behavioral-d{depth}@{version}: {len(items)} snapshots -> {target}')
