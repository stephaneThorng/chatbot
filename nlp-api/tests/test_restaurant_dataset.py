from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

from training.data_loader import load_jsonl


DATA_DIR = Path("training/data/restaurant")


def _read_lines(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def test_restaurant_dataset_files_exist_and_parse() -> None:
    train = DATA_DIR / "restaurant_train.jsonl"
    validation = DATA_DIR / "restaurant_validation.jsonl"
    evaluation = DATA_DIR / "restaurant_eval.jsonl"
    corpus = DATA_DIR / "restaurant_corpus.jsonl"

    assert train.exists()
    assert validation.exists()
    assert evaluation.exists()
    assert corpus.exists()

    assert len(load_jsonl(train)) == 240
    assert len(load_jsonl(validation)) == 30
    assert len(load_jsonl(evaluation)) == 30
    assert len(load_jsonl(corpus)) == 300


def test_restaurant_dataset_entity_spans_match_text() -> None:
    for path in DATA_DIR.glob("*.jsonl"):
        for payload in _read_lines(path):
            text = payload["text"]
            assert isinstance(text, str)
            for entity in payload["entities"]:
                extracted = text[entity["start"] : entity["end"]]
                assert extracted


def test_restaurant_dataset_has_no_duplicate_texts() -> None:
    for path in DATA_DIR.glob("*.jsonl"):
        payloads = _read_lines(path)
        texts = [payload["text"] for payload in payloads]
        assert len(texts) == len(set(texts))


def test_restaurant_dataset_splits_cover_all_intents() -> None:
    expected = {
        "reservation_create",
        "reservation_modify",
        "reservation_cancel",
        "menu_request",
        "opening_hours",
        "location_request",
        "pricing_request",
        "greeting_contact",
    }
    for name in ("restaurant_train.jsonl", "restaurant_validation.jsonl", "restaurant_eval.jsonl"):
        payloads = _read_lines(DATA_DIR / name)
        intents = {payload["intent"] for payload in payloads}
        assert intents == expected


def test_restaurant_dataset_contains_all_entity_types() -> None:
    payloads = _read_lines(DATA_DIR / "restaurant_corpus.jsonl")
    entity_types = {
        entity["type"]
        for payload in payloads
        for entity in payload["entities"]
    }
    assert entity_types == {
        "DATE",
        "TIME",
        "PEOPLE_COUNT",
        "PERSON",
        "PHONE",
        "EMAIL",
        "MENU_ITEM",
        "PRICE_ITEM",
        "LOCATION",
    }


def test_restaurant_dataset_distribution_matches_plan() -> None:
    payloads = _read_lines(DATA_DIR / "restaurant_corpus.jsonl")
    counts = Counter(payload["intent"] for payload in payloads)
    assert counts == {
        "reservation_create": 50,
        "reservation_modify": 35,
        "reservation_cancel": 25,
        "menu_request": 40,
        "opening_hours": 35,
        "location_request": 35,
        "pricing_request": 35,
        "greeting_contact": 45,
    }
