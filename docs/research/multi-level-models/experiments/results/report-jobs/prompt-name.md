You are naming the steps of a multi-level model of report-jobs, a small service that builds CSV
reports of orders: an HTTP API accepts a report request and queues a task; worker processes build
the CSV, upload it and mark the report done, retrying on upload failure; a periodic job requeues
reports that look stuck; an operator tool retries failed reports. The steps were found mechanically;
do not regroup them. Three levels, from the bottom:

- move: a few steps of one actor (database reads and commits, an upload, a message sent or received)
  that together make one change (or one failed attempt). Readers: developers working on concurrency.
  Mechanism words are fine (claim, commit, attempt); SQL, table, column, bucket and broker names are not.
- activation: one actor's uninterrupted part of an operation, up to a call to an outside service.
  Readers: developers.
- goal: one operation of one actor (an API call, a worker handling one message, the periodic job,
  the operator tool), seen as what that actor got done. Readers: product owners and support staff
  who know what the service does for its users and nothing of its storage. Use only words such a
  user would use.

For every step class give a name of at most five words, verb first, and one clause that starts
with "so that" and says what the step is for. Name moves first, then activations using your move
names, then goals. Different classes need different names. The same class must get the name you
would give it again tomorrow: prefer the plainest wording. A move's "function" names the code
line that closes it.

