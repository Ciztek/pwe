from fastapi import APIRouter, Depends

from ..db import get_session

router = APIRouter(prefix="/filter")


@router.get("/place")
async def filter_data(db=Depends(get_session)):
    return {"message": "Filter data endpoint"}
