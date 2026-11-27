# report-jobs-java

The Java twin of [report-jobs](../report-jobs/): the same system, with the same four bugs, as a Spring Boot service of about 430 lines, imports included. The bugs are listed in the [findings](../../docs/findings.md#s-the-hand-compiled-model-of-report-jobs). It exists so that Mental's Java support has a reference to be built against ([plan](../../docs/plans/java-reference.md)).

| Part | File | What it does |
|---|---|---|
| Project | [`pom.xml`](pom.xml) | A Maven project named `reports` on Spring Boot 4.2 and Java 25: 6 direct dependencies, 130 resolved artifacts. |
| API | [`ReportController.java`](src/main/java/app/ReportController.java) | `POST /reports` checks the date range, stores a `PENDING` row and queues a job. `GET /reports/{reportId}` returns the status. |
| Queue | [`ReportQueue.java`](src/main/java/app/ReportQueue.java) | Sends a report's ID to the RabbitMQ queue `reports`, with its retry count in a header. |
| Workers (two replicas) | [`ReportTasks.java`](src/main/java/app/ReportTasks.java) | `generateReport` claims the row, builds a CSV from `orders`, uploads it to S3 and marks the row `DONE`, with up to three retries. |
| Scheduler | [`ReportBeat.java`](src/main/java/app/ReportBeat.java) | `requeueStuck` runs every five minutes and re-queues rows stuck in `RUNNING`. |
| Operator tool | [`ReportCtl.java`](src/main/java/app/ReportCtl.java) | `reportctl retry` re-queues a `FAILED` report. |
| Data | [`Report.java`](src/main/java/app/Report.java), [`Order.java`](src/main/java/app/Order.java) and their repositories | Tables `reports` (with the `status` column) and `orders`. |
| Deployment | [`src/docker-compose.yml`](src/docker-compose.yml) | `api`, `worker` ×2, `beat`, RabbitMQ and Postgres, each a profile of one jar. |
| Reference model | [`model/`](model/) | The hand-written records a build is compared against: the build unit, its 130 pins and 251 sites, and their anchors. Tests only read it. |

## Where it differs from report-jobs

- **The queue is RabbitMQ.** A listener acknowledges a message after `generateReport` returns, which makes delivery at-least-once, like Celery's `task_acks_late`. The duplicate claim needs exactly that.
- **A retry is queued again at once,** without Celery's 30-second delay.
- **`requeueStuck` runs in the `beat` process** rather than as a task on a worker.
- **Each `save` commits on its own,** as Spring Data's repository methods do outside a transaction, just as report-jobs commits after each step.

## Building it

`mvn dependency:go-offline` fills Maven's cache, much as `uv sync` makes report-jobs' environment. `mvn compile` builds it. The reference check runs Maven offline after that:

```sh
cd examples/report-jobs-java && mvn dependency:go-offline
cd ../../extract && RUN_SLOW_TESTS=1 cargo test --test java_reference
```

It needs a JDK 25 first on `PATH`. Nothing here is run by Mental yet.
