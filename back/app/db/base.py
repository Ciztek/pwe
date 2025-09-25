import re

from pydantic import BaseModel
from sqlalchemy.ext.asyncio import (
    AsyncSession,
    async_sessionmaker,
    create_async_engine,
)
from sqlalchemy.orm import declarative_base

from ..config import get_package_config


class Config(BaseModel):
    uri: str


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


async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
