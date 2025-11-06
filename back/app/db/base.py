import re
from csv import DictReader
from datetime import date, timedelta
from pathlib import Path

from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import (
    AsyncSession,
    async_sessionmaker,
    create_async_engine,
)
from sqlalchemy.orm import declarative_base

from ..config import get_package_config


class Config(BaseModel):
    uri: str
    data_path: str


settings = get_package_config(__package__, Config)

Base = declarative_base()


class TableNameProvider:
    def __init_subclass__(cls, **kwargs):
        cls.__tablename__ = re.sub(
            r"(?<!^)(?=[A-Z])", "_", cls.__name__
        ).lower()
        super().__init_subclass__(**kwargs)


engine = create_async_engine(settings.uri, echo=True)

async_session = async_sessionmaker(
    bind=engine,
    expire_on_commit=False,
    class_=AsyncSession,
)


async def get_session():
    async with async_session() as session:
        yield session


async def fill_db(db: AsyncSession):
    from .models import DataPoint

    start_date = date(2021, 1, 1)
    end_date = date(2023, 3, 9)
    current = start_date

    result = await db.execute(select(DataPoint.date).distinct())
    existing_dates = {row for row in result.scalars().all()}

    batch_size = 30
    file_count = 0

    while current <= end_date:
        if current in existing_dates:
            current += timedelta(days=1)
            continue

        date_str = current.strftime("%m-%d-%Y")
        csv_path = Path(
            settings.data_path
            + f"/csse_covid_19_data/csse_covid_19_daily_reports/{date_str}.csv"
        )
        if csv_path.exists():
            with open(csv_path, newline="", encoding="utf-8") as f:
                reader = DictReader(f)
                dict_records = [
                    {
                        "date": current,
                        "us_county_id": r.get("FIPS") or None,
                        "us_county_name": r.get("Admin2") or None,
                        "province": r.get("Province_State")
                        or r.get("Province/State")
                        or None,
                        "country": r.get("Country_Region")
                        or r.get("Country/Region"),
                        "confirmed": int(r.get("Confirmed") or 0),
                        "deaths": int(r.get("Deaths") or 0),
                        "recovered": (
                            int(r.get("Recovered"))
                            if r.get("Recovered") not in (None, "")
                            else None
                        ),
                        "active": (
                            int(r.get("Active"))
                            if r.get("Active") not in (None, "")
                            else None
                        ),
                        "incident_rate": (
                            float(r.get("Incident_Rate"))
                            if r.get("Incident_Rate") not in (None, "")
                            else None
                        ),
                        "case_fatality_ratio": (
                            float(r.get("Case_Fatality_Ratio"))
                            if r.get("Case_Fatality_Ratio") not in (None, "")
                            else None
                        ),
                        "lat": (
                            float(r.get("Lat"))
                            if r.get("Lat") not in (None, "")
                            else None
                        ),
                        "long": (
                            float(r.get("Long_"))
                            if r.get("Long_") not in (None, "")
                            else None
                        ),
                    }
                    for r in reader
                ]

                if dict_records:
                    await db.run_sync(
                        lambda s: s.bulk_insert_mappings(
                            DataPoint.__mapper__, dict_records
                        )
                    )

        file_count += 1
        if file_count % batch_size == 0:
            await db.commit()

        current += timedelta(days=1)

    await db.commit()


async def place_db(db: AsyncSession):
    from sqlalchemy import distinct, select

    from .models import Continent, Country, County, DataPoint, State

    continent_map = {
        "US": "North America",
        "Canada": "North America",
        "Mexico": "North America",
        "Brazil": "South America",
        "Argentina": "South America",
        "France": "Europe",
        "Germany": "Europe",
        "Italy": "Europe",
        "Spain": "Europe",
        "China": "Asia",
        "Japan": "Asia",
        "India": "Asia",
        "Australia": "Oceania",
        "New Zealand": "Oceania",
        "South Africa": "Africa",
        "Egypt": "Africa",
        "Other": "Unknown",
    }

    print("[place_db] Building location hierarchy...")

    continents_cache: dict[str, Continent] = {
        getattr(c, "name"): c
        for c in (await db.execute(select(Continent))).scalars().all()
    }

    countries_cache: dict[tuple[str, str], Country] = {}
    for country in (await db.execute(select(Country))).scalars().all():
        continent = await db.get(Continent, country.continent_id)
        if continent:
            countries_cache[
                (getattr(continent, "name"), getattr(country, "name"))
            ] = country

    states_cache: dict[tuple[str, str], State] = {}
    for state in (await db.execute(select(State))).scalars().all():
        country = await db.get(Country, state.country_id)
        if country:
            states_cache[
                (getattr(country, "name"), getattr(state, "name"))
            ] = state

    counties_cache: dict[tuple[str, str], County] = {}
    for county in (await db.execute(select(County))).scalars().all():
        state = await db.get(State, county.state_id)
        if state:
            counties_cache[
                (getattr(state, "name"), getattr(county, "name"))
            ] = county

    # Fetch distinct country/province/county combos
    result = await db.execute(
        select(
            distinct(DataPoint.country),
            DataPoint.province,
            DataPoint.us_county_name,
        )
    )
    rows = result.all()

    # Process rows into hierarchy
    for country, province, county in rows:
        if not country:
            continue

        if province == "Unknown":
            province = None

        continent_name = continent_map.get(country, "Unknown")

        # --- Continent ---
        continent = continents_cache.get(continent_name)
        if not continent:
            continent = Continent(name=continent_name)
            db.add(continent)
            await db.flush()
            continents_cache[continent_name] = continent

        # --- Country ---
        country_key = (getattr(continent, "name"), country)
        country_obj = countries_cache.get(country_key)
        if not country_obj:
            country_obj = Country(name=country, continent_id=continent.id)
            db.add(country_obj)
            await db.flush()
            countries_cache[country_key] = country_obj

        # --- State / Province ---
        state_obj = None
        if province:
            state_key = (country, province)
            state_obj = states_cache.get(state_key)
            if not state_obj:
                state_obj = State(name=province, country_id=country_obj.id)
                db.add(state_obj)
                await db.flush()
                states_cache[state_key] = state_obj

        # --- County ---
        if county and state_obj and country == "US":
            county_key = (province, county)
            if county_key not in counties_cache:
                county_obj = County(name=county, state_id=state_obj.id)
                db.add(county_obj)
                counties_cache[county_key] = county_obj

    await db.commit()
    print("[place_db] Hierarchical place data updated successfully.")


async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
        await fill_db(AsyncSession(conn))
        await place_db(AsyncSession(conn))
