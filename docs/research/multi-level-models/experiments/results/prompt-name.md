You are naming the steps of a multi-level model of the Python job queue library rq.
The steps were found mechanically; do not regroup them. Three levels, from the bottom:

- move: a few Redis commands of one actor that together make one change (or one rejected attempt).
  Readers: developers working on concurrency. Mechanism words are fine (attempt, commit, claim);
  raw command names and key names are not.
- activation: one actor's uninterrupted part of an API call. Readers: developers.
- goal: one API call of one actor, seen as what that actor got done. Readers: product owners and
  support staff who know rq's user documentation and nothing of its storage. Use only words a
  user of rq would use.

For every step class give a name of at most five words, verb first, and one clause that starts
with "so that" and says what the step is for. Name moves first, then activations using your move
names, then goals. Different classes need different names. The same class must get the name you
would give it again tomorrow: prefer the plainest wording.

Answer with exactly one ```json block: {"moves": {id: {"name": ..., "so_that": ...}},
"activations": {...}, "goals": {...}} with every id from the sheet below.

An observable change is written as [status, places listing the job, type of latest result] before -> after; None means the job has no record.

## Sheet of step classes

```json
{
 "activations": {
  "a:cancel:0": {
   "api_call": "cancel",
   "moves": [
    "m:76056c",
    "m:be2344",
    "m:d39693",
    "m:daa541"
   ],
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['canceled', ['canceled', 'finished'], '1']",
    "['finished', ['finished'], '1'] -> ['canceled', ['canceled'], '1']",
    "['queued', ['queue'], None] -> ['canceled', ['canceled'], None]",
    "['queued', ['started'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['queued', [], None] -> ['canceled', ['canceled'], None]",
    "['started', ['started'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['started', ['started'], None] -> ['canceled', ['canceled'], None]"
   ],
   "part": 0
  },
  "a:cleanup:0": {
   "api_call": "cleanup",
   "moves": [
    "m:819711"
   ],
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['failed', ['failed', 'finished'], '2']",
    "['queued', ['started'], None] -> ['failed', ['failed'], '2']",
    "['started', ['started'], None] -> ['failed', ['failed'], '2']"
   ],
   "part": 0
  },
  "a:delete:0": {
   "api_call": "delete",
   "moves": [
    "m:1cdb1c",
    "m:3f68d8",
    "m:799b8b",
    "m:9a3f0c",
    "m:ec46fa",
    "m:feef01"
   ],
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['finished', [], '1']",
    "['finished', ['finished'], '1'] -> ['gone', ['finished'], '1']",
    "['finished', [], '1'] -> None",
    "['queued', ['queue'], None] -> ['queued', [], None]",
    "['queued', ['started'], None] -> ['queued', [], None]",
    "['queued', [], None] -> None",
    "['queued', [], None] -> ['gone', [], None]",
    "['started', ['started'], None] -> ['gone', ['started'], None]"
   ],
   "part": 0
  },
  "a:enqueue-dependent:0": {
   "api_call": "enqueue-dependent",
   "moves": [
    "m:183844",
    "m:509a00",
    "m:840bf6"
   ],
   "observable_changes": [
    "None -> ['deferred', ['deferred'], None]",
    "None -> ['queued', ['queue'], None]"
   ],
   "part": 0
  },
  "a:enqueue:0": {
   "api_call": "enqueue",
   "moves": [
    "m:257f55"
   ],
   "observable_changes": [
    "None -> ['queued', ['queue'], None]"
   ],
   "part": 0
  },
  "a:requeue:0": {
   "api_call": "requeue",
   "moves": [
    "m:10aebe",
    "m:a61c56",
    "m:f72349"
   ],
   "observable_changes": [
    "['failed', ['failed'], '2'] -> ['failed', [], '2']",
    "['failed', [], '2'] -> ['queued', ['queue'], '2']"
   ],
   "part": 0
  },
  "a:work:0": {
   "api_call": "work",
   "moves": [
    "m:34e4a3",
    "m:85ae07",
    "m:8e69df",
    "m:b7706e"
   ],
   "observable_changes": [
    "['canceled', ['canceled', 'started'], None] -> ['started', ['canceled', 'started'], None]",
    "['canceled', ['canceled'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['failed', ['failed'], '2'] -> ['started', ['failed'], '2']",
    "['gone', ['started'], None] -> ['started', ['started'], None]",
    "['gone', [], None] -> ['gone', ['started'], None]",
    "['gone', [], None] -> ['started', [], None]",
    "['queued', ['queue'], '2'] -> ['queued', [], '2']",
    "['queued', ['queue'], None] -> ['queued', [], None]"
   ],
   "part": 0
  },
  "a:work:1": {
   "api_call": "work",
   "moves": [
    "m:02365a",
    "m:0fccaa",
    "m:1ecc1d",
    "m:30455a",
    "m:4ecdcc",
    "m:6ed7ed",
    "m:ca9dee"
   ],
   "observable_changes": [
    "None -> ['finished', ['finished'], '1']",
    "None -> [None, [], None]",
    "['canceled', ['canceled', 'started'], None] -> ['finished', ['canceled', 'finished'], '1']",
    "['canceled', ['canceled'], None] -> ['finished', ['canceled', 'finished'], '1']",
    "['deferred', ['deferred'], None] -> ['queued', ['queue'], None]",
    "['failed', ['failed'], '2'] -> ['finished', ['failed', 'finished'], '1']",
    "['gone', ['started'], None] -> [None, ['started'], None]",
    "['started', ['canceled', 'started'], None] -> ['finished', ['canceled', 'finished'], '1']"
   ],
   "part": 1
  }
 },
 "goals": {
  "g:cancel": {
   "activations": [
    "a:cancel:0"
   ],
   "api_call": "cancel",
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['canceled', ['canceled', 'finished'], '1']",
    "['finished', ['finished'], '1'] -> ['canceled', ['canceled'], '1']",
    "['queued', ['queue'], None] -> ['canceled', ['canceled'], None]",
    "['queued', ['started'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['queued', [], None] -> ['canceled', ['canceled'], None]",
    "['started', ['started'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['started', ['started'], None] -> ['canceled', ['canceled'], None]"
   ],
   "returns": [
    "[\"exc\", \"ValueError\"]",
    "[\"ok\", \"canceled\"]"
   ]
  },
  "g:cleanup": {
   "activations": [
    "a:cleanup:0"
   ],
   "api_call": "cleanup",
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['failed', ['failed', 'finished'], '2']",
    "['queued', ['started'], None] -> ['failed', ['failed'], '2']",
    "['started', ['started'], None] -> ['failed', ['failed'], '2']"
   ],
   "returns": [
    "[\"ok\", \"cleaned\"]"
   ]
  },
  "g:delete": {
   "activations": [
    "a:delete:0"
   ],
   "api_call": "delete",
   "observable_changes": [
    "['finished', ['finished'], '1'] -> ['finished', [], '1']",
    "['finished', ['finished'], '1'] -> ['gone', ['finished'], '1']",
    "['finished', [], '1'] -> None",
    "['queued', ['queue'], None] -> ['queued', [], None]",
    "['queued', ['started'], None] -> ['queued', [], None]",
    "['queued', [], None] -> None",
    "['queued', [], None] -> ['gone', [], None]",
    "['started', ['started'], None] -> ['gone', ['started'], None]"
   ],
   "returns": [
    "[\"exc\", \"ValueError\"]",
    "[\"ok\", \"deleted\"]"
   ]
  },
  "g:enqueue": {
   "activations": [
    "a:enqueue:0"
   ],
   "api_call": "enqueue",
   "observable_changes": [
    "None -> ['queued', ['queue'], None]"
   ],
   "returns": [
    "[\"ok\", \"j1\"]"
   ]
  },
  "g:enqueue-dependent": {
   "activations": [
    "a:enqueue-dependent:0"
   ],
   "api_call": "enqueue-dependent",
   "observable_changes": [
    "None -> ['deferred', ['deferred'], None]",
    "None -> ['queued', ['queue'], None]"
   ],
   "returns": [
    "[\"ok\", \"j2\"]"
   ]
  },
  "g:requeue": {
   "activations": [
    "a:requeue:0"
   ],
   "api_call": "requeue",
   "observable_changes": [
    "['failed', ['failed'], '2'] -> ['failed', [], '2']",
    "['failed', [], '2'] -> ['queued', ['queue'], '2']"
   ],
   "returns": [
    "[\"ok\", \"requeued\"]"
   ]
  },
  "g:work": {
   "activations": [
    "a:work:0",
    "a:work:1"
   ],
   "api_call": "work",
   "observable_changes": [
    "None -> ['finished', ['finished'], '1']",
    "None -> [None, [], None]",
    "['canceled', ['canceled', 'started'], None] -> ['finished', ['canceled', 'finished'], '1']",
    "['canceled', ['canceled', 'started'], None] -> ['started', ['canceled', 'started'], None]",
    "['canceled', ['canceled'], None] -> ['canceled', ['canceled', 'started'], None]",
    "['canceled', ['canceled'], None] -> ['finished', ['canceled', 'finished'], '1']",
    "['deferred', ['deferred'], None] -> ['queued', ['queue'], None]",
    "['failed', ['failed'], '2'] -> ['finished', ['failed', 'finished'], '1']"
   ],
   "returns": [
    "[\"ok\", \"idle\"]",
    "[\"ok\", \"worked\"]"
   ]
  }
 },
 "moves": {
  "m:02365a": {
   "api_call": "work",
   "commands": [
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ]
   ],
   "function": "Job.heartbeat",
   "result": {
    "writes_key_families": [
     "rq:job:*"
    ]
   }
  },
  "m:0fccaa": {
   "api_call": "work",
   "commands": [
    [
     "WATCH",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HINCRBY",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HINCRBYFLOAT",
     [
      "rq:worker:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_success",
   "result": {
    "writes_key_families": [
     "rq:deferred:*",
     "rq:finished:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies",
     "rq:queue:*",
     "rq:results:*",
     "rq:worker:*"
    ]
   }
  },
  "m:10aebe": {
   "api_call": "requeue",
   "commands": [
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ]
   ],
   "function": "Job.save",
   "result": {
    "writes_key_families": [
     "rq:job:*"
    ]
   }
  },
  "m:183844": {
   "api_call": "enqueue-dependent",
   "commands": [
    [
     "WATCH",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ]
   ],
   "function": "Queue._enqueue_async_job",
   "result": {
    "writes_key_families": [
     "rq:job:*",
     "rq:queue:*"
    ]
   }
  },
  "m:1cdb1c": {
   "api_call": "delete",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:finished:*"
     ]
    ]
   ],
   "function": "BaseRegistry.remove",
   "result": {
    "writes_key_families": [
     "rq:finished:*"
    ]
   }
  },
  "m:1ecc1d": {
   "api_call": "work",
   "commands": [
    [
     "HDEL",
     [
      "rq:worker:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:wip:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:executions:*"
     ]
    ],
    [
     "HINCRBY",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HINCRBYFLOAT",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_failure",
   "result": {
    "writes_key_families": [
     "rq:execution:*:*",
     "rq:executions:*",
     "rq:job:*",
     "rq:queue:*",
     "rq:wip:*",
     "rq:worker:*"
    ]
   }
  },
  "m:257f55": {
   "api_call": "enqueue",
   "commands": [
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ]
   ],
   "function": "Queue._enqueue_async_job",
   "result": {
    "writes_key_families": [
     "rq:job:*",
     "rq:queue:*"
    ]
   }
  },
  "m:30455a": {
   "api_call": "work",
   "commands": [
    [
     "WATCH",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HINCRBY",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HINCRBYFLOAT",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "XADD",
     [
      "rq:results:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:results:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:finished:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HDEL",
     [
      "rq:worker:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:execution:*:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_success",
   "result": "transaction rejected, nothing written"
  },
  "m:34e4a3": {
   "api_call": "work",
   "commands": [
    [
     "LMOVE",
     [
      "rq:queue:*",
      "rq:queue:*:intermediate"
     ]
    ]
   ],
   "function": "Queue.lmove",
   "result": {
    "writes_key_families": [
     "rq:queue:*",
     "rq:queue:*:intermediate"
    ]
   }
  },
  "m:3f68d8": {
   "api_call": "delete",
   "commands": [
    [
     "ZRANGE",
     [
      "rq:executions:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:wip:*"
     ]
    ]
   ],
   "function": "StartedJobRegistry.remove_executions",
   "result": {
    "writes_key_families": [
     "rq:wip:*"
    ]
   }
  },
  "m:4ecdcc": {
   "api_call": "work",
   "commands": [
    [
     "WATCH",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HINCRBY",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HINCRBYFLOAT",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "XADD",
     [
      "rq:results:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:results:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:finished:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HDEL",
     [
      "rq:worker:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:execution:*:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_success",
   "result": {
    "writes_key_families": [
     "rq:execution:*:*",
     "rq:executions:*",
     "rq:finished:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies",
     "rq:results:*",
     "rq:wip:*",
     "rq:worker:*"
    ]
   }
  },
  "m:509a00": {
   "api_call": "enqueue-dependent",
   "commands": [
    [
     "WATCH",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "SADD",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SADD",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ]
   ],
   "function": "Queue.setup_dependencies",
   "result": {
    "writes_key_families": [
     "rq:deferred:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies"
    ]
   }
  },
  "m:6ed7ed": {
   "api_call": "work",
   "commands": [
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HDEL",
     [
      "rq:worker:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:wip:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:executions:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "ZADD",
     [
      "rq:failed:*"
     ]
    ],
    [
     "XADD",
     [
      "rq:results:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_failure",
   "result": {
    "writes_key_families": [
     "rq:execution:*:*",
     "rq:executions:*",
     "rq:failed:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies",
     "rq:results:*",
     "rq:wip:*",
     "rq:worker:*"
    ]
   }
  },
  "m:76056c": {
   "api_call": "cancel",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "LREM",
     [
      "rq:queue:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:finished:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:canceled:*"
     ]
    ]
   ],
   "function": "Job.cancel",
   "result": {
    "writes_key_families": [
     "rq:canceled:*",
     "rq:finished:*",
     "rq:job:*",
     "rq:queue:*"
    ]
   }
  },
  "m:799b8b": {
   "api_call": "delete",
   "commands": [
    [
     "ZRANGE",
     [
      "rq:executions:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:job:*",
      "rq:job:*:dependents",
      "rq:job::*:dependencies"
     ]
    ]
   ],
   "function": "Job.delete",
   "result": {
    "writes_key_families": [
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies"
    ]
   }
  },
  "m:819711": {
   "api_call": "cleanup",
   "commands": [
    [
     "ZRANGEBYSCORE",
     [
      "rq:wip:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "ZADD",
     [
      "rq:failed:*"
     ]
    ],
    [
     "XADD",
     [
      "rq:results:*"
     ]
    ],
    [
     "ZREMRANGEBYSCORE",
     [
      "rq:wip:*"
     ]
    ]
   ],
   "function": "StartedJobRegistry.cleanup",
   "result": {
    "writes_key_families": [
     "rq:failed:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies",
     "rq:results:*",
     "rq:wip:*"
    ]
   }
  },
  "m:840bf6": {
   "api_call": "enqueue-dependent",
   "commands": [
    [
     "WATCH",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "SADD",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SADD",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ]
   ],
   "function": "Queue.setup_dependencies",
   "result": "transaction rejected, nothing written"
  },
  "m:85ae07": {
   "api_call": "work",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:executions:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:executions:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:wip:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:worker:*"
     ]
    ]
   ],
   "function": "prepare_execution",
   "result": {
    "writes_key_families": [
     "rq:execution:*:*",
     "rq:executions:*",
     "rq:wip:*",
     "rq:worker:*"
    ]
   }
  },
  "m:8e69df": {
   "api_call": "work",
   "commands": [
    [
     "PERSIST",
     [
      "rq:job:*"
     ]
    ]
   ],
   "function": "Job.perform",
   "result": {
    "writes_key_families": [
     "rq:job:*"
    ]
   }
  },
  "m:9a3f0c": {
   "api_call": "delete",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "LREM",
     [
      "rq:queue:*"
     ]
    ]
   ],
   "function": "Queue.remove",
   "result": {
    "writes_key_families": [
     "rq:queue:*"
    ]
   }
  },
  "m:a61c56": {
   "api_call": "requeue",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:failed:*"
     ]
    ]
   ],
   "function": "BaseRegistry.requeue",
   "result": {
    "writes_key_families": [
     "rq:failed:*"
    ]
   }
  },
  "m:b7706e": {
   "api_call": "work",
   "commands": [
    [
     "HSET",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:worker:*"
     ]
    ],
    [
     "EXPIRE",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "LREM",
     [
      "rq:queue:*:intermediate"
     ]
    ]
   ],
   "function": "BaseWorker.prepare_job_execution",
   "result": {
    "writes_key_families": [
     "rq:job:*",
     "rq:queue:*:intermediate",
     "rq:worker:*"
    ]
   }
  },
  "m:be2344": {
   "api_call": "cancel",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "LREM",
     [
      "rq:queue:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:canceled:*"
     ]
    ]
   ],
   "function": "Job.cancel",
   "result": {
    "writes_key_families": [
     "rq:canceled:*",
     "rq:job:*",
     "rq:queue:*"
    ]
   }
  },
  "m:ca9dee": {
   "api_call": "work",
   "commands": [
    [
     "WATCH",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "WATCH",
     [
      "rq:job:*"
     ]
    ],
    [
     "SMEMBERS",
     [
      "rq:job::*:dependencies"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:deferred:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ],
    [
     "DEL",
     [
      "rq:job:*:dependents"
     ]
    ],
    [
     "HINCRBY",
     [
      "rq:worker:*"
     ]
    ],
    [
     "HINCRBYFLOAT",
     [
      "rq:worker:*"
     ]
    ]
   ],
   "function": "BaseWorker.handle_job_success",
   "result": {
    "writes_key_families": [
     "rq:deferred:*",
     "rq:execution:*:*",
     "rq:executions:*",
     "rq:finished:*",
     "rq:job:*",
     "rq:job:*:dependents",
     "rq:job::*:dependencies",
     "rq:queue:*",
     "rq:results:*",
     "rq:wip:*",
     "rq:worker:*"
    ]
   }
  },
  "m:d39693": {
   "api_call": "cancel",
   "commands": [
    [
     "ZRANGE",
     [
      "rq:executions:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "LREM",
     [
      "rq:queue:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:wip:*"
     ]
    ],
    [
     "ZADD",
     [
      "rq:canceled:*"
     ]
    ]
   ],
   "function": "Job.cancel",
   "result": {
    "writes_key_families": [
     "rq:canceled:*",
     "rq:job:*",
     "rq:queue:*",
     "rq:wip:*"
    ]
   }
  },
  "m:daa541": {
   "api_call": "cancel",
   "commands": [
    [
     "HGETALL",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREMRANGEBYSCORE",
     [
      "rq:executions:*"
     ]
    ]
   ],
   "function": "ExecutionRegistry.cleanup",
   "result": {
    "writes_key_families": [
     "rq:executions:*"
    ]
   }
  },
  "m:ec46fa": {
   "api_call": "delete",
   "commands": [
    [
     "ZRANGE",
     [
      "rq:executions:*"
     ]
    ],
    [
     "HGETALL",
     [
      "rq:execution:*:*"
     ]
    ],
    [
     "ZREM",
     [
      "rq:wip:*"
     ]
    ]
   ],
   "function": "StartedJobRegistry.remove_execution",
   "result": {
    "writes_key_families": [
     "rq:wip:*"
    ]
   }
  },
  "m:f72349": {
   "api_call": "requeue",
   "commands": [
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HSET",
     [
      "rq:job:*"
     ]
    ],
    [
     "RPUSH",
     [
      "rq:queue:*"
     ]
    ]
   ],
   "function": "BaseRegistry.requeue",
   "result": {
    "writes_key_families": [
     "rq:job:*",
     "rq:queue:*"
    ]
   }
  },
  "m:feef01": {
   "api_call": "delete",
   "commands": [
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "HGET",
     [
      "rq:job:*"
     ]
    ],
    [
     "ZREMRANGEBYSCORE",
     [
      "rq:executions:*"
     ]
    ]
   ],
   "function": "ExecutionRegistry.cleanup",
   "result": {
    "writes_key_families": [
     "rq:executions:*"
    ]
   }
  }
 }
}
```

## rq user documentation

### docs/docs/jobs.md
---
title: "RQ: Jobs"
layout: docs
---

For some use cases it might be useful have access to the current job ID or
instance from within the job function itself.  Or to store arbitrary data on
jobs.


## RQ's Job Object

### The Job Lifecycle

The life-cycle of a worker consists of a few phases:

1. _Queued_. When `queue.enqueue(foo)` is called, a `Job` will be created and it's ID
pushed into the queue. `job.get_status()` will return `queued`.
2. _Started_. When a worker picks up a job from queue, the job status will be set to `started`.
In this phase an `Execution` object will be created and it's `composite_key` put in `StartedJobRegistry`.
3. _Finished_. After an execution has ended, `execution` will be removed from `StartedJobRegistry`.
A `Result` object that holds the result of the execution will be created. Both the `Job` and `Result` key
will persist in Redis until the value of `result_ttl` is up. More details [here](/docs/results/).


#### Job Status

The status of a job can be one of the following:

* `queued`: The default status for created jobs, except for those that have dependencies, which will be created as `deferred`. These jobs have been placed in a queue and are ready to be executed.
* `finished`: The job has finished execution and is available through the finished job registry.
* `failed`: Jobs that encountered errors during execution or expired before being executed.
* `started`: The job has started execution. This status includes the job execution support mechanisms, such as setting the worker name and setting up heartbeat information.
* `deferred`: The job is not ready for execution because its dependencies have not finished successfully yet.
* `scheduled`: Jobs created to run at a future date or jobs that are retried after a retry interval.
* `stopped`: The job was stopped because the worker was stopped.
* `canceled`: The job has been manually canceled and will not be executed, even if it is part of a dependency chain.

These statuses can also be accessed from the job object using boolean properties, such as `job.is_finished`.


### Job Creation

When you enqueue a function, a job will be returned.  You may then access the
id property, which can later be used to retrieve the job.

```python
from rq import Queue
from redis import Redis
from somewhere import count_words_at_url

