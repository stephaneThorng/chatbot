# Repository Instructions

## Language and Style

- Code, docs, identifiers, comments, commits, and API names must be written in English.
- Keep implementation focused on the requested feature. Do not refactor unrelated NLP, backend, or training code.
- Prefer deterministic behavior for v1 chatbot responses. Do not introduce LLM-generated replies unless explicitly requested.
- When architecture rules conflict with a local `AGENTS.md`, the more specific file wins.

## Repository Layout

- `corebot-backend/` contains the Rust backend and is the active backend target.
- `model_training/` contains Python model training and ONNX export code.
- `backend/chatbot/` contains the legacy Kotlin backend. Do not extend it unless explicitly requested.
- `scripts/` contains repository automation and support scripts.

## Rust Backend Architecture

- Rust backend code lives under `corebot-backend/src`.
- Use feature-based hexagonal architecture under `corebot-backend/src/core/<feature>/`.
- Keep domain models free of framework annotations, HTTP serialization concerns, external client types, and artifact serialization concerns.
- Put application commands, use cases, application services, and port traits under `application`.
- Put inbound ports under `application/port/inbound`.
- Put outbound ports under `application/port/outbound`.
- Put HTTP routes and DTOs under `adapter/inbound/web`.
- Put persistence, external clients, runtime integrations, and gateways under `adapter/outbound`.
- Adapters may depend on application ports and domain types. They must not call use-case internals or other adapters directly.
- Application code may depend on domain types and port traits. It must not import `axum`, HTTP DTOs, ONNX runtime APIs, tokenizer APIs, filesystem APIs for adapters, or adapter modules.
- Domain code must not import `serde`, `axum`, `ort`, `tokenizers`, repositories, clients, or application services.

## Naming

- Use case structs use `{Action}{Feature}UseCase`.
- Input port traits use `{Action}{Feature}Port`.
- Output port traits use capability-oriented names suffixed with `Port`, such as `{Entity}RepositoryPort`, `{Service}RuntimePort`, or `{ExternalSystem}GatewayPort`.
- Command structs use `{Action}{Feature}Command`.
- Result structs use `{Action}{Feature}Result`.
- Route files use `routes.rs` inside `adapter/inbound/web`.
- HTTP DTOs use `{Action}{Feature}Request` and `{Action}{Feature}Response`.
- Mapper files use `{Action}{Feature}Mapper` naming in type names and `<action>_<feature>_mapper.rs` for files.

## Chatbot Rules

- The Rust backend owns conversation state, session lifecycle, slot filling, and deterministic reply generation.
- The Rust backend owns simple conversation acts such as greeting, thanks, and farewell.
- The NLU engine owns local model inference, preprocessing, tokenization, intent ranking, and BIO entity decoding.
- Do not add intent keyword classifiers to the Rust backend.
- Keep workflow states coarse and model detailed progress with generic requirements.
- Put requirement validation and transformation in value-type/domain classes or dedicated domain/application services, not in route handlers.
- Sessions are in-memory for v1 and must not be treated as durable storage.
- Restaurant business data is a static in-memory dataset for v1.

## Model Training Boundary

- Python training and ONNX export code owns training datasets, model construction, and artifact generation.
- Rust must consume exported artifacts through explicit runtime contracts.
- Keep Rust preprocessing aligned with the Python training format when changing NLU input construction.


