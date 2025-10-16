from datetime import datetime, timedelta
from http import HTTPStatus

from fastapi import APIRouter, Depends
from fastapi.responses import JSONResponse, RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from ...db import CovidData, get_session
from ...schemas import DataOutput, SimpleMessage

router = APIRouter(prefix="/data")

import functools
import logging
import time

logging.basicConfig(level=logging.INFO)


def log_time(func):
    @functools.wraps(func)
    async def wrapper(*args, **kwargs):
        start = time.perf_counter()
        result = await func(*args, **kwargs)
        end = time.perf_counter()
        logging.info(f"{func.__name__} executed in {end - start:.4f} seconds")
        return result

    return wrapper


@router.get("")
async def get_global_info():
    return RedirectResponse(url="/api/data/2021-01-01/2023-03-09")


@log_time
async def get_data_for_date(db: AsyncSession, date: datetime):
    result = await db.scalars(
        select(CovidData).where(CovidData.date == date.date())
    )
    return result.all()


@log_time
async def get_data_for_date_range(
    db: AsyncSession, start_date: datetime, end_date: datetime
):
    result = await db.scalars(
        select(CovidData).where(
            CovidData.date.between(start_date.date(), end_date.date())
        )
    )
    return result.all()


@log_time
@router.get("/date/{date}", response_model=DataOutput)
async def get_data_by_date(
    date: datetime, db: AsyncSession = Depends(get_session)
):
    data = await get_data_for_date(db, date)
    prev = await get_data_for_date(db, date - timedelta(days=1))
    if not data:
        return JSONResponse(
            {"message": f"No data found for date: {date.date()}"},
            status_code=HTTPStatus.NOT_FOUND,
        )

    def total(data, field):
        return sum(getattr(d, field, 0) or 0 for d in data)

    diff = {
        "confirmed": total(data, "confirmed") - total(prev, "confirmed"),
        "deaths": total(data, "deaths") - total(prev, "deaths"),
        "recovered": total(data, "recovered") - total(prev, "recovered"),
    }
    return DataOutput(
        date=date,
        confirmed=max(diff["confirmed"], 0),
        deaths=max(diff["deaths"], 0),
        recovered=max(diff["recovered"], 0),
    )


@log_time
@router.get("/date/{start_date}/{end_date}", response_model=DataOutput)
async def get_data_by_date_range(
    start_date: datetime,
    end_date: datetime,
    db: AsyncSession = Depends(get_session),
):
    if start_date > end_date:
        return JSONResponse(
            {"message": "Start date must be before or equal to end date."},
            status_code=HTTPStatus.BAD_REQUEST,
        )

    data = await get_data_for_date_range(db, start_date, end_date)
    if not data:
        return JSONResponse(
            {
                "message": f"No data found for date range: {start_date.date()} to {end_date.date()}"
            },
            status_code=HTTPStatus.NOT_FOUND,
        )

    def total(data, field):
        return sum(getattr(d, field, 0) or 0 for d in data)

    return DataOutput(
        date_range=f"{start_date.date()} to {end_date.date()}",
        confirmed=total(data, "confirmed"),
        deaths=total(data, "deaths"),
        recovered=total(data, "recovered"),
    )


# async def get_covid_data_for_date(
#     db: AsyncSession, date: datetime, country: str | None = None
# ):
#     """Fetch CovidData for a specific date and optional country."""
#     stmt = select(CovidData).where(CovidData.date == date.date())
#     if country:
#         stmt = stmt.where(CovidData.country == country)
#     result = await db.execute(stmt)
#     return result.scalars().all()


# async def get_daily_difference(
#     db: AsyncSession, date: datetime, country: str | None = None
# ):
#     """Compute daily new cases as difference from previous day."""
#     current = await get_covid_data_for_date(db, date, country)
#     prev = await get_covid_data_for_date(db, date - timedelta(days=1), country)

#     def total(data, field):
#         return sum(getattr(d, field, 0) or 0 for d in data)

#     return {
#         "confirmed": total(current, "confirmed") - total(prev, "confirmed"),
#         "deaths": total(current, "deaths") - total(prev, "deaths"),
#         "recovered": total(current, "recovered") - total(prev, "recovered"),
#     }


