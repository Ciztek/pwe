import sys
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
    while current <= end_date:
        date_str = current.strftime("%m-%d-%Y")
        csv_path = Path(
            settings.data_path
            + f"/csse_covid_19_data/csse_covid_19_daily_reports/{date_str}.csv"
        )
        print(f"Processing {csv_path.absolute().resolve()}")
        result = await db.execute(
            select(CovidData).where(CovidData.date == current)
        )
        existing = result.scalars().first()
        if existing:
            current += timedelta(days=1)
            continue
        if csv_path.exists():
            with open(csv_path, newline="", encoding="utf-8") as f:

                reader = DictReader(f)
                records = [
                    CovidData(
                        date=current,
                        us_county_id=record.get("FIPS"),
                        us_county_name=record.get("Admin2"),
                        province=record.get("Province_State")
                        or record.get("Province/State")
                        or "",
                        country=record.get("Country_Region")
                        or record.get("Country/Region")
                        or "",
                        confirmed=int(record.get("Confirmed") or 0),
                        deaths=int(record.get("Deaths") or 0),
                        recovered=(
                            int(record.get("Recovered") or 0)
                            if record.get("Recovered") not in (None, "")
                            else None
                        ),
                        active=(
                            int(record.get("Active") or 0)
                            if record.get("Active") not in (None, "")
                            else None
                        ),
                        incident_rate=(
                            float(record.get("Incident_Rate") or 0)
                            if record.get("Incident_Rate") not in (None, "")
                            else None
                        ),
                        case_fatality_ratio=(
                            float(record.get("Case_Fatality_Ratio") or 0)
                            if record.get("Case_Fatality_Ratio")
                            not in (None, "")
                            else None
                        ),
                    )
                    for record in reader
                ]
                db.add_all(records)
        current += timedelta(days=1)
    await db.commit()


@asynccontextmanager
async def lifespan(_: FastAPI):
    await init_db()
    async with async_session() as db:
        await fill_db(db)
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
