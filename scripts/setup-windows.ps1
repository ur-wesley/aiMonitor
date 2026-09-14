# Run from PowerShell: irm https://origin.cursor.com/... OR run locally:
#   powershell -ExecutionPolicy Bypass -File C:\Arbeit\aiMonitor\scripts\setup-windows.ps1

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
Write-Host "  cd C:\Arbeit\aiMonitor"
Write-Host "  dotnet publish -c Release -r win-x64 --self-contained -p:PublishReadyToRun=true"
Write-Host "  .\bin\Release\net10.0\win-x64\publish\aiMonitor.exe"
