"""spaCy-backed rule-based entity extraction."""

from __future__ import annotations

from collections.abc import Sequence
from typing import TYPE_CHECKING

import re

from src.config import Settings, settings

if TYPE_CHECKING:
    from src.models.ner_extractor import Entity


class SpacyEntityExtractor:
    """Centralizes rule-based extraction through a spaCy pipeline."""

    MENU_ITEM_TERMS = (
        "vegan options",
        "kids menu",
        "dessert menu",
        "drink list",
        "gluten free dishes",
        "chef specials",
        "seafood options",
        "lunch menu",
        "wine pairings",
        "brunch menu",
    )

    PRICE_ITEM_TERMS = (
        "tasting menu",
        "brunch buffet",
        "kids meals",
        "wine pairing",
        "steak special",
        "private dining menu",
        "seafood platter",
        "happy hour snacks",
        "date night set menu",
        "family meal",
    )

    LOCATION_TERMS = (
        "downtown",
        "main street",
        "riverside",
        "old town",
        "train station",
        "city center",
        "pine avenue",
        "art museum",
        "waterfront",
        "market square",
    )

    def __init__(self, config: Settings | None = None) -> None:
        self.config = config or settings
        self._nlp = self._build_pipeline()
        self._phone_patterns = (
            re.compile(r"\b(?:(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}|\d{3}[\s.-]?\d{4})\b", re.IGNORECASE),
        )
        self._date_patterns = (
            re.compile(r"\b(today|tomorrow|tonight|this weekend|next week)\b", re.IGNORECASE),
            re.compile(r"\b(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b", re.IGNORECASE),
            re.compile(
                r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}\b",
                re.IGNORECASE,
            ),
            re.compile(r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b", re.IGNORECASE),
        )
        self._time_patterns = (
            re.compile(r"\b\d{1,2}(?::\d{2})?\s?(?:am|pm)\b", re.IGNORECASE),
            re.compile(r"\b\d{1,2}:\d{2}\b", re.IGNORECASE),
            re.compile(r"\b(?:noon|midnight|this evening|tomorrow evening)\b", re.IGNORECASE),
        )

    def extract(self, text: str, entity_cls: type["Entity"]) -> list["Entity"]:
        """Extract entities from text using spaCy rules."""

        doc = self._nlp(text)
        entities = [
            entity_cls(
                type=span.label_,
                value=span.text,
                start=span.start_char,
                end=span.end_char,
                confidence=self._confidence_for_label(span.label_),
                source="spacy_rule",
            )
            for span in doc.ents
        ]
        entities.extend(self._extract_pattern_entities(text=text, entity_cls=entity_cls))
        return self._dedupe_entities(entities)

    def _build_pipeline(self):
        import spacy
        from spacy.pipeline import EntityRuler

        try:
            nlp = spacy.load(self.config.spacy_model, disable=["parser", "tagger", "lemmatizer", "attribute_ruler"])
        except Exception:
            nlp = spacy.blank("en")

        if "entity_ruler" in nlp.pipe_names:
            nlp.remove_pipe("entity_ruler")
        ruler = nlp.add_pipe("entity_ruler", config={"overwrite_ents": True})
        assert isinstance(ruler, EntityRuler)
        ruler.add_patterns(self._entity_ruler_patterns())
        return nlp

    def _entity_ruler_patterns(self) -> list[dict]:
        patterns: list[dict] = [
            {"label": "EMAIL", "pattern": [{"TEXT": {"REGEX": r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$"}}]},
            {
                "label": "PHONE",
                "pattern": [
                    {"TEXT": {"REGEX": r"^(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}$"}}
                ],
            },
            {
                "label": "PHONE",
                "pattern": [
                    {"TEXT": {"REGEX": r"^\d{3}[\s.-]?\d{4}$"}}
                ],
            },
            {"label": "PEOPLE_COUNT", "pattern": [{"LIKE_NUM": True}, {"LOWER": {"IN": ["people", "persons", "guests", "guest", "adults", "kids"]}}]},
        ]
        patterns.extend(self._phrase_patterns("MENU_ITEM", self.MENU_ITEM_TERMS))
        patterns.extend(self._phrase_patterns("PRICE_ITEM", self.PRICE_ITEM_TERMS))
        patterns.extend(self._phrase_patterns("LOCATION", self.LOCATION_TERMS))
        return patterns

    def _phrase_patterns(self, label: str, phrases: Sequence[str]) -> list[dict]:
        return [{"label": label, "pattern": [{"LOWER": token.lower()} for token in phrase.split()]} for phrase in phrases]

    def _extract_pattern_entities(self, text: str, entity_cls: type["Entity"]) -> list["Entity"]:
        entities: list["Entity"] = []
        for pattern in self._phone_patterns:
            for match in pattern.finditer(text):
                entities.append(entity_cls("PHONE", match.group(0), match.start(), match.end(), 0.91, "spacy_rule"))
        for pattern in self._date_patterns:
            for match in pattern.finditer(text):
                entities.append(entity_cls("DATE", match.group(0), match.start(), match.end(), 0.94, "spacy_rule"))
        for pattern in self._time_patterns:
            for match in pattern.finditer(text):
                entities.append(entity_cls("TIME", match.group(0), match.start(), match.end(), 0.95, "spacy_rule"))
        return entities

    def _confidence_for_label(self, label: str) -> float:
        default_confidence = {
            "EMAIL": 0.98,
            "PHONE": 0.91,
            "PEOPLE_COUNT": 0.9,
            "MENU_ITEM": 0.88,
            "PRICE_ITEM": 0.88,
            "LOCATION": 0.87,
        }
        return default_confidence.get(label, 0.85)

    def _dedupe_entities(self, entities: Sequence["Entity"]) -> list["Entity"]:
        sorted_entities = sorted(
            entities,
            key=lambda item: (item.start, -(item.end - item.start), -item.confidence, item.type),
        )
        deduped: list["Entity"] = []
        for entity in sorted_entities:
            duplicate = next(
                (
                    existing
                    for existing in deduped
                    if existing.start == entity.start and existing.end == entity.end and existing.type == entity.type
                ),
                None,
            )
            if duplicate is None:
                deduped.append(entity)
                continue
            if entity.confidence > duplicate.confidence:
                deduped.remove(duplicate)
                deduped.append(entity)
        deduped.sort(key=lambda item: (item.start, item.end, item.type))
        return deduped
