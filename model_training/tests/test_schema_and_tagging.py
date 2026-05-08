from __future__ import annotations

import pytest

from nlu_training.config import build_label_maps, build_ner_labels, load_config
from nlu_training.schema import DatasetValidationError, EntitySpan, TrainingExample, validate_examples
from nlu_training.tagging import align_bio_labels, build_tagged_text


def test_validate_rejects_provide_info_without_workflow_task() -> None:
    config = load_config("config.yaml")
    example = TrainingExample(
        text="Jean",
        lang="en",
        domain="restaurant",
        intent="provide_info",
        entities=(EntitySpan(0, 4, "person"),),
    )

    with pytest.raises(DatasetValidationError):
        validate_examples([example], config)


def test_tagged_text_omits_task_outside_workflow() -> None:
    example = TrainingExample(
        text="Hello",
        lang="en",
        domain="restaurant",
        intent="greeting",
        entities=(),
    )

    tagged = build_tagged_text(example)

    assert tagged.text == "[LANG=en] [DOMAIN=restaurant] Hello"
    assert "[TASK=" not in tagged.text


def test_tagged_text_shifts_entity_offsets_after_workflow_tags() -> None:
    example = TrainingExample(
        text="Jean",
        lang="en",
        domain="restaurant",
        intent="provide_info",
        entities=(EntitySpan(0, 4, "person"),),
        task="WF_BOOK",
    )

    tagged = build_tagged_text(example)

    assert tagged.text == "[TASK=WF_BOOK] [LANG=en] [DOMAIN=restaurant] Jean"
    assert tagged.entity_spans[0].start == tagged.text.index("Jean")
    assert tagged.entity_spans[0].end == tagged.text.index("Jean") + 4


def test_bio_alignment_keeps_custom_tags_outside_entities() -> None:
    config = load_config("config.yaml")
    ner_labels = build_ner_labels(config["entities"]["labels"])
    ner_label2id, ner_id2label = build_label_maps(ner_labels)
    example = TrainingExample(
        text="Jean",
        lang="en",
        domain="restaurant",
        intent="provide_info",
        entities=(EntitySpan(0, 4, "person"),),
        task="WF_BOOK",
    )
    tagged = build_tagged_text(example)
    jean_start = tagged.text.index("Jean")
    offsets = [
        (0, 0),
        (0, 5),
        (5, 13),
        (14, 23),
        (24, 43),
        (jean_start, jean_start + 2),
        (jean_start + 2, jean_start + 4),
        (0, 0),
    ]

    labels = [ner_id2label[label_id] if label_id != -100 else "IGN" for label_id in align_bio_labels(tagged, offsets, ner_label2id)]

    assert labels == ["IGN", "O", "O", "O", "O", "B-person", "I-person", "IGN"]
