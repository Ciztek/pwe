from datetime import datetime

from fastapi import APIRouter

from ..data import DailyData, bsearch, process_data
from ..schemas.data import Continent, Country, PlaceOutput
from ..utils import log_time

router = APIRouter(prefix="/place")


@router.get(
    "",
    response_model=PlaceOutput,
    description="Get place information",
)
async def get_place_info(data: list[DailyData] = process_data()):
    first_jan_2021_data = bsearch(
        data, key="date", target=int(datetime(2021, 1, 1).timestamp())
    )
    (log_time(lambda: print({d["country"] for d in first_jan_2021_data})))()
    return PlaceOutput(place=[])  # Dummy implementation
