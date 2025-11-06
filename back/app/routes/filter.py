import json

from fastapi import APIRouter, Depends

from ..db import get_db

router = APIRouter(prefix="/filter")


@router.get("/places")
async def filter_data(db=Depends(get_db)):
    querry = """
    SELECT
        json_group_array(
            json_object(
                'id', c.id,
                'name', c.name,
                'countries', (
                    SELECT json_group_array(
                        json_object(
                            'id', co.id,
                            'name', co.name,
                            'states', (
                                SELECT json_group_array(
                                    CASE
                                        WHEN co.name = 'US' THEN
                                            json_object(
                                                'id', s.id,
                                                'name', s.name,
                                                'counties', (
                                                    SELECT json_group_array(
                                                        json_object(
                                                            'id', ca.id,
                                                            'name', ca.name
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
