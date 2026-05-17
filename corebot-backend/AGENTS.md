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
- Chatbot-facing restaurant behavior belongs under `conversation`.
- Do not recreate a standalone `restaurant` hexagon or conversation-to-restaurant gateway layer.
- Chatbot-facing restaurant reads and reservation workflows live under `conversation/application/port/outbound/restaurant`, `conversation/application/service/restaurant`, and `conversation/adapter/outbound/postgres_restaurant`.
- Client-facing CRUD must live in a future `back_office` feature, not in `conversation`.
- `back_office` models must expose stable IDs and editable fields. Conversation projections may remain minimal and should not be stretched to serve CRUD forms.

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
- Prefer generic adapter structs over storing `Arc<T>` inside adapters. Use `Gateway<R>` and let the composition root decide whether to clone, box, or wrap dependencies.
- Use `Arc` at composition boundaries only when an already-built dependency must be shared by multiple owners.
- Keep `mod.rs` files small. They should declare modules and re-export public types, not contain repository or use-case logic.
- Put repository logic in explicit files named by responsibility, such as `menu_repository.rs`, `business_info_repository.rs`, `availability_repository.rs`, and `reservation_repository.rs`.
- Keep SQL row structs near the repository that owns the query. Row structs are query projections, not table mirrors; name them precisely when they contain only part of a table.

## Architecture Do / Don't

- Do keep application orchestration thin. Route first, then delegate to one owner of the behavior.
- Do return updated domain values from domain-owned `into_*` methods when state changes.
- Do keep intermediate application objects such as routing inputs or slot-update collections out of `domain/`.
- Do let the owning domain object update its nested state. `Conversation` updates `Workflow`; `Workflow` updates `SlotBag`.
- Do make workflow handlers the single execution path for workflow turns handled in application code.
- Do keep informational handlers stateless with respect to `Conversation`.
- Do keep compile-time typed enums for backend-owned concepts such as intents, slots, and entity types.

- Do not mutate external state deeply from non-owner application helpers.
- Do not expose nested mutable accessors such as `*_mut()` across layers when a domain-owned return-value API can express the change.
- Do not duplicate workflow execution in multiple places. Avoid splitting the same transition logic across processor, use cases, and handlers.
- Do not put gateway calls, i18n rendering, or adapter logic in domain types.
- Do not use raw strings for backend-owned slot or entity checks when a typed enum exists.
- Do not let catalogs become a second execution system. Shared metadata is fine; behavior belongs in handlers or domain objects.

## Naming Conventions

| Concept | Convention | Example |
|---------|------------|---------|
| File: command/result | `<feature>_command.rs` or `<bounded_context>_command.rs` | `conversation_command.rs` |
| File: use case trait | `<feature>_usecase.rs` or `<capability>_usecase.rs` | `conversation_usecase.rs`, `restaurant_menu_usecase.rs` |
| File: application service | `<service_name>.rs` | `conversation_service.rs`, `conversation_restaurant_service.rs` |
| File: port traits | `<capability>_port.rs`, `<entity>_repository_port.rs`, or `<capability>_gateway_port.rs` | `conversation_repository_port.rs`, `restaurant_menu_gateway_port.rs` |
| File: HTTP DTOs | `<action>_<feature>_dto.rs` | `send_message_dto.rs` |
| File: mapper | `<action>_<feature>_mapper.rs` | `send_message_mapper.rs` |
| Struct: use case | `{Action}{Feature}UseCase` | `HandleConversationUseCase` |
| Trait: inbound port | `{Action}{Feature}Port` | `HandleConversationPort` |
| Trait: outbound port | `{Capability}Port`, `{Entity}RepositoryPort`, or `{Capability}GatewayPort` | `NluModelRuntimePort`, `RestaurantReservationGatewayPort` |
| Struct: command | `{Action}{Feature}Command` | `HandleConversationCommand` |
| Struct: result | `{Action}{Feature}Result` | `HandleConversationResult` |
| Struct: HTTP request | `{Action}{Feature}Request` | `SendMessageRequest` |
| Struct: HTTP response | `{Action}{Feature}Response` | `SendMessageResponse` |
| PostgreSQL repository file | `<responsibility>_repository.rs` | `menu_repository.rs`, `reservation_repository.rs` |
| PostgreSQL row projection file | `models.rs` inside the responsibility folder | `postgres_restaurant/menu/models.rs` |
| Integration test file | `<feature>_routes_integration_test.rs` | `conversation_routes_integration_test.rs` |

