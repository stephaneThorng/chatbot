# NLP API Architecture

## Overview

The service is a stateless NLP engine intended to sit behind a chatbot backend. It receives a single utterance plus tenant domain metadata, resolves ranked business intents, utterance signals, and canonical entities, then returns the full analysis in one response. Conversation state and reply generation are deliberately excluded from this service.

## Runtime Flow

1. FastAPI starts through `src.main`.
2. `NLPService` initializes `IntentClassifier` and `NERExtractor`.
3. `ModelManager` downloads tokenizer and model artifacts from Hugging Face Hub using the configured revision and cache directory.
4. Each `/analyze` request normalizes text for model input and runs intent classification plus NER extraction concurrently in worker threads.
5. `IntentRanker` normalizes primary and alternative intent evidence into sorted candidates.
6. `EntityNormalizer` resolves raw entity spans into canonical values using `TemporalResolver` and deterministic value normalizers.
7. `UtteranceAnalyzer` classifies the utterance shape, such as business query, small talk, vague follow-up, frustration, out-of-domain, ambiguous, or unknown.
8. Metrics and structured logs are recorded for each request.

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
- `EntityNormalizer` canonicalizes extracted values after context-aware span mapping

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

### `src/services/intent_ranker.py`

- Converts classifier output into a sorted `intents` candidate list
- Keeps the primary candidate first and preserves source metadata
- Makes top-k routing evidence explicit for downstream backends

### `src/models/ner_extractor.py`

- Uses `AutoModelForTokenClassification` and BIO decoding when available
- Falls back to heuristic extraction for dates, times, counts, phone, and email
- Supplements model output with context-derived entities for short follow-ups when the caller indicates missing slots
- Emits span-level confidence and source metadata per entity

### `src/services/entity_normalizer.py`

- Converts extracted raw spans into canonical entity values
- Normalizes supported dates, times, and people counts
- Preserves `raw_value`, `resolution`, `normalization_status`, and non-fatal warnings

### `src/services/temporal_resolver.py`

- Resolves relative dates and time expressions against the service clock
- Uses `SERVICE_TIMEZONE`, defaulting to `Europe/Paris`
- Returns deterministic metadata such as `relative_date`, `weekday_date`, `month_day`, `time_12h`, and `time_24h`

### `src/services/utterance_analyzer.py`

- Classifies the utterance type independently of the business intent
- Emits signals such as `business_query`, `small_talk`, `vague_follow_up`, `clarification_request`, `frustration`, `out_of_domain`, `ambiguous`, and `unknown`
- Lets the backend avoid treating weak or non-business utterances as workflow commands

### `src/services/nlp_service.py`

- Orchestrates startup, health reporting, request analysis, and metric aggregation
- Converts classifier, ranker, NER, normalizer, and utterance outputs into API schemas
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
