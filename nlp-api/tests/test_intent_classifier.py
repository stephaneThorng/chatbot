from __future__ import annotations

from src.api.schemas import AnalysisContext, ContextSlots
from src.config import Settings
from src.models.intent_classifier import IntentClassifier


def test_regex_intent_classification_prefers_reservation() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("I need a new reservation for 2 on Friday at 8pm", "restaurant")
    assert result.name == "reservation_create"
    assert result.fast_path is True
    assert result.source == "regex"
    assert result.confidence >= 0.6


def test_regex_intent_classification_detects_status() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("Can you check my reservation?", "restaurant")
    assert result.name == "reservation_status"
    assert result.fast_path is True
    assert result.source == "regex"


def test_regex_intent_classification_leaves_farewell_to_backend() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("good bye", "restaurant")
    assert result.name == "unknown"


def test_regex_intent_classification_leaves_thanks_to_backend() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("thank you", "restaurant")
    assert result.name == "unknown"


def test_regex_intent_classification_detects_seafood_menu() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("Can I see the seafood options for weekend dinner?", "restaurant")
    assert result.name == "menu_request"
    assert result.fast_path is True
    assert result.source == "regex"


def test_context_classification_recovers_partial_follow_up() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "For 4",
        "restaurant",
        AnalysisContext(
            previous_intent="reservation_create",
            previous_slots=ContextSlots(date="tomorrow"),
        ),
    )
    assert result.name == "reservation_create"
    assert result.source == "context"
    assert result.fast_path is True


def test_context_classification_detects_modification_in_workflow() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "Actually make it 6 people instead",
        "restaurant",
        AnalysisContext(
            previous_intent="reservation_create",
            previous_slots=ContextSlots(
                date="tomorrow",
                time="7pm",
                name="Alex Carter",
            ),
            required_slots=["people", "date", "time", "name"],
        ),
    )
    assert result.name == "reservation_modify"
    assert result.source == "context"
    assert result.fast_path is True


def test_context_classification_keeps_previous_intent_for_date_time_follow_up() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "Tomorrow at 9pm",
        "restaurant",
        AnalysisContext(
            previous_intent="reservation_modify",
            previous_slots=ContextSlots(people="4", name="Alex Carter"),
            required_slots=["date", "time", "people", "name"],
        ),
    )
    assert result.name == "reservation_modify"
    assert result.source == "context"
    assert result.fast_path is True


def test_context_classification_keeps_contact_intent_for_email_follow_up() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "events@example.com",
        "restaurant",
        AnalysisContext(
            previous_intent="contact_request",
            required_slots=["email"],
        ),
    )
    assert result.name == "contact_request"
    assert result.source == "context"
    assert result.fast_path is True


def test_regex_intent_classification_detects_contact_request() -> None:
    classifier = IntentClassifier(Settings(use_hybrid_intent=True))
    result = classifier.classify("What is your phone number?", "restaurant")
    assert result.name == "contact_request"
    assert result.fast_path is True
    assert result.source == "regex"


def test_context_classification_keeps_location_intent_for_location_follow_up() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "Downtown",
        "restaurant",
        AnalysisContext(
            previous_intent="location_request",
            required_slots=["location"],
        ),
    )
    assert result.name == "location_request"
    assert result.source == "context"
    assert result.fast_path is True


def test_context_classification_keeps_previous_intent_for_slang_datetime_follow_up() -> None:
    classifier = IntentClassifier(Settings())
    result = classifier.classify(
        "tomorrow at 9pm",
        "restaurant",
        AnalysisContext(
            previous_intent="reservation_modify",
            previous_slots=ContextSlots(people="4", name="Alex Carter"),
            required_slots=["date", "time", "people", "name"],
        ),
    )
    assert result.name == "reservation_modify"
    assert result.source == "context"