redis_conn = Redis()
q = Queue(connection=redis_conn)  # no args implies the default queue

# Delay execution of count_words_at_url('http://nvie.com')
job = q.enqueue(count_words_at_url, 'http://nvie.com')
print('Job id: %s' % job.id)
```

Or if you want a predetermined job id, you may specify it when creating the job.
Custom job ids may contain only letters, numbers, underscores and dashes.

```python
job = q.enqueue(count_words_at_url, 'http://nvie.com', job_id='my_job_id')
```

A job can also be created directly with `Job.create()`.

```python
from rq.job import Job

job = Job.create(count_words_at_url, 'http://nvie.com')
print('Job id: %s' % job.id)
q.enqueue_job(job)

# create a job with a predetermined id
job = Job.create(count_words_at url, 'http://nvie.com', id='my_job_id')
```

The keyword arguments accepted by `create()` are:

* `timeout` specifies the maximum runtime of the job before it's interrupted
  and marked as `failed`. Its default unit is seconds and it can be an integer
  or a string representing an integer(e.g.  `2`, `'2'`). Furthermore, it can
  be a string with specify unit including hour, minute, second
  (e.g. `'1h'`, `'3m'`, `'5s'`).
* `result_ttl` specifies how long (in seconds) successful jobs and their
  results are kept. Expired jobs will be automatically deleted. Defaults to 500 seconds.
* `ttl` specifies the maximum queued time (in seconds) of the job before it's discarded.
  This argument defaults to `None` (infinite TTL).
* `failure_ttl` specifies how long (in seconds) failed jobs are kept (defaults to 1 year)
* `depends_on` specifies another job (or job id) that must complete before this
  job will be queued.
* `id` allows you to manually specify this job's id
* `description` to add additional description to the job
* `connection`
* `status`
* `origin` where this job was originally enqueued
* `meta` a dictionary holding custom status information on this job
* `args` and `kwargs`: use these to explicitly pass arguments and keyword to the
  underlying job function. This is useful if your function happens to have
  conflicting argument names with RQ, for example `description` or `ttl`.

In the last case, if you want to pass `description` and `ttl` keyword arguments
to your job and not to RQ's enqueue function, this is what you do:

```python
job = Job.create(count_words_at_url,
          ttl=30,  # This ttl will be used by RQ
          args=('http://nvie.com',),
          kwargs={
              'description': 'Function description', # This is passed on to count_words_at_url
              'ttl': 15  # This is passed on to count_words_at_url function
          })
