from datetime import datetime
from http import HTTPStatus

from fastapi import APIRouter, Depends
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from ...db import CovidData, get_session
from ...schemas import SimpleMessage

router = APIRouter(prefix="/hello")


# @router.get(
#     "",
#     response_model=SimpleMessage,
#     description="Dummy endpoint that reply a greeting",
#     responses={
#         HTTPStatus.OK: {
#             "model": SimpleMessage,
#             "content": {
#                 "application/json": {
#                     "example": SimpleMessage(
#                         message="Hello, World!"
#                     ).model_dump()
#                 }
#             },
#             "description": "greeting message",
#         }
#     },
# )
# async def example_json():
#     return JSONResponse({"message": "Hello, World!"})


@router.get("")
async def example_json():
    return JSONResponse({"message": "Hello, World!"})


class DataOutput(BaseModel):
    date: datetime
    confirmed: int
    deaths: int
    recovered: int


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
