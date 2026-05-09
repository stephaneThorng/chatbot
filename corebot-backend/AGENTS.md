# Rust Backend - Agent Instructions

## Language and Style

- All code, comments, identifiers, docs, and commit messages must be in English.
- Use `snake_case` for files, modules, variables, and functions.
- Use `PascalCase` for structs, enums, and traits.
- Use `SCREAMING_SNAKE_CASE` for constants.
- No abbreviations (`uc`, `cmd`, `repo`) unless universally known.
- No `.unwrap()` in production paths. Use typed errors or `unwrap_or_else` with a descriptive message at startup boundaries.
- One concept per file. File name must describe its content without opening it.

## Architecture

- Follow hexagonal architecture under `src/core/<feature>/`.
- `domain/` owns pure business concepts and value objects.
- `application/` owns commands, results, use cases, application services, and port traits. No framework imports allowed.
- `application/port/inbound/` owns inbound use-case traits.
- `application/port/outbound/` owns outbound dependency traits.
- `adapter/inbound/` owns inbound adapters such as HTTP, future gRPC, and CLI.
- `adapter/outbound/` owns outbound adapters such as repositories, gateways, external clients, and model runtimes.
- Domain structs must not derive `Serialize` or `Deserialize`. Only DTOs or artifact/config structs outside `domain/` do.
- Adapters depend on application port traits and domain types, never on use case structs directly.
- Application code may call output ports, but output adapters must not call application use cases or application decoding/orchestration helpers.
- Cross-feature calls must go through the target feature's public port, not through its adapter or concrete use case.

## Layer Responsibilities

| Layer | Owns | Must not own |
|-------|------|--------------|
| `domain/` | Business state, value objects, invariants, deterministic transformations independent of infrastructure | HTTP/JSON DTOs, `serde`, Axum, ONNX/tokenizer APIs, filesystem/runtime clients |
| `application/` | Commands, use cases, orchestration, port traits, application-level validation, mapping between port data and domain results | Web routing, DTO serialization, adapter construction, external client details |
| `application/port/inbound/` | Traits exposed to inbound adapters | Concrete adapter or use-case wiring |
| `application/port/outbound/` | Traits required by use cases to access storage, runtimes, gateways, or external services | Concrete implementation details |
| `adapter/inbound/` | Protocol-specific inbound mapping and handler glue | Business decisions, workflow logic, direct persistence/runtime calls |
| `adapter/outbound/` | Concrete repositories, gateways, external clients, runtime integrations | Application orchestration, domain policy, inbound adapter calls |

## Method Placement Rules

- Put methods that enforce business invariants on domain types.
- Put methods that coordinate multiple domain objects or ports in application use cases/services.
- Put methods that translate HTTP payloads in web DTO/mapper modules.
- Put methods that call external libraries or infrastructure in adapters.
- Put model/runtime decoding that creates domain results in application services unless it is purely infrastructure-specific.
- Put serialization-only structs near the boundary that reads/writes them, not in `domain/`.
- If a method needs both a concrete adapter dependency and domain policy, split it: adapter returns raw boundary data, application turns it into domain behavior.

## Naming Conventions

| Concept | Convention | Example |
|---------|------------|---------|
| File: command/result | `<feature>_command.rs` | `conversation_command.rs` |
| File: use case | `<feature>_usecase.rs` | `conversation_usecase.rs` |
| File: port traits | `<feature>_trait.rs` | `conversation_trait.rs` |
| File: HTTP DTOs | `<action>_<feature>_dto.rs` | `send_message_dto.rs` |
| File: mapper | `<action>_<feature>_mapper.rs` | `send_message_mapper.rs` |
| Struct: use case | `{Action}{Feature}UseCase` | `HandleConversationUseCase` |
| Trait: inbound port | `{Action}{Feature}Port` | `HandleConversationPort` |
| Trait: outbound port | `{Capability}Port` or `{Entity}RepositoryPort` | `NluModelRuntimePort` |
| Struct: command | `{Action}{Feature}Command` | `HandleConversationCommand` |
| Struct: result | `{Action}{Feature}Result` | `HandleConversationResult` |
| Struct: HTTP request | `{Action}{Feature}Request` | `SendMessageRequest` |
| Struct: HTTP response | `{Action}{Feature}Response` | `SendMessageResponse` |
| Integration test file | `<feature>_routes_integration_test.rs` | `conversation_routes_integration_test.rs` |

## File Structure Rules

```text
src/core/<feature>/
├── domain/
├── application/
│   ├── <feature>_command.rs
│   ├── <feature>_usecase.rs
│   ├── <application_service>.rs
│   └── port/
│       ├── inbound/
│       │   └── <feature>_trait.rs
│       └── outbound/
│           └── <dependency>_trait.rs
└── adapter/
    ├── inbound/
    │   └── web/
    │       ├── routes.rs
    │       ├── <action>_<feature>_dto.rs
    │       └── <action>_<feature>_mapper.rs
    └── outbound/
        └── <concrete_dependency>.rs
```

## Testing Rules

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the same file.
- Integration tests live in `tests/` at the crate root.
- Use helper functions such as `make_command` and `make_server` to eliminate test boilerplate.
- Do not test private internals when behavior can be tested through a public interface.
- Every use case must have tests for the main success path and relevant error/edge paths.
- Every `From<>` mapper impl must have a dedicated test.
- Integration tests must cover: 200 happy path, 415 missing Content-Type, 422 missing required fields.
- Architecture-sensitive changes must include at least one test at the layer where behavior is now owned.

## Chatbot Rules

- The Rust backend owns conversation state, session lifecycle, slot filling, and reply generation.
- The NLU engine owns intent classification and entity extraction when the local ONNX runtime is used.
- Do not duplicate model inference with keyword classifiers in conversation code.
- Sessions are in-memory for v1 and must not be treated as durable storage.
