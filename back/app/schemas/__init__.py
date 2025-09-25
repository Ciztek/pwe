from pydantic import BaseModel


class SimpleMessage(BaseModel):
    message: str


__all__ = ("SimpleMessage",)
