from csv import DictReader, DictWriter
from datetime import datetime, timedelta
from pathlib import Path
from typing import TypedDict

from pydantic import BaseModel

from .config import get_package_config
from .utils import daterange, log_time


class Config(BaseModel):
    raw_data_path: str
    preprocess_data_path: str


settings = get_package_config(__package__, Config)


@log_time
def preprocess_data(
    start_date: datetime = datetime(2021, 1, 1),
    end_date: datetime = datetime(2023, 3, 9),
):
    data_dir = Path(settings.raw_data_path).resolve()
    data: list = []
    for date in daterange(start_date, end_date, timedelta(days=1)):
        file_path = data_dir / f"{date.strftime('%m-%d-%Y')}.csv"

        if not file_path.exists():
            print(f"File {file_path} does not exist, skipping...")
            continue
        print(f"Processing file: {file_path}")
        with open(file_path, "r", encoding="utf-8") as f:
            reader = DictReader(f)
            data.extend(
                [{**row, "date": int(date.timestamp())} for row in reader]
            )

    if not data:
        print("No data to preprocess.")
        return []

    keys = data[0].keys()
    with open(
        settings.preprocess_data_path, "w", newline="", encoding="utf-8"
    ) as out_f:
        writer = DictWriter(out_f, fieldnames=keys)
        writer.writeheader()
        writer.writerows(data)
    return data
