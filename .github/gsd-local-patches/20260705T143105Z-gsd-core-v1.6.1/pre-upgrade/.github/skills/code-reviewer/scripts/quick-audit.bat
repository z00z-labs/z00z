@echo off
REM Quick Security Audit Script (Windows)
REM Runs basic SAST tools available in most Node.js projects
REM Usage: scripts\quick-audit.bat

echo Running Quick Security Audit...
echo =================================
echo.

setlocal enabledelayedexpansion
set CRITICAL_ISSUES=0
set HIGH_ISSUES=0

REM 1. npm audit
echo Running npm audit...
where npm >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    npm audit --audit-level=high
    if !ERRORLEVEL! EQU 0 (
        echo [32mNo high/critical vulnerabilities found[0m
    ) else (
        echo [31mnpm audit found vulnerabilities[0m
        set /a CRITICAL_ISSUES+=1
    )
) else (
    echo [33mnpm not found, skipping[0m
)
echo.

REM 2. ESLint
echo Running ESLint...
where npx >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    if exist ".eslintrc.js" (
        npx eslint . --ext .js,.jsx,.ts,.tsx --max-warnings 10
        if !ERRORLEVEL! EQU 0 (
            echo [32mESLint passed[0m
        ) else (
            echo [31mESLint found issues[0m
            set /a HIGH_ISSUES+=1
        )
    ) else (
        echo [33mESLint not configured, skipping[0m
    )
) else (
    echo [33mnpx not found, skipping[0m
)
echo.

REM 3. Prettier check
echo Running Prettier...
where npx >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    if exist ".prettierrc" (
        npx prettier --check . 2>nul
        if !ERRORLEVEL! EQU 0 (
            echo [32mCode formatting is consistent[0m
        ) else (
            echo [33mCode formatting inconsistencies found (auto-fixable)[0m
            echo   Run: npx prettier --write .
        )
    ) else (
        echo [33mPrettier not configured, skipping[0m
    )
) else (
    echo [33mnpx not found, skipping[0m
)
echo.

REM 4. TypeScript type check
echo Running TypeScript type check...
where npx >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    if exist "tsconfig.json" (
        npx tsc --noEmit
        if !ERRORLEVEL! EQU 0 (
            echo [32mTypeScript type check passed[0m
        ) else (
            echo [31mTypeScript type errors found[0m
            set /a HIGH_ISSUES+=1
        )
    ) else (
        echo [33mTypeScript not configured, skipping[0m
    )
) else (
    echo [33mnpx not found, skipping[0m
)
echo.

REM 5. Check for hardcoded secrets (basic findstr)
echo Checking for hardcoded secrets...
findstr /S /I /R "api_key.*=.*[\"'][a-zA-Z0-9]\{20,\}[\"'] password.*=.*[\"'][a-zA-Z0-9]\{20,\}[\"'] secret.*=.*[\"'][a-zA-Z0-9]\{20,\}[\"']" src\*.* 2>nul
if !ERRORLEVEL! EQU 0 (
    echo [31mPotential hardcoded secrets found![0m
    echo   Review the files above and move secrets to environment variables
    set /a CRITICAL_ISSUES+=1
) else (
    echo [32mNo obvious hardcoded secrets detected[0m
)
echo.

REM Final summary
echo =================================
echo Audit Summary
echo =================================
echo.

if !CRITICAL_ISSUES! GTR 0 (
    echo [31mCRITICAL ISSUES: !CRITICAL_ISSUES![0m
    echo    Must be fixed before deployment
)

if !HIGH_ISSUES! GTR 0 (
    echo [33mHIGH PRIORITY ISSUES: !HIGH_ISSUES![0m
    echo    Should be fixed within 48 hours
)

if !CRITICAL_ISSUES! EQU 0 if !HIGH_ISSUES! EQU 0 (
    echo [32mALL CHECKS PASSED[0m
    echo    Code is ready for review
)

echo.
echo For detailed analysis, consider running:
echo   - sonar-scanner (if SonarQube configured)
echo   - codeql database analyze (if CodeQL installed)
echo   - snyk test (if Snyk configured)
echo.

REM Exit with error code if critical issues found
if !CRITICAL_ISSUES! GTR 0 exit /b 1

exit /b 0
