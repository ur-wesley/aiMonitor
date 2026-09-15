# Run from PowerShell:
#   powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1

$ErrorActionPreference = "Stop"

if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
    Write-Error "WSL is required. Install it: wsl --install"
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$wslScript = Join-Path $scriptDir "setup-wsl.sh"
$wslPath = wsl wslpath -a $wslScript

Write-Host "Running Origin clone via WSL..."
wsl bash "$wslPath"

if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "Next: build and run aiMonitor"
Write-Host "  mise install"
Write-Host "  bun install"
Write-Host "  bun tauri build"
Write-Host "  .\src-tauri\target\release\aiMonitor.exe"
