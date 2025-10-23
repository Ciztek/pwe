from contextlib import asynccontextmanager
from csv import DictReader
from datetime import datetime
from pathlib import Path

from fastapi import FastAPI
from pydantic import BaseModel
from uvicorn import run

from .config import get_package_config
from .data import DailyData, data, preprocess_data


class Config(BaseModel):
    raw_data_path: str
    preprocess_data_path: str


settings = get_package_config(__package__, Config)


@asynccontextmanager
async def lifespan(_: FastAPI):
    global data
    if not Path(settings.preprocess_data_path).exists():
        data = preprocess_data()
    else:
        with open(settings.preprocess_data_path, "r") as f:
            reader = DictReader(f)
            data: list[DailyData] = [
                {
                    "date": int(r["date"]),
                    "country": r["country"],
                    "state": r["state"] or None,
                    "county": r["county"] or None,
                    "lat": float(r["lat"]),
                    "long": float(r["long"]),
                    "total_confirmed": int(r["total_confirmed"]),
                    "total_deaths": int(r["total_deaths"]),
                    "daily_confirmed": int(r["daily_confirmed"]),
                    "daily_deaths": int(r["daily_deaths"]),
                }
                for r in reader
            ]
    yield


app = FastAPI(docs_url="/docs", lifespan=lifespan)


def main():
    run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
