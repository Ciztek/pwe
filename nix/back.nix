{
  lib,
  python3Packages,
}:
python3Packages.buildPythonApplication {
  name = "EpiCoBack";
  version = "0.0.1";
  pyproject = true;

  src = ../back;

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
    description = "Backend for the Covid19-Dataviz project";
    license = lib.licenses.bsd3;
    maintainers = with lib.maintainers; [cizniarova];
    mainProgram = "EpiCoBack";
  };
}
