from csv import DictReader
from datetime import date, timedelta
from pathlib import Path

from aiosqlite import Connection, Row, connect
from countryinfo import CountryInfo
from pydantic import BaseModel

from ..config import get_package_config
from ..utils.wrapper import async_timed


class Config(BaseModel):
    uri: str
    data_path: str


settings = get_package_config(__package__, Config)


async def get_db():
    """Async context manager that yields an aiosqlite connection."""
    conn = await connect(settings.uri)
    conn.row_factory = Row
    await conn.execute("PRAGMA foreign_keys = ON;")
    try:
        yield conn
    finally:
        await conn.close()


@async_timed
async def init_schema(conn: Connection):
    """Create tables with proper AUTOINCREMENT and lat/lon columns."""
    await conn.executescript(
        """
    CREATE TABLE IF NOT EXISTS continent (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        lat REAL,
        lon REAL
    );

    CREATE TABLE IF NOT EXISTS country (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        continent_id INTEGER NOT NULL,
        lat REAL,
        lon REAL,
        FOREIGN KEY (continent_id) REFERENCES continent(id)
    );

    CREATE TABLE IF NOT EXISTS state (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        country_id INTEGER NOT NULL,
        lat REAL,
        lon REAL,
        FOREIGN KEY (country_id) REFERENCES country(id)
    );

    CREATE TABLE IF NOT EXISTS county (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        state_id INTEGER NOT NULL,
        lat REAL,
        lon REAL,
        FOREIGN KEY (state_id) REFERENCES state(id)
    );

    CREATE TABLE IF NOT EXISTS data_point (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        us_county_id TEXT,
        us_county_name TEXT,
        province TEXT,
        country TEXT NOT NULL,
        confirmed INTEGER NOT NULL,
        deaths INTEGER NOT NULL,
        active INTEGER,
        incident_rate REAL,
        case_fatality_ratio REAL,
        lat REAL,
        lon REAL
    );
    """
    )
    await conn.commit()