Answer with exactly one ```json block: {"moves": {id: {"name": ..., "so_that": ...}},
"activations": {...}, "goals": {...}} with every id from the sheet below.

An observable change is written as [status, has a download link, times the file was uploaded, stranded (pending with nothing queued to pick it up)] before -> after; None means no report record.

## Sheet of step classes

```json
{
 "activations": {
  "a:create:0": {
   "api_call": "create",
   "moves": [
    "m:1d90d1",
    "m:86e635"
   ],
   "observable_changes": [
    "('PENDING', False, 0, True) -> ('PENDING', False, 0, False)",
    "None -> ('PENDING', False, 0, True)"
   ],
   "part": 0
  },
  "a:requeue:0": {
   "api_call": "requeue",
   "moves": [
    "m:662f00",
    "m:c2770e"
   ],
   "observable_changes": [
    "('DONE', True, 1, False) -> ('PENDING', True, 1, True)",
    "('PENDING', True, 1, True) -> ('PENDING', True, 1, False)",
    "('RUNNING', False, 0, False) -> ('PENDING', False, 0, False)",
    "('RUNNING', False, 1, False) -> ('PENDING', False, 1, False)"
   ],
   "part": 0
  },
  "a:retry:0": {
   "api_call": "retry",
   "moves": [
    "m:091359",
    "m:560da1"
   ],
   "observable_changes": [
    "('FAILED', False, 0, False) -> ('PENDING', False, 0, False)",
    "('FAILED', False, 0, False) -> ('PENDING', False, 0, True)",
    "('PENDING', False, 0, True) -> ('PENDING', False, 0, False)"
   ],
   "part": 0
  },
  "a:work:0": {
   "api_call": "work",
   "moves": [
    "m:1b962d",
    "m:26ce19",
    "m:bb695c",
    "m:c22656"
   ],
   "observable_changes": [
    "('DONE', True, 1, False) -> ('DONE', True, 2, False)",
    "('DONE', True, 1, False) -> ('RUNNING', True, 1, False)",
    "('PENDING', False, 0, False) -> ('PENDING', False, 1, False)",
    "('PENDING', False, 0, False) -> ('RUNNING', False, 0, False)",
    "('RUNNING', False, 0, False) -> ('PENDING', False, 0, False)",
    "('RUNNING', False, 0, False) -> ('RUNNING', False, 1, False)",
    "('RUNNING', False, 1, False) -> ('RUNNING', False, 2, False)",
    "('RUNNING', True, 1, False) -> ('RUNNING', True, 2, False)"
   ],
   "part": 0
  },
  "a:work:1": {
   "api_call": "work",
   "moves": [
    "m:909260"
   ],
   "observable_changes": [
    "('PENDING', False, 1, False) -> ('DONE', True, 1, False)",
    "('RUNNING', False, 1, False) -> ('DONE', True, 1, False)",
    "('RUNNING', False, 2, False) -> ('DONE', True, 2, False)",
    "('RUNNING', True, 2, False) -> ('DONE', True, 2, False)"
   ],
   "part": 1
  }
 },
 "goals": {
  "g:create": {
   "activations": [
    "a:create:0"
   ],
   "api_call": "create",
   "observable_changes": [
    "('PENDING', False, 0, True) -> ('PENDING', False, 0, False)",
    "None -> ('PENDING', False, 0, True)"
   ],
   "returns": [
    "\"202 PENDING\"",
    "\"500 IntegrityError\""
   ]
  },
  "g:requeue": {
   "activations": [
    "a:requeue:0"
   ],
   "api_call": "requeue",
   "observable_changes": [
    "('DONE', True, 1, False) -> ('PENDING', True, 1, True)",
    "('PENDING', True, 1, True) -> ('PENDING', True, 1, False)",
    "('RUNNING', False, 0, False) -> ('PENDING', False, 0, False)",
    "('RUNNING', False, 1, False) -> ('PENDING', False, 1, False)"
   ],
   "returns": [
    "\"ok\""
   ]
  },
  "g:retry": {
   "activations": [
    "a:retry:0"
   ],
   "api_call": "retry",
   "observable_changes": [
    "('FAILED', False, 0, False) -> ('PENDING', False, 0, False)",
    "('FAILED', False, 0, False) -> ('PENDING', False, 0, True)",
    "('PENDING', False, 0, True) -> ('PENDING', False, 0, False)"
   ],
   "returns": [
    "\"requeued\""
   ]
  },
  "g:work": {
   "activations": [
    "a:work:0",
    "a:work:1"
   ],
   "api_call": "work",
   "observable_changes": [
    "('DONE', True, 1, False) -> ('DONE', True, 2, False)",
    "('DONE', True, 1, False) -> ('RUNNING', True, 1, False)",
    "('PENDING', False, 0, False) -> ('PENDING', False, 1, False)",
    "('PENDING', False, 0, False) -> ('RUNNING', False, 0, False)",
    "('PENDING', False, 1, False) -> ('DONE', True, 1, False)",
    "('RUNNING', False, 0, False) -> ('PENDING', False, 0, False)",
    "('RUNNING', False, 0, False) -> ('RUNNING', False, 1, False)",
    "('RUNNING', False, 1, False) -> ('DONE', True, 1, False)"
   ],
   "returns": [
    "\"empty\"",
    "\"ok\"",
    "\"retry\""
   ]
  }
 },
 "moves": {
  "m:091359": {
   "api_call": "retry",
   "commands": [
    [
     "SELECT",
     [
      "report:*"
     ]
    ],
    [
     "UPDATE",
     [
      "report:*",
      "reports:status"
     ]
    ]
   ],
   "function": "retry:23",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:status"
    ]
   }
  },
  "m:1b962d": {
   "api_call": "work",
   "commands": [
    [
     "UPDATE",
     [
      "report:*",
      "reports:status"
     ]
    ]
   ],
   "function": "generate_report:52",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:status"
    ]
   }
  },
  "m:1d90d1": {
   "api_call": "create",
   "commands": [
    [
     "SELECT",
     [
      "reports:key:*"
     ]
    ],
    [
     "INSERT",
     [
      "report:*",
      "reports:key:*",
      "reports:status"
     ]
    ]
   ],
   "function": "create_report:31",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:key:*",
     "reports:status"
    ]
   }
  },
  "m:26ce19": {
   "api_call": "work",
   "commands": [
    [
     "SELECT",
     [
      "orders"
     ]
    ],
    [
     "SELECT",
     [
      "orders"
     ]
    ],
    [
     "S3PUT",
     [
      "s3:reports/*.csv"
     ]
    ]
   ],
   "function": "generate_report:48",
   "result": "transaction rejected, nothing written"
  },
  "m:560da1": {
   "api_call": "retry",
   "commands": [
    [
     "SEND",
     [
      "broker:q"
     ]
    ]
   ],
   "function": "retry:24",
   "result": {
    "writes_key_families": [
     "broker:q"
    ]
   }
  },
  "m:662f00": {
   "api_call": "requeue",
   "commands": [
    [
     "SEND",
     [
      "broker:q"
     ]
    ]
   ],
   "function": "requeue_stuck:75",
   "result": {
    "writes_key_families": [
     "broker:q"
    ]
   }
  },
  "m:86e635": {
   "api_call": "create",
   "commands": [
    [
     "SEND",
     [
      "broker:q"
     ]
    ]
   ],
   "function": "create_report:33",
   "result": {
    "writes_key_families": [
     "broker:q"
    ]
   }
  },
  "m:909260": {
   "api_call": "work",
   "commands": [
    [
     "UPDATE",
     [
      "report:*",
      "reports:status"
     ]
    ]
   ],
   "function": "generate_report:59",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:status"
    ]
   }
  },
  "m:bb695c": {
   "api_call": "work",
   "commands": [
    [
     "SELECT",
     [
      "report:*"
     ]
    ],
    [
     "UPDATE",
     [
      "report:*",
      "reports:status"
     ]
    ]
   ],
   "function": "generate_report:32",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:status"
    ]
   }
  },
  "m:c22656": {
   "api_call": "work",
   "commands": [
    [
     "SELECT",
     [
      "orders"
     ]
    ],
    [
     "SELECT",
     [
      "orders"
     ]
    ],
    [
     "S3PUT",
     [
      "s3:reports/*.csv"
     ]
    ]
   ],
   "function": "generate_report:48",
   "result": {
    "writes_key_families": [
     "s3:reports/*.csv"
    ]
   }
  },
  "m:c2770e": {
   "api_call": "requeue",
   "commands": [
    [
     "SELECT",
     [
      "reports:status"
     ]
    ],
    [
     "UPDATE",
     [
      "report:*",
      "reports:status"
     ]
    ]
   ],
   "function": "requeue_stuck:73",
   "result": {
    "writes_key_families": [
     "report:*",
     "reports:status"
    ]
   }
  }
 }
}
```

## What the service does

# report-jobs

A small sample system and the first target for `mentalc`. It is about 150 lines of Python and contains four real bugs, listed in the [findings](../../docs/findings.md#s-the-hand-compiled-model-of-report-jobs).

| Part | File | What it does |
|---|---|---|
| API | [`src/app/api.py`](src/app/api.py) | `POST /reports` checks the date range, stores a `PENDING` row and queues a Celery task. `GET /reports/{id}` returns the status. |
| Workers (two replicas) | [`src/app/tasks.py`](src/app/tasks.py) | `generate_report` claims the row, builds a CSV from `orders`, uploads it to S3 and marks the row `DONE`, with up to three retries. `requeue_stuck` runs every five minutes and re-queues rows stuck in `RUNNING`. |
| Operator tool | [`src/app/cli.py`](src/app/cli.py) | `reportctl retry` re-queues a `FAILED` report. |
| Data | [`src/app/models.py`](src/app/models.py) | Tables `reports` (with the `status` column) and `orders`. |
| Deployment | [`src/docker-compose.yml`](src/docker-compose.yml) | `api`, `worker` ×2, `beat`, Redis and Postgres. |


