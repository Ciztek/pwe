{
  lib,
  python3Packages,
}:
python3Packages.buildPythonApplication {
  name = "covid-dataviz";
  version = "0.0.1";
  pyproject = true;

  src = ./.;

  build-system = [python3Packages.hatchling];

  dependencies = with python3Packages; [
    fastapi
    uvicorn
    sqlalchemy
    passlib
    aiosqlite
  ];

  optional-dependencies = with python3Packages; {
    dev = [
      fastapi-cli
      black
      isort
    ];
  };

  meta = {
    description = "Modular workflow runner with FastAPI, SQLAlchemy, and MariaDB";
    license = lib.licenses.bsd3;
    maintainers = with lib.maintainers; [sigmanificient];
    mainProgram = "area";
  };
}
