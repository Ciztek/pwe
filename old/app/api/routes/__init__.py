from . import data, hello

routers = [hello.router, data.router]

__all__ = ("routers",)
