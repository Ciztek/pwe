from csv import DictReader, DictWriter
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, TypedDict

from pydantic import BaseModel

from .config import get_package_config
from .utils import daterange, log_time


class Config(BaseModel):
    raw_data_path: str
    preprocess_data_path: str


settings = get_package_config(__package__, Config)


class DailyData(TypedDict):
    date: int
    country: str
    state: Optional[str]
    county: Optional[str]
    lat: float
    long: float
    total_confirmed: int
    total_deaths: int
    daily_confirmed: int
    daily_deaths: int


@log_time
def preprocess_data(
    start_date: datetime = datetime(2021, 1, 1),
    end_date: datetime = datetime(2023, 3, 9),
) -> List[DailyData]:
    data_dir = Path(settings.raw_data_path).resolve()
    all_data: List[DailyData] = []
    prev_day_map: Dict[tuple[str, Optional[str], Optional[str]], DailyData] = (
        {}
    )

    for date in daterange(start_date, end_date, timedelta(days=1)):
        file_path = data_dir / f"{date.strftime('%m-%d-%Y')}.csv"
        if not file_path.exists():
            print(f"File {file_path} does not exist, skipping...")
            continue

        print(f"Processing file: {file_path}")
        current_day_map: Dict[
            tuple[str, Optional[str], Optional[str]], DailyData
        ] = {}

        with open(file_path, "r", encoding="utf-8") as f:
            reader = DictReader(f)
            for row in reader:
                country = row.get("Country_Region", "").strip()
                state = row.get("Province_State", "").strip() or None
                county = row.get("Admin2", "").strip() or None

                key = (country, state, county)

                confirmed = int(row.get("Confirmed", 0) or 0)
                deaths = int(row.get("Deaths", 0) or 0)

                lat = float(row.get("Lat", 0.0) or 0.0)
                long = float(row.get("Long_", 0.0) or 0.0)

                prev = prev_day_map.get(key)

                daily_confirmed = confirmed - (
                    prev["total_confirmed"] if prev else 0
                )
                daily_deaths = deaths - (prev["total_deaths"] if prev else 0)

                daily_row: DailyData = {
                    "date": int(date.timestamp()),
                    "country": country,
                    "state": state,
                    "county": county,
                    "lat": lat,
                    "long": long,
                    "total_confirmed": confirmed,
                    "total_deaths": deaths,
                    "daily_confirmed": daily_confirmed,
                    "daily_deaths": daily_deaths,
                }

                current_day_map[key] = daily_row
                all_data.append(daily_row)

        prev_day_map = current_day_map

    if not all_data:
        print("No data to preprocess.")
        return []
    all_data.sort(
        key=lambda x: (
            x["date"],
            x["country"],
            x["state"] or "",
            x["county"] or "",
        )
    )
    keys = list(all_data[0].keys())
    with open(
        settings.preprocess_data_path, "w", newline="", encoding="utf-8"
    ) as out_f:
        writer = DictWriter(out_f, fieldnames=keys)
        writer.writeheader()
        writer.writerows(all_data)

    return all_data
