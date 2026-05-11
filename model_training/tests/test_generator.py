from __future__ import annotations

import re
from collections import Counter

from generate_dataset import build_english_rows, build_indonesian_rows
from nlu_training.config import load_config
from nlu_training.schema import EntitySpan, TrainingExample, validate_examples


def to_examples(rows: list[dict]) -> list[TrainingExample]:
    return [
        TrainingExample(
            text=row["text"],
            lang=row["lang"],
            domain=row["domain"],
            intent=row["intent"],
            task=row.get("task"),
            entities=tuple(EntitySpan(entity["start"], entity["end"], entity["type"]) for entity in row["entities"]),
        )
        for row in rows
    ]


def test_generator_rows_cover_languages_and_validate() -> None:
    config = load_config("config.yaml")
    rows = build_english_rows() + build_indonesian_rows()
    examples = to_examples(rows)

    validate_examples(examples, config)

    assert {row["lang"] for row in rows} == {"en", "id"}
    assert any(
        row.get("task") == "WF_RESERVATION_CREATE" and row["intent"] == "reservation_create"
        for row in rows
    )
    assert any(
        row.get("task") == "WF_RESERVATION_CANCEL" and row["intent"] == "reservation_cancel"
        for row in rows
    )
    assert any(row.get("task") == "WF_CHOICE" and row["intent"] == "affirmative" for row in rows)
    assert any(row["intent"] == "ask_price" for row in rows)
    assert {row["domain"] for row in rows} == {"restaurant"}


def test_generator_produces_expected_volume_without_duplicates() -> None:
    rows = build_english_rows() + build_indonesian_rows()

    assert 4800 <= len(rows) <= 5200
    assert len(rows) == 5000
    keys = [(row["domain"], row["lang"], row.get("task"), row["intent"], row["text"]) for row in rows]
    assert len(keys) == len(set(keys))
    assert Counter(row["lang"] for row in rows) == {"en": 2500, "id": 2500}


def test_generator_covers_reference_and_conversational_dates() -> None:
    rows = build_english_rows() + build_indonesian_rows()
    texts = [row["text"] for row in rows]
    references = [
        text[text.index("REST-") :].split()[0].strip("?.!,")
        for text in texts
        if "REST-" in text
    ]

    assert references
    assert all(re.fullmatch(r"REST-[A-Z0-9]{6,10}", reference) for reference in references)
    assert any("on July 8" in text for text in texts)
    assert any("next Tuesday" in text for text in texts)
    assert any("on August 23 2026" in text for text in texts)
    assert any("pada 8 Juli" in text for text in texts)
    assert any("Selasa depan" in text for text in texts)
    assert any("pada 23 Agustus 2026" in text for text in texts)


def test_generator_covers_structured_price_conditions() -> None:
    rows = build_english_rows() + build_indonesian_rows()

    conditional_rows = [
        row
        for row in rows
        if row["intent"] in {"ask_price", "ask_menu_general"}
        and {"price_comparator", "price_amount"}.issubset({entity["type"] for entity in row["entities"]})
    ]

    assert len(conditional_rows) > 100
    assert any(row["intent"] == "ask_price" for row in conditional_rows)
    assert any(row["intent"] == "ask_menu_general" for row in conditional_rows)


def test_english_generator_does_not_wrap_phrases_ungrammatically() -> None:
    rows = build_english_rows()
    texts = [row["text"] for row in rows]
    forbidden_patterns = [
        r"\bcan you I\b",
        r"\bI need to it is\b",
        r"\bI want to know (Can|Could|Do|Does|Is|Are|What|Which|How|My email)\b",
        r"^please [A-Z][a-z]+ [A-Z][a-z]+ if available$",
        r"^\w+ Can you\b",
        r"\? (today|for tonight|\.)$",
    ]

    for text in texts:
        for pattern in forbidden_patterns:
            assert not re.search(pattern, text), text
