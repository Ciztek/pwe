#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build PWE Karaoke MSI installer for Windows

.DESCRIPTION
    This script automates the build process for PWE Karaoke, including:
    - Checking for Rust toolchain installation
    - Installing Rust if needed
    - Checking for WiX Toolset
    - Building the release binary
    - Creating the MSI installer

.PARAMETER SkipRustInstall
    Skip automatic Rust installation if not found

.PARAMETER SkipBuild
    Skip building the release binary and use existing one

.PARAMETER Clean
    Clean build artifacts before building

.EXAMPLE
    .\build-msi.ps1
    Build the MSI installer with all checks

.EXAMPLE
    .\build-msi.ps1 -Clean
    Clean and rebuild everything
#>

param(
    [switch]$SkipRustInstall,
    [switch]$SkipBuild,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Color output functions
function Write-Success { param($Message) Write-Host "✓ $Message" -ForegroundColor Green }
function Write-Info { param($Message) Write-Host "ℹ $Message" -ForegroundColor Cyan }
function Write-Warning { param($Message) Write-Host "⚠ $Message" -ForegroundColor Yellow }
function Write-ErrorMsg { param($Message) Write-Host "✗ $Message" -ForegroundColor Red }
function Write-Step { param($Message) Write-Host "`n==> $Message" -ForegroundColor Magenta }

# Get project root
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ProjectRoot

Write-Host @"

╔═══════════════════════════════════════════════════════╗
║     PWE Karaoke - MSI Installer Build Script          ║
╚═══════════════════════════════════════════════════════╝

"@ -ForegroundColor Cyan

# ============================================================================
# Step 1: Check for Rust installation
# ============================================================================
Write-Step "Checking for Rust toolchain..."

$RustInstalled = $false
$CargoPath = $null

# Check common Rust installation paths
$PossibleCargoPaths = @(
    "$env:USERPROFILE\.cargo\bin\cargo.exe",
    "$env:CARGO_HOME\bin\cargo.exe",
    "C:\Users\$env:USERNAME\.cargo\bin\cargo.exe",
    "D:\cargo\bin\cargo.exe",
    "D:\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\cargo.exe",
    "D:\rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
)

foreach ($Path in $PossibleCargoPaths) {
    if (Test-Path $Path) {
        $CargoPath = $Path
        $RustInstalled = $true
        Write-Success "Found Rust at: $CargoPath"

        # Get Rust version
        $RustVersion = & $CargoPath --version
        Write-Info "Cargo version: $RustVersion"
        break
    }
}

# Try to find cargo in PATH
if (-not $RustInstalled) {
    $CargoInPath = Get-Command cargo -ErrorAction SilentlyContinue
    if ($CargoInPath) {
        $CargoPath = $CargoInPath.Source
        $RustInstalled = $true
        Write-Success "Found Rust in PATH: $CargoPath"
        $RustVersion = & cargo --version
        Write-Info "Cargo version: $RustVersion"
    }
}

# Offer to install Rust if not found
if (-not $RustInstalled) {
    Write-Warning "Rust toolchain not found!"

    if ($SkipRustInstall) {
        Write-ErrorMsg "Rust installation skipped. Cannot proceed without Rust."
        exit 1
    }

    Write-Host ""
    Write-Host "Rust is required to build PWE Karaoke." -ForegroundColor Yellow
    Write-Host "Would you like to install Rust now? (Y/N): " -NoNewline -ForegroundColor Yellow
    $Response = Read-Host

    if ($Response -eq 'Y' -or $Response -eq 'y') {
        Write-Step "Installing Rust toolchain..."
        Write-Info "Downloading rustup-init.exe..."

        $RustupUrl = "https://win.rustup.rs/x86_64"
        $RustupPath = "$env:TEMP\rustup-init.exe"

        try {
            Invoke-WebRequest -Uri $RustupUrl -OutFile $RustupPath -UseBasicParsing
            Write-Success "Downloaded rustup-init.exe"

            Write-Info "Running Rust installer (this may take a few minutes)..."
            Write-Info "Please follow the on-screen instructions..."
            & $RustupPath -y

            # Update PATH for current session
            $PossibleNewPaths = @(
                "$env:USERPROFILE\.cargo\bin",
                "D:\cargo\bin"
            )

            foreach ($NewPath in $PossibleNewPaths) {
                if (Test-Path "$NewPath\cargo.exe") {
                    $env:PATH += ";$NewPath"
                    $CargoPath = "$NewPath\cargo.exe"
                    break
                }
            }

            if (-not $CargoPath) {
                $CargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
            }

            if (Test-Path $CargoPath) {
                Write-Success "Rust installed successfully!"
                $RustVersion = & $CargoPath --version
                Write-Info "Cargo version: $RustVersion"
                $RustInstalled = $true
            } else {
                Write-ErrorMsg "Rust installation failed. Please install manually from: https://rustup.rs/"
                exit 1
            }
        }
        catch {
            Write-ErrorMsg "Failed to install Rust: $_"
            Write-Info "Please install Rust manually from: https://rustup.rs/"
            exit 1
        }
    }
    else {
        Write-ErrorMsg "Cannot proceed without Rust. Please install from: https://rustup.rs/"
        exit 1
    }
}

# ============================================================================
# Step 2: Check for WiX Toolset
# ============================================================================
Write-Step "Checking for WiX Toolset..."

$WixInstalled = $false
$CandlePath = $null
$LightPath = $null

# Check for WiX Toolset
$PossibleWixPaths = @(
    "C:\Program Files (x86)\WiX Toolset v3.14\bin",
    "C:\Program Files (x86)\WiX Toolset v3.13\bin",
    "C:\Program Files (x86)\WiX Toolset v3.11\bin",
    "C:\Program Files\WiX Toolset v3.14\bin"
)

foreach ($Path in $PossibleWixPaths) {
    $TestCandle = Join-Path $Path "candle.exe"
    $TestLight = Join-Path $Path "light.exe"

    if ((Test-Path $TestCandle) -and (Test-Path $TestLight)) {
        $CandlePath = $TestCandle
        $LightPath = $TestLight
        $WixInstalled = $true
        Write-Success "Found WiX Toolset at: $Path"
        break
    }
}

# Try PATH
if (-not $WixInstalled) {
    $CandleInPath = Get-Command candle.exe -ErrorAction SilentlyContinue
    $LightInPath = Get-Command light.exe -ErrorAction SilentlyContinue

    if ($CandleInPath -and $LightInPath) {
        $CandlePath = $CandleInPath.Source
        $LightPath = $LightInPath.Source
        $WixInstalled = $true
        Write-Success "Found WiX Toolset in PATH"
    }
}

if (-not $WixInstalled) {
    Write-ErrorMsg "WiX Toolset not found!"
    Write-Info "Please download and install WiX Toolset from:"
    Write-Info "https://wixtoolset.org/releases/"
    Write-Info "After installation, restart PowerShell and run this script again."
    exit 1
}

# ============================================================================
# Step 3: Parse project version from Cargo.toml
# ============================================================================
Write-Step "Reading project configuration..."

$CargoToml = Get-Content "Cargo.toml" -Raw
if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
    Write-Success "Project version: $Version"
} else {
    Write-ErrorMsg "Could not parse version from Cargo.toml"
    exit 1
}

