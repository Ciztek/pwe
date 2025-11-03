import re
from csv import DictReader
from datetime import date, timedelta
from pathlib import Path

from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import (
    AsyncSession,
    async_sessionmaker,
    create_async_engine,
)
from sqlalchemy.orm import declarative_base

from ..config import get_package_config


class Config(BaseModel):
    uri: str
    data_path: str


settings = get_package_config(__package__, Config)

Base = declarative_base()


class TableNameProvider:
    def __init_subclass__(cls, **kwargs):
        cls.__tablename__ = re.sub(
            r"(?<!^)(?=[A-Z])", "_", cls.__name__
        ).lower()
        super().__init_subclass__(**kwargs)


engine = create_async_engine(settings.uri, echo=True)

async_session = async_sessionmaker(
    bind=engine,
    expire_on_commit=False,
    class_=AsyncSession,
)


async def get_session():
    async with async_session() as session:
        yield session


async def fill_db(db: AsyncSession):
    from .models import DataPoint

    start_date = date(2021, 1, 1)
    end_date = date(2023, 3, 9)
    current = start_date

    result = await db.execute(select(DataPoint.date).distinct())
    existing_dates = {row for row in result.scalars().all()}

    batch_size = 30
    file_count = 0

    while current <= end_date:
        if current in existing_dates:
            current += timedelta(days=1)
            continue

        date_str = current.strftime("%m-%d-%Y")
        csv_path = Path(
            settings.data_path
            + f"/csse_covid_19_data/csse_covid_19_daily_reports/{date_str}.csv"
        )
        if csv_path.exists():
            with open(csv_path, newline="", encoding="utf-8") as f:
                reader = DictReader(f)
                dict_records = [
                    {
                        "date": current,
                        "us_county_id": r.get("FIPS") or None,
                        "us_county_name": r.get("Admin2") or None,
                        "province": r.get("Province_State")
                        or r.get("Province/State")
                        or None,
                        "country": r.get("Country_Region")
                        or r.get("Country/Region"),
                        "confirmed": int(r.get("Confirmed") or 0),
                        "deaths": int(r.get("Deaths") or 0),
                        "recovered": (
                            int(r.get("Recovered"))
                            if r.get("Recovered") not in (None, "")
                            else None
                        ),
                        "active": (
                            int(r.get("Active"))
                            if r.get("Active") not in (None, "")
                            else None
                        ),
                        "incident_rate": (
                            float(r.get("Incident_Rate"))
                            if r.get("Incident_Rate") not in (None, "")
                            else None
                        ),
                        "case_fatality_ratio": (
                            float(r.get("Case_Fatality_Ratio"))
                            if r.get("Case_Fatality_Ratio") not in (None, "")
                            else None
                        ),
                        "lat": (
                            float(r.get("Lat"))
                            if r.get("Lat") not in (None, "")
                            else None
                        ),
                        "long": (
                            float(r.get("Long_"))
                            if r.get("Long_") not in (None, "")
                            else None
                        ),
                    }
                    for r in reader
                ]

                if dict_records:
                    await db.run_sync(
                        lambda s: s.bulk_insert_mappings(
                            DataPoint.__mapper__, dict_records
                        )
                    )

        file_count += 1
        if file_count % batch_size == 0:
            await db.commit()

        current += timedelta(days=1)

    await db.commit()


async def place_db(db: AsyncSession):
    from .models import Continent, Country, County, DataPoint, State

    pass


async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
        await fill_db(AsyncSession(conn))
        await place_db(AsyncSession(conn))
