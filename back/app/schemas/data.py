from datetime import date as Date
from typing import TypedDict

from pydantic import BaseModel


class DataOutput(BaseModel):
    place: str | None = None
    date: Date | None = None
    date_range: str | None = None
    confirmed: int
    deaths: int
    recovered: int


class State(TypedDict):
    name: str
    county: list[str]


class Country(TypedDict):
    name: str
    state: list[State]


class Continent(TypedDict):
    name: str
    country: list[Country]


class PlaceOutput(BaseModel):
    place: list[Continent]
