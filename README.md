# Restaurant Chatbot

This repository contains the Rust backend chatbot and the Python model training project used to produce ONNX artifacts for local NLU inference.

- `corebot-backend`: Rust/Axum backend that owns HTTP handling, session state, conversation flow, restaurant data, and deterministic replies.
- `model_training`: Python training and ONNX export project for the local NLU model.

The current runtime architecture is local and in-memory for v1. The backend exposes a single HTTP endpoint for chat messages and consumes ONNX artifacts exported by `model_training`.

## Backend Scope

The Rust backend owns:

- conversation state and session lifecycle
- workflow slot filling
- deterministic reply generation
- restaurant domain data for v1
- local NLU orchestration through `nlu_engine`

The NLU model runtime returns:

- a primary intent
- ranked intent candidates
- extracted entities
- token-level NER labels for debugging

Simple conversation acts such as `greeting`, `thanks`, and `farewell` are handled in the backend conversation logic.

The backend uses feature-based hexagonal architecture under `corebot-backend/src/core/`:

- `domain` for pure domain models and invariants
- `application` for use cases, application services, and ports
- `adapter` for HTTP, in-memory persistence, gateways, and ONNX runtime integration

## Chat API

`POST /api/v1/conversation/send_message`

Request:

```json
{
  "message": "I want to book a table",
  "session_id": null
}
```

Response:

```json
{
  "session_id": "generated-session-id",
  "reply": "Not implemented yet",
  "detected_intent": "opening_hours"
}
```

When `session_id` is omitted, the backend creates a new session. Sessions use a sliding in-memory lifecycle for v1.

## Running Locally

Export the ONNX artifact directory first:

```powershell
$env:COREBOT_NLU_ONNX_DIR = "C:\path\to\model_training\outputs\restaurant_xlmr\onnx"
```

Start the Rust backend:

```powershell
cd corebot-backend
cargo run
```

The backend defaults to:

- Backend URL: `http://localhost:3000`
- Chat endpoint: `http://localhost:3000/api/v1/conversation/send_message`

## Testing

Run backend tests:

```powershell
cd corebot-backend
cargo test
```

Run model training tests:

```powershell
cd model_training
python -m pytest tests
```