# ============================================================================
# Step 4: Clean build artifacts (if requested)
# ============================================================================
if ($Clean) {
    Write-Step "Cleaning build artifacts..."

    if (Test-Path "target") {
        Write-Info "Removing target directory..."
        & $CargoPath clean
        Write-Success "Build artifacts cleaned"
    }
}

# ============================================================================
# Step 5: Build release binary
# ============================================================================
if (-not $SkipBuild) {
    Write-Step "Building release binary..."
    Write-Info "This may take several minutes on first build..."

    try {
        & $CargoPath build --release

        if ($LASTEXITCODE -ne 0) {
            Write-ErrorMsg "Build failed with exit code $LASTEXITCODE"
            exit 1
        }

        $ExePath = "target\release\pwe-karaoke.exe"
        if (Test-Path $ExePath) {
            $ExeSize = [math]::Round((Get-Item $ExePath).Length / 1MB, 2)
            Write-Success "Release binary built successfully!"
            Write-Info "Binary: $ExePath ($ExeSize MB)"
        } else {
            Write-ErrorMsg "Binary not found at expected location: $ExePath"
            exit 1
        }
    }
    catch {
        Write-ErrorMsg "Build failed: $_"
        exit 1
    }
} else {
    Write-Step "Skipping build (using existing binary)..."

    $ExePath = "target\release\pwe-karaoke.exe"
    if (Test-Path $ExePath) {
        $ExeSize = [math]::Round((Get-Item $ExePath).Length / 1MB, 2)
        Write-Success "Using existing binary: $ExePath ($ExeSize MB)"
    } else {
        Write-ErrorMsg "No existing binary found. Remove -SkipBuild flag to build."
        exit 1
    }
}

