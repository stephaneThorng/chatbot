"""Utterance-level signal analysis."""

from __future__ import annotations

import re
from dataclasses import dataclass

from src.api.schemas import IntentName, UtteranceKind


@dataclass(frozen=True, slots=True)
class UtteranceSignal:
    """High-level non-business signal detected from an utterance."""

    kind: UtteranceKind
    confidence: float
    source: str = "rule"


class UtteranceAnalyzer:
    """Detects non-business and ambiguous utterance categories."""

    SMALL_TALK_PATTERNS = (
        re.compile(r"^\s*how\s+are\s+you\b", re.IGNORECASE),
        re.compile(r"^\s*how\s+am\s+i\b", re.IGNORECASE),
        re.compile(r"^\s*i\s+am\b", re.IGNORECASE),
        re.compile(r"^\s*i['’]?m\b", re.IGNORECASE),
        re.compile(r"^\s*tell\s+me\s+something\s+random\b", re.IGNORECASE),
    )
    VAGUE_FOLLOW_UP_PATTERNS = (
        re.compile(r"^\s*what\s+else\s*\??\s*$", re.IGNORECASE),
        re.compile(r"^\s*anything\s+(else|more)\s*\??\s*$", re.IGNORECASE),
        re.compile(r"^\s*other\s+options\s*\??\s*$", re.IGNORECASE),
        re.compile(r"^\s*what\s+about\s+the\s+rest\s*\??\s*$", re.IGNORECASE),
    )
    CLARIFICATION_PATTERNS = (
        re.compile(r"^\s*what\s*\??\s*$", re.IGNORECASE),
        re.compile(r"^\s*what\s+do\s+you\s+mean\s*\??\s*$", re.IGNORECASE),
        re.compile(r"^\s*i\s+do\s+not\s+understand\s*\??\s*$", re.IGNORECASE),
    )
    FRUSTRATION_PATTERNS = (
        re.compile(r"^\s*w(?:tf|th)\s*\??\s*$", re.IGNORECASE),
        re.compile(r"\b(this\s+is\s+wrong|not\s+what\s+i\s+meant)\b", re.IGNORECASE),
    )

    def analyze(
        self,
        text: str,
        primary_intent: IntentName,
        primary_confidence: float,
        entity_count: int,
    ) -> UtteranceSignal:
        """Return the strongest utterance-level signal."""

        normalized = " ".join(text.strip().lower().split())
        if not normalized:
            return UtteranceSignal(UtteranceKind.UNKNOWN, 1.0)
        if any(pattern.search(normalized) for pattern in self.FRUSTRATION_PATTERNS):
            return UtteranceSignal(UtteranceKind.FRUSTRATION, 0.95)
        if any(pattern.search(normalized) for pattern in self.VAGUE_FOLLOW_UP_PATTERNS):
            return UtteranceSignal(UtteranceKind.VAGUE_FOLLOW_UP, 0.93)
        if any(pattern.search(normalized) for pattern in self.CLARIFICATION_PATTERNS):
            return UtteranceSignal(UtteranceKind.CLARIFICATION_REQUEST, 0.9)
        if any(pattern.search(normalized) for pattern in self.SMALL_TALK_PATTERNS):
            return UtteranceSignal(UtteranceKind.SMALL_TALK, 0.92)
        if primary_intent == IntentName.UNKNOWN and entity_count == 0:
            return UtteranceSignal(UtteranceKind.OUT_OF_DOMAIN, 0.88)
        if primary_intent == IntentName.UNKNOWN or primary_confidence < 0.4:
            return UtteranceSignal(UtteranceKind.UNKNOWN, 0.7)
        return UtteranceSignal(UtteranceKind.BUSINESS_QUERY, max(primary_confidence, 0.5))

