# NLP API

`nlp-api` is a production-oriented FastAPI language analysis engine for chatbot backends. It performs ranked intent classification, utterance signal detection, entity extraction, and entity normalization in a single `/analyze` call. It supports model versioning through Hugging Face Hub and includes a full training pipeline for intent and NER models.

## Features

- Hybrid intent classification with regex fast path, transformer fallback, and ranked candidates
- Utterance analysis for small talk, vague follow-ups, clarification requests, frustration, out-of-domain, ambiguous, and unknown messages
- Token-classification NER with BIO decoding and heuristic fallback
- Canonical entity normalization with raw spans, normalized values, resolution metadata, and warnings
- Rich responses with confidences, ranked intents, source metadata, and timing breakdowns
- Async FastAPI startup and request orchestration
- Hugging Face Hub model revision support for A/B testing
- Training scripts for intent, NER, and evaluation
- Docker assets, health checks, benchmark script, and pytest coverage

## Quick Start

```bash
py -3.11 -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
python -m spacy download en_core_web_sm
copy .env.example .env
python -m src.main
```

One-line alternatives:

```bash
python -m src.main
```

After `pip install -e .`, you can also use:

```bash
nlp-api
```

For local hot reload:

```bash
set UVICORN_RELOAD=true && python -m src.main
```

## Configuration

The service is configured through environment variables or `.env`.

```env
HF_MODEL_INTENT=artifacts/restaurant_intent
HF_MODEL_NER=artifacts/restaurant_ner
HF_MODEL_REVISION=main
HF_TOKEN=
HF_CACHE_DIR=./.cache/huggingface
SERVICE_PORT=8000
SERVICE_HOST=0.0.0.0
SERVICE_TIMEZONE=Europe/Paris
LOG_LEVEL=INFO
LOG_FILE=./logs/nlp-api.log
INTENT_CONFIDENCE_THRESHOLD=0.6
USE_HYBRID_INTENT=true
NER_CONFIDENCE_THRESHOLD=0.5
DEVICE=cpu
```

## API

### `POST /analyze`

```json
{
  "text": "Book a table for 4 people tomorrow at 7pm",
  "domain": "restaurant",
  "context": {
    "previous_slots": {
      "name": "Alex Carter"
    }
  }
}
```

### Context-aware follow-ups

The `context` field is important for short follow-ups. Without it, values like `For 5 people`, `Tomorrow at 9pm`, or `events@example.com` are ambiguous. With it, the API keeps the active business intent and extracts only the missing slots.

Supported context fields:

- `previous_intent`
- `current_intent`
- `previous_slots`
- `slots_filled`
- `required_slots`

Example multi-turn reservation workflow:

```json
{
  "text": "For 5 people",
  "domain": "restaurant",
  "context": {
    "previous_intent": "reservation_create",
    "previous_slots": {
      "date": "tomorrow",
      "time": "7pm"
    },
    "required_slots": ["people", "date", "time", "name"]
  }
}
```

Expected behavior:

- intent stays `reservation_create`
- source becomes `context`
- extracted entity is `PEOPLE_COUNT`
- `raw_value` preserves the user text
- `value` contains the canonical slot value when normalization succeeds

Another example:

```json
{
  "text": "events@example.com",
  "domain": "restaurant",
  "context": {
    "previous_intent": "contact_request",
    "required_slots": ["email"]
  }
}
```

Important: `events@example.com` does not imply `contact_request` by itself. The active intent comes from the previous turn. The text only provides the missing slot value.

```json
{
  "intent": {
    "name": "reservation_create",
    "confidence": 0.93,
    "fast_path": true,
    "source": "regex",
    "alternatives": {
      "reservation_modify": 0.12,
      "reservation_cancel": 0.08
    }
  },
  "intents": [
    {
      "name": "reservation_create",
      "confidence": 0.93,
      "source": "regex",
      "reason": "primary"
    },
    {
      "name": "reservation_modify",
      "confidence": 0.12,
      "source": "alternative",
      "reason": null
    }
  ],
  "utterance": {
    "kind": "business_query",
    "confidence": 0.92,
    "source": "rule"
  },
  "entities": [
    {
      "type": "PEOPLE_COUNT",
      "raw_value": "4 people",
      "value": "4",
      "start": 17,
      "end": 25,
      "confidence": 0.9,
      "source": "heuristic",
      "resolution": "count",
      "normalization_status": "normalized"
    },
    {
      "type": "DATE",
      "raw_value": "tomorrow",
      "value": "2026-05-04",
      "start": 26,
      "end": 34,
      "confidence": 0.9,
      "source": "heuristic",
      "resolution": "relative_date",
      "normalization_status": "normalized"
    },
    {
      "type": "TIME",
      "raw_value": "7pm",
      "value": "19:00",
      "start": 38,
      "end": 41,
      "confidence": 0.95,
      "source": "heuristic",
      "resolution": "time_12h",
      "normalization_status": "normalized"
    }
  ],
  "warnings": [],
  "processing_time_ms": 7.4,
  "processing_details": {
    "intent_ms": 2.1,
    "ner_ms": 1.8,
    "total_ms": 7.4
  },
  "model_info": {
    "intent_model": "artifacts/restaurant_intent",
    "ner_model": "artifacts/restaurant_ner",
    "revision": "main"
  }
}
```

### `GET /health`

Returns startup status, device, cache path, and whether the intent and NER transformer models were loaded successfully.

## Training

Training assets live in `training/`.

```bash
python -m training.train_intent_classifier --train data/intent_train.jsonl --validation data/intent_valid.jsonl
python -m training.train_ner_model --train data/ner_train.jsonl --validation data/ner_valid.jsonl
python -m training.evaluate --intent-model artifacts/intent --ner-model artifacts/ner --dataset data/eval.jsonl
```

See `training/README.md` for dataset shape and Hub upload flow.

The restaurant dataset currently annotates these entity types:
`DATE`, `TIME`, `PEOPLE_COUNT`, `PERSON`, `PHONE`, `EMAIL`, `MENU_ITEM`, `PRICE_ITEM`, and `LOCATION`.

The API normalizes supported `DATE`, `TIME`, and `PEOPLE_COUNT` entities after span extraction. Relative dates use the service clock and `SERVICE_TIMEZONE`.

## Testing

```bash
pytest tests
```

## Docker

```bash
docker compose -f docker/docker-compose.yml up --build
```

## Layout

- `src/`: API, service layer, model wrappers, config, logging, metrics
- `training/`: JSONL data loading, training, and evaluation scripts
- `tests/`: endpoint, model, classifier, and NER tests
- `scripts/`: model download, benchmark, and health-check utilities
- `docker/`: production image and local compose setup
