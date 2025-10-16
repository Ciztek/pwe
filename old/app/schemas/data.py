from datetime import datetime

from pydantic import BaseModel


class DataOutput(BaseModel):
    place: str | None = None
    date: datetime | None = None
    date_range: str | None = None
    confirmed: int
    deaths: int
    recovered: int


class PlaceOutput(BaseModel):
    countries: list[str]
    state: list[str]
    us_counties: list[str]
