# Building PWE Karaoke MSI Installer on Windows

## Quick Start

Simply run the PowerShell build script:

```powershell
.\build-msi.ps1
```

That's it! The script will:

- ✅ Check for Rust toolchain (install if needed)
- ✅ Check for WiX Toolset (prompt if missing)
- ✅ Build the release binary
- ✅ Create the MSI installer

## Prerequisites

### WiX Toolset (Required)

Download and install WiX Toolset 3.14 from:
<https://wixtoolset.org/releases/>

### Rust Toolchain (Optional - script can install)

The build script can automatically install Rust if it's not found. Alternatively, install manually from:
<https://rustup.rs/>

## Script Options

```powershell
# Full build (default)
.\build-msi.ps1

# Use existing binary (skip Rust build)
.\build-msi.ps1 -SkipBuild

# Clean and rebuild everything
.\build-msi.ps1 -Clean

# Skip automatic Rust installation
.\build-msi.ps1 -SkipRustInstall

# Combine options
.\build-msi.ps1 -Clean -SkipBuild
```

## Output Location

The MSI installer will be created at:

```pwsh
target\release\bundle\msi\pwe-karaoke_0.1.0_x64_en-US.msi
```

## Installation

### Option 1: Double-click

Double-click the `.msi` file in File Explorer

### Option 2: Command line

```powershell
msiexec /i "target\release\bundle\msi\pwe-karaoke_0.1.0_x64_en-US.msi"
```

### Option 3: Silent install

```powershell
msiexec /i "target\release\bundle\msi\pwe-karaoke_0.1.0_x64_en-US.msi" /qn
```

## Troubleshooting

### "WiX Toolset not found"

- Download and install from <https://wixtoolset.org/releases/>
- Restart PowerShell after installation
- Ensure `candle.exe` and `light.exe` are in your PATH

### "Rust toolchain not found"

- Run the script again and choose 'Y' when prompted to install Rust
- Or install manually from <https://rustup.rs/>
- Restart PowerShell after installation

### "Build failed"

- Check that all dependencies in `Cargo.toml` are available
- Ensure you have internet connection for first build (downloads dependencies)
- Try running with `-Clean` flag to rebuild from scratch

### "Cannot run script" / Execution Policy Error

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
.\build-msi.ps1
```

## What Gets Installed

The MSI installer:

- Installs PWE Karaoke to `C:\Program Files\pwe-karaoke\`
- Optionally adds the executable to system PATH
- Creates Start Menu shortcuts
- Registers for proper uninstallation via Windows Settings

## Updating Version

To release a new version:

1. Update version in `Cargo.toml`:

   ```toml
   [package]
   version = "0.2.0"

   [package.metadata.bundle]
   version = "0.2.0"
   ```

2. Run the build script:

   ```powershell
   .\build-msi.ps1 -Clean
   ```

3. New MSI will be: `pwe-karaoke_0.2.0_x64_en-US.msi`

## CI/CD Integration

For automated builds (GitHub Actions, etc.):

```yaml
- name: Build MSI Installer
  shell: pwsh
  run: |
    .\build-msi.ps1
```

## Additional Resources

- Project Documentation: `docs/`
- WiX Configuration: `wix/main.wxs`
- Installer Build Guide: `installer/README.md`
