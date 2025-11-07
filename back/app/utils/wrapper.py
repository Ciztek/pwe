import functools
import time


def async_timed(func):
    """Decorator to time an async function using perf_counter."""

    @functools.wraps(func)
    async def wrapper(*args, **kwargs):
        start = time.perf_counter()
        result = await func(*args, **kwargs)
        end = time.perf_counter()
        print(f"[{func.__name__}] Execution time: {end - start:.6f} seconds")
        return result

    return wrapper
