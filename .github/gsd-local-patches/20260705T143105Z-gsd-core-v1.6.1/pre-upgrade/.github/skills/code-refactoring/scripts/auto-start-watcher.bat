@echo off
REM Code Refactoring Skill - Auto-start Helper (Windows)
REM Silently starts watcher if not already running
REM Designed for use with generic session-start hooks

setlocal enabledelayedexpansion

REM Script directory
set "SCRIPT_DIR=%~dp0"

REM PID file location
set "PID_FILE=%SCRIPT_DIR%watcher.pid"

REM Default directory (can be overridden with argument)
set "WATCH_DIR=%~1"
if "%WATCH_DIR%"=="" set "WATCH_DIR=%CD%"

REM Check if already running
if exist "%PID_FILE%" (
    set /p WATCHER_PID=<"%PID_FILE%"
    tasklist /FI "PID eq !WATCHER_PID!" 2>NUL | find /I /N "node.exe">NUL
    if "!ERRORLEVEL!"=="0" (
        REM Already running, exit silently
        exit /b 0
    ) else (
        REM Stale PID file, clean up
        del "%PID_FILE%" 2>NUL
    )
)

REM Check if Node.js is installed
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    REM Node.js not installed, exit silently
    exit /b 0
)

REM Start watcher silently in background
start /B cmd /C "node "%SCRIPT_DIR%file-watcher.js" "%WATCH_DIR%" --quiet >> "%SCRIPT_DIR%watcher.log" 2>&1"

REM Wait for process to start
timeout /t 1 /nobreak >nul 2>&1

REM Get and save PID
for /f "tokens=2" %%a in ('tasklist /FI "IMAGENAME eq node.exe" /FO LIST ^| find "PID:"') do (
    set "NEW_PID=%%a"
    goto :save_pid
)

:save_pid
if defined NEW_PID (
    echo %NEW_PID% > "%PID_FILE%"
)

endlocal
exit /b 0