@async_timed
async def fill_db(conn: Connection):
    """Load CSV data efficiently without ORM overhead."""
    data_path = Path(settings.data_path)
    start_date = date(2021, 1, 1)
    end_date = date(2023, 3, 9)

    cursor = await conn.execute("SELECT DISTINCT date FROM data_point")
    existing_dates = {row["date"] async for row in cursor}
    await cursor.close()

    current = start_date
    batch_size = 0
    total_inserted = 0

    while current <= end_date:
        if str(current) in existing_dates:
            current += timedelta(days=1)
            continue

        csv_path = (
            data_path
            / f"csse_covid_19_data/csse_covid_19_daily_reports/{current:%m-%d-%Y}.csv"
        )
        if not csv_path.exists():
            current += timedelta(days=1)
            continue

        with open(csv_path, newline="", encoding="utf-8") as f:
            reader = DictReader(f)
            rows = [
                (
                    str(current),
                    r.get("FIPS") or None,
                    r.get("Admin2") or None,
                    r.get("Province_State") or r.get("Province/State") or None,
                    r.get("Country_Region") or r.get("Country/Region"),
                    int(r.get("Confirmed") or 0),
                    int(r.get("Deaths") or 0),
                    (
                        int(r.get("Active") or 0)
                        if r.get("Active") not in (None, "")
                        else None
                    ),
                    (
                        float(r.get("Incident_Rate") or 0)
                        if r.get("Incident_Rate") not in (None, "")
                        else None
                    ),
                    (
                        float(r.get("Case_Fatality_Ratio") or 0)
                        if r.get("Case_Fatality_Ratio") not in (None, "")
                        else None
                    ),
                    float(r.get("Lat") or 0),
                    float(r.get("Long_") or 0),
                )
                for r in reader
            ]

        await conn.executemany(
            """
            INSERT INTO data_point (
                date, us_county_id, us_county_name, province, country,
                confirmed, deaths, active,
                incident_rate, case_fatality_ratio, lat, lon
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
            rows,
        )

        total_inserted += len(rows)
        batch_size += 1
        if batch_size % 30 == 0:
            print(
                f"[fill_db] Done processing data for batch: {current - timedelta(days=30)} to {current}"
            )
            await conn.commit()

        current += timedelta(days=1)

    print(
        f"[fill_db] Done processing data for batch: {current - timedelta(days=batch_size)} to {current}, duration {batch_size} days."
    )
    await conn.commit()
    print(f"[fill_db] Inserted {total_inserted} data rows.")


@async_timed
async def place_db(conn: Connection):
    """Generate hierarchical place tables based on distinct data points."""
    print("[place_db] Building place hierarchy...")

    cursor = await conn.execute(
        """
        SELECT DISTINCT country, province, us_county_name
        FROM data_point
    """
    )
    rows = await cursor.fetchall()

    country_cache = {}

    for country, province, county in rows:
        if not country:
            continue

        province = None if province == "Unknown" else province

        # --- Use CountryInfoFacade for location data ---
        if country not in country_cache:
            try:
                info = CountryInfo(country)
                loc = info.get_location_info()
                region = loc.region() or loc.subregion() or "Unknown"
                latlng = loc.latlng() or [0, 0]
                lat, lon = latlng if len(latlng) == 2 else (0, 0)
            except Exception:
                region = "Unknown"
                lat, lon = 0, 0

            country_cache[country] = {
                "continent": region,
                "lat": lat,
                "lon": lon,
            }

        entry = country_cache[country]
        continent = entry["continent"]
        country_lat, country_lon = entry["lat"], entry["lon"]

        # Continent
        await conn.execute(
            "INSERT OR IGNORE INTO continent (name, lat, lon) VALUES (?, ?, ?)",
            (continent, None, None),
        )
        continent_id = (
            await (
                await conn.execute(
                    "SELECT id FROM continent WHERE name = ?", (continent,)
                )
            ).fetchone()
        )[0]

        # Country
        await conn.execute(
            "INSERT OR IGNORE INTO country (name, continent_id, lat, lon) VALUES (?, ?, ?, ?)",
            (country, continent_id, country_lat, country_lon),
        )
        country_id = (
            await (
                await conn.execute(
                    "SELECT id FROM country WHERE name = ?", (country,)
                )
            ).fetchone()
        )[0]

        # State
        if province:
            cursor = await conn.execute(
                """
                SELECT lat, lon
                FROM data_point
                WHERE province = ? AND lat IS NOT NULL AND lon IS NOT NULL
                LIMIT 1
                """,
                (province,),
            )
            row = await cursor.fetchone()
            state_lat, state_lon = (row["lat"], row["lon"]) if row else (0, 0)

            await conn.execute(
                "INSERT OR IGNORE INTO state (name, country_id, lat, lon) VALUES (?, ?, ?, ?)",
                (province, country_id, state_lat, state_lon),
            )
            state_id = (
                await (
                    await conn.execute(
                        "SELECT id FROM state WHERE name = ?", (province,)
                    )
                ).fetchone()
            )[0]

            # County
            if county and country == "US":
                cursor = await conn.execute(
                    """
                    SELECT lat, lon
                    FROM data_point
                    WHERE us_county_name = ? AND lat IS NOT NULL AND lon IS NOT NULL
                    LIMIT 1
                    """,
                    (county,),
                )
                crow = await cursor.fetchone()
                county_lat, county_lon = (
                    (crow["lat"], crow["lon"]) if crow else (0, 0)
                )

                await conn.execute(
                    "INSERT OR IGNORE INTO county (name, state_id, lat, lon) VALUES (?, ?, ?, ?)",
                    (county, state_id, county_lat, county_lon),
                )

    await conn.commit()
    print("[place_db] Hierarchical place data updated successfully.")


@async_timed
async def init_db():
    async with connect(settings.uri) as conn:
        conn.row_factory = Row
        await init_schema(conn)
        await fill_db(conn)
        await place_db(conn)
