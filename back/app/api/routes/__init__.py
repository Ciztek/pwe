from http import HTTPStatus

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from . import hello

routers = [hello.router]

__all__ = ("routers",)
