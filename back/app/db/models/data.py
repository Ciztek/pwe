from sqlalchemy import Column, Date, Float, ForeignKey, Integer, String
from sqlalchemy.orm import relationship

from ..base import Base, TableNameProvider


class DataPoint(Base, TableNameProvider):
    id = Column(Integer, primary_key=True, index=True)
    date = Column(Date, nullable=False, index=True)
    us_county_id = Column(String, nullable=True)
    us_county_name = Column(String, nullable=True)
    province = Column(String, nullable=True)
    country = Column(String, nullable=False)
    confirmed = Column(Integer, nullable=False)
    deaths = Column(Integer, nullable=False)
    recovered = Column(Integer, nullable=True)
    active = Column(Integer, nullable=True)
    incident_rate = Column(Float, nullable=True)
    case_fatality_ratio = Column(Float, nullable=True)
    lat = Column(Float, nullable=True)
    lon = Column(Float, nullable=True)


class Continent(Base):
    __tablename__ = "continent"

    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False, unique=True)

    countries = relationship(
        "Country", back_populates="continent", cascade="all, delete-orphan"
    )


class Country(Base):
    __tablename__ = "country"

    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False)
    continent_id = Column(Integer, ForeignKey("continent.id"), nullable=False)

    continent = relationship("Continent", back_populates="countries")
    states = relationship(
        "State", back_populates="country", cascade="all, delete-orphan"
    )


class State(Base):
    __tablename__ = "state"

    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False)
    country_id = Column(Integer, ForeignKey("country.id"), nullable=False)

    country = relationship("Country", back_populates="states")
    counties = relationship(
        "County", back_populates="state", cascade="all, delete-orphan"
    )


class County(Base):
    __tablename__ = "county"

    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False)
    state_id = Column(Integer, ForeignKey("state.id"), nullable=False)

    state = relationship("State", back_populates="counties")
