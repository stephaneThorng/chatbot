# Training Guide

## Dataset Format

Each JSONL line should contain:

```json
{
  "text": "Je souhaite reserver pour 4 personnes demain a 19h",
  "intent": "reservation",
  "entities": [
    {"start": 26, "end": 37, "type": "PEOPLE_COUNT"},
    {"start": 38, "end": 44, "type": "DATE"},
    {"start": 47, "end": 50, "type": "TIME"}
  ]
}
```

## Dataset Source Of Truth

The restaurant dataset in `training/data/restaurant/` is intended to be committed to Git.

- `restaurant_corpus.jsonl` is the canonical reviewed corpus.
- `restaurant_train.jsonl`, `restaurant_validation.jsonl`, and `restaurant_eval.jsonl` are the versioned split artifacts used by training and evaluation.
- `scripts/generate_restaurant_dataset.py` is the deterministic generator that can recreate those files.

Commit both the generator and the generated JSONL files so training remains reproducible across local environments and CI.

## Intent Training

```bash
python -m training.train_intent_classifier --train data/intent_train.jsonl --validation data/intent_valid.jsonl --output artifacts/intent
```

## NER Training

```bash
python -m training.train_ner_model --train data/ner_train.jsonl --validation data/ner_valid.jsonl --output artifacts/ner
```

## Upload to Hugging Face Hub

Both training scripts support `--push-to-hub` and `--hub-model-id`.

```bash
python -m training.train_intent_classifier --train data/train.jsonl --validation data/valid.jsonl --push-to-hub --hub-model-id your-org/nlp-intent-classifier
```

## Evaluation

```bash
python -m training.evaluate --intent-model artifacts/intent --ner-model artifacts/ner --dataset data/eval.jsonl
```