Additional naming rules:

- For chatbot restaurant behavior, prefer the `restaurant_` prefix only for conversation-owned restaurant capabilities, not for a standalone `restaurant` hexagon.
- Use `gateway_port` when `conversation` depends on a conversational capability and the boundary is expressed from the conversation side.
- Use `repository_port` when the boundary represents persistence semantics rather than conversational capability.
- Use `*_repository.rs` for concrete PostgreSQL implementations. Do not hide repository logic in `mod.rs`.
- Keep `mod.rs` for module declarations and re-exports only.

## File Structure Rules

Default feature layout:

```text
src/core/<feature>/
|-- domain/
|-- application/
|   |-- <feature>_command.rs
|   |-- <feature>_usecase.rs
|   |-- <application_service>.rs
|   `-- port/
|       |-- inbound/
|       |   `-- <feature>_trait.rs
|       `-- outbound/
|           `-- <dependency>_trait.rs
`-- adapter/
    |-- inbound/
    |   `-- web/
    |       |-- routes.rs
    |       |-- <action>_<feature>_dto.rs
    |       `-- <action>_<feature>_mapper.rs
    `-- outbound/
        `-- <concrete_dependency>.rs
```

Current chatbot restaurant structure under `conversation`:

```text
src/core/conversation/
|-- domain/
|-- application/
|   |-- conversation_processor.rs
|   |-- conversation_service.rs
|   |-- intent_handler/
|   |   |-- handler/
|   |   `-- restaurant_handler_registry_factory.rs
|   |-- service/
|   |   `-- restaurant/
|   `-- port/
|       |-- inbound/
|       `-- outbound/
|           |-- conversation_repository_port.rs
|           |-- language_detector_port.rs
|           |-- nlp_engine_gateway_port.rs
|           `-- restaurant/
|               |-- business_info_queries.rs
|               |-- menu_queries.rs
|               |-- reservation_queries.rs
|               |-- restaurant_business_info_repository_port.rs
|               |-- restaurant_opening_hours_gateway_port.rs
|               |-- restaurant_menu_gateway_port.rs
|               |-- restaurant_menu_repository_port.rs
|               `-- restaurant_reservation_gateway_port.rs
`-- adapter/
    |-- inbound/web/
    `-- outbound/
        |-- nlu_engine_gateway.rs
        `-- postgres_restaurant/
            |-- business_info/
            |   |-- business_info_repository.rs
            |   |-- models.rs
            |   `-- query_helpers.rs
            |-- menu/
            |   |-- menu_repository.rs
            |   |-- models.rs
            |   `-- query_helpers.rs
            |-- availability/
            |   |-- availability_repository.rs
            |   |-- models.rs
            |   `-- query_helpers.rs
            `-- reservation/
                |-- reservation_repository.rs
                |-- models.rs
                `-- query_helpers.rs
```

Rules for this area:

- New chatbot-facing restaurant behavior goes under `conversation`, not under a new autonomous `restaurant` feature.
- Keep separation visible by responsibility through subfolders such as `restaurant/`, `menu/`, `business_info/`, `availability/`, and `reservation/`.
- Prefer one responsibility per folder when SQLx logic, row models, and helpers belong together.
- Keep conversation adapters thin when they only translate one conversation-facing capability to one lower-level dependency.

Target client configuration structure:

```text
src/core/back_office/
|-- domain/
|   |-- menu/
|   |-- business_info/
|   |-- opening_hours/
|   `-- reservations/
|-- application/
|   |-- menu/
|   |-- business_info/
|   |-- opening_hours/
|   |-- reservations/
|   `-- port/
|       |-- inbound/
|       `-- outbound/
`-- adapter/
    |-- inbound/web/
    `-- outbound/postgres/
```

Rules for this area:

- `back_office` owns client-facing CRUD and setup screens.
- Use stable IDs and complete editable records here.
- Do not reuse conversation reply projections as CRUD view models.
- Organize back-office code by editable business responsibility rather than by technical layer alone.

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
- Restaurant data used by chatbot replies is read through conversation-owned restaurant ports.
- Restaurant configuration for clients belongs in `back_office`, even when it writes to the same PostgreSQL tables used by conversation.
