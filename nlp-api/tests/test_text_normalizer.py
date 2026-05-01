from __future__ import annotations

from src.config import Settings
from src.services.text_normalizer import TextNormalizer


class FakeSpellBackend:
    def __init__(self, mapping: dict[str, str]) -> None:
        self.mapping = mapping

    def correct(self, token: str) -> str | None:
        return self.mapping.get(token)


def test_text_normalizer_expands_slang_and_restaurant_abbreviations() -> None:
    normalizer = TextNormalizer(Settings())
    result = normalizer.normalize("pls book a tbl tmrw for 2 ppl")
    assert result.normalized_text == "please book a table tomorrow for 2 people"


def test_text_normalizer_expands_contractions() -> None:
    normalizer = TextNormalizer(Settings())
    result = normalizer.normalize("i can't make it tonight")
    assert result.normalized_text == "i cannot make it tonight"


def test_text_normalizer_handles_compact_people_and_reso() -> None:
    normalizer = TextNormalizer(Settings())
    result = normalizer.normalize("need a reso for 4ppl tmrw at 7")
    assert result.normalized_text == "need a reservation for 4 people tomorrow at 7"


def test_text_normalizer_leaves_protected_tokens_unchanged() -> None:
    normalizer = TextNormalizer(
        Settings(),
        spell_backend=FakeSpellBackend(
            {
                "events@example.com": "wrong@example.com",
                "555-0108": "000-0000",
                "7pm": "8pm",
            }
        ),
    )
    result = normalizer.normalize("events@example.com 555-0108 7pm")
    assert result.normalized_text == "events@example.com 555-0108 7pm"


def test_text_normalizer_uses_spell_backend_conservatively() -> None:
    normalizer = TextNormalizer(
        Settings(),
        spell_backend=FakeSpellBackend({"restarant": "restaurant"}),
    )
    result = normalizer.normalize("restarant booking")
    assert result.normalized_text == "restaurant booking"


def test_text_normalizer_falls_back_when_spell_backend_fails(monkeypatch) -> None:
    def raising_adapter():
        raise RuntimeError("boom")

    monkeypatch.setattr("src.services.text_normalizer.SymSpellAdapter", raising_adapter)
    normalizer = TextNormalizer(Settings(enable_spell_correction=True))
    result = normalizer.normalize("pls book tmrw")
    assert result.normalized_text == "please book tomorrow"
