import sys
import time
from contextlib import asynccontextmanager
from csv import DictReader
from datetime import date, datetime, timedelta
from operator import mod
from pathlib import Path

import uvicorn
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from .config import get_package_config
from .db import init_db
from .routes import routers


@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_db()
    for name, router in routers:
        print(
            f"Registering router from module: {name} with prefix: {router.prefix}"
        )
        app.include_router(router)
    yield


app = FastAPI(docs_url="/docs", lifespan=lifespan)


@app.middleware("http")
async def log_timing_info(request: Request, call_next):
    start_time = time.time()
    start_date = datetime.fromtimestamp(start_time).isoformat()
    print(f"[START] {request.method} {request.url.path} at {start_date}")

    response = await call_next(request)

    end_time = time.time()
    duration = end_time - start_time

    end_date = datetime.fromtimestamp(end_time).isoformat()

    response.headers["X-Start-Time"] = start_date
    response.headers["X-End-Time"] = end_date
    response.headers["X-Duration-Seconds"] = f"{duration * 1000:.2f} ms"

    print(f"[END]   {request.method} {request.url.path} at {end_date}")
    print(f"[DELTA] {duration * 1000:.2f} ms")
    return response


if "dev" in sys.argv:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=("http://localhost" "http://127.0.0.1:*"),
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )


def main():
    uvicorn.run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
