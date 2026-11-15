# report-jobs

A small sample system and the first target for `mentalc`. It is about 150 lines of Python and contains four real bugs, listed in the [findings](../../docs/findings.md#s-the-hand-compiled-model-of-report-jobs).

| Part | File | What it does |
|---|---|---|
| API | [`src/app/api.py`](src/app/api.py) | `POST /reports` checks the date range, stores a `PENDING` row and queues a Celery task. `GET /reports/{id}` returns the status. |
| Workers (two replicas) | [`src/app/tasks.py`](src/app/tasks.py) | `generate_report` claims the row, builds a CSV from `orders`, uploads it to S3 and marks the row `DONE`, with up to three retries. `requeue_stuck` runs every five minutes and re-queues rows stuck in `RUNNING`. |
| Operator tool | [`src/app/cli.py`](src/app/cli.py) | `reportctl retry` re-queues a `FAILED` report. |
| Data | [`src/app/models.py`](src/app/models.py) | Tables `reports` (with the `status` column) and `orders`. |
| Deployment | [`src/docker-compose.yml`](src/docker-compose.yml) | `api`, `worker` ×2, `beat`, Redis and Postgres. |

Nothing here is compiled yet. The [specification](../../docs/specification.md) uses this system for its examples and its acceptance tests.
