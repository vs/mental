import csv
import io
import os
from datetime import datetime, timedelta

import boto3
import requests
from botocore.exceptions import ClientError
from celery import Celery

from app.db import Session
from app.models import Order, Report, ReportStatus

celery = Celery("reports", broker=os.environ["CELERY_BROKER_URL"])
celery.conf.task_acks_late = True
celery.conf.beat_schedule = {
    "requeue-stuck": {"task": "app.tasks.requeue_stuck", "schedule": 300.0},
}

s3 = boto3.client("s3")
BUCKET = os.environ["REPORTS_BUCKET"]
WEBHOOK_URL = os.environ.get("WEBHOOK_URL")


@celery.task(bind=True, max_retries=3, default_retry_delay=30)
def generate_report(self, report_id: str):
    with Session() as s:
        report = s.get(Report, report_id)
        if report is None or report.status != ReportStatus.PENDING:
            return
        report.status = ReportStatus.RUNNING
        s.commit()

        buf = io.StringIO()
        writer = csv.writer(buf)
        page = 0
        while True:
            orders = (s.query(Order)
                      .filter(Order.created.between(report.start, report.end))
                      .order_by(Order.id).offset(page * 1000).limit(1000).all())
            if not orders:
                break
            writer.writerows((o.id, o.created, o.total_cents) for o in orders)
            page += 1

        key = f"reports/{report.id}.csv"
        try:
            s3.put_object(Bucket=BUCKET, Key=key, Body=buf.getvalue())
        except ClientError as exc:
            report.status = (ReportStatus.FAILED if self.request.retries >= self.max_retries
                             else ReportStatus.PENDING)
            s.commit()
            if report.status == ReportStatus.PENDING:
                raise self.retry(exc=exc)
            return

        report.status = ReportStatus.DONE
        report.url = f"s3://{BUCKET}/{key}"
        s.commit()

    if WEBHOOK_URL:
        requests.post(WEBHOOK_URL, json={"id": report_id, "status": "DONE"}, timeout=10)


@celery.task
def requeue_stuck():
    cutoff = datetime.utcnow() - timedelta(minutes=15)
    with Session() as s:
        stuck = s.query(Report).filter(Report.status == ReportStatus.RUNNING,
                                       Report.updated_at < cutoff).all()
        for report in stuck:
            report.status = ReportStatus.PENDING
        s.commit()
    for report in stuck:
        generate_report.delay(report.id)
