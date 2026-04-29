"""Fine-tune a NER token classifier."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

import yaml
from datasets import Dataset

from training.data_loader import iter_entity_types, load_jsonl


def _import_transformers() -> tuple[Any, Any, Any, Any]:
    from transformers import (
        AutoModelForTokenClassification,
        AutoTokenizer,
        DataCollatorForTokenClassification,
        Trainer,
        TrainingArguments,
    )

    return AutoTokenizer, AutoModelForTokenClassification, DataCollatorForTokenClassification, Trainer, TrainingArguments


def load_training_config() -> dict[str, Any]:
    return yaml.safe_load(Path("training/config.yaml").read_text(encoding="utf-8"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--train", required=True)
    parser.add_argument("--validation", required=True)
    parser.add_argument("--output", default="artifacts/ner")
    parser.add_argument("--push-to-hub", action="store_true")
    parser.add_argument("--hub-model-id")
    args = parser.parse_args()

    config = load_training_config()["ner_training"]
    train_rows = load_jsonl(args.train)
    valid_rows = load_jsonl(args.validation)
    entity_types = iter_entity_types(train_rows + valid_rows)
    labels = ["O"] + [f"{prefix}-{entity}" for entity in entity_types for prefix in ("B", "I")]
    label2id = {label: index for index, label in enumerate(labels)}
    id2label = {index: label for label, index in label2id.items()}

    AutoTokenizer, AutoModelForTokenClassification, DataCollatorForTokenClassification, Trainer, TrainingArguments = _import_transformers()
    tokenizer = AutoTokenizer.from_pretrained(config["model_name"])

    def align(example: dict[str, Any]) -> dict[str, Any]:
        encoded = tokenizer(example["text"], truncation=True, max_length=256, return_offsets_mapping=True)
        labels_for_tokens: list[int] = []
        for start, end in encoded["offset_mapping"]:
            if start == end:
                labels_for_tokens.append(-100)
                continue
            assigned = "O"
            for entity in example["entities"]:
                if start >= entity["start"] and end <= entity["end"]:
                    prefix = "B" if start == entity["start"] else "I"
                    assigned = f"{prefix}-{entity['type']}"
                    break
            labels_for_tokens.append(label2id[assigned])
        encoded["labels"] = labels_for_tokens
        return encoded

    train_dataset = Dataset.from_list(
        [{"text": row.text, "entities": [{"start": entity.start, "end": entity.end, "type": entity.type} for entity in row.entities]} for row in train_rows]
    ).map(align)
    valid_dataset = Dataset.from_list(
        [{"text": row.text, "entities": [{"start": entity.start, "end": entity.end, "type": entity.type} for entity in row.entities]} for row in valid_rows]
    ).map(align)

    model = AutoModelForTokenClassification.from_pretrained(
        config["model_name"],
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
        warmup_steps=int(config["warmup_steps"]),
        weight_decay=float(config["weight_decay"]),
        evaluation_strategy="epoch",
        save_strategy="epoch",
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
        data_collator=DataCollatorForTokenClassification(tokenizer),
    )
    trainer.train()
    trainer.save_model(args.output)
    tokenizer.save_pretrained(args.output)
    if args.push_to_hub:
        trainer.push_to_hub()


if __name__ == "__main__":
    main()
