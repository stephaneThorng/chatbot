"""Entity value canonicalization."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Sequence

from src.api.schemas import EntityType, NormalizationStatus
from src.models.ner_extractor import Entity
from src.services.temporal_resolver import TemporalResolver


@dataclass(frozen=True, slots=True)
class NormalizedEntity:
    """Entity with original span text and canonical value."""

    type: EntityType
    raw_value: str
    value: str
    start: int
    end: int
    confidence: float
    source: str
    resolution: str | None
    normalization_status: NormalizationStatus


@dataclass(frozen=True, slots=True)
class EntityNormalizationResult:
    """Normalized entities plus non-fatal warnings."""

    entities: list[NormalizedEntity]
    warnings: list[str]


class EntityNormalizer:
    """Canonicalizes entities while preserving original spans."""

    def __init__(self, temporal_resolver: TemporalResolver | None = None) -> None:
        self.temporal_resolver = temporal_resolver or TemporalResolver()

    def normalize(self, entities: Sequence[Entity]) -> EntityNormalizationResult:
        """Normalize supported entity types."""

        normalized_entities: list[NormalizedEntity] = []
        warnings: list[str] = []
        for entity in entities:
            normalized = self._normalize_entity(entity)
            if normalized.warning:
                warnings.append(normalized.warning)
            normalized_entities.append(normalized.entity)
        return EntityNormalizationResult(entities=normalized_entities, warnings=warnings)

    def _normalize_entity(self, entity: Entity):
        raw_value = entity.value
        if entity.type == EntityType.DATE:
            result = self.temporal_resolver.resolve_date(raw_value)
            if result:
                return _EntityWithWarning(
                    self._copy(entity, raw_value, result.value, result.resolution, NormalizationStatus.NORMALIZED),
                    result.warning,
                )
        if entity.type == EntityType.TIME:
            result = self.temporal_resolver.resolve_time(raw_value)
            if result and result.warning:
                return _EntityWithWarning(
                    self._copy(entity, raw_value, raw_value, result.resolution, NormalizationStatus.AMBIGUOUS),
                    result.warning,
                )
            if result:
                return _EntityWithWarning(
                    self._copy(entity, raw_value, result.value, result.resolution, NormalizationStatus.NORMALIZED),
                    None,
                )
        if entity.type == EntityType.PEOPLE_COUNT:
            if match := re.search(r"\d{1,3}", raw_value):
                return _EntityWithWarning(
                    self._copy(entity, raw_value, match.group(0), "party_size", NormalizationStatus.NORMALIZED),
                    None,
                )
        return _EntityWithWarning(
            self._copy(entity, raw_value, raw_value.strip(), "raw_only", NormalizationStatus.RAW_ONLY),
            None,
        )

    def _copy(
        self,
        entity: Entity,
        raw_value: str,
        value: str,
        resolution: str | None,
        status: NormalizationStatus,
    ) -> NormalizedEntity:
        return NormalizedEntity(
            type=entity.type,
            raw_value=raw_value,
            value=value,
            start=entity.start,
            end=entity.end,
            confidence=entity.confidence,
            source=entity.source,
            resolution=resolution,
            normalization_status=status,
        )


@dataclass(frozen=True, slots=True)
class _EntityWithWarning:
    entity: NormalizedEntity
    warning: str | None

