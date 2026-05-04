from __future__ import annotations

from datetime import date, timedelta

from fastapi.testclient import TestClient


def test_analyze_endpoint_returns_rich_response(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "Please create a completely new booking for 3 on today at 5:30pm; phone 555-0108",
            "domain": "restaurant",
            "context": {"previous_slots": {"name": "Alex"}},
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["name"] == "reservation_create"
    assert "processing_details" in payload
    assert any(entity["type"] in {"DATE", "TIME", "PHONE", "PERSON", "PEOPLE_COUNT"} for entity in payload["entities"])


def test_health_endpoint_returns_status(app) -> None:
    client = TestClient(app)
    response = client.get("/health")
    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] in {"ok", "degraded"}
    assert "models_loaded" in payload


def test_analyze_validation_error_is_400(app) -> None:
    client = TestClient(app)
    response = client.post("/analyze", json={"text": "   ", "domain": "restaurant"})
    assert response.status_code == 400
    assert response.json()["error"] == "validation_error"


def test_analyze_uses_context_for_partial_follow_up(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "4",
            "domain": "restaurant",
            "context": {
                "previous_intent": "reservation_create",
                "previous_slots": {"date": "tomorrow"},
                "required_slots": ["people", "date", "time"],
            },
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["source"] == "context"
    assert payload["intent"]["name"] == "reservation_create"
    assert any(entity["type"] == "PEOPLE_COUNT" and entity["value"] == "4" for entity in payload["entities"])


def test_analyze_uses_previous_turn_context_in_reservation_workflow(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "Actually make it 6 people instead",
            "domain": "restaurant",
            "context": {
                "previous_intent": "reservation_create",
                "previous_slots": {
                    "date": "tomorrow",
                    "time": "7pm",
                    "name": "Alex Carter",
                },
                "required_slots": ["people", "date", "time", "name"],
            },
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["source"] == "context"
    assert payload["intent"]["name"] == "reservation_modify"
    assert any(
        entity["type"] == "PEOPLE_COUNT" and entity["raw_value"] == "6 people" and entity["value"] == "6"
        for entity in payload["entities"]
    )


def test_analyze_normalizes_follow_up_slang_in_context(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "tmrw at 9pm",
            "domain": "restaurant",
            "context": {
                "previous_intent": "reservation_modify",
                "previous_slots": {"people": "4", "name": "Alex Carter"},
                "required_slots": ["date", "time", "people", "name"],
            },
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["source"] == "context"
    assert payload["intent"]["name"] == "reservation_modify"
    assert any(
        entity["type"] == "DATE" and entity["raw_value"] == "tmrw" and entity["normalization_status"] == "normalized"
        for entity in payload["entities"]
    )
    assert any(entity["type"] == "TIME" and entity["raw_value"] == "9pm" and entity["value"] == "21:00" for entity in payload["entities"])


def test_analyze_normalizes_compact_people_follow_up(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "for 5 ppl",
            "domain": "restaurant",
            "context": {
                "previous_intent": "reservation_create",
                "previous_slots": {"date": "tomorrow", "time": "7pm"},
                "required_slots": ["people", "date", "time", "name"],
            },
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["source"] == "context"
    assert payload["intent"]["name"] == "reservation_create"
    assert any(
        entity["type"] == "PEOPLE_COUNT" and entity["raw_value"] == "5 ppl" and entity["value"] == "5"
        for entity in payload["entities"]
    )


def test_analyze_returns_entity_offsets_against_original_text(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "pls book a tbl tmrw for 2 ppl",
            "domain": "restaurant",
        },
    )
    assert response.status_code == 200
    payload = response.json()
    date_entity = next(entity for entity in payload["entities"] if entity["type"] == "DATE")
    people_entity = next(entity for entity in payload["entities"] if entity["type"] == "PEOPLE_COUNT")
    assert date_entity["raw_value"] == "tmrw"
    assert date_entity["value"] == (date.today() + timedelta(days=1)).isoformat()
    assert people_entity["raw_value"] == "for 2 ppl"
    assert people_entity["value"] == "2"
    assert payload["intent"]["name"] == "reservation_create"


def test_analyze_returns_ranked_intents_and_utterance(app) -> None:
    client = TestClient(app)
    response = client.post("/analyze", json={"text": "what else?", "domain": "restaurant"})

    assert response.status_code == 200
    payload = response.json()
    confidences = [intent["confidence"] for intent in payload["intents"]]
    assert confidences == sorted(confidences, reverse=True)
    assert payload["utterance"]["kind"] == "vague_follow_up"
    assert "warnings" in payload


def test_analyze_classifies_non_business_utterances(app) -> None:
    client = TestClient(app)
    examples = {
        "carrot": "out_of_domain",
        "how are you?": "small_talk",
        "wtf?": "frustration",
    }

    for text, expected_kind in examples.items():
        response = client.post("/analyze", json={"text": text, "domain": "restaurant"})
        assert response.status_code == 200
        payload = response.json()
        assert payload["utterance"]["kind"] == expected_kind


def test_analyze_returns_canonical_reservation_entities(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={"text": "tomorrow at 7pm for 5 ppl", "domain": "restaurant"},
    )

    assert response.status_code == 200
    payload = response.json()
    date_entity = next(entity for entity in payload["entities"] if entity["type"] == "DATE")
    time_entity = next(entity for entity in payload["entities"] if entity["type"] == "TIME")
    people_entity = next(entity for entity in payload["entities"] if entity["type"] == "PEOPLE_COUNT")
    assert date_entity["raw_value"] == "tomorrow"
    assert date_entity["value"] == (date.today() + timedelta(days=1)).isoformat()
    assert time_entity["raw_value"] == "7pm"
    assert time_entity["value"] == "19:00"
    assert people_entity["raw_value"] == "for 5 ppl"
    assert people_entity["value"] == "5"
