import enum

from sqlalchemy import Column, Date, DateTime, Enum, Integer, String, func
from sqlalchemy.orm import declarative_base

Base = declarative_base()


class ReportStatus(enum.Enum):
    PENDING = "PENDING"
    RUNNING = "RUNNING"
    DONE = "DONE"
    FAILED = "FAILED"


class Report(Base):
    __tablename__ = "reports"

    id = Column(String, primary_key=True)
    idempotency_key = Column(String, unique=True, nullable=False)
    start = Column(Date, nullable=False)
    end = Column(Date, nullable=False)
    status = Column(Enum(ReportStatus), nullable=False, default=ReportStatus.PENDING)
    url = Column(String)
    updated_at = Column(DateTime, server_default=func.now(), onupdate=func.now())


class Order(Base):
    __tablename__ = "orders"

    id = Column(Integer, primary_key=True)
    created = Column(Date, nullable=False)
    total_cents = Column(Integer, nullable=False)