# ============================================================================
# Step 6: Create MSI installer with WiX
# ============================================================================
Write-Step "Building MSI installer..."

# Ensure output directory exists
$MsiOutputDir = "target\release\bundle\msi"
$WixObjDir = "target\wix"

New-Item -ItemType Directory -Force -Path $MsiOutputDir | Out-Null
New-Item -ItemType Directory -Force -Path $WixObjDir | Out-Null

# Compile WiX source
Write-Info "Compiling WiX source file..."
$WixObjFile = "$WixObjDir\main.wixobj"

try {
    & $CandlePath `
        -dCargoTargetBinDir="target\release" `
        -dVersion="$Version" `
        -arch x64 `
        -out $WixObjFile `
        "wix\main.wxs"

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "WiX compilation failed with exit code $LASTEXITCODE"
        exit 1
    }

    Write-Success "WiX source compiled"
}
catch {
    Write-ErrorMsg "WiX compilation failed: $_"
    exit 1
}

# Link to create MSI
Write-Info "Linking MSI installer..."
$MsiFile = "$MsiOutputDir\pwe-karaoke_${Version}_x64_en-US.msi"

try {
    & $LightPath `
        -out $MsiFile `
        $WixObjFile `
        -ext WixUIExtension `
        -spdb

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "MSI linking failed with exit code $LASTEXITCODE"
        exit 1
    }

    Write-Success "MSI installer created"
}
catch {
    Write-ErrorMsg "MSI linking failed: $_"
    exit 1
}

# ============================================================================
# Step 7: Display results
# ============================================================================
Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║           BUILD COMPLETED SUCCESSFULLY!               ║" -ForegroundColor Green
Write-Host "╚═══════════════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""

if (Test-Path $MsiFile) {
    $MsiSize = [math]::Round((Get-Item $MsiFile).Length / 1MB, 2)
    $MsiDate = (Get-Item $MsiFile).LastWriteTime

    Write-Host "MSI Installer Details:" -ForegroundColor Cyan
    Write-Host "  Location: $MsiFile" -ForegroundColor White
    Write-Host "  Size:     $MsiSize MB" -ForegroundColor White
    Write-Host "  Created:  $MsiDate" -ForegroundColor White
    Write-Host ""

    Write-Host "To install, run:" -ForegroundColor Yellow
    Write-Host "  msiexec /i `"$MsiFile`"" -ForegroundColor White
    Write-Host ""

    Write-Host "Or double-click the MSI file in File Explorer." -ForegroundColor Gray
} else {
    Write-ErrorMsg "MSI file not found at expected location!"
    exit 1
}

Write-Host ""
