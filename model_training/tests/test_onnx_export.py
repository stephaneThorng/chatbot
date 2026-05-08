from __future__ import annotations

import json

import torch
from transformers import XLMRobertaConfig

from nlu_training.model import MultiTaskNluModel
from nlu_training.onnx_export import (
    OnnxExportWrapper,
    build_export_contract,
    write_export_contract,
)


def test_export_wrapper_returns_named_model_heads() -> None:
    config = XLMRobertaConfig(
        vocab_size=32,
        hidden_size=16,
        intermediate_size=32,
        num_hidden_layers=1,
        num_attention_heads=2,
        max_position_embeddings=64,
    )
    config.num_intent_labels = 3
    config.num_ner_labels = 5
    model = MultiTaskNluModel(config, intent_class_weights=[1.0, 1.0, 1.0])
    wrapper = OnnxExportWrapper(model)

    intent_logits, ner_logits = wrapper(
        input_ids=torch.tensor([[0, 4, 5, 2]]),
        attention_mask=torch.tensor([[1, 1, 1, 1]]),
    )

    assert intent_logits.shape == (1, 3)
    assert ner_logits.shape == (1, 4, 5)


def test_contract_records_shared_preprocessing_rules(tmp_path) -> None:
    config = {
        "model": {"max_length": 160},
        "tags": {
            "languages": ["en", "id"],
            "domains": ["restaurant", "hotel"],
            "tasks": ["WF_BOOK", "WF_CANCEL"],
        },
        "intents": {
            "labels": ["book", "provide_info"],
        },
        "entities": {
            "labels": ["person", "date"],
        },
    }

    contract = build_export_contract(config)
    write_export_contract(tmp_path, contract)
    payload = json.loads((tmp_path / "onnx_contract.json").read_text(encoding="utf-8"))

    assert payload["model_inputs"] == ["input_ids", "attention_mask"]
    assert payload["model_outputs"] == ["intent_logits", "ner_logits"]
    assert payload["preprocessing"]["task_is_optional"] is True
    assert payload["preprocessing"]["context_tags_are_entities"] is False
    assert payload["labels"]["ner"] == ["O", "B-person", "I-person", "B-date", "I-date"]
