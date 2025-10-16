from sqlalchemy import Column, Date, Float, Integer, String

from ..base import Base, TableNameProvider


class CovidData(Base, TableNameProvider):
    id = Column(Integer, primary_key=True, index=True)
    date = Column(Date, nullable=False, index=True)
    us_county_id = Column(String, nullable=True)
    us_county_name = Column(String, nullable=True)
    province = Column(String, nullable=True)
    country = Column(String, nullable=True)
    confirmed = Column(Integer, nullable=True)
    deaths = Column(Integer, nullable=True)
    recovered = Column(Integer, nullable=True)
    active = Column(Integer, nullable=True)
    incident_rate = Column(Float, nullable=True)
    case_fatality_ratio = Column(Float, nullable=True)
