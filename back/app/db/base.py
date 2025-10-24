import sqlite3

from pydantic import BaseModel

from ..config import get_package_config


class Settings(BaseModel):
    database_name: str = "app.db"


settings = get_package_config(__package__, Settings)


def get_db_connection():
    conn = sqlite3.connect(settings.database_name)

    conn.row_factory = sqlite3.Row  # Dict like access to rows
    try:
        yield conn
    finally:
        conn.close()


def initialize_database():
    conn = sqlite3.connect(settings.database_name)
    cursor = conn.cursor()

    cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS EntryPoint (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date DATE NOT NULL,
            us_county_id TEXT,
            us_county_name TEXT,
            province TEXT,
            country TEXT NOT NULL,
            confirmed INTEGER,
            deaths INTEGER,
            recovered INTEGER,
            active INTEGER,
            incident_rate REAL,
            case_fatality_ratio REAL
        )
    """
    )

    conn.commit()
    conn.close()
