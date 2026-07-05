@echo off
REM Code Refactoring Skill - Stop File Watcher (Windows)

setlocal

REM Script directory
set "SCRIPT_DIR=%~dp0"

REM PID file location
set "PID_FILE=%SCRIPT_DIR%watcher.pid"

echo.
echo ╔════════════════════════════════════════════════════════════╗
echo ║  Stopping Code Refactoring File Watcher...                 ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Check if PID file exists
if not exist "%PID_FILE%" (
    echo [WARNING] File watcher is not running (no PID file found^)
    echo.
    exit /b 0
)

REM Read PID from file
set /p WATCHER_PID=<"%PID_FILE%"

REM Check if process is actually running
tasklist /FI "PID eq %WATCHER_PID%" 2>NUL | find /I /N "node.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo [INFO] Found running watcher with PID: %WATCHER_PID%
    echo [INFO] Stopping process...

    REM Kill the process
    taskkill /PID %WATCHER_PID% /F >nul 2>&1

    if "%ERRORLEVEL%"=="0" (
        echo [SUCCESS] File watcher stopped successfully!
    ) else (
        echo [WARNING] Failed to stop process, but continuing cleanup...
    )
) else (
    echo [WARNING] Process not found (may have already stopped^)
)

REM Clean up PID file
del "%PID_FILE%" 2>NUL

echo [INFO] Cleanup complete
echo.

endlocal
