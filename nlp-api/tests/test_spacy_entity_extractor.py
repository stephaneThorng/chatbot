from __future__ import annotations

from src.config import Settings
from src.models.ner_extractor import Entity
from src.services.spacy_entity_extractor import SpacyEntityExtractor


def test_spacy_entity_extractor_extracts_simple_rule_entities() -> None:
    extractor = SpacyEntityExtractor(Settings())
    entities = extractor.extract(
        "Please email events@example.com or call 555-0108 for 4 people",
        entity_cls=Entity,
    )
    values_by_type = {entity.type: entity.value for entity in entities}
    assert values_by_type["EMAIL"] == "events@example.com"
    assert values_by_type["PHONE"] == "555-0108"
    assert values_by_type["PEOPLE_COUNT"] == "4 people"


def test_spacy_entity_extractor_extracts_business_terms_and_temporal_entities() -> None:
    extractor = SpacyEntityExtractor(Settings())
    entities = extractor.extract(
        "How much is the private dining menu near Downtown tomorrow at 7:30pm?",
        entity_cls=Entity,
    )
    values_by_type = {entity.type: entity.value for entity in entities}
    assert values_by_type["PRICE_ITEM"] == "private dining menu"
    assert values_by_type["LOCATION"] == "Downtown"
    assert values_by_type["DATE"] == "tomorrow"
    assert values_by_type["TIME"] == "7:30pm"
