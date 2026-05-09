"""Configuration loading for NLU training."""

from __future__ import annotations

from ast import literal_eval
from pathlib import Path
from typing import Any

try:
    import yaml
except ModuleNotFoundError:  # pragma: no cover - fallback for minimal local runtimes
    yaml = None


def _parse_scalar(value: str) -> Any:
    lowered = value.lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    if lowered in {"null", "none"}:
        return None
    try:
        return literal_eval(value)
    except (SyntaxError, ValueError):
        return value


def _parse_simple_yaml(text: str) -> Any:
    lines = [
        (len(raw) - len(raw.lstrip(" ")), raw.strip())
        for raw in text.splitlines()
        if raw.strip() and not raw.lstrip().startswith("#")
    ]

    def parse_block(index: int, indent: int) -> tuple[Any, int]:
        if index >= len(lines):
            return {}, index

        current_indent, current_text = lines[index]
        if current_indent != indent:
            raise ValueError(f"Unexpected indentation at line {index + 1}")

        if current_text.startswith("- "):
            items: list[Any] = []
            while index < len(lines):
                line_indent, line_text = lines[index]
                if line_indent < indent:
                    break
                if line_indent != indent or not line_text.startswith("- "):
                    raise ValueError(f"Invalid list entry at line {index + 1}")
                item_value = line_text[2:].strip()
                index += 1
                if item_value:
                    items.append(_parse_scalar(item_value))
                else:
                    nested, index = parse_block(index, indent + 2)
                    items.append(nested)
            return items, index

        mapping: dict[str, Any] = {}
        while index < len(lines):
            line_indent, line_text = lines[index]
            if line_indent < indent:
                break
            if line_indent != indent or line_text.startswith("- "):
                raise ValueError(f"Invalid mapping entry at line {index + 1}")
            key, _, raw_value = line_text.partition(":")
            if not _:
                raise ValueError(f"Missing ':' in line {index + 1}")
            key = key.strip()
            value = raw_value.strip()
            index += 1
            if value:
                mapping[key] = _parse_scalar(value)
            else:
                nested, index = parse_block(index, indent + 2)
                mapping[key] = nested
        return mapping, index

    parsed, next_index = parse_block(0, 0)
    if next_index != len(lines):
        raise ValueError("Could not parse entire YAML document")
    return parsed


def load_config(path: str | Path = "config.yaml") -> dict[str, Any]:
    config_path = Path(path)
    with config_path.open("r", encoding="utf-8") as handle:
        if yaml is not None:
            return yaml.safe_load(handle)
        return _parse_simple_yaml(handle.read())


def build_label_maps(labels: list[str]) -> tuple[dict[str, int], dict[int, str]]:
    label2id = {label: index for index, label in enumerate(labels)}
    id2label = {index: label for label, index in label2id.items()}
    return label2id, id2label


def build_ner_labels(entity_labels: list[str]) -> list[str]:
    return ["O"] + [f"{prefix}-{label}" for label in entity_labels for prefix in ("B", "I")]
