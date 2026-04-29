"""JSONL training data utilities."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List


@dataclass(slots=True)
class EntityAnnotation:
    start: int
    end: int
    type: str


@dataclass(slots=True)
class TrainingExample:
    text: str
    intent: str
    entities: List[EntityAnnotation]


def load_jsonl(path: str | Path) -> List[TrainingExample]:
    """Load training examples from JSONL."""

    examples: List[TrainingExample] = []
    with Path(path).open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip():
                continue
            payload = json.loads(line)
            entities = [
                EntityAnnotation(
                    start=int(entity["start"]),
                    end=int(entity["end"]),
                    type=str(entity["type"]),
                )
                for entity in payload.get("entities", [])
            ]
            examples.append(
                TrainingExample(
                    text=str(payload["text"]),
                    intent=str(payload["intent"]),
                    entities=entities,
                )
            )
    return examples


def load_texts_and_labels(path: str | Path) -> tuple[list[str], list[str]]:
    """Return plain text and intent labels."""

    rows = load_jsonl(path)
    return [row.text for row in rows], [row.intent for row in rows]


def iter_entity_types(examples: Iterable[TrainingExample]) -> list[str]:
    """Collect unique entity labels."""

    values = {entity.type for example in examples for entity in example.entities}
    return sorted(values)
