"""Multi-task transformer model for intent classification and NER."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import torch
from torch import nn
from torch.nn import CrossEntropyLoss
from transformers import AutoConfig, AutoModel, PreTrainedModel
from transformers.modeling_outputs import ModelOutput


@dataclass
class NluModelOutput(ModelOutput):
    loss: torch.Tensor | None = None
    logits: torch.Tensor | None = None
    ner_logits: torch.Tensor | None = None


class MultiTaskNluModel(PreTrainedModel):
    config_class = AutoConfig
    base_model_prefix = "encoder"

    def __init__(
        self,
        config: Any,
        intent_class_weights: list[float] | None = None,
    ) -> None:
        super().__init__(config)
        self.num_intent_labels = int(config.num_intent_labels)
        self.num_ner_labels = int(config.num_ner_labels)
        self.encoder = AutoModel.from_config(config)
        dropout_prob = getattr(config, "classifier_dropout", None)
        if dropout_prob is None:
            dropout_prob = getattr(config, "hidden_dropout_prob", 0.1)
        self.dropout = nn.Dropout(dropout_prob)
        self.intent_classifier = nn.Linear(config.hidden_size, self.num_intent_labels)
        self.ner_classifier = nn.Linear(config.hidden_size, self.num_ner_labels)
        weights = intent_class_weights or [1.0] * self.num_intent_labels
        self.register_buffer("intent_class_weights", torch.tensor(weights, dtype=torch.float))
        self.post_init()

    @classmethod
    def from_base_model(
        cls,
        model_name: str,
        num_intent_labels: int,
        num_ner_labels: int,
        intent_label2id: dict[str, int],
        intent_id2label: dict[int, str],
        ner_label2id: dict[str, int],
        ner_id2label: dict[int, str],
        intent_class_weights: list[float] | None,
    ) -> "MultiTaskNluModel":
        config = AutoConfig.from_pretrained(model_name)
        config.num_intent_labels = num_intent_labels
        config.num_ner_labels = num_ner_labels
        config.intent_label2id = intent_label2id
        config.intent_id2label = {str(key): value for key, value in intent_id2label.items()}
        config.ner_label2id = ner_label2id
        config.ner_id2label = {str(key): value for key, value in ner_id2label.items()}
        model = cls(config, intent_class_weights=intent_class_weights)
        base_encoder = AutoModel.from_pretrained(model_name, config=config)
        model.encoder = base_encoder
        return model

    def forward(
        self,
        input_ids: torch.Tensor | None = None,
        attention_mask: torch.Tensor | None = None,
        token_type_ids: torch.Tensor | None = None,
        labels: torch.Tensor | None = None,
        ner_labels: torch.Tensor | None = None,
        **kwargs: Any,
    ) -> NluModelOutput:
        encoder_inputs: dict[str, Any] = {
            "input_ids": input_ids,
            "attention_mask": attention_mask,
        }
        if token_type_ids is not None:
            encoder_inputs["token_type_ids"] = token_type_ids
        outputs = self.encoder(**encoder_inputs)
        sequence_output = self.dropout(outputs.last_hidden_state)
        pooled_output = sequence_output[:, 0, :]

        intent_logits = self.intent_classifier(pooled_output)
        ner_logits = self.ner_classifier(sequence_output)

        loss = None
        if labels is not None:
            intent_loss = CrossEntropyLoss(weight=self.intent_class_weights)(intent_logits, labels)
            loss = intent_loss
        if ner_labels is not None:
            ner_loss = CrossEntropyLoss(ignore_index=-100)(
                ner_logits.view(-1, self.num_ner_labels),
                ner_labels.view(-1),
            )
            loss = ner_loss if loss is None else loss + ner_loss

        return NluModelOutput(
            loss=loss,
            logits=intent_logits,
            ner_logits=ner_logits,
        )
