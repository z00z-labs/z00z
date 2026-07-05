@echo off
REM Code Refactoring Skill - Start File Watcher (Windows)
REM Usage: start-watcher.bat [directory] [options]

setlocal

REM Default directory is current working directory
set "WATCH_DIR=%~1"
if "%WATCH_DIR%"=="" set "WATCH_DIR=%CD%"

REM Script directory
set "SCRIPT_DIR=%~dp0"

REM PID file location
set "PID_FILE=%SCRIPT_DIR%watcher.pid"

REM Log file location
set "LOG_FILE=%SCRIPT_DIR%watcher.log"

echo.
echo ╔════════════════════════════════════════════════════════════╗
echo ║  Starting Code Refactoring File Watcher...                 ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Check if already running
if exist "%PID_FILE%" (
    set /p OLD_PID=<"%PID_FILE%"
    tasklist /FI "PID eq !OLD_PID!" 2>NUL | find /I /N "node.exe">NUL
    if "!ERRORLEVEL!"=="0" (
        echo [ERROR] File watcher is already running with PID: !OLD_PID!
        echo.
        echo To stop it, run: stop-watcher.bat
        echo To check status, run: watcher-status.bat
        echo.
        exit /b 1
    ) else (
        echo [INFO] Cleaning up stale PID file...
        del "%PID_FILE%" 2>NUL
    )
)

REM Check if Node.js is installed
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Node.js is not installed or not in PATH!
    echo.
    echo Please install Node.js from: https://nodejs.org/
    echo.
    exit /b 1
)

echo [INFO] Starting file watcher for directory: %WATCH_DIR%
echo [INFO] Log file: %LOG_FILE%
echo.

REM Start the watcher in background
start /B cmd /C "node "%SCRIPT_DIR%file-watcher.js" "%WATCH_DIR%" %~2 %~3 %~4 %~5 >> "%LOG_FILE%" 2>&1"

REM Wait a moment for process to start
timeout /t 2 /nobreak >nul

REM Get the PID of the started process
for /f "tokens=2" %%a in ('tasklist /FI "IMAGENAME eq node.exe" /FO LIST ^| find "PID:"') do (
    set "WATCHER_PID=%%a"
    goto :found_pid
)

:found_pid
if defined WATCHER_PID (
    echo %WATCHER_PID% > "%PID_FILE%"
    echo [SUCCESS] File watcher started successfully!
    echo [INFO] PID: %WATCHER_PID%
    echo.
    echo To view live output: tail -f "%LOG_FILE%"
    echo To stop watcher: stop-watcher.bat
    echo To check status: watcher-status.bat
    echo.
) else (
    echo [ERROR] Failed to start file watcher!
    echo Check the log file for details: %LOG_FILE%
    echo.
    exit /b 1
)

endlocal
