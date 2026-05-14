"""Export a trained multi-task NLU model to ONNX for Rust inference."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from transformers import AutoConfig, AutoTokenizer

from nlu_training.config import load_config
from nlu_training.model import MultiTaskNluModel
from nlu_training.onnx_export import (
    build_export_contract,
    copy_runtime_artifacts,
    export_model_to_onnx,
    validate_onnx_export,
    write_export_contract,
)
from nlu_training.schema import load_jsonl, validate_examples


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="config.yaml")
    parser.add_argument("--model-dir", default="outputs/restaurant_xlmr")
    parser.add_argument("--output-dir")
    parser.add_argument("--validation-dataset")
    parser.add_argument("--max-validation-examples", type=int, default=4)
    parser.add_argument("--skip-validation", action="store_true")
    parser.add_argument("--opset", type=int, default=17)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    config = load_config(args.config)
    model_dir = Path(args.model_dir)
    output_dir = Path(args.output_dir or model_dir / "onnx")
    output_dir.mkdir(parents=True, exist_ok=True)

    tokenizer = AutoTokenizer.from_pretrained(model_dir)
    model_config = AutoConfig.from_pretrained(model_dir)
    setattr(model_config, "_attn_implementation", "eager")
    setattr(model_config, "_attn_implementation_internal", "eager")
    setattr(model_config, "attn_implementation", "eager")
    model = MultiTaskNluModel.from_pretrained(
        model_dir,
        config=model_config,
        attn_implementation="eager",
    )
    contract = build_export_contract(config)

    export_model_to_onnx(
        model=model,
        output_path=output_dir / "model.onnx",
        max_length=contract.max_length,
        opset_version=args.opset,
    )
    copy_runtime_artifacts(model_dir, output_dir)
    write_export_contract(output_dir, contract)

    summary = {
        "onnx_path": str(output_dir / "model.onnx"),
        "contract_path": str(output_dir / "onnx_contract.json"),
        "validation": None,
    }
    if not args.skip_validation:
        dataset_path = args.validation_dataset or config["data"]["validation"]
        examples = load_jsonl(dataset_path)
        validate_examples(examples, config)
        validation_summary = validate_onnx_export(
            model=model,
            tokenizer=tokenizer,
            examples=examples,
            config=config,
            onnx_path=output_dir / "model.onnx",
            max_examples=args.max_validation_examples,
        )
        (output_dir / "onnx_validation.json").write_text(
            json.dumps(validation_summary, indent=2, ensure_ascii=False, sort_keys=True),
            encoding="utf-8",
        )
        summary["validation"] = validation_summary

    print(json.dumps(summary, indent=2, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
