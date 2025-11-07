from http import HTTPStatus

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from ..schemas import SimpleMessage

router = APIRouter(prefix="/hello")


@router.get(
    "",
    response_model=SimpleMessage,
    description="Dummy endpoint that reply a greeting",
    responses={
        HTTPStatus.OK: {
            "model": SimpleMessage,
            "content": {
                "application/json": {
                    "example": SimpleMessage(
                        message="Hello, World!"
                    ).model_dump()
                }
            },
        }
    },
)
async def example_json():
    return SimpleMessage(message="Hello, World!")
