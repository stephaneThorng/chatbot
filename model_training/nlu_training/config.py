"""Configuration loading for NLU training."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


def load_config(path: str | Path = "config.yaml") -> dict[str, Any]:
    config_path = Path(path)
    with config_path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def build_label_maps(labels: list[str]) -> tuple[dict[str, int], dict[int, str]]:
    label2id = {label: index for index, label in enumerate(labels)}
    id2label = {index: label for label, index in label2id.items()}
    return label2id, id2label


def build_ner_labels(entity_labels: list[str]) -> list[str]:
    return ["O"] + [f"{prefix}-{label}" for label in entity_labels for prefix in ("B", "I")]
