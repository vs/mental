import uuid
from datetime import date

from fastapi import FastAPI, Header, HTTPException
from kombu.exceptions import OperationalError
from pydantic import BaseModel

from app.db import Session
from app.models import Report, ReportStatus
from app.tasks import generate_report

app = FastAPI()


class ReportRequest(BaseModel):
    start: date
    end: date


@app.post("/reports", status_code=202)
def create_report(req: ReportRequest, idempotency_key: str = Header(...)):
    if (req.end - req.start).days > 366:
        raise HTTPException(400, "date range longer than 366 days")
    with Session() as s:
        existing = s.query(Report).filter_by(idempotency_key=idempotency_key).one_or_none()
        if existing:
            return {"id": existing.id, "status": existing.status.value}
        report = Report(id=str(uuid.uuid4()), idempotency_key=idempotency_key,
                        start=req.start, end=req.end, status=ReportStatus.PENDING)
        s.add(report)
        s.commit()
    try:
        generate_report.delay(report.id)
    except OperationalError:
        raise HTTPException(503, "queue unavailable")
    return {"id": report.id, "status": "PENDING"}


@app.get("/reports/{report_id}")
def get_report(report_id: str):
    with Session() as s:
        report = s.get(Report, report_id)
        if report is None:
            raise HTTPException(404, "not found")
        return {"id": report.id, "status": report.status.value, "url": report.url}
