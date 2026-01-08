@echo off
REM PWE Karaoke MSI Builder - Windows Batch Wrapper
REM This file allows double-clicking to build the MSI installer

echo.
echo ================================================
echo   PWE Karaoke - MSI Installer Builder
echo ================================================
echo.

REM Check if PowerShell is available
where pwsh >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Using PowerShell Core...
    pwsh -ExecutionPolicy Bypass -File "%~dp0build-msi.ps1"
) else (
    where powershell >nul 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo Using Windows PowerShell...
        powershell -ExecutionPolicy Bypass -File "%~dp0build-msi.ps1"
    ) else (
        echo ERROR: PowerShell not found!
        echo Please install PowerShell or run build-msi.ps1 directly.
        pause
        exit /b 1
    )
)

echo.
echo ================================================
echo   Build process completed
echo ================================================
echo.
pause