```

### Retrieving Jobs

All job information is stored in Redis. You can inspect a job and its attributes
by using `Job.fetch()`.

```python
from redis import Redis
from rq.job import Job

redis = Redis()
job = Job.fetch('my_job_id', connection=redis)
print('Status: %s' % job.get_status())
```

Some interesting job attributes include:
* `job.get_status(refresh=True)` Possible values are `queued`, `started`,
  `deferred`, `finished`, `stopped`, `scheduled`, `canceled` and `failed`. If `refresh` is
  `True` fresh values are fetched from Redis.
* `job.get_meta(refresh=True)` Returns custom `job.meta` dict containing user
  stored data. If `refresh` is `True` fresh values are fetched from Redis.
* `job.origin` queue name of this job
* `job.func_name`
* `job.args` arguments passed to the underlying job function
* `job.kwargs` key word arguments passed to the underlying job function
* `job.result` stores the return value of the job being executed, will return `None` prior to job execution. Results are kept according to the `result_ttl` parameter (500 seconds by default).
* `job.enqueued_at`
* `job.started_at`
* `job.ended_at`
* `job.exc_info` stores exception information if job doesn't finish successfully.
* `job.last_heartbeat` the latest timestamp that's periodically updated when the job is executing. Can be used to determine if the job is still active.
* `job.worker_name` returns the worker name currently executing this job.
* `job.refresh()` Update the job instance object with values fetched from Redis.

If you want to efficiently fetch a large number of jobs, use `Job.fetch_many()`.

```python
jobs = Job.fetch_many(['foo_id', 'bar_id'], connection=redis)
for job in jobs:
    print('Job %s: %s' % (job.id, job.func_name))
