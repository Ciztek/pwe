import sys
import time
from contextlib import asynccontextmanager
from csv import DictReader
from datetime import date, timedelta
from pathlib import Path

import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from .api.routes import routers
from .config import get_package_config
from .db import async_session, init_db
from .db.models import CovidData


class Config(BaseModel):
    data_path: str


settings = get_package_config(__package__, Config)


async def fill_db(db: AsyncSession):
    start_date = date(2021, 1, 1)
    end_date = date(2023, 3, 9)
    current = start_date

    result = await db.execute(select(CovidData.date).distinct())
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
                            int(r.get("Recovered") or 0)
                            if r.get("Recovered") not in (None, "")
                            else None
                        ),
                        "active": (
                            int(r.get("Active") or 0)
                            if r.get("Active") not in (None, "")
                            else None
                        ),
                        "incident_rate": (
                            float(r.get("Incident_Rate") or 0)
                            if r.get("Incident_Rate") not in (None, "")
                            else None
                        ),
                        "case_fatality_ratio": (
                            float(r.get("Case_Fatality_Ratio") or 0)
                            if r.get("Case_Fatality_Ratio") not in (None, "")
                            else None
                        ),
                    }
                    for r in reader
                ]

                if dict_records:
                    await db.run_sync(
                        lambda s: s.bulk_insert_mappings(
                            CovidData.__mapper__, dict_records
                        )
                    )

        file_count += 1
        if file_count % batch_size == 0:
            await db.commit()

        current += timedelta(days=1)

    await db.commit()


@asynccontextmanager
async def lifespan(_: FastAPI):
    await init_db()
    async with async_session() as db:
        start = time.perf_counter()
        await fill_db(db)
        end = time.perf_counter()
        print(f"DB filled in {end - start:.2f} seconds")
    yield


app = FastAPI(docs_url="/docs", lifespan=lifespan)

for router in routers:
    app.include_router(router, prefix="/api")


if "dev" in sys.argv:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=("http://localhost" "http://127.0.0.1:*"),
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )


def main():
    uvicorn.run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
