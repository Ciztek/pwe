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


@router.get("/data", response_model=DataOutput)
async def get_data(
    db: Connection = Depends(get_db),
    start_date: Optional[date] = Query(
        None, description="Start of date range"
    ),
    end_date: Optional[date] = Query(None, description="End of date range"),
    date_: Optional[date] = Query(
        None, alias="date", description="Exact date to filter"
    ),
    continent: Optional[str] = None,
    country: Optional[str] = None,
    state: Optional[str] = None,
    county: Optional[str] = None,
):
    """
    Query cumulative or delta data depending on the filters.

    - If `date` is given → return totals up to that date.
    - If `start_date` and `end_date` are given → return *difference* between those dates.
    - Location filters (continent, country, state, county) are combinable (and are case-insensitive).
    - Location and date filters are combinable.
    If no date filters are given, return latest available totals.
    """

    filters = []
    params = {}

    if county:
        filters.append("LOWER(us_county_name) = LOWER(:county)")
        params["county"] = county
    if state:
        filters.append("LOWER(province) = LOWER(:state)")
        params["state"] = state
    if country:
        filters.append("LOWER(country) = LOWER(:country)")
        params["country"] = country
    if continent:
        # continent via subquery mapping (avoid duplicate joins)
        filters.append(
            """
            LOWER(country) IN (
                SELECT LOWER(co.name)
                FROM country co
                JOIN continent ct ON co.continent_id = ct.id
                WHERE LOWER(ct.name) = LOWER(:continent)
            )
        """
        )
        params["continent"] = continent

    where_clause = f"WHERE {' AND '.join(filters)}" if filters else ""

    # helper to get total for specific date
    async def get_total(date_value: date) -> dict[str, int]:
        q = f"""
        SELECT
            COALESCE(SUM(confirmed), 0) AS confirmed,
            COALESCE(SUM(deaths), 0) AS deaths,
            COALESCE(SUM(recovered), 0) AS recovered
        FROM data_point
        {where_clause}
        {'AND' if filters else 'WHERE'} date = :date
        """
        result = await db.execute(q, {**params, "date": str(date_value)})
        row = await result.fetchone()
        assert row is not None, "Due to COALESCE, row should never be None"
        return {
            "confirmed": row["confirmed"],
            "deaths": row["deaths"],
            "recovered": row["recovered"],
        }

    # exact-date mode
    if date_:
        totals = await get_total(date_)
        return DataOutput(
            place=county or state or country or continent or "Global",
            date=date_,
            confirmed=totals["confirmed"],
            deaths=totals["deaths"],
            recovered=totals["recovered"],
        )

    # date-range mode → difference between end and (start - 1)
    if start_date and end_date:
        totals_end = await get_total(end_date)

        prev_date_query = await db.execute(
            "SELECT MAX(date) AS prev FROM data_point WHERE date < :start",
            {"start": str(start_date)},
        )
        prev_row = await prev_date_query.fetchone()
        prev_date = prev_row["prev"]

        if prev_date:
            totals_prev = await get_total(prev_date)
        else:
            totals_prev = {"confirmed": 0, "deaths": 0, "recovered": 0}

        deltas = {
            k: max(totals_end[k] - totals_prev[k], 0) for k in totals_end
        }

        return DataOutput(
            place=county or state or country or continent or "Global",
            date_range=f"{start_date} → {end_date}",
            confirmed=deltas["confirmed"],
            deaths=deltas["deaths"],
            recovered=deltas["recovered"],
        )

    # default: latest available date
    latest_q = await db.execute("SELECT MAX(date) AS latest FROM data_point")
    latest_date = (await latest_q.fetchone())["latest"]

    totals = await get_total(latest_date)
    return DataOutput(
        place=county or state or country or continent or "Global",
        date=latest_date,
        confirmed=totals["confirmed"],
        deaths=totals["deaths"],
        recovered=totals["recovered"],
    )