```


## Job Executions

_New in 2.0_

When a job is being executed, RQ stores it's execution data in Redis. You can access this data
via `Execution` objects.

```python
from redis import Redis
from rq.job import Job

redis = Redis()
job = Job.fetch('my_job_id', connection=redis)
executions = job.get_executions()  # Returns all current executions
execution = job.get_executions()[0]  # Retrieves a single execution
print(execution.created_at)  # When did this execution start?
print(execution.last_heartbeat)  # Worker's last heartbeat
```

`Execution` objects have a few properties:
* `id`: ID of an execution.
* `job`: the `Job` object that owns this execution instance
* `composite_key`: a combination of `job.id` and `execution.id`, formatted as `<job_id>:<execution_id>`
* `created_at`: returns a datetime object representing the start of this execution
* `last_heartbeat`: worker's last heartbeat


## Stopping a Currently Executing Job
_New in version 1.7.0_

You can use `send_stop_job_command()` to tell a worker to immediately stop a currently executing job. A job that's stopped will be sent to [FailedJobRegistry](/docs/results/#dealing-with-exceptions).

```python
from redis import Redis
from rq.command import send_stop_job_command

redis = Redis()

# This will raise an exception if job is invalid or not currently executing
send_stop_job_command(redis, job_id)
```

Unlike failed jobs, stopped jobs will *not* be automatically retried if retry is configured. Subclasses of `Worker` which override `handle_job_failure()` should likewise take care to handle jobs with a `stopped` status appropriately.

## Canceling a Job
_New in version 1.10.0_

To prevent a job from running, cancel a job, use `job.cancel()`.

```python
from redis import Redis
from rq.job import Job
from rq.registry import CanceledJobRegistry
from .queue import Queue

