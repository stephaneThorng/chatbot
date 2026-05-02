"""Request and response schemas."""

from __future__ import annotations

from datetime import datetime
from enum import StrEnum
from typing import Dict, List

from pydantic import BaseModel, ConfigDict, Field, field_validator


class SchemaModel(BaseModel):
    """Base schema configuration shared across API models."""

    model_config = ConfigDict(protected_namespaces=())


class IntentName(StrEnum):
    """Supported intent labels."""

    RESERVATION_CREATE = "reservation_create"
    RESERVATION_MODIFY = "reservation_modify"
    RESERVATION_CANCEL = "reservation_cancel"
    RESERVATION_STATUS = "reservation_status"
    MENU_REQUEST = "menu_request"
    OPENING_HOURS = "opening_hours"
    LOCATION_REQUEST = "location_request"
    PRICING_REQUEST = "pricing_request"
    CONTACT_REQUEST = "contact_request"
    UNKNOWN = "unknown"


class EntityType(StrEnum):
    """Supported entity labels."""

    DATE = "DATE"
    TIME = "TIME"
    PEOPLE_COUNT = "PEOPLE_COUNT"
    PERSON = "PERSON"
    PHONE = "PHONE"
    EMAIL = "EMAIL"
    MENU_ITEM = "MENU_ITEM"
    PRICE_ITEM = "PRICE_ITEM"
    LOCATION = "LOCATION"


class SlotName(StrEnum):
    """Supported conversational slot names."""

    DATE = "date"
    TIME = "time"
    PEOPLE = "people"
    NAME = "name"
    PHONE = "phone"
    EMAIL = "email"
    MENU_ITEM = "menu_item"
    PRICE_ITEM = "price_item"
    LOCATION = "location"


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

    current_intent: IntentName | None = None
    previous_intent: IntentName | None = None
    previous_slots: ContextSlots | None = None
    slots_filled: ContextSlots | None = None
    required_slots: List[SlotName] = Field(default_factory=list)


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

    name: IntentName
    confidence: float = Field(..., ge=0.0, le=1.0)
    fast_path: bool = False
    source: str
    alternatives: Dict[str, float] = Field(default_factory=dict)


class EntityResponse(SchemaModel):
    """Entity payload."""

    type: EntityType
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
