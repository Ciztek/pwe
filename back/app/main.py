from contextlib import asynccontextmanager
from csv import DictReader
from pathlib import Path

from fastapi import FastAPI
from pydantic import BaseModel
from uvicorn import run

from .config import get_package_config
from .data import preprocess_data


class Config(BaseModel):
    raw_data_path: str
    preprocess_data_path: str


settings = get_package_config(__package__, Config)


@asynccontextmanager
async def lifespan(_: FastAPI):
    if not Path(settings.preprocess_data_path).exists():
        data = preprocess_data()
    else:
        with open(settings.preprocess_data_path, "r") as f:
            data: list = list(DictReader(f))
    print(len(data))
    yield


app = FastAPI(docs_url="/docs", lifespan=lifespan)


def main():
    run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
