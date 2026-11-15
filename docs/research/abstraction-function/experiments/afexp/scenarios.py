"""Scenarios: an initial Redis state and a program of rq API calls per actor.

TRAIN scenarios may be read by the derivation algorithms that learn from runs.
HELD_OUT scenarios are only ever used to evaluate them.
"""

from __future__ import annotations

import datetime
from collections.abc import Callable
from typing import Any

import redis

import afjobs
from afexp import harness
from afexp.harness import FROZEN_NOW, Op

TRAIN = ['work-ok', 'fail-retry', 'requeue-vs-work', 'dependency']
HELD_OUT = ['cancel-vs-work', 'cleanup-vs-work', 'two-workers', 'delete-vs-work']
ALL = TRAIN + HELD_OUT

Scenario = tuple[Callable[[redis.Redis], None], dict[str, list[Op]]]


def build() -> dict[str, Scenario]:
    from rq import Queue, Retry, SimpleWorker
    from rq.job import Job
    from rq.registry import FailedJobRegistry, StartedJobRegistry
    from rq.timeouts import BaseDeathPenalty

    class NoDeath(BaseDeathPenalty):
        # Signals only work on the main thread; the experiments never time a job out.
        def setup_death_penalty(self) -> None:
            pass

        def cancel_death_penalty(self) -> None:
            pass

    def queue(conn: redis.Redis) -> Any:
        return Queue('default', connection=conn)

    def work_once(conn: redis.Redis, name: str) -> str:
        worker = SimpleWorker([queue(conn)], connection=conn, name=name)
        worker.death_penalty_class = NoDeath
        # shortcut: skips the worker's periodic registry maintenance, which the
        # cleanup-vs-work scenario exercises on its own; upgrade if maintenance races matter.
        worker.last_cleaned_at = harness.rq_now()
        found = worker.dequeue_job_and_maintain_ttl(None)
        if found is None:
            return 'idle'
        job, q = found
        worker.execute_job(job, q)
        return 'worked'

    def work(name: str) -> Op:
        return Op('take', lambda conn: work_once(conn, name))

    def enqueue_ok(job_id: str, **kwargs: Any) -> Callable[[redis.Redis], Any]:
        return lambda conn: queue(conn).enqueue(afjobs.ok, job_id, job_id=job_id, **kwargs).id

    def enqueue_boom(job_id: str, **kwargs: Any) -> Callable[[redis.Redis], Any]:
        return lambda conn: queue(conn).enqueue(afjobs.boom, job_id, job_id=job_id, **kwargs).id

    def cancel(conn: redis.Redis) -> str:
        Job.fetch('j1', connection=conn).cancel()
        return 'canceled'

    def delete(conn: redis.Redis) -> str:
        Job.fetch('j1', connection=conn).delete()
        return 'deleted'

    def requeue(conn: redis.Redis) -> str:
        FailedJobRegistry('default', connection=conn).requeue('j1')
        return 'requeued'

    far_future = (FROZEN_NOW + datetime.timedelta(days=30)).timestamp()

    def cleanup(conn: redis.Redis) -> str:
        StartedJobRegistry('default', connection=conn).cleanup(timestamp=far_future)
        return 'cleaned'

    def setup_enqueued_ok(conn: redis.Redis) -> None:
        enqueue_ok('j1')(conn)

    def setup_failed(conn: redis.Redis) -> None:
        enqueue_boom('j1')(conn)
        work_once(conn, 'w0')

    def nothing(conn: redis.Redis) -> None:
        pass

    return {
        'work-ok': (nothing, {'c': [Op('enqueue', enqueue_ok('j1'))], 'w1': [work('w1')]}),
        'fail-retry': (
            lambda conn: enqueue_boom('j1', retry=Retry(max=1))(conn),
            {'w1': [work('w1')], 'w2': [work('w2')]},
        ),
        'requeue-vs-work': (setup_failed, {'c': [Op('requeue', requeue)], 'w1': [work('w1')]}),
        'dependency': (
            setup_enqueued_ok,
            {'c': [Op('enqueue-dependent', enqueue_ok('j2', depends_on='j1'))], 'w1': [work('w1'), work('w1')]},
        ),
        'cancel-vs-work': (setup_enqueued_ok, {'c': [Op('cancel', cancel)], 'w1': [work('w1')]}),
        'cleanup-vs-work': (setup_enqueued_ok, {'m': [Op('cleanup', cleanup)], 'w1': [work('w1')]}),
        'two-workers': (setup_enqueued_ok, {'w1': [work('w1')], 'w2': [work('w2')]}),
        'delete-vs-work': (setup_enqueued_ok, {'c': [Op('delete', delete)], 'w1': [work('w1')]}),
    }
