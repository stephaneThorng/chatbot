"""Rank intent classifier output into ordered candidates."""

from __future__ import annotations

from dataclasses import dataclass

from src.api.schemas import IntentName
from src.models.intent_classifier import IntentResult


@dataclass(frozen=True, slots=True)
class RankedIntent:
    """Ranked business intent candidate."""

    name: IntentName
    confidence: float
    source: str
    reason: str | None = None


class IntentRanker:
    """Converts classifier output into a stable ranked candidate list."""

    def rank(self, result: IntentResult) -> list[RankedIntent]:
        """Return primary intent plus alternatives sorted by confidence."""

        candidates: dict[IntentName, RankedIntent] = {
            result.name: RankedIntent(
                name=result.name,
                confidence=result.confidence,
                source=result.source,
                reason="primary",
            )
        }
        for name, confidence in result.alternatives.items():
            if name in candidates:
                continue
            candidates[name] = RankedIntent(
                name=name,
                confidence=confidence,
                source=result.source,
                reason="alternative",
            )
        return sorted(
            candidates.values(),
            key=lambda candidate: candidate.confidence,
            reverse=True,
        )

