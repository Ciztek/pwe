{
  lib,
  buildPythonPackage,
  poetry-core,
  pytestCheckHook,
  fetchPypi,
  norbert,

  httpx,
  pandas,
  typer,
  tensorflow,
  ffmpeg-python,
  librosa,
  numpy,
  numba,
  ...
}:

buildPythonPackage rec {
  pname = "spleeter";
  version = "2.4.2";
  pyproject = true;

  src = fetchPypi {
    inherit pname version;
    sha256 = "sha256-5W1IMs+gAdk3usI3NJcWtuNXnKI4YlBKUcEMI5PZ8/A=";
  };


  buildInputs = [ poetry-core ];
  build-system = [poetry-core];


  dependencies = [
    tensorflow
    numpy
    ffmpeg-python
    norbert
    librosa
    numba

    pandas
    httpx
    typer
  ];

  pythonRelaxDeps = ["typer" "httpx" "tensorflow"];

  pythonImportsCheck = ["spleeter"];

  nativeCheckInputs = [pytestCheckHook];

  meta = {
    # homepage = "https://github.com/porimol/countryinfo";
    # description = "Data about countries, ISO info and states/provinces within them";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [
      cizniarova
    ];
  };
}
