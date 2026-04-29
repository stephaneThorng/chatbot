from __future__ import annotations

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
