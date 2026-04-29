"""Main NLP orchestration service."""

from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from typing import Any, Dict

from src.api.schemas import (
    AnalysisResponse,
    EntityResponse,
    HealthResponse,
    IntentResponse,
    ModelInfo,
    ProcessingDetails,
)
from src.config import Settings, settings
from src.models.intent_classifier import IntentClassifier
from src.models.ner_extractor import NERExtractor
from src.services.model_manager import ModelManager
from src.utils.logger import get_logger
from src.utils.metrics import metrics_collector


logger = get_logger(__name__)


class NLPService:
    """Coordinates classifier, NER, metrics, and health state."""

    def __init__(
        self,
        config: Settings | None = None,
        model_manager: ModelManager | None = None,
        intent_classifier: IntentClassifier | None = None,
        ner_extractor: NERExtractor | None = None,
    ) -> None:
        self.config = config or settings
        self.model_manager = model_manager or ModelManager(self.config)
        self.intent_classifier = intent_classifier or IntentClassifier(self.config)
        self.ner_extractor = ner_extractor or NERExtractor(self.config)
        self.startup_error: str | None = None

    async def initialize(self) -> None:
        """Load Hugging Face artifacts."""

        try:
            bundle = await self.model_manager.download_bundle()
        except Exception as exc:
            self.startup_error = str(exc)
            logger.exception("Model initialization failed.")
            return

        self.intent_classifier.bind_artifacts(bundle.intent_tokenizer, bundle.intent_model)
        self.ner_extractor.bind_artifacts(bundle.ner_tokenizer, bundle.ner_model)
        self.startup_error = None
        logger.info("Transformer models loaded successfully.")

    async def analyze(
        self,
        text: str,
        domain: str,
        context: Dict[str, Any] | None = None,
    ) -> AnalysisResponse:
        """Run complete analysis for a single utterance."""

        if not text.strip():
            raise ValueError("text must not be empty")

        request_metrics = metrics_collector.track_request()
        started = asyncio.get_running_loop().time()
        intent_task = asyncio.to_thread(self.intent_classifier.classify, text, domain, context)
        ner_task = asyncio.to_thread(self.ner_extractor.extract, text)

        try:
            intent_result, ner_result = await asyncio.gather(intent_task, ner_task)
            total_ms = (asyncio.get_running_loop().time() - started) * 1000
            request_metrics.intent_name = intent_result.name
            request_metrics.intent_confidence = intent_result.confidence
            request_metrics.entity_count = len(ner_result.entities)
            request_metrics.fast_path = intent_result.fast_path
            metrics_collector.finalize_request(request_metrics, success=True)
            return AnalysisResponse(
                intent=IntentResponse(
                    name=intent_result.name,
                    confidence=intent_result.confidence,
                    fast_path=intent_result.fast_path,
                    source=intent_result.source,
                    alternatives=intent_result.alternatives,
                ),
                entities=[
                    EntityResponse(
                        type=entity.type,
                        value=entity.value,
                        start=entity.start,
                        end=entity.end,
                        confidence=entity.confidence,
                        source=entity.source,
                    )
                    for entity in ner_result.entities
                ],
                processing_time_ms=round(total_ms, 3),
                processing_details=ProcessingDetails(
                    intent_ms=round(intent_result.processing_time_ms, 3),
                    ner_ms=round(ner_result.processing_time_ms, 3),
                    total_ms=round(total_ms, 3),
                ),
                model_info=ModelInfo(
                    intent_model=self.config.hf_model_intent,
                    ner_model=self.config.hf_model_ner,
                    revision=self.config.hf_model_revision,
                ),
            )
        except Exception:
            metrics_collector.finalize_request(request_metrics, success=False)
            logger.exception("Analysis failed.")
            raise

    def health(self) -> HealthResponse:
        """Return service health snapshot."""

        models_loaded = {
            "intent": self.intent_classifier.is_loaded,
            "ner": self.ner_extractor.is_loaded,
        }
        status_value = "ok" if all(models_loaded.values()) else "degraded"
        return HealthResponse(
            status=status_value,
            models_loaded=models_loaded,
            device=self.config.normalized_device,
            cache_dir=str(self.config.cache_dir),
            timestamp=datetime.now(timezone.utc),
        )
