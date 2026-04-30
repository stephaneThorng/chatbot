# NLP API Architecture

## Overview

The service is a stateless NLP component intended to sit behind a chatbot backend. It receives a single utterance plus tenant domain metadata, resolves intent and entities, and returns the full result in one response. Conversation state is deliberately excluded from this service.

## Runtime Flow

1. FastAPI starts through `src.main`.
2. `NLPService` initializes `IntentClassifier` and `NERExtractor`.
3. `ModelManager` downloads tokenizer and model artifacts from Hugging Face Hub using the configured revision and cache directory.
4. Each `/analyze` request runs intent classification and NER extraction concurrently in worker threads.
5. Metrics and structured logs are recorded for each request.

## Context Handling

The service is stateless, but it is context-aware. The caller can send a typed `context` object to help resolve short follow-ups.

Current context contract:

- `previous_intent`: intent from the previous turn
- `current_intent`: active intent already resolved upstream
- `previous_slots`: slots already known
- `slots_filled`: alternative slot bag from upstream
- `required_slots`: slots still expected for the current workflow

This is used in two places:

- `IntentClassifier` keeps the previous intent for short slot-only follow-ups
- `NERExtractor` filters or supplements extraction toward missing slots only

Typical examples:

- `For 5 people` with `previous_intent=reservation_create` and missing `people` keeps `reservation_create`
- `Tomorrow at 9pm` with `previous_intent=reservation_modify` and missing `date` / `time` keeps `reservation_modify`
- `events@example.com` does not define an intent alone; with `required_slots=["email"]`, it fills the email slot for the active workflow

## Components

### `src/config.py`

- Centralized settings via `pydantic-settings`
- Encapsulates environment-based service, logging, and model configuration
- Provides built-in domain regex patterns for the hybrid path

### `src/models/intent_classifier.py`

- Attempts domain-specific regex classification first when enabled
- Falls back to `AutoModelForSequenceClassification`
- Resolves short follow-ups from context before regex or model fallback
- Returns primary intent, alternatives, source, and timing details

### `src/models/ner_extractor.py`

- Uses `AutoModelForTokenClassification` and BIO decoding when available
- Falls back to heuristic extraction for dates, times, counts, phone, and email
- Supplements model output with context-derived entities for short follow-ups when the caller indicates missing slots
- Emits span-level confidence and source metadata per entity

### `src/services/nlp_service.py`

- Orchestrates startup, health reporting, request analysis, and metric aggregation
- Converts model-layer results into API schemas
- Keeps the application stateless and async at the boundary

### `training/`

- `data_loader.py` parses JSONL examples
- `train_intent_classifier.py` prepares a classification dataset and fine-tunes a transformer head
- `train_ner_model.py` aligns entity spans to tokens for token classification
- `evaluate.py` computes intent metrics and seqeval NER metrics

## Error Handling

- Validation errors return HTTP 400 via FastAPI request validation
- Missing service initialization returns HTTP 503
- Model download failures are logged with tracebacks and surfaced via `/health`
- The runtime can still serve degraded heuristic analysis if model downloads fail

## Model Versioning

Model revisioning is handled by Hugging Face `revision` support. The same deployment can be pointed at `main`, a release tag, or a branch:

```python
AutoModelForSequenceClassification.from_pretrained(
    "your-org/nlp-intent-classifier",
    revision="v1.1",
)
```

That makes A/B testing a configuration concern rather than a code change.

## Operational Notes

- Logs go to stdout plus a rotating file handler
- JSON file logs can be enabled with `LOG_JSON=true`
- Metrics are kept in-process and exposed to application code for future export
- Docker image uses a slim Python base and runs `uvicorn` on port `8000`