redis = Redis()
job = Job.fetch('my_job_id', connection=redis)
job.cancel()

job.get_status()  # Job status is CANCELED

registry = CanceledJobRegistry(job.origin, connection=job.connection)
print(job in registry)  # Job is in CanceledJobRegistry
```

Canceling a job will remove:
1. Sets job status to `CANCELED`
2. Removes job from queue
3. Puts job into `CanceledJobRegistry`

Note that `job.cancel()` does **not** delete the job itself from Redis. If you want to
delete the job from Redis and reclaim memory, use `job.delete()`.

Note: if you want to enqueue the dependents of the job you
are trying to cancel use the following:

```python
from rq import cancel_job
cancel_job(
  '2eafc1e6-48c2-464b-a0ff-88fd199d039c',
  enqueue_dependents=True
)
```

## Job / Queue Creation with Custom Serializer

When creating a job or queue, you can pass in a custom serializer that will be used for serializing / de-serializing job arguments. Serializers must implement `loads` and `dumps`. The default serializer is `pickle`.

> **Warning:** RQ uses [`pickle`](https://docs.python.org/3/library/pickle.html#module-pickle) as its default serializer, which **is not secure**. Only run RQ against Redis instances that you trust. It is possible to construct malicious pickle data that will execute arbitrary code during unpickling.

To avoid pickle, use an alternative serializer, such as `JSONSerializer`, when enqueueing and processing jobs. JSON only supports primitive argument types (str, int, float, bool, list, dict, None).

For example, to enqueue jobs with the JSON serializer, pass either the `'json'` shorthand or the `JSONSerializer` class itself:

```python
from rq import Queue
from rq.serializers import JSONSerializer

