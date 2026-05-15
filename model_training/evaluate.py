"""Evaluate a trained multi-task NLU model."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import torch
from seqeval.metrics import classification_report, f1_score
from sklearn.metrics import accuracy_score, classification_report as intent_classification_report
from transformers import AutoConfig, AutoTokenizer

from nlu_training.config import build_label_maps, build_ner_labels, load_config
from nlu_training.data import NluDataCollator, NluDataset
from nlu_training.model import MultiTaskNluModel
from nlu_training.schema import load_jsonl, validate_examples


def json_safe(value: Any) -> Any:
    if isinstance(value, dict):
        return {str(key): json_safe(item) for key, item in value.items()}
    if isinstance(value, list):
        return [json_safe(item) for item in value]
    if hasattr(value, "item"):
        return value.item()
    return value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="config.yaml")
    parser.add_argument("--model-dir", default="outputs/restaurant_xlmr")
    parser.add_argument("--dataset")
    parser.add_argument("--output")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    config = load_config(args.config)
    dataset_path = args.dataset or config["data"]["eval"]
    output_path = Path(args.output or Path(args.model_dir) / "eval_metrics.json")
    examples = load_jsonl(dataset_path)
    validate_examples(examples, config)

    intent_labels = list(config["intents"]["labels"])
    ner_labels = build_ner_labels(list(config["entities"]["labels"]))
    intent_label2id, intent_id2label = build_label_maps(intent_labels)
    ner_label2id, ner_id2label = build_label_maps(ner_labels)
    tokenizer = AutoTokenizer.from_pretrained(args.model_dir)
    model_config = AutoConfig.from_pretrained(args.model_dir)
    model = MultiTaskNluModel.from_pretrained(args.model_dir, config=model_config)
    model.eval()

    dataset = NluDataset(examples, tokenizer, intent_label2id, ner_label2id, int(config["model"]["max_length"]))
    collator = NluDataCollator(tokenizer)

    intent_predictions: list[int] = []
    intent_truth: list[int] = []
    ner_predictions: list[list[str]] = []
    ner_truth: list[list[str]] = []

    with torch.no_grad():
        for row in dataset:
            batch = collator([row])
            batch.pop("tagged_text", None)
            outputs = model(**batch)
            intent_predictions.append(int(outputs.logits.argmax(dim=-1).item()))
            intent_truth.append(int(batch["labels"].item()))

            predicted_ner_ids = outputs.ner_logits.argmax(dim=-1).squeeze(0).tolist()
            truth_ner_ids = batch["ner_labels"].squeeze(0).tolist()
            row_predicted: list[str] = []
            row_truth: list[str] = []
            for predicted_id, truth_id in zip(predicted_ner_ids, truth_ner_ids):
                if truth_id == -100:
                    continue
                row_predicted.append(ner_id2label[int(predicted_id)])
                row_truth.append(ner_id2label[int(truth_id)])
            ner_predictions.append(row_predicted)
            ner_truth.append(row_truth)

    metrics: dict[str, Any] = {
        "intent_accuracy": float(accuracy_score(intent_truth, intent_predictions)),
        "intent_report": intent_classification_report(
            intent_truth,
            intent_predictions,
            labels=list(range(len(intent_labels))),
            target_names=intent_labels,
            zero_division=0,
            output_dict=True,
        ),
        "ner_f1": float(f1_score(ner_truth, ner_predictions)),
        "ner_report": classification_report(ner_truth, ner_predictions, zero_division=0, output_dict=True),
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    safe_metrics = json_safe(metrics)
    output_path.write_text(json.dumps(safe_metrics, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(safe_metrics, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
