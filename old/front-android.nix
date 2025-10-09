{
  stdenv
, lib
, front-web
, jdk21_headless
, nodejs_20
, pnpm
, gradle
}:
stdenv.mkDerivation (finalAttrs:{
  pname = "EpiCoBile";
  version = "0.0.1";
  src = ../frontend;

  mitmCache = gradle.fetchDeps {
    inherit (finalAttrs) pname;
    # data = ./deps.json;
  };

  nativeBuildInputs = [
    jdk21_headless
    nodejs_20
    pnpm
    gradle
  ];

  gradleBuildTasks = [ "assembleRelease" ];
  gradleFlags = [ "--no-daemon" "--offline" ];

  preBuild = ''
    cp -r ${front-web}/. frontend
  '';


  installPhase = ''
    mkdir -p $out
    cp -r android/app/build/outputs $out/
  '';

  meta = {
    description = "EpiCovid Android app (Capacitor)";
    license = lib.licenses.bsd3;
    maintainers = with lib.maintainers; [ cizniarova ];
  };
})
