from __future__ import annotations

from src.api.schemas import AnalysisContext, ContextSlots
from src.config import Settings
from src.models.ner_extractor import NERExtractor


def test_heuristic_ner_extracts_expected_entities() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract("Book a table for 4 people tomorrow at 7pm")
    entity_types = {entity.type for entity in result.entities}
    assert "PEOPLE_COUNT" in entity_types
    assert "DATE" in entity_types
    assert "TIME" in entity_types


def test_heuristic_ner_extracts_menu_price_and_location_entities() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract(
        "How much is the private dining menu and do you have vegan options near Downtown?"
    )
    values_by_type = {entity.type: entity.value for entity in result.entities}
    assert values_by_type["PRICE_ITEM"] == "private dining menu"
    assert values_by_type["MENU_ITEM"] == "vegan options"
    assert values_by_type["LOCATION"] == "Downtown"


def test_contextual_ner_extracts_missing_people_slot_from_short_reply() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract_with_context(
        "4",
        AnalysisContext(
            previous_intent="reservation",
            previous_slots=ContextSlots(date="tomorrow"),
            required_slots=["people", "date", "time"],
        ),
    )
    assert len(result.entities) == 1
    assert result.entities[0].type == "PEOPLE_COUNT"
    assert result.entities[0].value == "4"


def test_contextual_ner_filters_to_missing_slot_in_follow_up_workflow() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract_with_context(
        "Actually make it 6 people instead",
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
    assert len(result.entities) == 1
    assert result.entities[0].type == "PEOPLE_COUNT"
    assert result.entities[0].value == "6 people"


def test_contextual_ner_extracts_location_from_short_follow_up() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract_with_context(
        "Downtown",
        AnalysisContext(
            previous_intent="location_request",
            required_slots=["location"],
        ),
    )
    assert len(result.entities) == 1
    assert result.entities[0].type == "LOCATION"
    assert result.entities[0].value == "Downtown"


def test_contextual_ner_extracts_email_from_short_follow_up() -> None:
    extractor = NERExtractor(Settings())
    result = extractor.extract_with_context(
        "events@example.com",
        AnalysisContext(
            previous_intent="greeting_contact",
            required_slots=["email"],
        ),
    )
    assert len(result.entities) == 1
    assert result.entities[0].type == "EMAIL"
    assert result.entities[0].value == "events@example.com"


def test_bio_decoder_merges_contiguous_tokens() -> None:
    extractor = NERExtractor(Settings())
    entities = extractor._decode_bio(
        text="Alex Carter",
        tags=["B-PERSON", "I-PERSON"],
        confidences=[0.8, 0.9],
        offsets=[(0, 4), (5, 11)],
    )
    assert len(entities) == 1
    assert entities[0].value == "Alex Carter"
