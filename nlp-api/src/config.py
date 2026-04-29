"""Application configuration."""

from __future__ import annotations

from pathlib import Path
from typing import Dict, List

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


DEFAULT_REGEX_PATTERNS: Dict[str, Dict[str, List[str]]] = {
    "restaurant": {
        "reservation": [r"\breserv", r"\btable\b", r"\bpersonnes?\b", r"\bdemain\b"],
        "menu": [r"\bmenu\b", r"\bcarte\b", r"\bplat"],
        "horaires": [r"\bhoraire", r"\bouvert", r"\bheure"],
        "localisation": [r"\badresse\b", r"\bou\b", r"\bsitue"],
    },
    "hotel": {
        "reservation": [r"\breserv", r"\bchambre\b", r"\bnight\b"],
        "check_in": [r"\barrivee\b", r"\bcheck[\s-]?in\b"],
        "check_out": [r"\bdepart\b", r"\bcheck[\s-]?out\b"],
        "services": [r"\bspa\b", r"\bpetit dejeuner\b", r"\bparking\b"],
    },
    "spa": {
        "reservation": [r"\brdv\b", r"\breserv", r"\bappointment\b"],
        "services": [r"\bmassage\b", r"\bsoin\b", r"\bsauna\b"],
        "pricing": [r"\bprix\b", r"\btarif\b", r"\bcombien\b"],
    },
}


class Settings(BaseSettings):
    """Environment-backed service settings."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    hf_model_intent: str = "your-org/nlp-intent-classifier"
    hf_model_ner: str = "your-org/nlp-ner-model"
    hf_model_revision: str = "main"
    hf_token: str | None = None
    hf_cache_dir: str = "./models"

    service_port: int = 8000
    service_host: str = "0.0.0.0"

    log_level: str = "INFO"
    log_file: str = "./logs/nlp-api.log"
    log_json: bool = False

    intent_confidence_threshold: float = Field(default=0.6, ge=0.0, le=1.0)
    use_hybrid_intent: bool = True
    ner_confidence_threshold: float = Field(default=0.5, ge=0.0, le=1.0)

    device: str = "cpu"
    uvicorn_workers: int = 1
    regex_patterns: Dict[str, Dict[str, List[str]]] = Field(
        default_factory=lambda: DEFAULT_REGEX_PATTERNS.copy()
    )

    @property
    def cache_dir(self) -> Path:
        path = Path(self.hf_cache_dir)
        path.mkdir(parents=True, exist_ok=True)
        return path

    @property
    def log_path(self) -> Path:
        path = Path(self.log_file)
        path.parent.mkdir(parents=True, exist_ok=True)
        return path

    @property
    def normalized_device(self) -> str:
        return self.device.lower()


settings = Settings()
