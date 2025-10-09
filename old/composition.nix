{
  androidenv,
  includeNDK ? false,
  includeSystemImages ? false,
  includeEmulator ? false,

  buildToolsVersions ? [ "34.0.0" ],
  platformVersions ? [ "34" "35" ],

  systemImageTypes ? [ "google_apis" ],
  abiVersions ? [ "arm64-v8a" ],

  extraLicenses ? [
    "android-sdk-preview-license"
    "android-googletv-license"
    "android-sdk-arm-dbt-license"
    "google-gdk-license"
    "intel-android-extra-license"
    "intel-android-sysimage-license"
    "mips-android-sysimage-license"
  ]
}:
androidenv.composeAndroidPackages {
  inherit
    includeNDK
    includeSystemImages
    includeEmulator
    buildToolsVersions
    platformVersions
    systemImageTypes
    abiVersions
    extraLicenses;
}
