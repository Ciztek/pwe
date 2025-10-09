from datetime import datetime
from http import HTTPStatus

from fastapi import APIRouter, Depends
from fastapi.responses import JSONResponse, RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from ...db import CovidData, get_session
from ...schemas import DataOutput, SimpleMessage

router = APIRouter(prefix="/data")


@router.get("")
async def get_global_info():
    return RedirectResponse(url="/api/data/2021-01-01/2023-03-09")


@router.get(
    "/{date}",
    response_model=DataOutput,
    description="Get COVID-19 data for a specific date",
    responses={
        HTTPStatus.OK: {
            "model": DataOutput,
            "content": {
                "application/json": {
                    "example": DataOutput(
                        date=datetime(2023, 3, 9),
                        confirmed=123456,
                        deaths=7890,
                        recovered=100000,
                    ).model_dump()
                }
            },
            "description": "COVID-19 data for the specified date",
        },
        HTTPStatus.NOT_FOUND: {
            "model": SimpleMessage,
            "content": {
                "application/json": {
                    "example": SimpleMessage(
                        message="No data found for date: 2023-03-09"
                    ).model_dump()
                }
            },
            "description": "No data found for the specified date",
        },
    },
)
async def get_date_info(
    date: datetime, db: AsyncSession = Depends(get_session)
):
    result = await db.execute(
        select(CovidData).where(CovidData.date == date.date())
    )
    data = result.scalars().all()
    if not data:
        return JSONResponse(
            {"message": f"No data found for date: {date.date()}"},
            status_code=HTTPStatus.NOT_FOUND,
        )
    return DataOutput(
        date=date,
        confirmed=sum(getattr(d, "confirmed", 0) for d in data),
        deaths=sum(getattr(d, "deaths", 0) for d in data),
        recovered=sum(getattr(d, "recovered", 0) for d in data),
    )


@router.get(
    "/{start_date}/{end_date}",
    response_model=DataOutput,
    description="Get COVID-19 data for a specific date range",
    responses={
        HTTPStatus.OK: {
            "model": DataOutput,
            "content": {
                "application/json": {
                    "example": DataOutput(
                        date_range="2021-01-01 to 2021-01-31",
                        confirmed=123456,
                        deaths=7890,
                        recovered=100000,
                    ).model_dump()
                }
            },
            "description": "COVID-19 data for the specified date range",
        },
        HTTPStatus.NOT_FOUND: {
            "model": SimpleMessage,
            "content": {
                "application/json": {
                    "example": SimpleMessage(
                        message="No data found for date range: 2021-01-01 to 2021-01-31"
                    ).model_dump()
                }
            },
            "description": "No data found for the specified date range",
        },
    },
)
async def get_date_range_info(
    start_date: datetime,
    end_date: datetime,
    db: AsyncSession = Depends(get_session),
):
    if start_date > end_date:
        return JSONResponse(
            {"message": "Start date must be before or equal to end date."},
            status_code=HTTPStatus.BAD_REQUEST,
        )
    result = await db.execute(
        select(CovidData).where(
            CovidData.date.between(start_date.date(), end_date.date())
        )
    )
    data = result.scalars().all()
    if not data:
        return JSONResponse(
            {
                "message": f"No data found for date range: {start_date.date()} to {end_date.date()}"
            },
            status_code=HTTPStatus.NOT_FOUND,
        )
    return DataOutput(
        date_range=f"{start_date.date()} to {end_date.date()}",
        confirmed=sum(getattr(d, "confirmed", 0) or 0 for d in data),
        deaths=sum(getattr(d, "deaths", 0) or 0 for d in data),
        recovered=sum(getattr(d, "recovered", 0) or 0 for d in data),
    )


@router.get("/{country}")
async def get_country_info(
    country: str, db: AsyncSession = Depends(get_session)
):
    result = await db.execute(
        select(CovidData).where(CovidData.country == country)
    )
    data = result.scalars().all()
    if not data:
        return JSONResponse(
            {"message": f"No data found for country: {country}"},
            status_code=HTTPStatus.NOT_FOUND,
        )
    return DataOutput(
        place=country,
        confirmed=sum(getattr(d, "confirmed", 0) or 0 for d in data),
        deaths=sum(getattr(d, "deaths", 0) or 0 for d in data),
        recovered=sum(getattr(d, "recovered", 0) or 0 for d in data),
    )


@router.get("/{date}/{country}")
async def get_date_country_info(
    date: datetime, country: str, db: AsyncSession = Depends(get_session)
):
    result = await db.execute(
        select(CovidData).where(
            (CovidData.date == date.date()) & (CovidData.country == country)
        )
    )
    data = result.scalars().all()
    if not data:
        return JSONResponse(
            {
                "message": f"No data found for country: {country} on date: {date.date()}"
            },
            status_code=HTTPStatus.NOT_FOUND,
        )
    return DataOutput(
        place=country,
        date=date,
        confirmed=sum(getattr(d, "confirmed", 0) or 0 for d in data),
        deaths=sum(getattr(d, "deaths", 0) or 0 for d in data),
        recovered=sum(getattr(d, "recovered", 0) or 0 for d in data),
    )


@router.get("/{start_date}/{end_date}/{country}")
async def get_date_range_country_info(
    start_date: datetime,
    end_date: datetime,
    country: str,
    db: AsyncSession = Depends(get_session),
):
    if start_date > end_date:
        return JSONResponse(
            {"message": "Start date must be before or equal to end date."},
            status_code=HTTPStatus.BAD_REQUEST,
        )
    result = await db.execute(
        select(CovidData).where(
            (CovidData.date.between(start_date.date(), end_date.date()))
            & (CovidData.country == country)
        )
    )
    data = result.scalars().all()
    if not data:
        return JSONResponse(
            {
                "message": f"No data found for country: {country} in date range: {start_date.date()} to {end_date.date()}"
            },
            status_code=HTTPStatus.NOT_FOUND,
        )
    return DataOutput(
        place=country,
        date_range=f"{start_date.date()} to {end_date.date()}",
        confirmed=sum(getattr(d, "confirmed", 0) or 0 for d in data),
        deaths=sum(getattr(d, "deaths", 0) or 0 for d in data),
        recovered=sum(getattr(d, "recovered", 0) or 0 for d in data),
    )
