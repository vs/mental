"""Seeded changes to rq 2.12, each labelled before any result was seen.

harmful: a change a user of rq would call a bug in job handling.
benign: a change to internal representation, bookkeeping or timing that a
user of rq would not notice in the state of their jobs.

A good business-level abstraction shows every harmful change and hides every
benign one. Run as a script to create the mutant copies:

    python -I mutants.py <rq-2.12 checkout> <output dir>
"""

from __future__ import annotations

import pathlib
import shutil
import sys

NONATOMIC_DEQUEUE = """head = connection.lindex(queue_key, 0)
            result = None
            if head is not None:
                connection.lrem(queue_key, 1, head)
                connection.rpush(intermediate_queue.key, head)
                result = head"""

MUTANTS: list[tuple[str, str, str, str, str]] = [
    # name, label, file, old, new
    ('no-started-status', 'harmful', 'rq/job.py',
     '        self._status = JobStatus.STARTED\n        mapping: Mapping = {',
     '        mapping: Mapping = {'),
    ('success-skips-finished-registry', 'harmful', 'rq/job.py',
     '            finished_job_registry.add(self, result_ttl, pipeline)', '            pass'),
    ('failure-marks-finished', 'harmful', 'rq/worker/base.py',
     '                if not retry:\n                    job.set_status(JobStatus.FAILED, pipeline=pipeline)',
     '                if not retry:\n                    job.set_status(JobStatus.FINISHED, pipeline=pipeline)'),
    ('cancel-skips-canceled-registry', 'harmful', 'rq/job.py',
     '                registry.add(self, pipeline=pipe)', '                pass'),
    ('requeue-keeps-failed-entry', 'harmful', 'rq/registry.py',
     '        result = self.connection.zrem(self.key, job.id)',
     '        result = self.connection.zscore(self.key, job.id) is not None'),
    ('nonatomic-dequeue', 'harmful', 'rq/queue.py',
     'result = cast(Any | None, connection.lmove(queue_key, intermediate_queue.key))', NONATOMIC_DEQUEUE),
    ('delete-keeps-registries', 'harmful', 'rq/job.py',
     '        self._remove_from_registries(pipeline=pipeline, remove_from_queue=remove_from_queue)\n', '        pass\n'),
    ('retry-never-counts-down', 'harmful', 'rq/job.py',
     '        self.retries_left = self.retries_left - 1', '        self.retries_left = self.retries_left'),
    ('abandoned-job-not-recorded', 'harmful', 'rq/registry.py',
     "                        job._handle_failure(exc_string, pipeline, worker_name='')", '                        pass'),
    ('dependents-never-enqueued', 'harmful', 'rq/queue.py',
     '                # There\'s no dependents\n                if not dependent_job_ids:',
     '                dependent_job_ids = set()\n                if not dependent_job_ids:'),
    ('no-success-counter', 'benign', 'rq/worker/base.py',
     '                    self.increment_successful_job_count(pipeline=pipeline)\n', '                    pass\n'),
    ('heartbeat-ttl', 'benign', 'rq/worker/worker_classes.py',
     '            return int(job.timeout or DEFAULT_WORKER_TTL) + 60', '            return int(job.timeout or DEFAULT_WORKER_TTL) + 90'),
    ('result-ttl-default', 'benign', 'rq/defaults.py', 'DEFAULT_RESULT_TTL = 500', 'DEFAULT_RESULT_TTL = 600'),
    ('reorder-inside-transaction', 'benign', 'rq/worker/base.py',
     '                    self.increment_successful_job_count(pipeline=pipeline)\n'
     '                    self.increment_total_working_time(job.ended_at - job.started_at, pipeline)  # type: ignore\n',
     '                    self.increment_total_working_time(job.ended_at - job.started_at, pipeline)  # type: ignore\n'
     '                    self.increment_successful_job_count(pipeline=pipeline)\n'),
    ('worker-state-word', 'benign', 'rq/worker/base.py', "    BUSY = 'busy'", "    BUSY = 'working'"),
    ('log-text', 'benign', 'rq/worker/base.py', "'Successfully completed %s job in", "'Completed %s job in"),
    ('execution-key-layout', 'benign', 'rq/executions.py', "return f'rq:execution:{self.composite_key}'", "return f'rq:exec:{self.composite_key}'"),
    ('extra-read', 'benign', 'rq/worker/base.py',
     '        with self.connection.pipeline() as pipeline:\n            self.set_current_job_id(job.id, pipeline=pipeline)',
     '        self.connection.exists(job.key)\n        with self.connection.pipeline() as pipeline:\n            self.set_current_job_id(job.id, pipeline=pipeline)'),
    ('worker-extra-field', 'benign', 'rq/worker/base.py',
     "        connection.hset(self.key, 'state', state)",
     "        connection.hset(self.key, mapping={'state': state, 'state_note': 'v2'})"),
    ('result-extra-field', 'benign', 'rq/results.py',
     '        result = connection.xadd(key, self.serialize(), maxlen=10)',
     "        result = connection.xadd(key, {**self.serialize(), 'schema_note': 'v2'}, maxlen=10)"),
]


def create(source: pathlib.Path, out: pathlib.Path) -> None:
    for name, _label, rel, old, new in MUTANTS:
        target = out / name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(source / 'rq', target / 'rq')
        shutil.copytree(source / 'docs', target / 'docs')
        path = target / rel
        text = path.read_text()
        if text.count(old) != 1:
            raise SystemExit(f'mutant {name}: the text to replace occurs {text.count(old)} times in {rel}; fix the mutant')
        path.write_text(text.replace(old, new))
        print(f'{name}: {rel}')


if __name__ == '__main__':
    create(pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]))
