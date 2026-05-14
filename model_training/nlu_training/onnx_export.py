"""ONNX export helpers for the multi-task NLU model."""

from __future__ import annotations

import json
import shutil
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import torch
from torch import nn

from nlu_training.config import build_label_maps, build_ner_labels
from nlu_training.data import NluDataCollator, NluDataset
from nlu_training.schema import TrainingExample


@dataclass(frozen=True, slots=True)
class OnnxContract:
    model_input_names: tuple[str, ...]
    model_output_names: tuple[str, ...]
    max_length: int
    default_domain: str
    default_language: str
    available_languages: tuple[str, ...]
    available_domains: tuple[str, ...]
    available_tasks: tuple[str, ...]
    intents: tuple[str, ...]
    ner_labels: tuple[str, ...]

    def to_dict(self) -> dict[str, Any]:
        return {
            "format_version": 1,
            "model_inputs": list(self.model_input_names),
            "model_outputs": list(self.model_output_names),
            "max_length": self.max_length,
            "preprocessing": {
                "tagged_input_template": "[TASK={task}] [LANG={lang}] [DOMAIN={domain}] {text}",
                "task_is_optional": True,
                "task_tag_omitted_when_missing": True,
                "default_domain": self.default_domain,
                "default_language": self.default_language,
                "available_languages": list(self.available_languages),
                "available_domains": list(self.available_domains),
                "available_tasks": list(self.available_tasks),
                "context_tags_are_entities": False,
            },
            "labels": {
                "intents": list(self.intents),
                "ner": list(self.ner_labels),
            },
        }


