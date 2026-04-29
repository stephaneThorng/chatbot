from __future__ import annotations

from fastapi.testclient import TestClient


def test_analyze_endpoint_returns_rich_response(app) -> None:
    client = TestClient(app)
    response = client.post(
        "/analyze",
        json={
            "text": "Book a table for 4 people tomorrow at 7pm",
            "domain": "restaurant",
            "context": {"previous_slots": {"name": "Alex"}},
        },
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["intent"]["name"] == "reservation"
    assert "processing_details" in payload
    assert any(entity["type"] == "PEOPLE_COUNT" for entity in payload["entities"])


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
