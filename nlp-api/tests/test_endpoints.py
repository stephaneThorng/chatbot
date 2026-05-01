from __future__ import annotations

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
    assert any(entity["type"] == "PEOPLE_COUNT" and entity["value"] == "6 people" for entity in payload["entities"])


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
    assert any(entity["type"] == "DATE" and entity["value"] == "tmrw" for entity in payload["entities"])
    assert any(entity["type"] == "TIME" and entity["value"] == "9pm" for entity in payload["entities"])


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
    assert any(entity["type"] == "PEOPLE_COUNT" and entity["value"] == "5 ppl" for entity in payload["entities"])


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
    assert date_entity["value"] == "tmrw"
    assert people_entity["value"] == "2 ppl"
    assert payload["intent"]["name"] == "reservation_create"
