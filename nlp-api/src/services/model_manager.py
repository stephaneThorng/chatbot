"""Model download and cache management."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Any, Callable

from src.config import Settings, settings


def _import_transformers() -> tuple[Any, Any, Any, Any]:
    from transformers import (
        AutoModelForSequenceClassification,
        AutoModelForTokenClassification,
        AutoTokenizer,
        PreTrainedTokenizerBase,
    )

    return (
        AutoTokenizer,
        AutoModelForSequenceClassification,
        AutoModelForTokenClassification,
        PreTrainedTokenizerBase,
    )


@dataclass(slots=True)
class ModelBundle:
    """Loaded intent and NER artifacts."""

    intent_tokenizer: Any
    intent_model: Any
    ner_tokenizer: Any
    ner_model: Any


class ModelManager:
    """Loads models from Hugging Face Hub."""

    def __init__(
        self,
        config: Settings | None = None,
        importer: Callable[[], tuple[Any, Any, Any, Any]] = _import_transformers,
    ) -> None:
        self.config = config or settings
        self.importer = importer

    async def download_bundle(self) -> ModelBundle:
        """Download tokenizers and models concurrently."""

        return await asyncio.to_thread(self._download_bundle_sync)

    def _download_bundle_sync(self) -> ModelBundle:
        AutoTokenizer, AutoModelForSequenceClassification, AutoModelForTokenClassification, _ = self.importer()

        common_kwargs = {
            "revision": self.config.hf_model_revision,
            "token": self.config.hf_token,
            "cache_dir": str(self.config.cache_dir),
        }

        intent_tokenizer = AutoTokenizer.from_pretrained(self.config.hf_model_intent, **common_kwargs)
        intent_model = AutoModelForSequenceClassification.from_pretrained(self.config.hf_model_intent, **common_kwargs)
        ner_tokenizer = AutoTokenizer.from_pretrained(self.config.hf_model_ner, **common_kwargs)
        ner_model = AutoModelForTokenClassification.from_pretrained(self.config.hf_model_ner, **common_kwargs)
        return ModelBundle(
            intent_tokenizer=intent_tokenizer,
            intent_model=intent_model,
            ner_tokenizer=ner_tokenizer,
            ner_model=ner_model,
        )
