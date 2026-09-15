@echo off
setlocal EnableExtensions
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 exit /b 1

set "MSVC_BIN=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64"
set "PATH=%MSVC_BIN%;C:\Users\parac\AppData\Local\mise\dotnet-root;C:\Users\parac\AppData\Local\mise\shims;%SystemRoot%\System32;%SystemRoot%;%SystemRoot%\System32\Wbem;%SystemRoot%\System32\WindowsPowerShell\v1.0"

echo Using:
where link
where dotnet

cd /d D:\projects\aimonitor

if exist global.json.bak del /f /q global.json.bak
copy /y global.json global.json.bak >nul
powershell -NoProfile -Command "(Get-Content -Raw global.json) -replace '10.0.303','10.0.300' | Set-Content -NoNewline global.json"
if errorlevel 1 (
  echo Failed to patch global.json
  exit /b 1
)

if exist artifacts\local-aot rmdir /s /q artifacts\local-aot
mkdir artifacts\local-aot 2>nul

dotnet publish aiMonitor.csproj -c Release -r win-x64 -o artifacts\local-aot -p:IlcUseEnvironmentalTools=true "-p:CppLinker=%MSVC_BIN%\link.exe"
set PUBLISH_EXIT=%ERRORLEVEL%

copy /y global.json.bak global.json >nul
del /f /q global.json.bak >nul

if %PUBLISH_EXIT% NEQ 0 exit /b %PUBLISH_EXIT%

echo.
echo Publish output:
dir /a-d artifacts\local-aot
exit /b 0
