from __future__ import annotations

import re
from collections import Counter

from generate_dataset import INTENT_ORDER, build_english_rows, split_rows
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


def test_generator_rows_cover_language_and_validate() -> None:
    config = load_config("config.yaml")
    rows = build_english_rows()
    examples = to_examples(rows)

    validate_examples(examples, config)

    assert {row["lang"] for row in rows} == {"en"}
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
    rows = build_english_rows()

    assert len(rows) == 1500
    keys = [(row["domain"], row["lang"], row.get("task"), row["intent"], row["text"]) for row in rows]
    assert len(keys) == len(set(keys))
    assert Counter(row["lang"] for row in rows) == {"en": 1500}


def test_generator_covers_reference_and_conversational_dates() -> None:
    rows = build_english_rows()
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


def test_generator_covers_structured_price_conditions() -> None:
    rows = build_english_rows()

    conditional_rows = [
        row
        for row in rows
        if row["intent"] in {"ask_price", "ask_menu_general"}
        and {"price_comparator", "price_amount"}.issubset({entity["type"] for entity in row["entities"]})
    ]

    assert len(conditional_rows) > 60
    assert any(row["intent"] == "ask_price" for row in conditional_rows)
    assert any(row["intent"] == "ask_menu_general" for row in conditional_rows)


def test_english_generator_does_not_emit_known_broken_patterns() -> None:
    rows = build_english_rows()
    texts = [row["text"] for row in rows]
    forbidden_patterns = [
        r"\bcan you I\b",
        r"\bI need to it is\b",
        r"\bI want to know\b",
        r"\bMy email is\b",
        r"\bfor dinner\b",
    ]

    for text in texts:
        for pattern in forbidden_patterns:
            assert not re.search(pattern, text), text


def test_generator_covers_reservation_create_without_entities() -> None:
    rows = build_english_rows()
    zero_entity_rows = [
        row
        for row in rows
        if row["intent"] == "reservation_create"
        and row.get("task") is None
        and not row["entities"]
    ]

    assert len(zero_entity_rows) >= 8
    assert any(row["text"] == "book a reservation" for row in zero_entity_rows)
    assert any(row["text"] == "i want to book" for row in zero_entity_rows)
    assert any("book a table for me" in row["text"] for row in zero_entity_rows)


def test_generator_covers_bare_people_count_slot_replies() -> None:
    rows = build_english_rows()
    workflow_rows = [
        row
        for row in rows
        if row["intent"] == "reservation_create"
        and row.get("task") == "WF_RESERVATION_CREATE"
    ]
    texts = {row["text"] for row in workflow_rows}

    assert {"4", "10", "for 4", "for 10", "for 4 people", "for 10 people"}.issubset(texts)


def test_generator_covers_choice_short_forms() -> None:
    rows = build_english_rows()
    choice_rows = [
        row for row in rows if row.get("task") == "WF_CHOICE"
    ]
    affirmative = {row["text"] for row in choice_rows if row["intent"] == "affirmative"}
    negative = {row["text"] for row in choice_rows if row["intent"] == "negative"}

    assert {"y", "yes", "Yes", "okay"}.issubset(affirmative)
    assert {"n", "no", "No", "nope"}.issubset(negative)


def test_split_rows_keep_intents_sorted_in_output() -> None:
    train, validation, eval_rows = split_rows(build_english_rows())
    order = {intent: index for index, intent in enumerate(INTENT_ORDER)}

    for split in (train, validation, eval_rows):
        split_order = [order[row["intent"]] for row in split]
        assert split_order == sorted(split_order)
