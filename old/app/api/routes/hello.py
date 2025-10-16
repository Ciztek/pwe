from fastapi import APIRouter, Depends
from sqlalchemy import select

from ...db import get_session
from ...db.models.data import CovidData
from ...schemas import SimpleMessage
from ...schemas.data import PlaceOutput

router = APIRouter()


@router.get("/hello", response_model=SimpleMessage)
async def example_json():
    return SimpleMessage(message="Hello, World!")


@router.get("/places", response_model=PlaceOutput)
async def get_places(db=Depends(get_session)):
    query = select(
        CovidData.country, CovidData.province, CovidData.us_county_name
    ).distinct()

    result = await db.execute(query)
    rows = result.all()
    countries = sorted({r[0] for r in rows if r[0]})
    states = sorted({r[1] for r in rows if r[1]})
    counties = sorted({r[2] for r in rows if r[2]})
    return PlaceOutput(
        countries=countries,
        state=states,
        us_counties=counties,
    )