# @router.get(
#     "/{date}",
#     response_model=DataOutput,
#     description="Get daily new COVID-19 cases for a specific date",
# )
# async def get_date_info(
#     date: datetime, db: AsyncSession = Depends(get_session)
# ):
#     current = await get_covid_data_for_date(db, date)
#     if not current:
#         return JSONResponse(
#             {"message": f"No data found for date: {date.date()}"},
#             status_code=HTTPStatus.NOT_FOUND,
#         )

#     diff = await get_daily_difference(db, date)

#     return DataOutput(
#         date=date,
#         confirmed=max(diff["confirmed"], 0),
#         deaths=max(diff["deaths"], 0),
#         recovered=max(diff["recovered"], 0),
#     )


# @router.get(
#     "/{start_date}/{end_date}",
#     response_model=DataOutput,
#     description="Get COVID-19 data for a specific date range (daily new cases aggregated)",
# )
# async def get_date_range_info(
#     start_date: datetime,
#     end_date: datetime,
#     db: AsyncSession = Depends(get_session),
# ):
#     if start_date > end_date:
#         return JSONResponse(
#             {"message": "Start date must be before or equal to end date."},
#             status_code=HTTPStatus.BAD_REQUEST,
#         )

#     total_confirmed = total_deaths = total_recovered = 0

#     date = start_date
#     while date <= end_date:
#         diff = await get_daily_difference(db, date)
#         total_confirmed += max(diff["confirmed"], 0)
#         total_deaths += max(diff["deaths"], 0)
#         total_recovered += max(diff["recovered"], 0)
#         date += timedelta(days=1)

#     return DataOutput(
#         date_range=f"{start_date.date()} to {end_date.date()}",
#         confirmed=total_confirmed,
#         deaths=total_deaths,
#         recovered=total_recovered,
#     )


# @router.get("/{country}")
# async def get_country_info(
#     country: str, db: AsyncSession = Depends(get_session)
# ):
#     result = await db.execute(
#         select(CovidData).where(CovidData.country == country)
#     )
#     data = result.scalars().all()
#     if not data:
#         return JSONResponse(
#             {"message": f"No data found for country: {country}"},
#             status_code=HTTPStatus.NOT_FOUND,
#         )

#     latest = max(data, key=lambda d: d.date)
#     return DataOutput(
#         place=country,
#         confirmed=getattr(latest, "confirmed", 0),
#         deaths=getattr(latest, "deaths", 0),
#         recovered=getattr(latest, "recovered", 0),
#     )


# @router.get("/{date}/{country}")
# async def get_date_country_info(
#     date: datetime, country: str, db: AsyncSession = Depends(get_session)
# ):
#     diff = await get_daily_difference(db, date, country)
#     data = await get_covid_data_for_date(db, date, country)
#     if not data:
#         return JSONResponse(
#             {
#                 "message": f"No data found for country: {country} on date: {date.date()}"
#             },
#             status_code=HTTPStatus.NOT_FOUND,
#         )
#     return DataOutput(
#         place=country,
#         date=date,
#         confirmed=max(diff["confirmed"], 0),
#         deaths=max(diff["deaths"], 0),
#         recovered=max(diff["recovered"], 0),
#     )


# @router.get("/{start_date}/{end_date}/{country}")
# async def get_date_range_country_info(
#     start_date: datetime,
#     end_date: datetime,
#     country: str,
#     db: AsyncSession = Depends(get_session),
# ):
#     if start_date > end_date:
#         return JSONResponse(
#             {"message": "Start date must be before or equal to end date."},
#             status_code=HTTPStatus.BAD_REQUEST,
#         )

#     total_confirmed = total_deaths = total_recovered = 0
#     date = start_date
#     while date <= end_date:
#         diff = await get_daily_difference(db, date, country)
#         total_confirmed += max(diff["confirmed"], 0)
#         total_deaths += max(diff["deaths"], 0)
#         total_recovered += max(diff["recovered"], 0)
#         date += timedelta(days=1)

#     return DataOutput(
#         place=country,
#         date_range=f"{start_date.date()} to {end_date.date()}",
#         confirmed=total_confirmed,
#         deaths=total_deaths,
#         recovered=total_recovered,
#     )