queue = Queue(connection=connection, serializer='json')  # or: serializer=JSONSerializer
job = queue.enqueue('my_module.count_words', 'https://example.com')
```

Then run workers with the same serializer. The `--serializer` option accepts the shorthands `json` and `pickle`, or a full dotted path such as `rq.serializers.JSONSerializer`:

```console
$ rq worker --serializer json
$ rq worker --serializer rq.serializers.JSONSerializer
```

## Accessing The "current" Job from within the job function

Since job functions are regular Python functions, you must retrieve the
job in order to inspect or update the job's attributes.  To do this from within
the function, you can use:

```python
from rq import get_current_job

def add(x, y):
    job = get_current_job()
    print('Current job: %s' % (job.id,))
    return x + y
```

Note that calling get_current_job() outside of the context of a job function will return `None`.


## Storing arbitrary data on jobs

_Improved in 0.8.0._

To add/update custom status information on this job, you have access to the
`meta` property, which allows you to store arbitrary pickleable data on the job
itself:

```python
import socket

def add(x, y):
    job = get_current_job()
    job.meta['handled_by'] = socket.gethostname()
    job.save_meta()

    # do more work
    time.sleep(1)
    return x + y
```


## Time to live for job in queue

A job has two TTLs, one for the job result, `result_ttl`, and one for the job itself, `ttl`.
The latter is used if you have a job that shouldn't be executed after a certain amount of time.

```python
# When creating the job:
job = Job.create(func=say_hello,
                 result_ttl=600,  # how long (in seconds) to keep the job (if successful) and its results
                 ttl=43,  # maximum queued time (in seconds) of the job before it's discarded.
                )

