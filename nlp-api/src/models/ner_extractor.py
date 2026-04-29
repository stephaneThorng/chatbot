"""NER extraction with transformer and heuristic fallback."""

from __future__ import annotations

import re
import time
from dataclasses import dataclass
from typing import Any, Iterable, List, Sequence

from src.config import Settings, settings


@dataclass(slots=True)
class Entity:
    """Resolved entity span."""

    type: str
    value: str
    start: int
    end: int
    confidence: float
    source: str


@dataclass(slots=True)
class NERResult:
    """NER response wrapper."""

    entities: List[Entity]
    processing_time_ms: float


class NERExtractor:
    """Entity extractor."""

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
        self.model: Any | None = None
        self.tokenizer: Any | None = None
        self.id2label: dict[int, str] = {}

    def bind_artifacts(self, tokenizer: Any, model: Any) -> None:
        """Attach pretrained artifacts."""

        self.tokenizer = tokenizer
        self.model = model
        raw_labels = getattr(model.config, "id2label", {}) or {}
        self.id2label = {int(key): value for key, value in raw_labels.items()}

    def extract(self, text: str) -> NERResult:
        """Extract named entities from text."""

        started = time.perf_counter()
        if self.model is not None and self.tokenizer is not None:
            try:
                entities = self._extract_with_model(text)
                return NERResult(
                    entities=entities,
                    processing_time_ms=(time.perf_counter() - started) * 1000,
                )
            except Exception:
                pass
        entities = self._extract_with_heuristics(text)
        return NERResult(
            entities=entities,
            processing_time_ms=(time.perf_counter() - started) * 1000,
        )

    def _extract_with_model(self, text: str) -> List[Entity]:
        import torch

        encoded = self.tokenizer(
            text,
            return_offsets_mapping=True,
            return_tensors="pt",
            truncation=True,
            max_length=256,
        )
        offsets = encoded.pop("offset_mapping")[0].tolist()
        device = "cuda" if self.config.normalized_device == "cuda" and torch.cuda.is_available() else "cpu"
        encoded = {key: value.to(device) for key, value in encoded.items()}

        with torch.no_grad():
            logits = self.model(**encoded).logits[0]

        predictions = torch.argmax(logits, dim=-1).tolist()
        confidences = torch.max(torch.softmax(logits, dim=-1), dim=-1).values.tolist()
        tags = [self.id2label.get(int(prediction), "O") for prediction in predictions]
        return self._decode_bio(text=text, tags=tags, confidences=confidences, offsets=offsets)

    def _extract_with_heuristics(self, text: str) -> List[Entity]:
        menu_pattern = "|".join(re.escape(term) for term in self.MENU_ITEM_TERMS)
        price_pattern = "|".join(re.escape(term) for term in self.PRICE_ITEM_TERMS)
        location_pattern = "|".join(re.escape(term) for term in self.LOCATION_TERMS)
        patterns = {
            "DATE": [
                (r"\b(today|tomorrow|tonight|this weekend|next week)\b", 0.9),
                (
                    r"\b(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
                    0.9,
                ),
                (
                    r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}\b",
                    0.94,
                ),
                (r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b", 0.94),
            ],
            "TIME": [
                (r"\b\d{1,2}(?::\d{2})?\s?(?:am|pm)\b", 0.95),
                (r"\b\d{1,2}:\d{2}\b", 0.95),
                (r"\b(?:noon|midnight|this evening|tomorrow evening)\b", 0.88),
            ],
            "PEOPLE_COUNT": [
                (r"\b\d+\s*(people|persons|guests?|adults?|kids?)\b", 0.9),
            ],
            "EMAIL": [
                (r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b", 0.98),
            ],
            "PHONE": [
                (r"\b(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}\b", 0.91),
            ],
            "MENU_ITEM": [
                (rf"\b(?:{menu_pattern})\b", 0.88),
            ],
            "PRICE_ITEM": [
                (rf"\b(?:{price_pattern})\b", 0.88),
            ],
            "LOCATION": [
                (rf"\b(?:{location_pattern})\b", 0.87),
            ],
        }
        entities: List[Entity] = []
        for entity_type, rules in patterns.items():
            for pattern, confidence in rules:
                for match in re.finditer(pattern, text, flags=re.IGNORECASE):
                    entities.append(
                        Entity(
                            type=entity_type,
                            value=match.group(0),
                            start=match.start(),
                            end=match.end(),
                            confidence=confidence,
                            source="heuristic",
                        )
                    )
        return self._dedupe_entities(entities)

    def _dedupe_entities(self, entities: Sequence[Entity]) -> List[Entity]:
        """Keep the strongest non-duplicate heuristic spans."""

        sorted_entities = sorted(
            entities,
            key=lambda item: (item.start, -(item.end - item.start), -item.confidence, item.type),
        )
        deduped: List[Entity] = []
        for entity in sorted_entities:
            duplicate = next(
                (
                    existing
                    for existing in deduped
                    if existing.start == entity.start and existing.end == entity.end
                ),
                None,
            )
            if duplicate is not None:
                if entity.confidence > duplicate.confidence:
                    deduped.remove(duplicate)
                    deduped.append(entity)
                continue
            deduped.append(entity)
        deduped.sort(key=lambda item: (item.start, item.end, item.type))
        return deduped

    def _decode_bio(
        self,
        text: str,
        tags: Sequence[str],
        confidences: Sequence[float],
        offsets: Iterable[Sequence[int]],
    ) -> List[Entity]:
        entities: List[Entity] = []
        active: Entity | None = None
        for tag, confidence, offset in zip(tags, confidences, offsets):
            start, end = int(offset[0]), int(offset[1])
            if start == end:
                continue
            if tag == "O":
                if active is not None:
                    entities.append(active)
                    active = None
                continue
            prefix, _, entity_type = tag.partition("-")
            if prefix == "B" or active is None or active.type != entity_type:
                if active is not None:
                    entities.append(active)
                active = Entity(
                    type=entity_type,
                    value=text[start:end],
                    start=start,
                    end=end,
                    confidence=float(confidence),
                    source="ner_model",
                )
                continue
            active.value = text[active.start:end]
            active.end = end
            active.confidence = max(active.confidence, float(confidence))
        if active is not None:
            entities.append(active)
        return [entity for entity in entities if entity.confidence >= self.config.ner_confidence_threshold]

    @property
    def is_loaded(self) -> bool:
        """Whether a transformer model is available."""

        return self.model is not None and self.tokenizer is not None
