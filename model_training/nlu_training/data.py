"""Torch dataset and collator for multi-task NLU training."""

from __future__ import annotations

from typing import Any

import torch
from torch.utils.data import Dataset

from nlu_training.schema import TrainingExample
from nlu_training.tagging import encode_example


class NluDataset(Dataset):
    def __init__(
        self,
        examples: list[TrainingExample],
        tokenizer: Any,
        intent_label2id: dict[str, int],
        ner_label2id: dict[str, int],
        max_length: int,
    ) -> None:
        self.rows = [
            encode_example(
                example=example,
                tokenizer=tokenizer,
                intent_label2id=intent_label2id,
                ner_label2id=ner_label2id,
                max_length=max_length,
            )
            for example in examples
        ]

    def __len__(self) -> int:
        return len(self.rows)

    def __getitem__(self, index: int) -> dict[str, Any]:
        return self.rows[index]


class NluDataCollator:
    def __init__(self, tokenizer: Any) -> None:
        self.tokenizer = tokenizer

    def __call__(self, features: list[dict[str, Any]]) -> dict[str, torch.Tensor]:
        features = [dict(feature) for feature in features]
        tagged_texts = [feature.pop("tagged_text", None) for feature in features]
        ner_labels = [feature.pop("ner_labels") for feature in features]
        intent_labels = [feature.pop("labels") for feature in features]
        batch = self.tokenizer.pad(features, padding=True, return_tensors="pt")

        max_length = batch["input_ids"].shape[1]
        padded_ner_labels = [
            labels + [-100] * (max_length - len(labels))
            for labels in ner_labels
        ]
        batch["labels"] = torch.tensor(intent_labels, dtype=torch.long)
        batch["ner_labels"] = torch.tensor(padded_ner_labels, dtype=torch.long)
        if all(value is not None for value in tagged_texts):
            batch["tagged_text"] = tagged_texts
        return batch
