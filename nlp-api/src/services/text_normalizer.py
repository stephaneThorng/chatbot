"""Lightweight English text normalization for intent and NER."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Protocol

from src.config import Settings, settings


SLANG_MAP = {
    "pls": "please",
    "plz": "please",
    "thx": "thanks",
    "tmrw": "tomorrow",
    "tmr": "tomorrow",
    "u": "you",
    "ur": "your",
    "wtf": "what the fuck",
    "idk": "i do not know",
    "veg": "vegan",
    "gf": "gluten free",
    "tbl": "table",
    "reso": "reservation",
}

CONTRACTION_MAP = {
    "i'm": "i am",
    "can't": "cannot",
    "don't": "do not",
    "won't": "will not",
    "isn't": "is not",
    "aren't": "are not",
    "didn't": "did not",
    "doesn't": "does not",
    "i've": "i have",
    "we're": "we are",
    "they're": "they are",
    "you're": "you are",
    "that's": "that is",
    "what's": "what is",
    "it's": "it is",
}

PROTECTED_TOKEN_PATTERNS = (
    re.compile(r"^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$", re.IGNORECASE),
    re.compile(r"^(?:(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}|\d{3}[\s.-]?\d{4})$"),
    re.compile(r"^\d{1,2}(?::\d{2})?\s?(?:am|pm)$", re.IGNORECASE),
    re.compile(r"^\d+$"),
)


class SpellCorrectionBackend(Protocol):
    """Spell correction adapter protocol."""

    def correct(self, token: str) -> str | None:
        """Return a corrected token or None when no safe correction exists."""


@dataclass(frozen=True, slots=True)
class NormalizationReplacement:
    """Single normalization replacement record."""

    original: str
    normalized: str
    start: int
    end: int
    kind: str


@dataclass(frozen=True, slots=True)
class NormalizationResult:
    """Text normalization output."""

    original_text: str
    normalized_text: str
    replacements: tuple[NormalizationReplacement, ...]
    offset_map: tuple[int, ...]

    def map_span_to_original(self, start: int, end: int) -> tuple[int, int]:
        """Map a normalized-text span back to the original-text span."""

        if not self.offset_map:
            return start, end
        if start < 0:
            start = 0
        if end < start:
            end = start
        if start >= len(self.offset_map):
            return len(self.original_text), len(self.original_text)
        if end == 0:
            return 0, 0
        mapped_segment = self.offset_map[start : min(end, len(self.offset_map))]
        if not mapped_segment:
            anchor = self.offset_map[min(start, len(self.offset_map) - 1)]
            return anchor, anchor
        original_start = min(mapped_segment)
        original_end = max(mapped_segment) + 1
        return original_start, original_end


class SymSpellAdapter:
    """Thin adapter around symspellpy."""

    def __init__(self) -> None:
        from symspellpy import SymSpell, Verbosity  # type: ignore

        self._verbosity = Verbosity
        self._symspell = SymSpell(max_dictionary_edit_distance=1, prefix_length=7)
        for word in self._default_words():
            self._symspell.create_dictionary_entry(word, 1)

    def correct(self, token: str) -> str | None:
        suggestions = self._symspell.lookup(token, self._verbosity.TOP, max_edit_distance=1)
        if not suggestions:
            return None
        best = suggestions[0]
        if best.term == token:
            return None
        return str(best.term)

    def _default_words(self) -> tuple[str, ...]:
        return (
            "reservation",
            "book",
            "booking",
            "table",
            "people",
            "tomorrow",
            "today",
            "tonight",
            "vegan",
            "gluten",
            "free",
            "menu",
            "price",
            "location",
            "downtown",
            "contact",
            "email",
            "phone",
            "cancel",
            "change",
            "modify",
            "please",
            "thanks",
        )


class TextNormalizer:
    """Centralized low-maintenance text normalization."""

    TOKEN_PATTERN = re.compile(r"\S+")

    def __init__(
        self,
        config: Settings | None = None,
        spell_backend: SpellCorrectionBackend | None = None,
    ) -> None:
        self.config = config or settings
        self._spell_backend = spell_backend
        self._spell_backend_ready = spell_backend is not None

    def normalize(self, text: str) -> NormalizationResult:
        """Normalize text while preserving a map back to original spans."""

        if not self.config.enable_text_normalization:
            offset_map = self._identity_offset_map(text)
            return NormalizationResult(
                original_text=text,
                normalized_text=text,
                replacements=(),
                offset_map=offset_map,
            )

        original_text = text
        replacements: list[NormalizationReplacement] = []
        tokens: list[tuple[str, tuple[int, ...]]] = []
        for match in self.TOKEN_PATTERN.finditer(original_text):
            token = match.group(0)
            indices = tuple(range(match.start(), match.end()))
            tokens.append((token, indices))

        normalized_tokens: list[tuple[str, tuple[int, ...]]] = []
        for index, (token, indices) in enumerate(tokens):
            lowered = token.lower()
            next_token = tokens[index + 1][0].lower() if index + 1 < len(tokens) else None

            if next_token == "ppl" and lowered.isdigit():
                normalized_tokens.append((token, indices))
                continue

            compact_people_match = re.fullmatch(r"(\d{1,2})ppl", lowered)
            if compact_people_match:
                normalized_tokens.extend(
                    self._split_normalized_token(f"{compact_people_match.group(1)} people", indices)
                )
                replacements.append(
                    NormalizationReplacement(
                        original=token,
                        normalized=f"{compact_people_match.group(1)} people",
                        start=indices[0],
                        end=indices[-1] + 1,
                        kind="slang",
                    )
                )
                continue

            if lowered == "ppl" and normalized_tokens:
                previous_text, previous_indices = normalized_tokens[-1]
                if previous_text.isdigit():
                    normalized_tokens[-1] = (previous_text, previous_indices)
                    replacement_text = "people"
                    normalized_tokens.append((replacement_text, indices))
                    replacements.append(
                        NormalizationReplacement(
                            original="ppl",
                            normalized=replacement_text,
                            start=indices[0],
                            end=indices[-1] + 1,
                            kind="slang",
                        )
                    )
                    continue

            normalized_token = token
            normalized_indices = indices
            replacement_kind: str | None = None

            if lowered in SLANG_MAP:
                normalized_token = SLANG_MAP[lowered]
                replacement_kind = "slang"
            elif lowered in CONTRACTION_MAP:
                normalized_token = CONTRACTION_MAP[lowered]
                replacement_kind = "contraction"
            elif self.config.enable_spell_correction and self._should_spell_correct(lowered):
                corrected = self._spell_backend_or_none().correct(lowered) if self._spell_backend_or_none() else None
                if corrected and corrected != lowered:
                    normalized_token = corrected
                    replacement_kind = "spell"

            if replacement_kind is not None and normalized_token != token:
                replacements.append(
                    NormalizationReplacement(
                        original=token,
                        normalized=normalized_token,
                        start=indices[0],
                        end=indices[-1] + 1,
                        kind=replacement_kind,
                    )
                )

            normalized_tokens.extend(self._split_normalized_token(normalized_token, normalized_indices))

        normalized_text, offset_map = self._join_tokens(normalized_tokens)
        normalized_text = " ".join(normalized_text.strip().split())
        offset_map = tuple(offset_map[: len(normalized_text)])
        return NormalizationResult(
            original_text=original_text,
            normalized_text=normalized_text,
            replacements=tuple(replacements),
            offset_map=offset_map,
        )

    def _spell_backend_or_none(self) -> SpellCorrectionBackend | None:
        if self._spell_backend_ready:
            return self._spell_backend
        self._spell_backend_ready = True
        try:
            self._spell_backend = SymSpellAdapter()
        except Exception:
            self._spell_backend = None
        return self._spell_backend

    def _should_spell_correct(self, token: str) -> bool:
        if len(token) < 4:
            return False
        if any(character.isdigit() for character in token):
            return False
        if "@" in token:
            return False
        if token in SLANG_MAP or token in CONTRACTION_MAP:
            return False
        return not any(pattern.fullmatch(token) for pattern in PROTECTED_TOKEN_PATTERNS)

    def _split_normalized_token(self, token: str, indices: tuple[int, ...]) -> list[tuple[str, tuple[int, ...]]]:
        parts = token.split()
        if len(parts) == 1:
            return [(token, indices)]
        if not indices:
            return [(part, ()) for part in parts]
        return [(part, indices) for part in parts]

    def _join_tokens(self, tokens: list[tuple[str, tuple[int, ...]]]) -> tuple[str, list[int]]:
        if not tokens:
            return "", []
        pieces: list[str] = []
        offset_map: list[int] = []
        for index, (token, indices) in enumerate(tokens):
            if index > 0:
                pieces.append(" ")
                anchor = tokens[index - 1][1][-1] if tokens[index - 1][1] else 0
                offset_map.append(anchor)
            pieces.append(token)
            if not indices:
                offset_map.extend([0] * len(token))
                continue
            anchor = indices[0]
            offset_map.extend(indices[min(position, len(indices) - 1)] if len(indices) > 1 else anchor for position in range(len(token)))
        return "".join(pieces), offset_map

    def _identity_offset_map(self, text: str) -> tuple[int, ...]:
        return tuple(range(len(text)))
