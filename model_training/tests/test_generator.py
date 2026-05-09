from __future__ import annotations

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
