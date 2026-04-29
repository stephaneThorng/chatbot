# NLP API Architecture

## Overview

The service is a stateless NLP component intended to sit behind a chatbot backend. It receives a single utterance plus tenant domain metadata, resolves intent and entities, and returns the full result in one response. Conversation state is deliberately excluded from this service.

## Runtime Flow

1. FastAPI starts through `src.main`.
2. `NLPService` initializes `IntentClassifier` and `NERExtractor`.
3. `ModelManager` downloads tokenizer and model artifacts from Hugging Face Hub using the configured revision and cache directory.
4. Each `/analyze` request runs intent classification and NER extraction concurrently in worker threads.
5. Metrics and structured logs are recorded for each request.

## Components

### `src/config.py`

- Centralized settings via `pydantic-settings`
- Encapsulates environment-based service, logging, and model configuration
- Provides built-in domain regex patterns for the hybrid path

### `src/models/intent_classifier.py`

- Attempts domain-specific regex classification first when enabled
- Falls back to `AutoModelForSequenceClassification`
- Returns primary intent, alternatives, source, and timing details

### `src/models/ner_extractor.py`

- Uses `AutoModelForTokenClassification` and BIO decoding when available
- Falls back to heuristic extraction for dates, times, counts, phone, and email
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
