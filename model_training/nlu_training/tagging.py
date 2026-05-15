"""Tagged input and BIO conversion."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from nlu_training.schema import EntitySpan, TrainingExample


@dataclass(frozen=True, slots=True)
class TaggedExample:
    text: str
    entity_spans: tuple[EntitySpan, ...]
    prefix_length: int


def build_tagged_text(example: TrainingExample) -> TaggedExample:
    tags = []
    if example.task is not None:
        tags.append(f"[TASK={example.task}]")
    if example.slot is not None:
        tags.append(f"[SLOT={example.slot}]")
    tags.append(f"[LANG={example.lang}]")
    tags.append(f"[DOMAIN={example.domain}]")
    prefix = " ".join(tags)
    tagged_text = f"{prefix} {example.text}"
    shift = len(prefix) + 1
    shifted_entities = tuple(
        EntitySpan(
            start=entity.start + shift,
            end=entity.end + shift,
            type=entity.type,
        )
        for entity in example.entities
    )
    return TaggedExample(text=tagged_text, entity_spans=shifted_entities, prefix_length=shift)


def align_bio_labels(
    tagged: TaggedExample,
    offset_mapping: list[tuple[int, int]],
    label2id: dict[str, int],
) -> list[int]:
    token_labels: list[int] = []
    entity_token_started: set[int] = set()

    for token_start, token_end in offset_mapping:
        if token_start == token_end:
            token_labels.append(-100)
            continue

        assigned = "O"
        for entity_index, entity in enumerate(tagged.entity_spans):
            if token_end <= entity.start or token_start >= entity.end:
                continue
            prefix = "B" if entity_index not in entity_token_started else "I"
            assigned = f"{prefix}-{entity.type}"
            entity_token_started.add(entity_index)
            break
        token_labels.append(label2id[assigned])

    return token_labels


def encode_example(
    example: TrainingExample,
    tokenizer: Any,
    intent_label2id: dict[str, int],
    ner_label2id: dict[str, int],
    max_length: int,
) -> dict[str, Any]:
    tagged = build_tagged_text(example)
    encoded = tokenizer(
        tagged.text,
        truncation=True,
        max_length=max_length,
        return_offsets_mapping=True,
    )
    offsets = [(int(start), int(end)) for start, end in encoded.pop("offset_mapping")]
    encoded["labels"] = intent_label2id[example.intent]
    encoded["ner_labels"] = align_bio_labels(tagged, offsets, ner_label2id)
    encoded["tagged_text"] = tagged.text
    return encoded


def debug_bio_row(example: TrainingExample, tokenizer: Any, ner_label2id: dict[str, int], max_length: int) -> dict[str, Any]:
    id2label = {index: label for label, index in ner_label2id.items()}
    tagged = build_tagged_text(example)
    encoded = tokenizer(
        tagged.text,
        truncation=True,
        max_length=max_length,
        return_offsets_mapping=True,
    )
    offsets = [(int(start), int(end)) for start, end in encoded["offset_mapping"]]
    label_ids = align_bio_labels(tagged, offsets, ner_label2id)
    labels = ["IGN" if label_id == -100 else id2label[label_id] for label_id in label_ids]
    return {
        "text": tagged.text,
        "tokens": tokenizer.convert_ids_to_tokens(encoded["input_ids"]),
        "labels": labels,
        "offsets": offsets,
        "intent": example.intent,
    }
