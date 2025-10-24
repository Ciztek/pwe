from __future__ import annotations

from collections import defaultdict
from datetime import datetime
from typing import get_type_hints

from fastapi import APIRouter, Path
from pydantic import BaseModel, ValidationInfo, field_validator

from ..data import DailyData, bsearch, process_data
from ..schemas.data import Continent, Country, PlaceOutput, State

router = APIRouter(prefix="/data")

data: list[DailyData] | None = None

VALID_KEYS = list(DailyData.__annotations__.keys())


class PreciseParams(BaseModel):
    key: str
    target: str  # initially a string from the URL

    @field_validator("key")
    def validate_key(cls, v):
        if v not in VALID_KEYS:
            raise ValueError(f"Invalid key: {v}")
        return v

    @field_validator("target")
    def ensure_target_matches_key_type(cls, v, info: ValidationInfo):
        key = info.data.get("key")

        if key is None:
            return v  # key validation will catch this
        expected_type = get_type_hints(DailyData).get(key)

        if expected_type is None:
            raise ValueError(f"Unknown key: {key}")

        try:
            if expected_type == int:
                return int(v)
            elif expected_type == float:
                return float(v)
            elif expected_type == str:
                return str(v)
            else:
                return v
        except (TypeError, ValueError):
            raise ValueError(
                f"target must be of type {expected_type.__name__} for key '{key}'"
            )


@router.get(
    "/{key}/{target}",
    response_model=list[DailyData],
    description="Get data filtered by a specific key and target value",
)
async def get_place_info(query: PreciseParams = Path(...)):
    global data

    if data is None:
        data = process_data()
    filtered = bsearch(data, query.key, query.target)
    return filtered


# @router.get("/{key}/{start}/{end}", response_model=list[DailyData], description="Get data filtered by a specific key and target value within a range")
# async def get_data_in_range(key: str, start: int, end: int):
#     global data

#     if data is None:
#         data = process_data()

#     filtered = [d for d in data if d[key] >= start and d[key] <= end]
#     return filtered
