from __future__ import annotations

from collections import defaultdict
from datetime import datetime

from fastapi import APIRouter

from ..data import DailyData, process_data
from ..schemas.data import Continent, Country, PlaceOutput, State

router = APIRouter(prefix="/place")

test: list[DailyData] | None = None


def build_place_output(data: list[DailyData]) -> PlaceOutput:
    # continent -> country -> state -> counties[]
    structure = defaultdict(lambda: defaultdict(lambda: defaultdict(set)))

    for d in data:
        continent = country_to_continent.get(d["country"], "Unknown")
        country = d["country"]
        state = d["state"] or "Unknown"
        county = d["county"] or "Unknown"

        structure[continent][country][state].add(county)

    continents: list[Continent] = []

    for continent_name, countries in structure.items():
        continent_countries: list[Country] = []

        for country_name, states in countries.items():
            country_states: list[State] = []

            for state_name, counties in states.items():
                country_states.append(
                    State(name=state_name, county=list(counties))
                )

            continent_countries.append(
                Country(name=country_name, state=country_states)
            )

        continents.append(
            Continent(name=continent_name, country=continent_countries)
        )

    return PlaceOutput(place=continents)


@router.get(
    "",
    response_model=PlaceOutput,
    description="Get place information",
)
async def get_place_info():
    global test

    if test is None:
        test = process_data()

    data = test

    target_date = int(datetime(2021, 1, 1).timestamp())
    data = [d for d in data if d["date"] == target_date]
    return build_place_output(data)


country_to_continent = {
    "United States": "North America",
    "Canada": "North America",
    "Mexico": "North America",
    "Brazil": "South America",
    "France": "Europe",
    "Germany": "Europe",
    "India": "Asia",
    "China": "Asia",
    "South Africa": "Africa",
    "Australia": "Oceania",
}
