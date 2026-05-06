#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "========================================"
echo "Chatbot Launcher"
echo "========================================"
echo

echo "Starting services..."
echo

echo "[1/3] Starting NLP API on port 8000..."
cd nlp-api
.venv311/bin/python -m src.main &
NLP_PID=$!
cd ..

echo "   NLP API started (PID: $NLP_PID)"
sleep 3

echo
echo "[2/3] Starting Ktor backend on port 8080..."
cd backend/chatbot
./gradlew run &
KTOR_PID=$!
cd ../..

echo "   Ktor backend started (PID: $KTOR_PID)"

echo "   Waiting for backend to be ready..."
while ! curl -s http://localhost:8080/health > /dev/null 2>&1; do
    sleep 3
done
echo "   Backend is ready!"

echo
echo "========================================"
echo "All services running!"
echo "========================================"
echo
echo "Endpoints:"
echo "   - NLP API:    http://localhost:8000"
echo "   - Backend:   http://localhost:8080"
echo "   - API Doc:   http://localhost:8080/openapi"
echo
echo

read -p "Launch chat CLI? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Starting CLI..."
    cd backend/chatbot
    ./gradlew chatCli --console plain &
    cd ../..
fi

echo
echo "Press Enter to stop servers..."
read

echo
echo "Stopping servers..."
kill $NLP_PID $KTOR_PID 2>/dev/null || true

echo "Done."