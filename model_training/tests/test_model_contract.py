from __future__ import annotations

from pathlib import Path

import torch
from transformers import XLMRobertaConfig

from nlu_training.model import MultiTaskNluModel


def test_multitask_model_forward_and_save_contract(tmp_path: Path) -> None:
    config = XLMRobertaConfig(
        vocab_size=32,
        hidden_size=16,
        intermediate_size=32,
        num_hidden_layers=1,
        num_attention_heads=2,
        max_position_embeddings=32,
    )
    config.num_intent_labels = 3
    config.num_ner_labels = 5
    model = MultiTaskNluModel(config, intent_class_weights=[1.0, 2.0, 0.5])

    outputs = model(
        input_ids=torch.tensor([[0, 4, 5, 2]]),
        attention_mask=torch.tensor([[1, 1, 1, 1]]),
        labels=torch.tensor([1]),
        ner_labels=torch.tensor([[-100, 1, 2, -100]]),
    )

    assert outputs.loss is not None
    assert outputs.logits.shape == (1, 3)
    assert outputs.ner_logits.shape == (1, 4, 5)

    model.save_pretrained(tmp_path)
    assert (tmp_path / "config.json").exists()
    assert any(path.name.startswith("model") for path in tmp_path.iterdir())
