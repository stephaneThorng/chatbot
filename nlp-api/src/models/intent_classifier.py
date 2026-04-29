"""Intent classification with regex and transformer fallback."""

from __future__ import annotations

import re
import time
from dataclasses import dataclass
from typing import Any, Dict

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
        context: Dict[str, Any] | None = None,
    ) -> IntentResult:
        """Resolve the most likely intent."""

        del context
        started = time.perf_counter()
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
