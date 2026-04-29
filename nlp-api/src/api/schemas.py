"""Request and response schemas."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from pydantic import BaseModel, Field, field_validator


class AnalysisRequest(BaseModel):
    """Incoming analysis request."""

    text: str = Field(..., min_length=1, max_length=2000)
    domain: str = Field(..., min_length=1, max_length=64)
    context: Dict[str, Any] | None = None

    @field_validator("text")
    @classmethod
    def validate_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("text must not be blank")
        return cleaned

    @field_validator("domain")
    @classmethod
    def validate_domain(cls, value: str) -> str:
        cleaned = value.strip().lower()
        if not cleaned:
            raise ValueError("domain must not be blank")
        return cleaned


class IntentResponse(BaseModel):
    """Intent payload."""

    name: str
    confidence: float = Field(..., ge=0.0, le=1.0)
    fast_path: bool = False
    source: str
    alternatives: Dict[str, float] = Field(default_factory=dict)


class EntityResponse(BaseModel):
    """Entity payload."""

    type: str
    value: str
    start: int = Field(..., ge=0)
    end: int = Field(..., ge=0)
    confidence: float = Field(..., ge=0.0, le=1.0)
    source: str


class ProcessingDetails(BaseModel):
    """Timing breakdown."""

    intent_ms: float = Field(..., ge=0.0)
    ner_ms: float = Field(..., ge=0.0)
    total_ms: float = Field(..., ge=0.0)


class ModelInfo(BaseModel):
    """Model version details."""

    intent_model: str
    ner_model: str
    revision: str


class AnalysisResponse(BaseModel):
    """Combined NLP result."""

    intent: IntentResponse
    entities: List[EntityResponse] = Field(default_factory=list)
    processing_time_ms: float = Field(..., ge=0.0)
    processing_details: ProcessingDetails
    model_info: ModelInfo


class HealthResponse(BaseModel):
    """Health payload."""

    status: str
    models_loaded: Dict[str, bool]
    device: str
    cache_dir: str
    timestamp: datetime


class ErrorResponse(BaseModel):
    """Error payload."""

    error: str
    message: str
    details: Dict[str, Any] | None = None
