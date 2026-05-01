"""Application configuration."""

from __future__ import annotations

from pathlib import Path
from typing import Dict, List

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


DEFAULT_REGEX_PATTERNS: Dict[str, Dict[str, List[str]]] = {
    "restaurant": {
        "reservation_create": [
            r"\bbook\b",
            r"\bnew reservation\b",
            r"\bnew booking\b",
            r"\btable booking\b",
            r"\bbook a table\b",
        ],
        "reservation_modify": [
            r"\bchange\b",
            r"\bupdate\b",
            r"\bmove\b",
            r"\breschedule\b",
            r"\bmodify\b",
            r"\bexisting reservation\b",
        ],
        "reservation_cancel": [
            r"\bcancel\b",
            r"\bdrop\b",
            r"\bremove\b",
        ],
        "menu_request": [
            r"\bmenu\b",
            r"\bvegan\b",
            r"\bgluten free\b",
            r"\bdessert\b",
            r"\bdrink list\b",
        ],
        "opening_hours": [
            r"\bopen\b",
            r"\bclose\b",
            r"\bhours\b",
            r"\bopening hours\b",
        ],
        "location_request": [
            r"\baddress\b",
            r"\blocated\b",
            r"\bwhere are you\b",
            r"\bnear\b",
            r"\bparking\b",
        ],
        "pricing_request": [
            r"\bprice\b",
            r"\bcost\b",
            r"\bhow much\b",
            r"\bprice range\b",
        ],
        "greeting_contact": [
            r"\bhello\b",
            r"\bhi\b",
            r"\bphone\b",
            r"\bemail\b",
            r"\bcontact\b",
        ],
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

    hf_model_intent: str = "artifacts/restaurant_intent"
    hf_model_ner: str = "artifacts/restaurant_ner"
    hf_model_revision: str = "main"
    hf_token: str | None = None
    hf_cache_dir: str = "./.cache/huggingface"

    service_port: int = 8000
    service_host: str = "0.0.0.0"

    log_level: str = "INFO"
    log_file: str = "./logs/nlp-api.log"
    log_json: bool = False

    intent_confidence_threshold: float = Field(default=0.6, ge=0.0, le=1.0)
    use_hybrid_intent: bool = True
    ner_confidence_threshold: float = Field(default=0.5, ge=0.0, le=1.0)
    enable_text_normalization: bool = True
    enable_spell_correction: bool = True
    spacy_model: str = "en_core_web_sm"

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
