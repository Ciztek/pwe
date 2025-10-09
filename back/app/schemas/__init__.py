from pydantic import BaseModel

from .data import DataOutput


class SimpleMessage(BaseModel):
    message: str


__all__ = ("SimpleMessage", "DataOutput")
