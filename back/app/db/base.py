from csv import DictReader
from datetime import date, timedelta
from pathlib import Path

from aiosqlite import Connection, Row, connect
from pydantic import BaseModel

from ..config import get_package_config
from ..utils.continent_map import CONTINENT_MAP
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
    """Create tables with proper AUTOINCREMENT (independent counters)."""
    await conn.executescript(
        """
    CREATE TABLE IF NOT EXISTS continent (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL
    );

    CREATE TABLE IF NOT EXISTS country (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        continent_id INTEGER NOT NULL,
        FOREIGN KEY (continent_id) REFERENCES continent(id)
    );

    CREATE TABLE IF NOT EXISTS state (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        country_id INTEGER NOT NULL,
        FOREIGN KEY (country_id) REFERENCES country(id)
    );

    CREATE TABLE IF NOT EXISTS county (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        state_id INTEGER NOT NULL,
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
        recovered INTEGER,
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
                        int(r.get("Recovered") or 0)
                        if r.get("Recovered") not in (None, "")
                        else None
                    ),
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
                confirmed, deaths, recovered, active,
                incident_rate, case_fatality_ratio, lat, lon
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

    for country, province, county in rows:
        if not country:
            continue

        province = None if province == "Unknown" else province
        continent = CONTINENT_MAP.get(country, "Unknown")

        # Continent
        await conn.execute(
            "INSERT OR IGNORE INTO continent (name) VALUES (?)", (continent,)
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
            "INSERT OR IGNORE INTO country (name, continent_id) VALUES (?, ?)",
            (country, continent_id),
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
            await conn.execute(
                "INSERT OR IGNORE INTO state (name, country_id) VALUES (?, ?)",
                (province, country_id),
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
                await conn.execute(
                    "INSERT OR IGNORE INTO county (name, state_id) VALUES (?, ?)",
                    (county, state_id),
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
