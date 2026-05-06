#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "========================================"
echo "Chatbot Model Training"
echo "========================================"
echo

echo "Training requires the .venv311 Python environment."
echo

if [ ! -f "nlp-api/.venv311/bin/python" ]; then
    echo "ERROR: .venv311 not found. Run install.sh first."
    exit 1
fi

PYTHON="nlp-api/.venv311/bin/python"

echo "[1/3] Training intent classifier..."
echo "   Command: python -m training.train_intent_classifier ..."
echo
cd nlp-api
$PYTHON -m training.train_intent_classifier \
    --train training/data/restaurant/restaurant_train.jsonl \
    --validation training/data/restaurant/restaurant_validation.jsonl \
    --output artifacts/restaurant_intent \
    --model-name nreimers/MiniLM-L6-H384-uncased

cd "$SCRIPT_DIR/.."
echo "   Intent training complete."

echo
echo "[2/3] Training NER model..."
echo "   Command: python -m training.train_ner_model ..."
echo
cd nlp-api
$PYTHON -m training.train_ner_model \
    --train training/data/restaurant/restaurant_train.jsonl \
    --validation training/data/restaurant/restaurant_validation.jsonl \
    --output artifacts/restaurant_ner \
    --model-name nreimers/MiniLM-L6-H384-uncased

cd "$SCRIPT_DIR/.."
echo "   NER training complete."

echo
echo "[3/3] Running evaluation..."
echo "   Command: python -m training.evaluate ..."
echo
cd nlp-api
$PYTHON -m training.evaluate \
    --intent-model artifacts/restaurant_intent \
    --ner-model artifacts/restaurant_ner \
    --dataset training/data/restaurant/restaurant_eval.jsonl \
    --output artifacts/eval_results.json

cd "$SCRIPT_DIR/.."
echo "   Evaluation complete."

echo
echo "========================================"
echo "Training complete!"
echo "========================================"
echo
echo "Artifacts:"
echo "   - nlp-api/artifacts/restaurant_intent"
echo "   - nlp-api/artifacts/restaurant_ner"
echo "   - nlp-api/artifacts/eval_results.json"
echo