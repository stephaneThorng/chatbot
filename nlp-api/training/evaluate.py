"""Evaluation helpers for intent and NER models."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from seqeval.metrics import classification_report as ner_report
from sklearn.metrics import classification_report as intent_report

from training.data_loader import load_jsonl


def _import_transformers() -> tuple[Any, Any, Any]:
    from transformers import AutoModelForSequenceClassification, AutoModelForTokenClassification, AutoTokenizer

    return AutoTokenizer, AutoModelForSequenceClassification, AutoModelForTokenClassification


def evaluate_intent_model(model_path: str, dataset_path: str) -> dict[str, Any]:
    """Evaluate an intent model against a JSONL dataset."""

    AutoTokenizer, AutoModelForSequenceClassification, _ = _import_transformers()
    tokenizer = AutoTokenizer.from_pretrained(model_path)
    model = AutoModelForSequenceClassification.from_pretrained(model_path)
    rows = load_jsonl(dataset_path)
    predictions: list[str] = []
    references = [row.intent for row in rows]
    id2label = {int(key): value for key, value in (getattr(model.config, "id2label", {}) or {}).items()}
    import torch

    for row in rows:
        encoded = tokenizer(row.text, return_tensors="pt", truncation=True)
        with torch.no_grad():
            prediction = int(torch.argmax(model(**encoded).logits, dim=-1).item())
        predictions.append(id2label[prediction])
    return intent_report(references, predictions, output_dict=True, zero_division=0)


def evaluate_ner_model(model_path: str, dataset_path: str) -> dict[str, str]:
    """Return seqeval classification report for a token-classification model."""

    AutoTokenizer, _, AutoModelForTokenClassification = _import_transformers()
    tokenizer = AutoTokenizer.from_pretrained(model_path)
    model = AutoModelForTokenClassification.from_pretrained(model_path)
    rows = load_jsonl(dataset_path)
    true_labels: list[list[str]] = []
    pred_labels: list[list[str]] = []
    id2label = {int(key): value for key, value in (getattr(model.config, "id2label", {}) or {}).items()}
    import torch

    for row in rows:
        encoded = tokenizer(row.text, return_offsets_mapping=True, return_tensors="pt", truncation=True)
        offsets = encoded.pop("offset_mapping")[0].tolist()
        with torch.no_grad():
            predictions = torch.argmax(model(**encoded).logits, dim=-1)[0].tolist()
        gold = ["O"] * len(offsets)
        for entity in row.entities:
            for index, (start, end) in enumerate(offsets):
                if start == end:
                    continue
                if start >= entity.start and end <= entity.end:
                    prefix = "B" if start == entity.start else "I"
                    gold[index] = f"{prefix}-{entity.type}"
        pred = [id2label.get(int(label), "O") for label in predictions]
        true_labels.append(gold)
        pred_labels.append(pred)
    return {"report": ner_report(true_labels, pred_labels, zero_division=0)}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--intent-model", required=True)
    parser.add_argument("--ner-model", required=True)
    parser.add_argument("--dataset", required=True)
    parser.add_argument("--output", default="evaluation.json")
    args = parser.parse_args()

    payload = {
        "intent": evaluate_intent_model(args.intent_model, args.dataset),
        "ner": evaluate_ner_model(args.ner_model, args.dataset),
    }
    Path(args.output).write_text(json.dumps(payload, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
