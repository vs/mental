"""Abstraction functions that need nothing but a snapshot.

Each function maps a normalised Redis snapshot to a JSON-able value. The
evaluator appends the job-body effect counts to every value, so all functions
compare on how they abstract Redis, not on whether they see the outside world.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from typing import Any

from afexp.harness import digest, key_pattern

Snapshot = dict[str, Any]
AF = Callable[[Snapshot], Any]

JOB_KEY = re.compile(r'^rq:job:([^:]+)$')
CONTAINERS = ('list', 'set', 'zset')


def job_ids(snap: Snapshot) -> list[str]:
    return sorted(m.group(1) for k, v in snap.items() if v['t'] == 'hash' and (m := JOB_KEY.match(k)))


def members(entry: dict[str, Any]) -> list[str]:
    if entry['t'] == 'zset':
        return [m for m, _score in entry['v']]
    if entry['t'] in ('list', 'set'):
        return list(entry['v'])
    return []


def holds(entry: dict[str, Any], job_id: str) -> bool:
    return any(m == job_id or m.startswith(job_id + ':') for m in members(entry))


def mentioned_jobs(snap: Snapshot) -> list[str]:
    ids = set(job_ids(snap))
    for k, v in snap.items():
        for m in members(v):
            if re.fullmatch(r'j\d+(:\S+)?', m):
                ids.add(m.split(':')[0])
    return sorted(ids)


def identity(snap: Snapshot) -> Any:
    return digest(snap)


def status_only(snap: Snapshot) -> Any:
    """Mental's current mechanical rule: the enum field that code branches on."""
    return [[j, snap[f'rq:job:{j}']['v'].get('status')] for j in job_ids(snap)]


def membership(snap: Snapshot) -> Any:
    """Shape abstraction: which key families hold each job, and whether its hash exists."""
    out = []
    for j in mentioned_jobs(snap):
        families = sorted({key_pattern(k) for k, v in snap.items() if v['t'] in CONTAINERS and holds(v, j)})
        out.append([j, f'rq:job:{j}' in snap, families])
    return out


def read_projection(read_set: set[tuple[str, str]]) -> AF:
    """Cone of influence: keep only the locations some code path reads.

    read_set holds (key family, field) pairs; field '*' means the whole key.
    """

    whole = {family for family, field in read_set if field == '*'}
    fields: dict[str, set[str]] = {}
    for family, field in read_set:
        if field != '*':
            fields.setdefault(family, set()).add(field)

    def af(snap: Snapshot) -> Any:
        out = {}
        for k, v in snap.items():
            family = key_pattern(k)
            if family in whole:
                out[k] = v['v']
            elif family in fields and v['t'] == 'hash':
                out[k] = {f: v['v'].get(f) for f in sorted(fields[family])}
        return out

    return af


REGISTRIES = {
    'queue': 'rq:queue:default',
    'started': 'rq:wip:default',
    'finished': 'rq:finished:default',
    'failed': 'rq:failed:default',
    'deferred': 'rq:deferred:default',
    'scheduled': 'rq:scheduled:default',
    'canceled': 'rq:canceled:default',
}


def reference(snap: Snapshot) -> Any:
    """The oracle, written from rq's user documentation before any result was seen.

    Per job: its status, the documented queue and registries listing it, and the
    type of its latest result. Worker bookkeeping, executions, heartbeats, TTLs and
    the intermediate queue are implementation detail.
    """
    out = []
    for j in mentioned_jobs(snap):
        job = snap.get(f'rq:job:{j}')
        status = job['v'].get('status') if job else 'gone'
        where = sorted(name for name, key in REGISTRIES.items() if key in snap and holds(snap[key], j))
        results = snap.get(f'rq:results:{j}')
        last = results['v'][-1].get('type') if results and results['v'] else None
        out.append([j, status, where, last])
    return out


EXPECTED_LOCATION = {
    'queued': ['queue'],
    'started': ['started'],
    'finished': ['finished'],
    'failed': ['failed'],
    'stopped': ['failed'],
    'canceled': ['canceled'],
    'deferred': ['deferred'],
    'scheduled': ['scheduled'],
    'gone': [],
}


def doc_invariant_violations(snap: Snapshot) -> list[str]:
    """Jobs breaking the documented invariant: a job is listed in exactly the
    queue or registry that matches its status, and a deleted job nowhere."""
    bad = []
    for j, status, where, _last in reference(snap):
        stray = [k for k, v in snap.items() if v['t'] in CONTAINERS and k.endswith(':intermediate') and holds(v, j)]
        if where != EXPECTED_LOCATION.get(status, where) or stray:
            bad.append(j)
    return bad


PURE: dict[str, AF] = {
    'identity': identity,
    'status-only': status_only,
    'membership': membership,
    'reference': reference,
}

READ_COMMANDS = {
    'GET', 'HGET', 'HGETALL', 'HMGET', 'HEXISTS', 'EXISTS', 'LRANGE', 'LLEN', 'LPOS', 'LINDEX',
    'SMEMBERS', 'SISMEMBER', 'SCARD', 'ZRANGE', 'ZRANGEBYSCORE', 'ZSCORE', 'ZCARD', 'XRANGE',
    'XREVRANGE', 'TYPE', 'LMOVE', 'LPOP', 'BLMOVE', 'WATCH', 'EVALSHA',
}


def reads_of(label: str) -> list[tuple[str, str]]:
    """Locations a step label reads, as (key family, field) pairs."""
    if label.startswith('EXEC['):
        parts = label[5:-1].split('; ')
    else:
        parts = [label]
    out = []
    for part in parts:
        words = part.split(' ')
        if len(words) < 2 or words[0] not in READ_COMMANDS:
            continue
        if words[0] in ('HGET', 'HMGET', 'HEXISTS') and len(words) > 2:
            out.extend((words[1], f) for f in words[2].split(','))
        else:
            out.append((words[1], '*'))
    return out
