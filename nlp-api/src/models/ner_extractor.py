"""NER extraction with transformer and heuristic fallback."""

from __future__ import annotations

import re
import time
from dataclasses import dataclass
from typing import Any, Iterable, List, Sequence

from src.api.schemas import AnalysisContext, ContextSlots
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
        return self.extract_with_context(text=text, context=None)

    def extract_with_context(
        self,
        text: str,
        context: AnalysisContext | None = None,
    ) -> NERResult:
        """Extract named entities from text with optional context hints."""

        started = time.perf_counter()
        if self.model is not None and self.tokenizer is not None:
            try:
                entities = self._extract_with_model(text, context=context)
                entities = self._merge_contextual_entities(text=text, entities=entities, context=context)
                return NERResult(
                    entities=entities,
                    processing_time_ms=(time.perf_counter() - started) * 1000,
                )
            except Exception:
                pass
        entities = self._extract_with_heuristics(text, context=context)
        return NERResult(
            entities=entities,
            processing_time_ms=(time.perf_counter() - started) * 1000,
        )

    def _extract_with_model(
        self,
        text: str,
        context: AnalysisContext | None = None,
    ) -> List[Entity]:
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
        entities = self._decode_bio(text=text, tags=tags, confidences=confidences, offsets=offsets)
        return self._apply_context_filters(text=text, entities=entities, context=context)

    def _merge_contextual_entities(
        self,
        text: str,
        entities: Sequence[Entity],
        context: AnalysisContext | None,
    ) -> List[Entity]:
        """Supplement model output with context-derived entities when expected slots are still missing."""

        if not context:
            return list(entities)
        contextual_entities = self._extract_contextual_entities(text=text, context=context)
        merged = self._dedupe_entities([*entities, *contextual_entities])
        return self._apply_context_filters(text=text, entities=merged, context=context)

    def _extract_with_heuristics(
        self,
        text: str,
        context: AnalysisContext | None = None,
    ) -> List[Entity]:
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
        entities.extend(self._extract_contextual_entities(text=text, context=context))
        return self._apply_context_filters(
            text=text,
            entities=self._dedupe_entities(entities),
            context=context,
        )

    def _extract_contextual_entities(
        self,
        text: str,
        context: AnalysisContext | None,
    ) -> List[Entity]:
        """Infer likely slot values from short follow-ups when context narrows the target."""

        if not context:
            return []
        missing_slots = self._infer_missing_slots(context)
        normalized_text = text.strip()
        entities: List[Entity] = []
        if not normalized_text:
            return entities

        if "people" in missing_slots:
            match = re.fullmatch(r"\s*(\d{1,2})\s*", normalized_text)
            if match:
                entities.append(
                    Entity(
                        type="PEOPLE_COUNT",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.86,
                        source="context",
                    )
                )
        if "time" in missing_slots:
            match = re.fullmatch(r"\s*(\d{1,2}(?::\d{2})?\s?(?:am|pm))\s*", normalized_text, flags=re.IGNORECASE)
            if match:
                entities.append(
                    Entity(
                        type="TIME",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.86,
                        source="context",
                    )
                )
        if "date" in missing_slots:
            match = re.fullmatch(
                r"\s*(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\s*",
                normalized_text,
                flags=re.IGNORECASE,
            )
            if match:
                entities.append(
                    Entity(
                        type="DATE",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.86,
                        source="context",
                    )
                )
        if "name" in missing_slots:
            match = re.fullmatch(r"\s*([A-Z][a-z]+(?:\s+[A-Z][a-z]+){0,2})\s*", normalized_text)
            if match:
                entities.append(
                    Entity(
                        type="PERSON",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.8,
                        source="context",
                    )
                )
        if "phone" in missing_slots:
            match = re.fullmatch(
                r"\s*((?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4})\s*",
                normalized_text,
            )
            if match:
                entities.append(
                    Entity(
                        type="PHONE",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.86,
                        source="context",
                    )
                )
        if "email" in missing_slots:
            match = re.fullmatch(r"\s*([A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,})\s*", normalized_text, flags=re.IGNORECASE)
            if match:
                entities.append(
                    Entity(
                        type="EMAIL",
                        value=match.group(1),
                        start=match.start(1),
                        end=match.end(1),
                        confidence=0.86,
                        source="context",
                    )
                )
        if "menu_item" in missing_slots:
            for term in self.MENU_ITEM_TERMS:
                match = re.search(rf"\b{re.escape(term)}\b", normalized_text, flags=re.IGNORECASE)
                if match:
                    entities.append(
                        Entity(
                            type="MENU_ITEM",
                            value=match.group(0),
                            start=match.start(),
                            end=match.end(),
                            confidence=0.84,
                            source="context",
                        )
                    )
                    break
        if "price_item" in missing_slots:
            for term in self.PRICE_ITEM_TERMS:
                match = re.search(rf"\b{re.escape(term)}\b", normalized_text, flags=re.IGNORECASE)
                if match:
                    entities.append(
                        Entity(
                            type="PRICE_ITEM",
                            value=match.group(0),
                            start=match.start(),
                            end=match.end(),
                            confidence=0.84,
                            source="context",
                        )
                    )
                    break
        if "location" in missing_slots:
            for term in self.LOCATION_TERMS:
                match = re.search(rf"\b{re.escape(term)}\b", normalized_text, flags=re.IGNORECASE)
                if match:
                    entities.append(
                        Entity(
                            type="LOCATION",
                            value=match.group(0),
                            start=match.start(),
                            end=match.end(),
                            confidence=0.84,
                            source="context",
                        )
                    )
                    break
        return entities

    def _apply_context_filters(
        self,
        text: str,
        entities: Sequence[Entity],
        context: AnalysisContext | None,
    ) -> List[Entity]:
        """Prefer entities that fill still-missing contextual slots."""

        del text
        if not context:
            return list(entities)
        missing_slots = self._infer_missing_slots(context)
        if not missing_slots:
            return list(entities)
        filtered = [
            entity
            for entity in entities
            if self._entity_type_to_slot(entity.type) in missing_slots
        ]
        return filtered or list(entities)

    def _infer_missing_slots(self, context: AnalysisContext) -> set[str]:
        """Compute which conversational slots are still expected."""

        previous_slots = context.previous_slots or context.slots_filled or ContextSlots()
        previous_slot_values = previous_slots.model_dump(exclude_none=True)
        required_slots = context.required_slots or []
        missing_slots = {
            str(slot).lower()
            for slot in required_slots
            if str(slot).lower() not in {str(key).lower() for key in previous_slot_values}
        }
        previous_intent = str(context.current_intent or context.previous_intent or "").lower()
        if previous_intent.startswith("reservation") and not required_slots:
            default_reservation_slots = {"people", "date", "time", "name"}
            provided = {str(key).lower() for key in previous_slot_values}
            missing_slots |= default_reservation_slots - provided
        return missing_slots

    def _entity_type_to_slot(self, entity_type: str) -> str:
        """Map entity types to contextual slot keys."""

        mapping = {
            "DATE": "date",
            "TIME": "time",
            "PEOPLE_COUNT": "people",
            "PERSON": "name",
            "PHONE": "phone",
            "EMAIL": "email",
            "MENU_ITEM": "menu_item",
            "PRICE_ITEM": "price_item",
            "LOCATION": "location",
        }
        return mapping.get(entity_type, entity_type.lower())

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
