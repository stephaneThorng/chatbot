# NLP API

`nlp-api` is a production-oriented FastAPI service for multi-tenant chatbot backends. It performs intent classification and named entity recognition in a single `/analyze` call, supports model versioning through Hugging Face Hub, and includes a full training pipeline for intent and NER models.

## Features

- Hybrid intent classification with regex fast path and transformer fallback
- Token-classification NER with BIO decoding and heuristic fallback
- Rich responses with confidences, alternatives, source metadata, and timing breakdowns
- Async FastAPI startup and request orchestration
- Hugging Face Hub model revision support for A/B testing
- Training scripts for intent, NER, and evaluation
- Docker assets, health checks, benchmark script, and pytest coverage

## Quick Start

```bash
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
copy .env.example .env
uvicorn src.main:app --reload --host 0.0.0.0 --port 8000
```

## Configuration

The service is configured through environment variables or `.env`.

```env
HF_MODEL_INTENT=your-org/nlp-intent-classifier
HF_MODEL_NER=your-org/nlp-ner-model
HF_MODEL_REVISION=main
HF_TOKEN=
HF_CACHE_DIR=./models
SERVICE_PORT=8000
SERVICE_HOST=0.0.0.0
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

```json
{
  "intent": {
    "name": "reservation",
    "confidence": 0.93,
    "fast_path": true,
    "source": "regex",
    "alternatives": {
      "horaires": 0.12,
      "menu": 0.08
    }
  },
  "entities": [
    {
      "type": "PEOPLE_COUNT",
      "value": "4 people",
      "start": 17,
      "end": 25,
      "confidence": 0.9,
      "source": "heuristic"
    },
    {
      "type": "DATE",
      "value": "tomorrow",
      "start": 26,
      "end": 34,
      "confidence": 0.9,
      "source": "heuristic"
    },
    {
      "type": "TIME",
      "value": "7pm",
      "start": 38,
      "end": 41,
      "confidence": 0.95,
      "source": "heuristic"
    }
  ],
  "processing_time_ms": 7.4,
  "processing_details": {
    "intent_ms": 2.1,
    "ner_ms": 1.8,
    "total_ms": 7.4
  },
  "model_info": {
    "intent_model": "your-org/nlp-intent-classifier",
    "ner_model": "your-org/nlp-ner-model",
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
