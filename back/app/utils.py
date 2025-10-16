from datetime import datetime, timedelta
from inspect import iscoroutine
from time import perf_counter


def log_time(func):
    async def async_wrapper(*args, **kwargs):
        start = perf_counter()
        result = await func(*args, **kwargs)
        end = perf_counter()
        print(f"{func.__name__} executed in {end - start:.4f} seconds")
        return result

    def sync_wrapper(*args, **kwargs):
        start = perf_counter()
        result = func(*args, **kwargs)
        end = perf_counter()
        print(f"{func.__name__} executed in {end - start:.4f} seconds")
        return result

    return async_wrapper if iscoroutine(func) else sync_wrapper


def daterange(start: datetime, end: datetime, step: timedelta):
    current = start
    while current <= end:
        yield current
        current += step
