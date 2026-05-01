"""NER extraction with transformer and heuristic fallback."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any, Iterable, List, Sequence

from src.api.schemas import AnalysisContext
from src.config import Settings, settings
from src.services.context_resolver import ContextEntityHint, ContextResolver
from src.services.spacy_entity_extractor import SpacyEntityExtractor


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

    def __init__(
        self,
        config: Settings | None = None,
        context_resolver: ContextResolver | None = None,
        spacy_entity_extractor: SpacyEntityExtractor | None = None,
    ) -> None:
        self.config = config or settings
        self.context_resolver = context_resolver or ContextResolver()
        self.spacy_entity_extractor = spacy_entity_extractor or SpacyEntityExtractor(self.config)
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
        rule_entities = self.spacy_entity_extractor.extract(text=text, entity_cls=Entity)
        contextual_entities = self._context_hints_to_entities(
            self.context_resolver.extract_contextual_entities(text=text, context=context)
        )
        if self.model is not None and self.tokenizer is not None:
            try:
                entities = self._extract_with_model(text, context=context)
                entities = self._merge_entities([*entities, *rule_entities, *contextual_entities])
                entities = self._apply_context_filters(entities=entities, context=context)
                return NERResult(
                    entities=entities,
                    processing_time_ms=(time.perf_counter() - started) * 1000,
                )
            except Exception:
                pass
        entities = self._merge_entities([*rule_entities, *contextual_entities])
        entities = self._apply_context_filters(entities=entities, context=context)
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
        return self._decode_bio(text=text, tags=tags, confidences=confidences, offsets=offsets)

    def _apply_context_filters(
        self,
        entities: Sequence[Entity],
        context: AnalysisContext | None,
    ) -> List[Entity]:
        """Prefer entities that fill still-missing contextual slots."""

        if not context:
            return list(entities)
        allowed_types = self.context_resolver.filter_entity_types(
            entity_types=[entity.type for entity in entities],
            context=context,
        )
        if not allowed_types:
            return list(entities)
        filtered = [entity for entity in entities if entity.type in allowed_types]
        return filtered or list(entities)

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

    def _context_hints_to_entities(self, hints: Sequence[ContextEntityHint]) -> list[Entity]:
        return [
            Entity(
                type=hint.type,
                value=hint.value,
                start=hint.start,
                end=hint.end,
                confidence=hint.confidence,
                source=hint.source,
            )
            for hint in hints
        ]

    def _merge_entities(self, entities: Sequence[Entity]) -> list[Entity]:
        return self._dedupe_entities(entities)

    @property
    def is_loaded(self) -> bool:
        """Whether a transformer model is available."""

        return self.model is not None and self.tokenizer is not None
