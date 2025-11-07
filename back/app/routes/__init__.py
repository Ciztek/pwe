from fastapi import APIRouter

from . import filter, hello

routers: list[tuple[str, APIRouter]] = []

for mod in (filter, hello):
    assert hasattr(
        mod, "router"
    ), f"Module {mod.__name__} is missing 'router' attribute"
    assert isinstance(
        mod.router, APIRouter
    ), f"'router' in module {mod.__name__} is not an APIRouter instance"
    routers.append(("/".join(mod.__name__.split(".")[1:]), mod.router))


__all__ = ("routers",)
