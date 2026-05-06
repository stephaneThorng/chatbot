@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Chatbot Test Environment Setup
echo ========================================
echo.

set "PROJECT_ROOT=%~dp0.."
cd /d "%PROJECT_ROOT%"

set FAILED=0

echo [1/4] Checking Java 21...
java -version 2>&1 | findstr /C:"21" >nul
if %errorlevel% neq 0 (
    echo    ERROR: Java 21 not found. Please install JDK 21.
    set FAILED=1
) else (
    echo    OK: Java 21 found.
)

echo.
echo [2/4] Checking Python 3.11...
where python >nul 2>&1
if %errorlevel% neq 0 (
    echo    ERROR: Python not found. Please install Python 3.11.
    set FAILED=1
) else (
    for /f "delims=" %%v in ('python --version 2^>^&1') do set PYVER=%%v
    echo    Found: !PYVER!
)

if %FAILED% equ 1 goto :failed

echo.
echo [3/4] Checking runtime venv (.venv311)...
if not exist "%PROJECT_ROOT%\nlp-api\.venv311\Scripts\python.exe" (
    echo    Creating .venv311 with Python 3.11...
    cd /d "%PROJECT_ROOT%\nlp-api"
    python -m venv .venv311
    cd /d "%PROJECT_ROOT%"
)

echo    Installing dependencies...
"%PROJECT_ROOT%\nlp-api\.venv311\Scripts\python.exe" -m pip install -r "%PROJECT_ROOT%\nlp-api\requirements.txt" >nul 2>&1
if %errorlevel% neq 0 (
    echo    WARNING: Could not install all dependencies.
)

echo.
echo [4/4] Downloading spaCy model...
"%PROJECT_ROOT%\nlp-api\.venv311\Scripts\python.exe" -m spacy download en_core_web_sm >nul 2>&1
if %errorlevel% neq 0 (
    echo    WARNING: Could not download spaCy model. Continuing...
) else (
    echo    spaCy model installed.
)

echo.
echo ========================================
echo Setup complete!
echo ========================================
echo.
echo To launch the chatbot:
echo   scripts\launch.bat
echo.
echo To train models:
echo   scripts\train.bat
echo.
goto :end

:failed
echo.
echo ========================================
echo Setup FAILED
echo ========================================
echo Please install missing prerequisites and try again.
exit /b 1

:end