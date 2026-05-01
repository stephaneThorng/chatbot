from __future__ import annotations

import pytest

from src.config import Settings
from src.main import create_app
from src.services.nlp_service import NLPService


class FakeService(NLPService):
    def __init__(self) -> None:
        super().__init__(config=Settings(use_hybrid_intent=True))


@pytest.fixture
def fake_service() -> NLPService:
    service = FakeService()
    return service


@pytest.fixture
def app(fake_service):
    return create_app(service=fake_service)
