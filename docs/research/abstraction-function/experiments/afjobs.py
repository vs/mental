"""Job bodies for the experiments. EFFECTS counts how often each job body ran:
the one effect outside Redis that a user of rq would notice.

ON_START marks the business-level boundary between a worker taking a job and
completing it; the harness sets it so that the job body splits the work
operation into two upper steps.
"""

from collections import Counter
from collections.abc import Callable

EFFECTS: Counter[str] = Counter()
ON_START: list[Callable[[], None]] = []


def _started() -> None:
    for hook in ON_START:
        hook()


def ok(tag: str) -> int:
    _started()
    EFFECTS[tag] += 1
    return 1


def boom(tag: str) -> int:
    _started()
    EFFECTS[tag] += 1
    raise ValueError(f'job {tag} fails on purpose')
