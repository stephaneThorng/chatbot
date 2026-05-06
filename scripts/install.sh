#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "========================================"
echo "Chatbot Test Environment Setup"
echo "========================================"
echo

FAILED=0

echo "[1/4] Checking Java 21..."
if ! command -v java &> /dev/null; then
    echo "   ERROR: Java not found. Please install JDK 21."
    FAILED=1
else
    JAVA_VERSION=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)
    if [ "$JAVA_VERSION" != "21" ]; then
        echo "   WARNING: Found Java $JAVA_VERSION, expected 21."
    else
        echo "   OK: Java 21 found."
    fi
fi

echo
echo "[2/4] Checking Python 3.11..."
PYTHON_CMD=""
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
fi
if [ -z "$PYTHON_CMD" ]; then
    echo "   ERROR: Python not found. Please install Python 3.11."
    FAILED=1
else
    PYVER=$($PYTHON_CMD --version 2>&1)
    echo "   Found: $PYVER"
fi

if [ $FAILED -eq 1 ]; then
    echo
    echo "========================================"
    echo "Setup FAILED"
    echo "========================================"
    exit 1
fi

echo
echo "[3/4] Checking runtime venv (.venv311)..."
if [ ! -f "nlp-api/.venv311/bin/python" ]; then
    echo "   Creating .venv311 with Python 3.11..."
    cd nlp-api
    $PYTHON_CMD -m venv .venv311
    cd ..
fi

echo "   Installing dependencies..."
nlp-api/.venv311/bin/pip install -r nlp-api/requirements.txt --quiet

echo
echo "[4/4] Downloading spaCy model..."
nlp-api/.venv311/bin/python -m spacy download en_core_web_sm --quiet 2>/dev/null || echo "   WARNING: Could not download spaCy model."

echo
echo "========================================"
echo "Setup complete!"
echo "========================================"
echo
echo "To launch the chatbot:"
echo "   ./scripts/launch.sh"
echo
echo "To train models:"
echo "   ./scripts/train.sh"
echo