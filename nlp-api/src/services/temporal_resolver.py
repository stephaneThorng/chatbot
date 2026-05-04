"""Date and time normalization helpers."""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass
from datetime import date, datetime, timedelta
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from src.config import Settings, settings


@dataclass(frozen=True, slots=True)
class ResolutionResult:
    """Canonical resolved value plus metadata."""

    value: str
    resolution: str
    warning: str | None = None


class TemporalResolver:
    """Resolves natural-language dates and times using the service clock."""

    MONTHS = {
        "january": 1,
        "february": 2,
        "march": 3,
        "april": 4,
        "may": 5,
        "june": 6,
        "july": 7,
        "august": 8,
        "september": 9,
        "october": 10,
        "november": 11,
        "december": 12,
    }
    WEEKDAYS = {
        "monday": 0,
        "tuesday": 1,
        "wednesday": 2,
        "thursday": 3,
        "friday": 4,
        "saturday": 5,
        "sunday": 6,
    }

    def __init__(
        self,
        config: Settings | None = None,
        now_provider: Callable[[], datetime | date] | None = None,
    ) -> None:
        self.config = config or settings
        self._now_provider = now_provider

    def resolve_date(self, raw_value: str) -> ResolutionResult | None:
        """Resolve a supported date phrase to ISO date."""

        normalized = self._normalize(raw_value)
        today = self._today()
        if normalized in {"today"}:
            return ResolutionResult(today.isoformat(), "relative_date")
        if normalized in {"tomorrow", "tmrw", "tmr"}:
            return ResolutionResult((today + timedelta(days=1)).isoformat(), "relative_date")
        if normalized == "tonight":
            return ResolutionResult(today.isoformat(), "relative_date")
        if match := re.fullmatch(r"in\s+(\d{1,2})\s+weeks?", normalized):
            return ResolutionResult((today + timedelta(weeks=int(match.group(1)))).isoformat(), "relative_date")
        if match := re.fullmatch(r"(?:next\s+)?(monday|tuesday|wednesday|thursday|friday|saturday|sunday)", normalized):
            weekday = self.WEEKDAYS[match.group(1)]
            days_ahead = (weekday - today.weekday()) % 7
            if days_ahead == 0 or normalized.startswith("next "):
                days_ahead += 7
            return ResolutionResult((today + timedelta(days=days_ahead)).isoformat(), "weekday_date")
        if match := re.fullmatch(
            r"(january|february|march|april|may|june|july|august|september|october|november|december)\s+(\d{1,2})",
            normalized,
        ):
            month = self.MONTHS[match.group(1)]
            day = int(match.group(2))
            candidate = date(today.year, month, day)
            if candidate < today:
                candidate = date(today.year + 1, month, day)
            return ResolutionResult(candidate.isoformat(), "month_day")
        return None

    def resolve_time(self, raw_value: str) -> ResolutionResult | None:
        """Resolve a supported time phrase to HH:mm."""

        normalized = self._normalize(raw_value)
        if normalized in {"tomorrow evening", "this evening"}:
            return ResolutionResult(
                value=raw_value,
                resolution="part_of_day",
                warning=f"Ambiguous time expression: {raw_value}",
            )
        if normalized == "noon":
            return ResolutionResult("12:00", "named_time")
        if normalized == "midnight":
            return ResolutionResult("00:00", "named_time")
        if match := re.fullmatch(r"(\d{1,2})(?::(\d{2}))?\s*(am|pm)", normalized):
            hour = int(match.group(1))
            minute = int(match.group(2) or "0")
            suffix = match.group(3)
            if hour == 12:
                hour = 0
            if suffix == "pm":
                hour += 12
            return ResolutionResult(f"{hour:02d}:{minute:02d}", "time_12h")
        if match := re.fullmatch(r"(\d{1,2})[:h](\d{2})", normalized):
            hour = int(match.group(1))
            minute = int(match.group(2))
            if 0 <= hour <= 23 and 0 <= minute <= 59:
                return ResolutionResult(f"{hour:02d}:{minute:02d}", "time_24h")
        return None

    def _today(self) -> date:
        now = self._now_provider() if self._now_provider else self._now()
        if isinstance(now, datetime):
            return now.date()
        if isinstance(now, date):
            return now
        raise TypeError("now_provider must return datetime or date")

    def _now(self) -> datetime:
        try:
            return datetime.now(ZoneInfo(self.config.service_timezone))
        except ZoneInfoNotFoundError:
            return datetime.now()

    def _normalize(self, value: str) -> str:
        return " ".join(value.strip().lower().replace(".", "").split())
