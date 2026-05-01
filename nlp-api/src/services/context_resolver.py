"""Conversational context resolution for intent and slot follow-ups."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Iterable, Sequence

from src.api.schemas import AnalysisContext, ContextSlots


@dataclass(frozen=True, slots=True)
class ContextIntentDecision:
    """Resolved intent from conversational context."""

    name: str
    confidence: float
    alternatives: dict[str, float]
    source: str = "context"
    fast_path: bool = True


@dataclass(frozen=True, slots=True)
class ContextEntityHint:
    """Context-derived entity candidate."""

    type: str
    value: str
    start: int
    end: int
    confidence: float
    source: str = "context"


class ContextResolver:
    """Centralizes short follow-up and missing-slot logic."""

    CONTEXT_MODIFICATION_MARKERS = (
        "instead",
        "actually",
        "change",
        "update",
        "move",
        "rather",
        "non",
        "plutot",
        "en fait",
        "changez",
        "attendez",
    )

    CONTEXT_CANCELLATION_MARKERS = (
        "cancel",
        "drop",
        "remove",
        "annule",
        "supprime",
    )

    SLOT_PATTERNS: dict[str, tuple[str, ...]] = {
        "people": (
            r"^\d{1,2}$",
            r"^for\s+\d{1,2}$",
            r"^for\s+\d{1,2}\s*(people|persons|guests?|adults?|kids?)$",
            r"^\d{1,2}\s*(people|persons|guests?|adults?|kids?)$",
        ),
        "date": (
            r"^(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)$",
            r"^(this\s+)?(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$",
            r"^(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}$",
            r"^(today|tomorrow|tonight)\s+at\s+\d{1,2}(?::\d{2})?\s?(am|pm)$",
        ),
        "time": (
            r"^\d{1,2}(?::\d{2})?\s?(am|pm)$",
            r"^(at\s+)?\d{1,2}(?::\d{2})?\s?(am|pm)$",
        ),
        "name": (
            r"^[a-z]+(?:\s+[a-z]+){1,2}$",
        ),
        "phone": (
            r"^(?:(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}|\d{3}[\s.-]?\d{4})$",
        ),
        "email": (
            r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$",
        ),
        "menu_item": (
            r"^(the\s+)?[a-z]+(?:[\s-]+[a-z]+){0,4}$",
        ),
        "price_item": (
            r"^(the\s+)?[a-z]+(?:[\s-]+[a-z]+){0,4}$",
        ),
        "location": (
            r"^[a-z]+(?:\s+[a-z]+){0,3}$",
        ),
    }

    DEFAULT_RESERVATION_SLOTS = {"people", "date", "time", "name"}

    def infer_missing_slots(self, context: AnalysisContext | None) -> set[str]:
        """Compute which conversational slots are still expected."""

        if not context:
            return set()
        previous_slots = context.previous_slots or context.slots_filled or ContextSlots()
        previous_slot_values = previous_slots.model_dump(exclude_none=True)
        required_slots = context.required_slots or []
        missing_slots = {
            str(slot).lower()
            for slot in required_slots
            if str(slot).lower() not in {str(key).lower() for key in previous_slot_values}
        }
        previous_intent = self.get_previous_intent(context).lower()
        if previous_intent.startswith("reservation") and not required_slots:
            provided = {str(key).lower() for key in previous_slot_values}
            missing_slots |= self.DEFAULT_RESERVATION_SLOTS - provided
        return missing_slots

    def get_previous_intent(self, context: AnalysisContext | None) -> str:
        """Return active prior intent from context."""

        if not context:
            return ""
        return str(context.current_intent or context.previous_intent or "").strip()

    def resolve_intent(
        self,
        text: str,
        context: AnalysisContext | None,
        available_intents: Iterable[str],
    ) -> ContextIntentDecision | None:
        """Resolve short follow-up intent from conversational context."""

        if not context:
            return None
        normalized_text = text.strip().lower()
        if not normalized_text:
            return None

        previous_intent = self.get_previous_intent(context)
        if not previous_intent:
            return None

        previous_intent = self._align_to_available_intent(previous_intent, set(available_intents))

        if any(marker in normalized_text for marker in self.CONTEXT_CANCELLATION_MARKERS):
            return self._context_decision(
                name="reservation_cancel",
                fallback=previous_intent,
                confidence=0.94,
                available_intents=set(available_intents),
            )

        if any(marker in normalized_text for marker in self.CONTEXT_MODIFICATION_MARKERS):
            return self._context_decision(
                name="reservation_modify",
                fallback=previous_intent,
                confidence=0.93,
                available_intents=set(available_intents),
            )

        if self.is_short_follow_up(text, context):
            return ContextIntentDecision(
                name=previous_intent,
                confidence=0.88,
                alternatives={},
            )
        return None

    def is_short_follow_up(self, text: str, context: AnalysisContext | None) -> bool:
        """Detect short contextual replies such as slot-only continuations."""

        if not context:
            return False
        normalized_text = text.strip().lower()
        if not normalized_text:
            return False
        missing_slots = self.infer_missing_slots(context)
        if not ((context.previous_slots or context.slots_filled or missing_slots)):
            return False
        if any(self.matches_slot_value_shape(normalized_text, slot) for slot in missing_slots):
            return True
        if self.matches_multi_slot_follow_up(normalized_text, missing_slots):
            return True
        generic_patterns = (
            r"^\d{1,2}$",
            r"^for\s+\d{1,2}$",
            r"^\d{1,2}\s*(people|persons|guests?|personnes?)$",
            r"^\d{1,2}(?::\d{2})?\s?(am|pm)$",
            r"^(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)$",
            r"^[a-z]+(?:\s+[a-z]+){0,2}$",
        )
        return any(re.fullmatch(pattern, normalized_text) for pattern in generic_patterns)

    def matches_slot_value_shape(self, normalized_text: str, slot: str) -> bool:
        """Check whether text looks like a value for the given slot."""

        return any(re.fullmatch(pattern, normalized_text) for pattern in self.SLOT_PATTERNS.get(slot, ()))

    def matches_multi_slot_follow_up(self, normalized_text: str, missing_slots: set[str]) -> bool:
        """Detect short follow-ups with multiple slot values."""

        has_date = "date" in missing_slots and any(
            re.search(pattern, normalized_text)
            for pattern in (
                r"\b(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
                r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}\b",
            )
        )
        has_time = "time" in missing_slots and any(
            re.search(pattern, normalized_text)
            for pattern in (
                r"\b\d{1,2}(?::\d{2})?\s?(am|pm)\b",
                r"\b\d{1,2}:\d{2}\b",
            )
        )
        return has_date and has_time

    def extract_contextual_entities(self, text: str, context: AnalysisContext | None) -> list[ContextEntityHint]:
        """Infer likely slot values from short follow-ups when context narrows the target."""

        if not context:
            return []
        missing_slots = self.infer_missing_slots(context)
        normalized_text = text.strip()
        if not normalized_text:
            return []

        hints: list[ContextEntityHint] = []
        hints.extend(self._match_single_value_hints(normalized_text, missing_slots))
        hints.extend(self._match_phrase_hints(normalized_text, missing_slots))
        return hints

    def filter_entity_types(self, entity_types: Sequence[str], context: AnalysisContext | None) -> set[str]:
        """Return entity types that are preferred given the current context."""

        if not context:
            return set(entity_types)
        missing_slots = self.infer_missing_slots(context)
        if not missing_slots:
            return set(entity_types)
        return {
            entity_type
            for entity_type in entity_types
            if self.entity_type_to_slot(entity_type) in missing_slots
        }

    def entity_type_to_slot(self, entity_type: str) -> str:
        """Map entity types to slot keys."""

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

    def _align_to_available_intent(self, previous_intent: str, available_intents: set[str]) -> str:
        if previous_intent in available_intents or not available_intents:
            return previous_intent
        return next(
            (intent for intent in available_intents if previous_intent in intent or intent in previous_intent),
            previous_intent,
        )

    def _context_decision(
        self,
        name: str,
        fallback: str,
        confidence: float,
        available_intents: set[str],
    ) -> ContextIntentDecision:
        resolved_name = name if name in available_intents or not available_intents else fallback
        alternatives = {fallback: round(1.0 - confidence, 6)} if fallback != resolved_name else {}
        return ContextIntentDecision(
            name=resolved_name,
            confidence=confidence,
            alternatives=alternatives,
        )

    def _match_single_value_hints(self, normalized_text: str, missing_slots: set[str]) -> list[ContextEntityHint]:
        hints: list[ContextEntityHint] = []
        if "people" in missing_slots:
            match = re.fullmatch(r"\s*(\d{1,2}|for\s+\d{1,2}(?:\s+\w+)?)\s*", normalized_text, flags=re.IGNORECASE)
            if match:
                hints.append(ContextEntityHint("PEOPLE_COUNT", match.group(1), match.start(1), match.end(1), 0.86))
        if "time" in missing_slots:
            match = re.search(r"\b(\d{1,2}(?::\d{2})?\s?(?:am|pm)|noon|midnight)\b", normalized_text, flags=re.IGNORECASE)
            if match:
                hints.append(ContextEntityHint("TIME", match.group(1), match.start(1), match.end(1), 0.86))
        if "date" in missing_slots:
            match = re.search(
                r"\b(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
                normalized_text,
                flags=re.IGNORECASE,
            )
            if match:
                hints.append(ContextEntityHint("DATE", match.group(1), match.start(1), match.end(1), 0.86))
        if "name" in missing_slots:
            match = re.fullmatch(r"\s*([A-Z][a-z]+(?:\s+[A-Z][a-z]+){0,2})\s*", normalized_text)
            if match:
                hints.append(ContextEntityHint("PERSON", match.group(1), match.start(1), match.end(1), 0.8))
        if "phone" in missing_slots:
            match = re.fullmatch(
                r"\s*((?:(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}|\d{3}[\s.-]?\d{4}))\s*",
                normalized_text,
            )
            if match:
                hints.append(ContextEntityHint("PHONE", match.group(1), match.start(1), match.end(1), 0.86))
        if "email" in missing_slots:
            match = re.fullmatch(r"\s*([A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,})\s*", normalized_text, flags=re.IGNORECASE)
            if match:
                hints.append(ContextEntityHint("EMAIL", match.group(1), match.start(1), match.end(1), 0.86))
        return hints

    def _match_phrase_hints(self, normalized_text: str, missing_slots: set[str]) -> list[ContextEntityHint]:
        hints: list[ContextEntityHint] = []
        phrase_specs = (
            ("menu_item", "MENU_ITEM"),
            ("price_item", "PRICE_ITEM"),
            ("location", "LOCATION"),
        )
        for slot_name, entity_type in phrase_specs:
            if slot_name not in missing_slots:
                continue
            if not re.fullmatch(r"[A-Za-z][A-Za-z\s-]{1,60}", normalized_text):
                continue
            stripped = normalized_text.strip()
            hints.append(
                ContextEntityHint(
                    entity_type,
                    stripped,
                    normalized_text.index(stripped),
                    normalized_text.index(stripped) + len(stripped),
                    0.84,
                )
            )
        return hints