class OnnxExportWrapper(nn.Module):
    """Expose a tuple output that is easy to serialize to ONNX."""

    def __init__(self, model: nn.Module) -> None:
        super().__init__()
        self.model = model

    def forward(self, input_ids: torch.Tensor, attention_mask: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        outputs = self.model(
            input_ids=input_ids,
            attention_mask=attention_mask,
        )
        return outputs.logits, outputs.ner_logits


def build_export_contract(config: dict[str, Any]) -> OnnxContract:
    intents = tuple(config["intents"]["labels"])
    ner_labels = tuple(build_ner_labels(list(config["entities"]["labels"])))
    languages = tuple(config["tags"]["languages"])
    domains = tuple(config["tags"]["domains"])
    tasks = tuple(config["tags"]["tasks"])
    return OnnxContract(
        model_input_names=("input_ids", "attention_mask"),
        model_output_names=("intent_logits", "ner_logits"),
        max_length=int(config["model"]["max_length"]),
        default_domain=str(domains[0]),
        default_language=str(languages[0]),
        available_languages=languages,
        available_domains=domains,
        available_tasks=tasks,
        intents=intents,
        ner_labels=ner_labels,
    )


def write_export_contract(output_dir: str | Path, contract: OnnxContract) -> None:
    target = Path(output_dir) / "onnx_contract.json"
    target.write_text(json.dumps(contract.to_dict(), indent=2, sort_keys=True), encoding="utf-8")


def copy_runtime_artifacts(model_dir: str | Path, output_dir: str | Path) -> None:
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    for file_name in (
        "tokenizer.json",
        "tokenizer_config.json",
        "special_tokens_map.json",
        "sentencepiece.bpe.model",
        "sentencepiece.model",
        "label_maps.json",
        "training_config.yaml",
    ):
        source = Path(model_dir) / file_name
        if source.exists():
            shutil.copy2(source, output_path / file_name)


def export_model_to_onnx(
    model: nn.Module,
    output_path: str | Path,
    max_length: int,
    opset_version: int = 17,
) -> None:
    wrapper = OnnxExportWrapper(model.eval())
    dummy_input_ids = torch.ones((1, max_length), dtype=torch.long)
    dummy_attention_mask = torch.ones((1, max_length), dtype=torch.long)
    with patched_transformers_sdpa_mask_for_onnx_export():
        torch.onnx.export(
            wrapper,
            (dummy_input_ids, dummy_attention_mask),
            str(output_path),
            input_names=["input_ids", "attention_mask"],
            output_names=["intent_logits", "ner_logits"],
            dynamic_axes={
                "input_ids": {0: "batch_size", 1: "sequence_length"},
                "attention_mask": {0: "batch_size", 1: "sequence_length"},
                "intent_logits": {0: "batch_size"},
                "ner_logits": {0: "batch_size", 1: "sequence_length"},
            },
            opset_version=opset_version,
            do_constant_folding=True,
            dynamo=False,
        )


@contextmanager
def patched_transformers_sdpa_mask_for_onnx_export():
    from transformers import masking_utils

    original_sdpa_mask = masking_utils.sdpa_mask

    def patched_sdpa_mask(*args, **kwargs):
        q_length = kwargs.get("q_length")
        if isinstance(q_length, torch.Tensor) and q_length.dim() == 0:
            kwargs["q_length"] = torch.arange(q_length, device=q_length.device)
            kwargs.setdefault("q_offset", 0)
        return original_sdpa_mask(*args, **kwargs)

    masking_utils.sdpa_mask = patched_sdpa_mask
    try:
        yield
    finally:
        masking_utils.sdpa_mask = original_sdpa_mask


def validate_onnx_export(
    *,
    model: nn.Module,
    tokenizer: Any,
    examples: list[TrainingExample],
    config: dict[str, Any],
    onnx_path: str | Path,
    max_examples: int = 4,
    atol: float = 1.0e-4,
    rtol: float = 1.0e-4,
) -> dict[str, Any]:
    import onnxruntime as ort

    intent_labels = list(config["intents"]["labels"])
    ner_labels = build_ner_labels(list(config["entities"]["labels"]))
    intent_label2id, _ = build_label_maps(intent_labels)
    ner_label2id, _ = build_label_maps(ner_labels)

    dataset = NluDataset(examples[:max_examples], tokenizer, intent_label2id, ner_label2id, int(config["model"]["max_length"]))
    collator = NluDataCollator(tokenizer)
    session = ort.InferenceSession(str(onnx_path), providers=["CPUExecutionProvider"])
    model.eval()

    validated_rows: list[dict[str, Any]] = []
    with torch.no_grad():
        for row in dataset:
            batch = collator([row])
            batch.pop("tagged_text", None)

            torch_outputs = model(
                input_ids=batch["input_ids"],
                attention_mask=batch["attention_mask"],
            )
            onnx_outputs = session.run(
                ["intent_logits", "ner_logits"],
                {
                    "input_ids": batch["input_ids"].cpu().numpy(),
                    "attention_mask": batch["attention_mask"].cpu().numpy(),
                },
            )

            intent_logits_torch = torch_outputs.logits.cpu().numpy()
            ner_logits_torch = torch_outputs.ner_logits.cpu().numpy()
            intent_logits_onnx, ner_logits_onnx = onnx_outputs

            if not torch.allclose(
                torch.from_numpy(intent_logits_torch),
                torch.from_numpy(intent_logits_onnx),
                atol=atol,
                rtol=rtol,
            ):
                raise ValueError("ONNX intent logits diverge from PyTorch outputs beyond tolerance")
            if not torch.allclose(
                torch.from_numpy(ner_logits_torch),
                torch.from_numpy(ner_logits_onnx),
                atol=atol,
                rtol=rtol,
            ):
                raise ValueError("ONNX NER logits diverge from PyTorch outputs beyond tolerance")

            validated_rows.append(
                {
                    "tagged_text": row["tagged_text"],
                    "intent_argmax_match": int(intent_logits_torch.argmax(axis=-1)[0]) == int(intent_logits_onnx.argmax(axis=-1)[0]),
                    "ner_argmax_match": (
                        torch.from_numpy(ner_logits_torch).argmax(dim=-1).cpu().tolist()
                        == torch.from_numpy(ner_logits_onnx).argmax(dim=-1).cpu().tolist()
                    ),
                }
            )

    return {
        "validated_examples": len(validated_rows),
        "atol": atol,
        "rtol": rtol,
        "rows": validated_rows,
    }
