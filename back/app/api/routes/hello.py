from fastapi import APIRouter

from ...schemas import SimpleMessage

router = APIRouter(prefix="/hello")


@router.get("")
async def example_json():
    return SimpleMessage(message="Hello, World!")
