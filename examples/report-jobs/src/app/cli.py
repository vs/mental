import click

from app.db import Session
from app.models import Report, ReportStatus
from app.tasks import generate_report


@click.group()
def reportctl():
    pass


@reportctl.command()
@click.argument("report_id")
def retry(report_id):
    with Session() as s:
        report = s.get(Report, report_id)
        if report is None:
            raise click.ClickException("no such report")
        if report.status != ReportStatus.FAILED:
            raise click.ClickException(f"report is {report.status.value}, not FAILED")
        report.status = ReportStatus.PENDING
        s.commit()
    generate_report.delay(report_id)
    click.echo("requeued")


if __name__ == "__main__":
    reportctl()
