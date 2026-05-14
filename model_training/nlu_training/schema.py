"""Dataset schema and validation."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True, slots=True)
class EntitySpan:
    start: int
    end: int
    type: str


@dataclass(frozen=True, slots=True)
class TrainingExample:
    text: str
    lang: str
    domain: str
    intent: str
    entities: tuple[EntitySpan, ...]
    task: str | None = None


class DatasetValidationError(ValueError):
    """Raised when a dataset row does not match the configured schema."""


def load_jsonl(path: str | Path) -> list[TrainingExample]:
    examples: list[TrainingExample] = []
    with Path(path).open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip():
                continue
            payload = json.loads(line)
            try:
                entities = tuple(
                    EntitySpan(
                        start=int(entity["start"]),
                        end=int(entity["end"]),
                        type=str(entity["type"]),
                    )
                    for entity in payload.get("entities", [])
                )
                task_value = payload.get("task")
                examples.append(
                    TrainingExample(
                        text=str(payload["text"]),
                        lang=str(payload["lang"]),
                        domain=str(payload["domain"]),
                        intent=str(payload["intent"]),
                        entities=entities,
                        task=str(task_value) if task_value is not None else None,
                    )
                )
            except KeyError as exc:
                raise DatasetValidationError(f"{path}:{line_number}: missing field {exc}") from exc
    return examples


def write_jsonl(path: str | Path, rows: list[dict[str, Any]]) -> None:
    output_path = Path(path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False) + "\n")


def validate_examples(examples: list[TrainingExample], config: dict[str, Any]) -> None:
    intent_labels = set(config["intents"]["labels"])
    entity_labels = set(config["entities"]["labels"])
    intent_labels_by_domain = {
        domain: set(labels) for domain, labels in config["intents"].get("domains", {}).items()
    }
    entity_labels_by_domain = {
        domain: set(labels) for domain, labels in config["entities"].get("domains", {}).items()
    }
    languages = set(config["tags"]["languages"])
    domains = set(config["tags"]["domains"])
    tasks = set(config["tags"]["tasks"])
    workflow_intents = {"reservation_create", "reservation_cancel"}
    workflow_choice_intents = {"affirmative", "negative", "unknown"}

    for index, example in enumerate(examples, start=1):
        prefix = f"row {index}"
        if not example.text.strip():
            raise DatasetValidationError(f"{prefix}: text must not be blank")
        if example.lang not in languages:
            raise DatasetValidationError(f"{prefix}: unsupported lang {example.lang!r}")
        if example.domain not in domains:
            raise DatasetValidationError(f"{prefix}: unsupported domain {example.domain!r}")
        if example.intent not in intent_labels:
            raise DatasetValidationError(f"{prefix}: unsupported intent {example.intent!r}")
        if example.domain in intent_labels_by_domain and example.intent not in intent_labels_by_domain[example.domain]:
            raise DatasetValidationError(
                f"{prefix}: intent {example.intent!r} is not allowed for domain {example.domain!r}"
            )
        if example.task is not None and example.task not in tasks:
            raise DatasetValidationError(f"{prefix}: unsupported task {example.task!r}")
        if example.task == "WF_RESERVATION_CREATE" and example.intent not in {
            "reservation_create",
            "cancel",
            "unknown",
        }:
            raise DatasetValidationError(
                f"{prefix}: WF_RESERVATION_CREATE only supports reservation_create, cancel, or unknown"
            )
        if example.task == "WF_RESERVATION_CANCEL" and example.intent not in {
            "reservation_cancel",
            "cancel",
            "unknown",
        }:
            raise DatasetValidationError(
                f"{prefix}: WF_RESERVATION_CANCEL only supports reservation_cancel, cancel, or unknown"
            )
        if example.task == "WF_CHOICE" and example.intent not in workflow_choice_intents:
            raise DatasetValidationError(
                f"{prefix}: WF_CHOICE only supports affirmative, negative, or unknown"
            )
        if example.task is None and example.intent == "cancel":
            raise DatasetValidationError(f"{prefix}: cancel requires an active workflow task")
        if example.intent in workflow_intents and example.task is not None and not example.entities:
            raise DatasetValidationError(
                f"{prefix}: workflow slot-collection rows must carry entities"
            )

        previous_end = -1
        for entity in sorted(example.entities, key=lambda value: (value.start, value.end)):
            if entity.type not in entity_labels:
                raise DatasetValidationError(f"{prefix}: unsupported entity type {entity.type!r}")
            if example.domain in entity_labels_by_domain and entity.type not in entity_labels_by_domain[example.domain]:
                raise DatasetValidationError(
                    f"{prefix}: entity {entity.type!r} is not allowed for domain {example.domain!r}"
                )
            if entity.start < 0 or entity.end <= entity.start or entity.end > len(example.text):
                raise DatasetValidationError(f"{prefix}: invalid span {entity.start}:{entity.end}")
            if entity.start < previous_end:
                raise DatasetValidationError(f"{prefix}: overlapping entity span at {entity.start}:{entity.end}")
            if not example.text[entity.start : entity.end].strip():
                raise DatasetValidationError(f"{prefix}: entity span must not be blank")
            previous_end = entity.end
