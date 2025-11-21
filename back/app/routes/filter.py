import json
from datetime import date
from typing import Optional

from aiosqlite import Connection
from fastapi import APIRouter, Depends, Query

from ..db import get_db
from ..schemas import DataOutput

router = APIRouter(prefix="/filter")


@router.get("/places")
async def filter_data(db=Depends(get_db)):
    querry = """
    SELECT
        json_group_array(
            json_object(
                'id', c.id,
                'name', c.name,
                'lat', c.lat,
                'lon', c.lon,
                'countries', (
                    SELECT json_group_array(
                        json_object(
                            'id', co.id,
                            'name', co.name,
                            'lat', co.lat,
                            'lon', co.lon,
                            'states', (
                                SELECT json_group_array(
                                    CASE
                                        WHEN co.name = 'US' THEN
                                            json_object(
                                                'id', s.id,
                                                'name', s.name,
                                                'lat', s.lat,
                                                'lon', s.lon,
                                                'counties', (
                                                    SELECT json_group_array(
                                                        json_object(
                                                            'id', ca.id,
                                                            'name', ca.name,
                                                            'lat', ca.lat,
                                                            'lon', ca.lon
                                                        )
                                                    )
                                                    FROM county ca
                                                    WHERE ca.state_id = s.id
                                                )
                                            )
                                        ELSE
                                            json_object(
                                                'id', s.id,
                                                'name', s.name
                                            )
                                    END
                                )
                                FROM state s
                                WHERE s.country_id = co.id
                            )
                        )
                    )
                    FROM country co
                    WHERE co.continent_id = c.id
                )
            )
        ) AS continents_json
    FROM continent c;
    """
    result = await db.execute(querry)
    row = await result.fetchone()
    continents_json_str = row[0] if row and row[0] else "[]"

    continents_json = json.loads(continents_json_str)
    return {"continents": continents_json}


@router.get("/dates")
async def get_dates(db=Depends(get_db)):
    query = """
    SELECT DISTINCT date FROM data_point ORDER BY date;
    """
    result = await db.execute(query)
    rows = await result.fetchall()
    dates = [row[0] for row in rows]
    return {"dates": dates}


@router.get("/data")
async def get_data(
    db: Connection = Depends(get_db),
    start_date: Optional[date] = Query(None),
    end_date: Optional[date] = Query(None),
    date_: Optional[date] = Query(None, alias="date"),
    continent: Optional[str] = None,
    country: Optional[str] = None,
    state: Optional[str] = None,
    county: Optional[str] = None,
):
    """
    Return SUM(daily_confirmed) and SUM(daily_deaths) over the date range with nested breakdown by place.

    - If date= is provided → treat it as start_date=end_date
    - If no date filters → use the full range available
    - If place filters are provided → drill down to that place in the result
    - Otherwise return global summary with continent breakdown
    """

    if date_:
        start_date = end_date = date_

    if not start_date or not end_date:
        row = await (
            await db.execute("SELECT MIN(date), MAX(date) FROM data_point")
        ).fetchone()
        assert row is not None
        start_date = start_date or date.fromisoformat(row[0])
        end_date = end_date or date.fromisoformat(row[1])

    params = {
        "start": start_date.isoformat(),
        "end": end_date.isoformat(),
        "continent": continent,
        "country": country,
        "state": state,
        "county": county,
    }

    where_clauses = ["dp.date BETWEEN :start AND :end"]

    if continent:
        where_clauses.append("LOWER(continent.name) = LOWER(:continent)")
    if country:
        where_clauses.append("LOWER(country.name) = LOWER(:country)")
    if state:
        where_clauses.append("LOWER(state.name) = LOWER(:state)")
    if county:
        where_clauses.append("LOWER(county.name) = LOWER(:county)")

    where_sql = " AND ".join(where_clauses)

    query = f"""
    SELECT
        continent.name AS continent_name,
        country.name AS country_name,
        state.name AS state_name,
        county.name AS county_name,
        SUM(dp.daily_confirmed) AS confirmed,
        SUM(dp.daily_deaths) AS deaths
    FROM data_point dp
    LEFT JOIN country   ON country.name = dp.country
    LEFT JOIN continent ON continent.id = country.continent_id
    LEFT JOIN state     ON state.name = dp.province AND state.country_id = country.id
    LEFT JOIN county    ON county.name = dp.us_county_name AND county.state_id = state.id
    WHERE {where_sql}
    GROUP BY continent_name, country_name, state_name, county_name
    """

    rows = await (await db.execute(query, params)).fetchall()

    def make_node(place):
        return {
            "place": place,
            "confirmed": 0,
            "deaths": 0,
            "date_range": {
                "start": start_date.isoformat(),
                "end": end_date.isoformat(),
            },
        }

    world_confirmed = 0
    world_deaths = 0
    continents = {}

    for (
        continent_name,
        country_name,
        state_name,
        county_name,
        confirmed,
        deaths,
    ) in rows:

        world_confirmed += confirmed or 0
        world_deaths += deaths or 0

        if continent_name not in continents:
            continents[continent_name] = make_node(continent_name)

        cont = continents[continent_name]
        cont["confirmed"] += confirmed or 0
        cont["deaths"] += deaths or 0

        if country_name is None:
            continue

        cont.setdefault("detail", [])
        country_node = next(
            (c for c in cont["detail"] if c["place"] == country_name), None
        )

        if not country_node:
            country_node = make_node(country_name)
            cont["detail"].append(country_node)

        country_node["confirmed"] += confirmed or 0
        country_node["deaths"] += deaths or 0

        if state_name is None:
            continue

        country_node.setdefault("detail", [])
        state_node = next(
            (s for s in country_node["detail"] if s["place"] == state_name),
            None,
        )

        if not state_node:
            state_node = make_node(state_name)
            country_node["detail"].append(state_node)

        state_node["confirmed"] += confirmed or 0
        state_node["deaths"] += deaths or 0

        if country_name == "US" and county_name is not None:
            state_node.setdefault("detail", [])
            county_node = next(
                (x for x in state_node["detail"] if x["place"] == county_name),
                None,
            )

            if not county_node:
                county_node = make_node(county_name)
                state_node["detail"].append(county_node)

            county_node["confirmed"] += confirmed or 0
            county_node["deaths"] += deaths or 0

    result = make_node("Global")
    result["confirmed"] = world_confirmed
    result["deaths"] = world_deaths
    result["detail"] = list(continents.values())

    # Fine grain the result to match filter inputs
    def drill_down(root):
        # Priority: county > state > country > continent
        if county:
            # find the state first
            for cont in root.get("detail", []):
                for ctry in cont.get("detail", []):
                    for st in ctry.get("detail", []):
                        for cy in st.get("detail", []):
                            if cy["place"].lower() == county.lower():
                                return cy
        if state:
            for cont in root.get("detail", []):
                for ctry in cont.get("detail", []):
                    for st in ctry.get("detail", []):
                        if st["place"].lower() == state.lower():
                            return st
        if country:
            for cont in root.get("detail", []):
                for ctry in cont.get("detail", []):
                    if ctry["place"].lower() == country.lower():
                        return ctry
        if continent:
            for cont in root.get("detail", []):
                if cont["place"].lower() == continent.lower():
                    return cont
        return root

    return drill_down(result)
