"""Intent classification with regex and transformer fallback."""

from __future__ import annotations

import re
import time
from dataclasses import dataclass
from typing import Any, Dict

from src.api.schemas import AnalysisContext
from src.config import Settings, settings


@dataclass(slots=True)
class IntentResult:
    """Classifier output."""

    name: str
    confidence: float
    fast_path: bool
    source: str
    alternatives: Dict[str, float]
    processing_time_ms: float


class IntentClassifier:
    """Hybrid intent classifier."""

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

    def __init__(self, config: Settings | None = None) -> None:
        self.config = config or settings
        self.model: Any | None = None
        self.tokenizer: Any | None = None
        self.label_map: Dict[int, str] = {}
        self.regex_patterns = self._compile_patterns()

    def _compile_patterns(self) -> Dict[str, Dict[str, re.Pattern[str]]]:
        compiled: Dict[str, Dict[str, re.Pattern[str]]] = {}
        for domain, intents in self.config.regex_patterns.items():
            compiled[domain] = {
                intent: re.compile("|".join(patterns), re.IGNORECASE)
                for intent, patterns in intents.items()
            }
        return compiled

    def bind_artifacts(self, tokenizer: Any, model: Any) -> None:
        """Attach pretrained artifacts."""

        self.tokenizer = tokenizer
        self.model = model
        raw_labels = getattr(model.config, "id2label", {}) or {}
        self.label_map = {int(key): value for key, value in raw_labels.items()}

    def classify(
        self,
        text: str,
        domain: str,
        context: AnalysisContext | None = None,
    ) -> IntentResult:
        """Resolve the most likely intent."""

        started = time.perf_counter()
        context_result = self._classify_with_context(text=text, domain=domain, context=context)
        if context_result is not None:
            context_result.processing_time_ms = (time.perf_counter() - started) * 1000
            return context_result

        if self.config.use_hybrid_intent:
            regex_result = self._classify_with_regex(text=text, domain=domain)
            if regex_result and regex_result.confidence >= self.config.intent_confidence_threshold:
                regex_result.processing_time_ms = (time.perf_counter() - started) * 1000
                return regex_result

        if self.model is None or self.tokenizer is None:
            fallback = regex_result or IntentResult(
                name="unknown",
                confidence=0.0,
                fast_path=False,
                source="unavailable",
                alternatives={},
                processing_time_ms=0.0,
            )
            fallback.processing_time_ms = (time.perf_counter() - started) * 1000
            return fallback

        encoded = self.tokenizer(
            text,
            return_tensors="pt",
            truncation=True,
            max_length=256,
        )
        import torch

        device = "cuda" if self.config.normalized_device == "cuda" and torch.cuda.is_available() else "cpu"
        encoded = {key: value.to(device) for key, value in encoded.items()}

        with torch.no_grad():
            logits = self.model(**encoded).logits[0]

        probabilities = torch.softmax(logits, dim=-1)
        top_values, top_indices = torch.topk(probabilities, k=min(5, probabilities.shape[0]))
        top_pairs = [(self.label_map.get(int(idx), f"label_{int(idx)}"), float(value)) for idx, value in zip(top_indices, top_values)]
        primary_label, primary_score = top_pairs[0]
        alternatives = {label: round(score, 6) for label, score in top_pairs[1:]}
        return IntentResult(
            name=primary_label,
            confidence=round(primary_score, 6),
            fast_path=False,
            source="intent_model",
            alternatives=alternatives,
            processing_time_ms=(time.perf_counter() - started) * 1000,
        )

    def _classify_with_context(
        self,
        text: str,
        domain: str,
        context: AnalysisContext | None,
    ) -> IntentResult | None:
        """Use conversational context to disambiguate short follow-ups."""

        if not context:
            return None

        normalized_text = text.strip().lower()
        if not normalized_text:
            return None

        previous_intent = str(context.current_intent or context.previous_intent or "").strip()
        if not previous_intent:
            return None

        available_intents = set(self.regex_patterns.get(domain, {}).keys()) | set(self.label_map.values())
        if previous_intent not in available_intents and available_intents:
            fallback_previous = next(
                (intent for intent in available_intents if previous_intent in intent or intent in previous_intent),
                previous_intent,
            )
            previous_intent = fallback_previous

        if any(marker in normalized_text for marker in self.CONTEXT_CANCELLATION_MARKERS):
            return self._context_result(name="reservation_cancel", fallback=previous_intent, confidence=0.94)

        if any(marker in normalized_text for marker in self.CONTEXT_MODIFICATION_MARKERS):
            return self._context_result(name="reservation_modify", fallback=previous_intent, confidence=0.93)

        if self._looks_like_follow_up(text, context):
            return IntentResult(
                name=previous_intent,
                confidence=0.88,
                fast_path=True,
                source="context",
                alternatives={},
                processing_time_ms=0.0,
            )
        return None

    def _context_result(self, name: str, fallback: str, confidence: float) -> IntentResult:
        """Build a context-driven intent result using supported labels when possible."""

        available_intents = set(self.label_map.values()) | {
            intent
            for domain_patterns in self.regex_patterns.values()
            for intent in domain_patterns
        }
        resolved_name = name if name in available_intents or not available_intents else fallback
        alternatives = {fallback: round(1.0 - confidence, 6)} if fallback != resolved_name else {}
        return IntentResult(
            name=resolved_name,
            confidence=confidence,
            fast_path=True,
            source="context",
            alternatives=alternatives,
            processing_time_ms=0.0,
        )

    def _looks_like_follow_up(self, text: str, context: AnalysisContext) -> bool:
        """Detect short contextual replies such as slot-only continuations."""

        normalized_text = text.strip().lower()
        missing_slots = self._infer_missing_slots(context)
        if not (context.previous_slots or context.slots_filled or missing_slots):
            return False

        if any(self._matches_slot_value_shape(normalized_text, slot) for slot in missing_slots):
            return True
        if self._matches_multi_slot_follow_up(normalized_text, missing_slots):
            return True

        slot_like_patterns = [
            r"^\d{1,2}$",
            r"^for\s+\d{1,2}$",
            r"^\d{1,2}\s*(people|persons|guests?|personnes?)$",
            r"^\d{1,2}(?::\d{2})?\s?(am|pm)$",
            r"^(today|tomorrow|tonight|this weekend|next week|monday|tuesday|wednesday|thursday|friday|saturday|sunday)$",
            r"^[a-z]+(?:\s+[a-z]+){0,2}$",
        ]
        has_slot_shape = any(re.fullmatch(pattern, normalized_text) for pattern in slot_like_patterns)
        return has_slot_shape

    def _infer_missing_slots(self, context: AnalysisContext) -> set[str]:
        """Return the contextual slots that are still expected."""

        previous_slots = context.previous_slots or context.slots_filled or None
        previous_slot_values = previous_slots.model_dump(exclude_none=True) if previous_slots else {}
        required_slots = {str(slot).lower() for slot in (context.required_slots or [])}
        if not required_slots:
            return set()
        return {
            slot
            for slot in required_slots
            if slot not in {str(key).lower() for key in previous_slot_values}
        }

    def _matches_slot_value_shape(self, normalized_text: str, slot: str) -> bool:
        """Check whether text looks like a value for the given conversational slot."""

        slot_patterns = {
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
            ),
            "time": (
                r"^\d{1,2}(?::\d{2})?\s?(am|pm)$",
                r"^(at\s+)?\d{1,2}(?::\d{2})?\s?(am|pm)$",
            ),
            "name": (
                r"^[a-z]+(?:\s+[a-z]+){1,2}$",
            ),
            "phone": (
                r"^(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}$",
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
        patterns = slot_patterns.get(slot, ())
        return any(re.fullmatch(pattern, normalized_text) for pattern in patterns)

    def _matches_multi_slot_follow_up(self, normalized_text: str, missing_slots: set[str]) -> bool:
        """Detect short follow-ups that contain multiple slot values in one sentence."""

        has_date = "date" in missing_slots and any(
            re.search(
                pattern,
                normalized_text,
            )
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

    def _classify_with_regex(self, text: str, domain: str) -> IntentResult | None:
        patterns = self.regex_patterns.get(domain) or self.regex_patterns.get("restaurant", {})
        scored: list[tuple[str, float]] = []
        normalized = text.lower()
        for intent, pattern in patterns.items():
            matches = list(pattern.finditer(normalized))
            if not matches:
                continue
            longest = max(len(match.group(0)) for match in matches)
            score = min(0.55 + 0.08 * len(matches) + 0.01 * longest, 0.99)
            scored.append((intent, round(score, 6)))
        if not scored:
            return None
        scored.sort(key=lambda item: item[1], reverse=True)
        winner, confidence = scored[0]
        alternatives = {intent: score for intent, score in scored[1:5]}
        return IntentResult(
            name=winner,
            confidence=confidence,
            fast_path=True,
            source="regex",
            alternatives=alternatives,
            processing_time_ms=0.0,
        )

    @property
    def is_loaded(self) -> bool:
        """Whether a transformer model is available."""

        return self.model is not None and self.tokenizer is not None
