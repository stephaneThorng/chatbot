# Repository Instructions

## Language and Style

- Code, docs, identifiers, comments, commits, and API names must be written in English.
- Keep implementation focused on the requested feature. Do not refactor unrelated NLP or backend code.
- Prefer deterministic behavior for v1 chatbot responses. Do not introduce LLM-generated replies unless explicitly requested.

## Backend Architecture

- Kotlin backend code lives under `backend/chatbot`.
- Use feature-based hexagonal architecture under `core/<feature>`.
- Keep domain models free of framework annotations and HTTP serialization concerns.
- Put application commands and use cases under `application`.
- Put outbound interfaces under `application/port/out`.
- Put Ktor routes and DTOs under `adapter/in/web`.
- Put in-memory persistence and external clients under `adapter/out`.

## Naming

- Use case classes use `{Action}{Feature}UseCase`.
- Repository interfaces use `{Entity}Repository`.
- Ktor route files use `{Feature}Routes`.
- HTTP DTOs use `{Action}{Feature}Request` and `{Feature}Response`.
- Mapper objects use `{Feature}WebMapper` or `{Feature}PersistenceMapper`.

## Chatbot Rules

- The Kotlin backend owns conversation state, session lifecycle, slot filling, and reply generation.
- The Kotlin backend owns simple conversation acts such as greeting, thanks, and farewell.
- The Python NLP API owns intent classification and entity extraction.
- Do not add intent keyword classifiers to the Kotlin backend.
- Keep workflow states coarse and model detailed progress with generic requirements.
- Put requirement validation and transformation in value-type classes, not in route handlers.
- Sessions are in-memory for v1 and must not be treated as durable storage.
- Restaurant business data is a static in-memory dataset for v1.
