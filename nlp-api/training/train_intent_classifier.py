"""Fine-tune an intent classifier."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

import yaml
from datasets import Dataset

from training.data_loader import load_texts_and_labels


def _import_transformers() -> tuple[Any, Any, Any, Any]:
    from transformers import (
        AutoModelForSequenceClassification,
        AutoTokenizer,
        Trainer,
        TrainingArguments,
    )

    return AutoTokenizer, AutoModelForSequenceClassification, Trainer, TrainingArguments


def load_training_config() -> dict[str, Any]:
    return yaml.safe_load(Path("training/config.yaml").read_text(encoding="utf-8"))


def resolve_warmup_steps(
    sample_count: int,
    batch_size: int,
    num_epochs: int,
    configured_warmup_steps: int,
) -> int:
    """Cap warmup to a sensible fraction of total optimizer steps."""

    steps_per_epoch = max(1, math.ceil(sample_count / batch_size))
    total_steps = max(1, steps_per_epoch * num_epochs)
    capped_warmup = max(0, min(configured_warmup_steps, total_steps // 10))
    return capped_warmup


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--train", required=True)
    parser.add_argument("--validation", required=True)
    parser.add_argument("--output", default="artifacts/intent")
    parser.add_argument("--model-name")
    parser.add_argument("--push-to-hub", action="store_true")
    parser.add_argument("--hub-model-id")
    args = parser.parse_args()

    config = load_training_config()["intent_training"]
    model_name = args.model_name or config["model_name"]
    train_texts, train_labels = load_texts_and_labels(args.train)
    valid_texts, valid_labels = load_texts_and_labels(args.validation)
    labels = sorted(set(train_labels + valid_labels))
    label2id = {label: index for index, label in enumerate(labels)}
    id2label = {index: label for label, index in label2id.items()}

    AutoTokenizer, AutoModelForSequenceClassification, Trainer, TrainingArguments = _import_transformers()
    tokenizer = AutoTokenizer.from_pretrained(model_name)

    def tokenize(batch: dict[str, list[str]]) -> dict[str, Any]:
        encoded = tokenizer(batch["text"], truncation=True, padding="max_length", max_length=256)
        encoded["labels"] = [label2id[label] for label in batch["intent"]]
        return encoded

    train_dataset = Dataset.from_dict({"text": train_texts, "intent": train_labels}).map(tokenize, batched=True)
    valid_dataset = Dataset.from_dict({"text": valid_texts, "intent": valid_labels}).map(tokenize, batched=True)
    effective_warmup_steps = resolve_warmup_steps(
        sample_count=len(train_texts),
        batch_size=int(config["batch_size"]),
        num_epochs=int(config["num_epochs"]),
        configured_warmup_steps=int(config["warmup_steps"]),
    )

    model = AutoModelForSequenceClassification.from_pretrained(
        model_name,
        num_labels=len(labels),
        label2id=label2id,
        id2label=id2label,
    )
    training_args = TrainingArguments(
        output_dir=args.output,
        learning_rate=float(config["learning_rate"]),
        per_device_train_batch_size=int(config["batch_size"]),
        per_device_eval_batch_size=int(config["batch_size"]),
        num_train_epochs=int(config["num_epochs"]),
        warmup_steps=effective_warmup_steps,
        weight_decay=float(config["weight_decay"]),
        evaluation_strategy="epoch",
        save_strategy="epoch",
        logging_strategy="steps",
        logging_steps=25,
        load_best_model_at_end=True,
        push_to_hub=args.push_to_hub,
        hub_model_id=args.hub_model_id,
    )
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=valid_dataset,
        tokenizer=tokenizer,
    )
    trainer.train()
    trainer.save_model(args.output)
    tokenizer.save_pretrained(args.output)
    if args.push_to_hub:
        trainer.push_to_hub()


if __name__ == "__main__":
    main()
