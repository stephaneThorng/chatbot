"""Request and response schemas."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator


class SchemaModel(BaseModel):
    """Base schema configuration shared across API models."""

    model_config = ConfigDict(protected_namespaces=())


class ContextSlots(SchemaModel):
    """Known conversational slots from upstream orchestration."""

    date: str | None = None
    time: str | None = None
    people: str | None = None
    name: str | None = None
    phone: str | None = None
    email: str | None = None
    menu_item: str | None = None
    price_item: str | None = None
    location: str | None = None


class AnalysisContext(SchemaModel):
    """Typed conversation context passed by the caller."""

    current_intent: str | None = None
    previous_intent: str | None = None
    previous_slots: ContextSlots | None = None
    slots_filled: ContextSlots | None = None
    required_slots: List[
        Literal[
            "date",
            "time",
            "people",
            "name",
            "phone",
            "email",
            "menu_item",
            "price_item",
            "location",
        ]
    ] = Field(default_factory=list)


class AnalysisRequest(SchemaModel):
    """Incoming analysis request."""

    text: str = Field(..., min_length=1, max_length=2000)
    domain: str = Field(..., min_length=1, max_length=64)
    context: AnalysisContext | None = None

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


class IntentResponse(SchemaModel):
    """Intent payload."""

    name: str
    confidence: float = Field(..., ge=0.0, le=1.0)
    fast_path: bool = False
    source: str
    alternatives: Dict[str, float] = Field(default_factory=dict)


class EntityResponse(SchemaModel):
    """Entity payload."""

    type: str
    value: str
    start: int = Field(..., ge=0)
    end: int = Field(..., ge=0)
    confidence: float = Field(..., ge=0.0, le=1.0)
    source: str


class ProcessingDetails(SchemaModel):
    """Timing breakdown."""

    intent_ms: float = Field(..., ge=0.0)
    ner_ms: float = Field(..., ge=0.0)
    total_ms: float = Field(..., ge=0.0)


class ModelInfo(SchemaModel):
    """Model version details."""

    intent_model: str
    ner_model: str
    revision: str


class AnalysisResponse(SchemaModel):
    """Combined NLP result."""

    intent: IntentResponse
    entities: List[EntityResponse] = Field(default_factory=list)
    processing_time_ms: float = Field(..., ge=0.0)
    processing_details: ProcessingDetails
    model_info: ModelInfo


class HealthResponse(SchemaModel):
    """Health payload."""

    status: str
    models_loaded: Dict[str, bool]
    device: str
    cache_dir: str
    timestamp: datetime


class ErrorResponse(SchemaModel):
    """Error payload."""

    error: str
    message: str
    details: Dict[str, object] | None = None
