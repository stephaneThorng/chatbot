# Training Guide

This guide is the operational reference for retraining the restaurant models locally.

It covers:
- environment setup
- dataset files
- intent training
- NER training
- evaluation
- metric reading
- cleanup after training

## 1. Python Environment

Use the dedicated Python 3.11 virtual environment.

```powershell
uv venv .venv311 --python 3.11
uv pip install --python .\.venv311\Scripts\python.exe -r requirements.txt
uv pip install --python .\.venv311\Scripts\python.exe pyarrow==14.0.2 accelerate==0.24.0
```

If `uv` is not available, create a standard venv and install with `pip`, but keep Python `3.11`.

## 2. Dataset Files

The restaurant dataset lives in `C:\Users\Stephyu\git\chatbot\nlp-api\training\data\restaurant:1`.

- `restaurant_corpus.jsonl`: canonical reviewed corpus
- `restaurant_train.jsonl`: training split
- `restaurant_validation.jsonl`: validation split
- `restaurant_eval.jsonl`: held-out evaluation split

Current dataset sizes:
- corpus: `700`
- train: `560`
- validation: `70`
- eval: `70`

The dataset generator is:

- `C:\Users\Stephyu\git\chatbot\nlp-api\scripts\generate_restaurant_dataset.py:1`

To regenerate the dataset:

```powershell
.\.venv311\Scripts\python.exe scripts\generate_restaurant_dataset.py
```

To validate the dataset:

```powershell
.\.venv311\Scripts\python.exe -m pytest tests\test_restaurant_dataset.py
```

## 3. Current Recommended Backbone

Current lightweight backbone:

- `microsoft/MiniLM-L12-H384-uncased`

It is the current best compromise for this project:
- much lighter than `xlm-roberta-base`
- better than the original DistilBERT baseline on the restaurant intent task
- final intent + NER artifacts fit within the project memory target

## 4. Intent Training

Train the intent model with MiniLM:

```powershell
.\.venv311\Scripts\python.exe -m training.train_intent_classifier `
  --train training/data/restaurant/restaurant_train.jsonl `
  --validation training/data/restaurant/restaurant_validation.jsonl `
  --output artifacts/restaurant_intent `
  --model-name microsoft/MiniLM-L12-H384-uncased
```

Notes:
- `--model-name` overrides the default backbone from `training/config.yaml`
- `--output` is the final model directory used later by the API

## 5. NER Training

Train the NER model with MiniLM:

```powershell
.\.venv311\Scripts\python.exe -m training.train_ner_model `
  --train training/data/restaurant/restaurant_train.jsonl `
  --validation training/data/restaurant/restaurant_validation.jsonl `
  --output artifacts/restaurant_ner `
  --model-name microsoft/MiniLM-L12-H384-uncased
```

## 6. Evaluation

Run full evaluation on the held-out split:

```powershell
.\.venv311\Scripts\python.exe -m training.evaluate `
  --intent-model artifacts/restaurant_intent `
  --ner-model artifacts/restaurant_ner `
  --dataset training/data/restaurant/restaurant_eval.jsonl `
  --output artifacts/minilm_comparison_eval.json
```

This produces a JSON file with:
- intent metrics
- NER seqeval report

## 7. How To Read The Metrics

### Intent

The key value is:
- `accuracy`

Example:
- `0.9667` means 96.67% of eval utterances got the correct intent

Also inspect per-intent:
- `precision`
- `recall`
- `f1-score`

Practical reading:
- low `precision` = this intent is predicted too often by mistake
- low `recall` = this intent is missed too often
- low `f1-score` = overall weakness for that intent

For this project, the main intent to watch is:
- `reservation_create`

because it is the class most likely to be confused with:
- `reservation_modify`

### NER

The important values are:
- `micro avg`
- `macro avg`
- per-entity scores

Practical reading:
- `micro avg` reflects overall extraction quality
- `macro avg` reveals whether one entity type is weak even if the total score still looks good

For this project, the most useful business entities to inspect manually are:
- `PEOPLE_COUNT`
- `DATE`
- `TIME`
- `MENU_ITEM`
- `PRICE_ITEM`
- `LOCATION`

If `MENU_ITEM`, `PRICE_ITEM`, or `LOCATION` are weaker than the slot entities, that usually means the runtime heuristics still need improvement even if the trained model is acceptable.

## 8. Real API Testing

Run the API locally:

```powershell
.\.venv311\Scripts\python.exe -m uvicorn src.main:app --host 127.0.0.1 --port 8000
```

Then test:

```powershell
curl -X POST http://127.0.0.1:8000/analyze `
  -H "Content-Type: application/json" `
  -d "{\"text\":\"Book a table for 4 people tomorrow at 7pm under Alex Carter\",\"domain\":\"restaurant\"}"
```

### Test model only

To bypass regex and test the true intent model:

```powershell
$env:USE_HYBRID_INTENT="false"
.\.venv311\Scripts\python.exe -m uvicorn src.main:app --host 127.0.0.1 --port 8000
```

This is useful when you want to know whether improvements come from:
- the transformer
- or the regex fast-path

## 9. Current Recommended Runtime Artifacts

Current default runtime artifacts:

- `C:\Users\Stephyu\git\chatbot\nlp-api\artifacts\restaurant_intent:1`
- `C:\Users\Stephyu\git\chatbot\nlp-api\artifacts\restaurant_ner:1`

Current final artifact sizes:
- intent: about `128 MB`
- NER: about `128 MB`

Measured runtime memory with both models loaded:
- about `508 MB` RSS in the local test process

## 10. Cleanup After Training

Training creates `checkpoint-*` folders that are not needed for inference.

Keep only the final artifact files:
- `config.json`
- `model.safetensors`
- tokenizer files

Delete checkpoint folders if you do not need resume capability.

You can inspect artifact sizes with:

```powershell
$paths = @(
  'artifacts/restaurant_intent',
  'artifacts/restaurant_ner'
)
foreach ($p in $paths) {
  $size = (Get-ChildItem $p -Recurse -File | Measure-Object -Property Length -Sum).Sum
  [PSCustomObject]@{Path=$p; SizeMB=[math]::Round($size/1MB,2)}
}
```

## 11. Default Validation Commands

After training or code changes, run:

```powershell
.\.venv311\Scripts\python.exe -m pytest tests
```

Optional dataset-only validation:

```powershell
.\.venv311\Scripts\python.exe -m pytest tests\test_restaurant_dataset.py
```

## 12. When To Retrain

Retrain if:
- you changed the dataset
- you changed the intent taxonomy
- you changed the entity taxonomy
- you changed the backbone

Do not retrain just because you changed:
- logging
- API routes
- docs
- heuristics only

## 13. Useful Rule Of Thumb

For this repo:
- if intent quality drops, check `reservation_create` vs `reservation_modify` first
- if NER looks fine in eval but weak in the API, inspect runtime heuristics in `C:\Users\Stephyu\git\chatbot\nlp-api\src\models\ner_extractor.py:1`
