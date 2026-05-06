@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Chatbot Launcher
echo ========================================
echo.

set "PROJECT_ROOT=%~dp0.."
cd /d "%PROJECT_ROOT%"

echo Starting services...
echo.

echo [1/3] Starting NLP API on port 8000...
start "NLP API" /D "%PROJECT_ROOT%\nlp-api" cmd /k ".venv311\Scripts\python.exe -m src.main"
timeout /t 4 >nul
echo    NLP API started. Check the "NLP API" window for status.

echo.
echo [2/3] Starting Ktor backend on port 8080...
start "Ktor Backend" /D "%PROJECT_ROOT%\backend\chatbot" cmd /k "gradlew.bat run"
echo    Waiting for backend to be ready...
:retry
timeout /t 5 >nul
curl.exe -s http://localhost:8080/health >nul 2>&1
if %errorlevel% neq 0 goto :retry
echo    Backend is ready!

echo.
echo ========================================
echo All services running!
echo ========================================
echo.
echo Endpoints:
echo   - NLP API:    http://localhost:8000
echo   - Backend:  http://localhost:8080
echo   - API Doc:  http://localhost:8080/openapi
echo.
echo.

set /p LAUNCH_CLI="Launch chat CLI? (y/n): "
if /i "%LAUNCH_CLI%"=="y" (
    echo Starting CLI...
    start "Chat CLI" /D "%PROJECT_ROOT%\backend\chatbot" cmd /k "gradlew.bat chatCli --console plain -PchatbotApiUrl=http://localhost:8080/api/v1/chat/messages"
)

echo.
echo Press any key to stop servers and exit...
pause >nul

echo.
echo Stopping servers...
taskkill /F /FI "WINDOWTITLE eq NLP API"  >nul 2>&1
taskkill /F /FI "WINDOWTITLE eq Ktor Backend" >nul 2>&1
taskkill /F /FI "WINDOWTITLE eq Chat CLI" >nul 2>&1
taskkill /F /IM python.exe >nul 2>&1
taskkill /F /IM java.exe >nul 2>&1

echo Done.