# or when queueing a new job:
job = q.enqueue(count_words_at_url,
                'http://nvie.com',
                result_ttl=600,  # how long to keep the job (if successful) and its results
                ttl=43  # maximum queued time
               )
```

## Job Position in Queue

For user feedback or debuging it is possible to get the position of a job
within the work queue. This allows to track the job processing through the
queue.

This function iterates over all jobs within the queue and therefore does
perform poorly on very large job queues.

```python
from rq import Queue
from redis import Redis
from hello import say_hello

redis_conn = Redis()
q = Queue(connection=redis_conn)

job = q.enqueue(say_hello)
job2 = q.enqueue(say_hello)

job2.get_position()
# returns 1

q.get_job_position(job)
# return 0
```

## Failed Jobs

If a job fails during execution, the worker will put the job in a FailedJobRegistry.
On the Job instance, the `is_failed` property will be true. FailedJobRegistry
can be accessed through `queue.failed_job_registry`.

```python
from redis import Redis
from rq import Queue
from rq.job import Job


def div_by_zero(x):
    return x / 0


connection = Redis()
queue = Queue(connection=connection)
job = queue.enqueue(div_by_zero, 1)
registry = queue.failed_job_registry

worker = Worker([queue])
worker.work(burst=True)

assert len(registry) == 1  # Failed jobs are kept in FailedJobRegistry
```

By default, failed jobs are kept for 1 year. You can change this by specifying
`failure_ttl` (in seconds) when enqueueing jobs.

```python
job = queue.enqueue(foo_job, failure_ttl=300)  # 5 minutes in seconds
```


### Requeuing Failed Jobs

If you need to manually requeue failed jobs, here's how to do it:

```python
from redis import Redis
from rq import Queue

