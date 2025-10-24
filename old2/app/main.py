from contextlib import asynccontextmanager

from fastapi import FastAPI
from pydantic import BaseModel
from uvicorn import run

from .config import get_package_config
from .data import process_data
from .routes import routers
from .utils import log_time


class Config(BaseModel):
    raw_data_path: str
    preprocess_data_path: str


settings = get_package_config(__package__, Config)


@asynccontextmanager
async def lifespan(_: FastAPI):
    (log_time(lambda: process_data()))()
    yield
    # print(f"Shutting down app at {datetime.now()}")
    # data = next(process_data())
    # french_data = bsearch(data, key="country", target="France")
    # first_date_data = bsearch(french_data, key="date", target=int(datetime(2021, 1, 1).timestamp()))
    # print(f"France regions names at 2021-01-01: {[d['state'] for d in first_date_data]}")


app = FastAPI(docs_url="/docs", lifespan=lifespan)

for router in routers:
    app.include_router(router)


def main():
    run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
