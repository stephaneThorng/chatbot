from __future__ import annotations

from src.config import Settings
from src.models.intent_classifier import IntentClassifier


def test_regex_intent_classification_prefers_reservation() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify("Je veux reserv une table pour 2 personnes", "restaurant")
    assert result.name == "reservation"
    assert result.fast_path is True
    assert result.source == "regex"
    assert result.confidence >= 0.6
