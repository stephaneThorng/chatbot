"""Logging configuration."""

from __future__ import annotations

import logging
import logging.config
from logging.handlers import TimedRotatingFileHandler

from src.config import Settings, settings


_configured = False

try:
    from pythonjsonlogger.json import JsonFormatter
except ModuleNotFoundError:
    JsonFormatter = None


def configure_logging(config: Settings | None = None) -> None:
    """Configure console and rotating-file logging once."""

    global _configured
    if _configured:
        return

    active = config or settings
    file_handler = TimedRotatingFileHandler(
        filename=str(active.log_path),
        when="midnight",
        backupCount=7,
        encoding="utf-8",
    )
    if active.log_json and JsonFormatter is not None:
        file_handler.setFormatter(JsonFormatter("%(asctime)s %(name)s %(levelname)s %(message)s"))
    else:
        file_handler.setFormatter(logging.Formatter("%(asctime)s %(name)s %(levelname)s %(message)s"))

    console_handler = logging.StreamHandler()
    console_handler.setFormatter(logging.Formatter("%(asctime)s %(levelname)s %(name)s %(message)s"))

    root = logging.getLogger()
    root.setLevel(active.log_level.upper())
    root.handlers.clear()
    root.addHandler(console_handler)
    root.addHandler(file_handler)
    _configured = True


def get_logger(name: str) -> logging.Logger:
    """Return a configured logger."""

    configure_logging()
    return logging.getLogger(name)
