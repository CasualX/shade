@echo off
setlocal

set SCRIPT_DIR=%~dp0
for %%I in ("%SCRIPT_DIR%..\..") do set ROOT_DIR=%%~fI

pushd "%ROOT_DIR%"
if errorlevel 1 exit /b 1

cargo build --release --package webgl --target wasm32-unknown-unknown
if errorlevel 1 (
	popd
	exit /b 1
)

copy /Y "%ROOT_DIR%\target\wasm32-unknown-unknown\release\webgl.wasm" "%SCRIPT_DIR%html\webgl.wasm" >nul
if errorlevel 1 (
	popd
	exit /b 1
)

echo Copied webgl.wasm to %SCRIPT_DIR%html\webgl.wasm
popd
