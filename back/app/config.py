import importlib
import os
import tomllib
from types import ModuleType
from typing import Any, Type, TypeVar

from pydantic import BaseModel


def load_config(path: str) -> dict[str, Any]:
    """Load configuration from a TOML file."""
    assert os.path.exists(path)

    with open(path, "rb") as f:
        config = tomllib.load(f)
    return config


config = load_config("config.toml")
T = TypeVar("T", bound=BaseModel)


def get_package_config(package_name: str | None, type_: Type[T]) -> T:
    """
    Get configuration for a specific package and validate it against a Pydantic model.
    Args:
        package_name: The name of the package to get the configuration for.
        type_: A Pydantic model class to validate the configuration against.
    Returns:
        An instance of the Pydantic model with the configuration data.
    """
    assert issubclass(type_, BaseModel)
    assert package_name is not None

    package_layers: list[str] = package_name.removeprefix("app.").split(".")
    cfg = config
    for pkg in package_layers:
        cfg = cfg.get(pkg, {})
    return type_(**cfg)
