{
  lib,
  buildPythonPackage,
  fetchFromGitHub,
  poetry-core,
  setuptools,
  scipy,
  pytestCheckHook,
  fetchPypi,
}:
buildPythonPackage rec {
  pname = "norbert";
  version = "0.2.1";
  pyproject = true;

  src = fetchPypi {
    inherit pname version;
    sha256 = "sha256-vUy8JSfwVQuBv0JlwaZLNSyrf3Hk48gj0wtxpzaN504=";
  };

  buildInputs = [
    setuptools
  ];
  # build-system = [poetry-core];

  dependencies = [
    scipy
  ];

  # pythonRelaxDeps = ["typer"];

  pythonImportsCheck = ["norbert"];
  doCheck = false;

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
