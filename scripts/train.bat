@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Chatbot Model Training
echo ========================================
echo.

set "PROJECT_ROOT=%~dp0.."
cd /d "%PROJECT_ROOT%"

if not exist "%PROJECT_ROOT%\nlp-api\.venv311\Scripts\python.exe" (
    echo ERROR: .venv311 not found. Run install.bat first.
    exit /b 1
)

echo [1/3] Training intent classifier...
echo    Command: python -m training.train_intent_classifier ...
echo.
cd /d "%PROJECT_ROOT%\nlp-api"
call .venv311\Scripts\python.exe -m training.train_intent_classifier --train training\data\restaurant\restaurant_train.jsonl --validation training\data\restaurant\restaurant_validation.jsonl --output artifacts\restaurant_intent --model-name nreimers/MiniLM-L6-H384-uncased
if %errorlevel% neq 0 (
    echo    Training failed.
    cd /d "%PROJECT_ROOT%"
    exit /b 1
)
cd /d "%PROJECT_ROOT%"
echo    Intent training complete.

echo.
echo [2/3] Training NER model...
echo    Command: python -m training.train_ner_model ...
echo.
cd /d "%PROJECT_ROOT%\nlp-api"
call .venv311\Scripts\python.exe -m training.train_ner_model --train training\data\restaurant\restaurant_train.jsonl --validation training\data\restaurant\restaurant_validation.jsonl --output artifacts\restaurant_ner --model-name nreimers/MiniLM-L6-H384-uncased
if %errorlevel% neq 0 (
    echo    Training failed.
    cd /d "%PROJECT_ROOT%"
    exit /b 1
)
cd /d "%PROJECT_ROOT%"
echo    NER training complete.

echo.
echo [3/3] Running evaluation...
echo    Command: python -m training.evaluate ...
echo.
cd /d "%PROJECT_ROOT%\nlp-api"
call .venv311\Scripts\python.exe -m training.evaluate --intent-model artifacts\restaurant_intent --ner-model artifacts\restaurant_ner --dataset training\data\restaurant\restaurant_eval.jsonl --output artifacts\eval_results.json
if %errorlevel% neq 0 (
    echo    Evaluation failed.
    cd /d "%PROJECT_ROOT%"
    exit /b 1
)
cd /d "%PROJECT_ROOT%"
echo    Evaluation complete.

echo.
echo ========================================
echo Training complete!
echo ========================================
echo.
echo Artifacts:
echo   - nlp-api\artifacts\restaurant_intent
echo   - nlp-api\artifacts\restaurant_ner
echo   - nlp-api\artifacts\eval_results.json
echo.