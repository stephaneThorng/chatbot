"""Train a single multi-task NLU model."""

from __future__ import annotations

import argparse
import json
import random
from pathlib import Path
from typing import Any

import torch
import yaml
from sklearn.metrics import accuracy_score, precision_recall_fscore_support
from transformers import AutoTokenizer, Trainer, TrainingArguments, set_seed

from nlu_training.config import build_label_maps, build_ner_labels, load_config
from nlu_training.data import NluDataCollator, NluDataset
from nlu_training.model import MultiTaskNluModel
from nlu_training.schema import load_jsonl, validate_examples
from nlu_training.tagging import debug_bio_row


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="config.yaml")
    parser.add_argument("--train")
    parser.add_argument("--validation")
    parser.add_argument("--output")
    parser.add_argument("--model-name")
    parser.add_argument("--epochs", type=float)
    parser.add_argument("--max-train-samples", type=int)
    parser.add_argument("--max-validation-samples", type=int)
    return parser.parse_args()


def limit_examples(examples: list[Any], limit: int | None, seed: int) -> list[Any]:
    if limit is None or len(examples) <= limit:
        return examples
    rng = random.Random(seed)
    selected = list(examples)
    rng.shuffle(selected)
    return selected[:limit]


def compute_intent_metrics(eval_prediction: Any) -> dict[str, float]:
    predictions, labels = eval_prediction
    if isinstance(predictions, tuple):
        intent_logits = predictions[0]
    else:
        intent_logits = predictions
    if isinstance(labels, tuple):
        labels = labels[0]
    predicted = intent_logits.argmax(axis=-1)
    precision, recall, f1, _ = precision_recall_fscore_support(labels, predicted, average="macro", zero_division=0)
    return {
        "intent_accuracy": float(accuracy_score(labels, predicted)),
        "intent_precision_macro": float(precision),
        "intent_recall_macro": float(recall),
        "intent_f1_macro": float(f1),
    }


def main() -> None:
    args = parse_args()
    config = load_config(args.config)
    seed = int(config["training"]["seed"])
    set_seed(seed)

    train_path = args.train or config["data"]["train"]
    validation_path = args.validation or config["data"]["validation"]
    output_dir = Path(args.output or config["training"]["output_dir"])
    model_name = args.model_name or config["model"]["name"]
    max_length = int(config["model"]["max_length"])

    train_examples = limit_examples(load_jsonl(train_path), args.max_train_samples, seed)
    validation_examples = limit_examples(load_jsonl(validation_path), args.max_validation_samples, seed)
    validate_examples(train_examples, config)
    validate_examples(validation_examples, config)

    intent_labels = list(config["intents"]["labels"])
    ner_labels = build_ner_labels(list(config["entities"]["labels"]))
    intent_label2id, intent_id2label = build_label_maps(intent_labels)
    ner_label2id, ner_id2label = build_label_maps(ner_labels)
    intent_weights = [
        float(config["intents"]["class_weights"].get(label, 1.0))
        for label in intent_labels
    ]

    tokenizer = AutoTokenizer.from_pretrained(model_name)
    train_dataset = NluDataset(train_examples, tokenizer, intent_label2id, ner_label2id, max_length)
    validation_dataset = NluDataset(validation_examples, tokenizer, intent_label2id, ner_label2id, max_length)
    model = MultiTaskNluModel.from_base_model(
        model_name=model_name,
        num_intent_labels=len(intent_labels),
        num_ner_labels=len(ner_labels),
        intent_label2id=intent_label2id,
        intent_id2label=intent_id2label,
        ner_label2id=ner_label2id,
        ner_id2label=ner_id2label,
        intent_class_weights=intent_weights,
    )

    training_config = config["training"]
    training_args = TrainingArguments(
        output_dir=str(output_dir),
        num_train_epochs=float(args.epochs or training_config["num_epochs"]),
        per_device_train_batch_size=int(training_config["batch_size"]),
        per_device_eval_batch_size=int(training_config["batch_size"]),
        learning_rate=float(training_config["learning_rate"]),
        weight_decay=float(training_config["weight_decay"]),
        warmup_ratio=float(training_config["warmup_ratio"]),
        eval_strategy="epoch",
        save_strategy="epoch",
        logging_steps=int(training_config["logging_steps"]),
        load_best_model_at_end=True,
        metric_for_best_model="intent_f1_macro",
        greater_is_better=True,
        remove_unused_columns=False,
        report_to=[],
    )

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=validation_dataset,
        processing_class=tokenizer,
        data_collator=NluDataCollator(tokenizer),
        compute_metrics=compute_intent_metrics,
    )
    trainer.train()
    metrics = trainer.evaluate()

    output_dir.mkdir(parents=True, exist_ok=True)
    trainer.save_model(str(output_dir))
    tokenizer.save_pretrained(str(output_dir))
    (output_dir / "label_maps.json").write_text(
        json.dumps(
            {
                "intent_label2id": intent_label2id,
                "intent_id2label": intent_id2label,
                "ner_label2id": ner_label2id,
                "ner_id2label": ner_id2label,
            },
            indent=2,
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    (output_dir / "training_config.yaml").write_text(yaml.safe_dump(config, sort_keys=False), encoding="utf-8")
    (output_dir / "metrics.json").write_text(json.dumps(metrics, indent=2, sort_keys=True), encoding="utf-8")
    preview = [
        debug_bio_row(example, tokenizer, ner_label2id, max_length)
        for example in validation_examples[:10]
    ]
    (output_dir / "debug_bio_preview.json").write_text(json.dumps(preview, indent=2, ensure_ascii=False), encoding="utf-8")

    print(json.dumps(metrics, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
