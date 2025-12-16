{ pkgs, pythonVersion ? "python310" }:

let
  python = pkgs.${pythonVersion};

  # Helper for PyPI packages
  mkPypi = { pname, version, sha256, propagatedBuildInputs ? [] }:
    python.pkgs.buildPythonPackage {
      inherit pname version propagatedBuildInputs;
      src = pkgs.fetchPypi {
        inherit pname version sha256;
      };
    };

  # --- Pinned Dependencies ---
  # httpx_019 = mkPypi {
  #   pname = "httpx";
  #   version = "0.19.0";
  #   sha256 = "";
  # };

  # pandas_1 = mkPypi {
  #   pname = "pandas";
  #   version = "1.5.3";
  #   # propagatedBuildInputs = [ numpy_1 ];
  #   sha256 = "";
  # };

  # typer_03 = mkPypi {
  #   pname = "typer";
  #   version = "0.3.2";
  #   sha256 = "";
  # };

  # tensorflow_212 = mkPypi {
  #   pname = "tensorflow";
  #   version = "2.12.1";
  #   sha256 = "";
  # };

  # tf_io_gcs = mkPypi {
  #   pname = "tensorflow-io-gcs-filesystem";
  #   version = "0.32.0";
  #   sha256 = "";
  # };

  # --- Local packages ---
  # norbert = python.pkgs.buildPythonPackage {
  #   pname = "norbert";
  #   version = "local";
  #   src = ./norbert;
  # };

  # spleeter = python.pkgs.buildPythonPackage {
  #   pname = "spleeter";
  #   version = "local";
  #   src = ./spleeter;
  #   propagatedBuildInputs = [
  #     pandas_1
  #     httpx_019
  #     typer_03
  #     tensorflow_212
  #     tf_io_gcs
  #     norbert
  #   ];
  # };

in python.withPackages (_: [
  # pandas_1
  # httpx_019
  # typer_03
  # tensorflow_212
  # tf_io_gcs
  # norbert
  # spleeter
])
