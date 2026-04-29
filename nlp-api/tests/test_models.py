from __future__ import annotations

from types import SimpleNamespace

from src.services.model_manager import ModelManager


class StubTokenizer:
    @classmethod
    def from_pretrained(cls, *args, **kwargs):
        return {"args": args, "kwargs": kwargs}


class StubSequenceModel:
    config = SimpleNamespace(id2label={0: "reservation"})

    @classmethod
    def from_pretrained(cls, *args, **kwargs):
        return cls()


class StubTokenModel:
    config = SimpleNamespace(id2label={0: "O", 1: "B-DATE"})

    @classmethod
    def from_pretrained(cls, *args, **kwargs):
        return cls()


def test_model_manager_download_bundle_uses_configured_revision() -> None:
    def importer():
        return StubTokenizer, StubSequenceModel, StubTokenModel, object

    manager = ModelManager(importer=importer)
    bundle = manager._download_bundle_sync()
    assert bundle.intent_tokenizer["kwargs"]["revision"] == "main"
    assert isinstance(bundle.intent_model, StubSequenceModel)
    assert isinstance(bundle.ner_model, StubTokenModel)