connection = Redis()
queue = Queue(connection=connection)
registry = queue.failed_job_registry

# This is how to get jobs from FailedJobRegistry
for job_id in registry.get_job_ids():
    registry.requeue(job_id)  # Puts job back in its original queue

assert len(registry) == 0  # Registry will be empty when job is requeued
```

Starting from version 1.5.0, RQ also allows you to [automatically retry
failed jobs](/docs/exceptions/#retrying-failed-jobs).


### Requeuing Failed Jobs via CLI

RQ also provides a CLI tool that makes requeuing failed jobs easy.

```console
# This will requeue foo_job_id and bar_job_id from myqueue's failed job registry
rq requeue --queue myqueue -u redis://localhost:6379 foo_job_id bar_job_id

# This command will requeue all jobs in myqueue's failed job registry
rq requeue --queue myqueue -u redis://localhost:6379 --all
```


### docs/docs/job_registries.md
---
title: "RQ: Job Registries"
layout: docs
---

Each queue maintains a set of Job Registries:
* `StartedJobRegistry` Holds currently executing jobs. Jobs are added right before they are
executed and removed right after completion (success or failure).
* `FinishedJobRegistry` Holds successfully completed jobs.
* `FailedJobRegistry` Holds jobs that have been executed, but didn't finish successfully.
* `DeferredJobRegistry` Holds deferred jobs (jobs that depend on another job and are waiting for that
job to finish).
* `ScheduledJobRegistry` Holds scheduled jobs.
* `CanceledJobRegistry` Holds canceled jobs.

You can get the number of jobs in a registry, the ids of the jobs in the registry, and more.
Below is an example using a `StartedJobRegistry`.
```python
import time
from redis import Redis
from rq import Queue
from rq.registry import StartedJobRegistry
from somewhere import count_words_at_url

redis = Redis()
queue = Queue(connection=redis)
job = queue.enqueue(count_words_at_url, 'http://nvie.com')

# get StartedJobRegistry by queue
registry = StartedJobRegistry(queue=queue)

# or get StartedJobRegistry by queue name and connection
registry2 = StartedJobRegistry(name='my_queue', connection=redis)

# sleep for a moment while job is taken off the queue
time.sleep(0.1)

print('Queue associated with the registry: %s' % registry.get_queue())
print('Number of jobs in registry %s' % registry.count)

# get the list of ids for the jobs in the registry
print('IDs in registry %s' % registry.get_job_ids())

# test if a job is in the registry using the job instance or job id
print('Job in registry %s' % (job in registry))
print('Job in registry %s' % (job.id in registry))
```

_New in version 1.2.0_

You can quickly access job registries from `Queue` objects.

```python
from redis import Redis
from rq import Queue

redis = Redis()
queue = Queue(connection=redis)

queue.started_job_registry  # Returns StartedJobRegistry
queue.deferred_job_registry   # Returns DeferredJobRegistry
queue.finished_job_registry  # Returns FinishedJobRegistry
queue.failed_job_registry  # Returns FailedJobRegistry
queue.scheduled_job_registry  # Returns ScheduledJobRegistry
```

## Removing Jobs

_New in version 1.2.0_

To remove a job from a job registry, use `registry.remove()`. This is useful
when you want to manually remove jobs from a registry, such as deleting failed
jobs before they expire from `FailedJobRegistry`.

```python
from redis import Redis
from rq import Queue
from rq.registry import FailedJobRegistry

redis = Redis()
queue = Queue(connection=redis)
registry = FailedJobRegistry(queue=queue)

# This is how to remove a job from a registry
for job_id in registry.get_job_ids():
    registry.remove(job_id)

# If you want to remove a job from a registry AND delete the job,
# use `delete_job=True`
for job_id in registry.get_job_ids():
    registry.remove(job_id, delete_job=True)
```

