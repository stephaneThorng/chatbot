# corebot-backend Architecture

## Overview

`corebot-backend` is the Rust backend for the chatbot. It uses Axum for HTTP routing and follows feature-based hexagonal architecture under `src/core/<feature>/`.

The main architectural goal is strict separation of concerns:

- domain code owns business concepts and invariants;
- application code owns use-case orchestration and ports;
- adapters own concrete protocols, runtimes, clients, repositories, and DTOs.

## Target Feature Ownership

The long-term target is to keep two separate product responsibilities:

- `conversation`: owns the chatbot experience, including restaurant-oriented conversational reads and reservation workflows.
- `back_office`: future feature owning the client-facing restaurant configuration UI and CRUD use cases.

There is no standalone `restaurant` feature. Chatbot-facing restaurant behavior belongs in `conversation`; client-facing restaurant setup belongs in the future `back_office` feature.

The important separation is:

- conversational behavior: deterministic replies, intents, workflows, slot filling, restaurant data projections for the bot;
- client configuration behavior: editable restaurant records, IDs, complete forms, validation for business owners.

Both may use the same PostgreSQL schema, but they should not share the same application models by default. Conversation models can be small projections. Back-office models should carry stable IDs and editable fields.

## Target Module Structure

```text
src/
|-- lib.rs
|-- main.rs
`-- core/
    |-- conversation/
    |   |-- domain/
    |   |-- application/
    |   |   |-- intent_handler/
    |   |   |-- service/
    |   |   |   `-- restaurant/
    |   |   `-- port/
    |   |       |-- inbound/
    |   |       `-- outbound/
    |   |           `-- restaurant/
    |   `-- adapter/
    |       |-- inbound/web/
    |       `-- outbound/
    |           `-- postgres_restaurant/
    |               |-- menu/
    |               |-- business_info/
    |               |-- availability/
    |               `-- reservation/
    |-- nlu_engine/
    |   |-- domain/
    |   |-- application/
    |   `-- adapter/outbound/
    `-- back_office/
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

The `back_office` feature does not need to exist immediately. When introduced, its subfolders must make the client-facing responsibility visible: menu editing, opening-hours editing, closures, contacts, payment methods, facilities, facts, tables, and reservation settings.

## Layer Rules

| Layer | May depend on | Must not import |
|-------|---------------|-----------------|
| `domain/` | Rust standard library and pure domain modules | `serde`, `axum`, `ort`, `tokenizers`, adapters, application services |
| `application/` | domain types, application services, inbound/outbound port traits | Axum, HTTP DTOs, adapter modules, concrete repositories/clients/runtimes |
| `application/port/inbound/` | command/result/domain types | adapters, concrete use-case construction |
| `application/port/outbound/` | domain types and boundary data types needed by use cases | concrete adapter implementations |
| `adapter/inbound/` | input ports, commands/results, DTOs, mappers, framework crates | domain policy, persistence/runtime implementation details |
| `adapter/outbound/` | output ports, domain types, concrete infrastructure crates | inbound adapters, use-case orchestration internals |

Dependency direction must point inward:

```text
adapter/inbound
        |
        v
application ports/use cases -> domain
        ^
        |
adapter/outbound
```

Cross-feature dependency must use a port. A feature must not reach into another feature's adapter or concrete use case. Do not recreate a standalone `restaurant` feature or a conversation-to-restaurant gateway layer.

## Method and Object Placement

- Put business state and invariants in `domain/`.
- Put application commands/results in `application/*_command.rs` unless the feature has a justified split.
- Put use-case coordination in `application/*_usecase.rs`.
- Put reusable application transformations or decoders in `application/<service>.rs`.
- Put inbound protocol mapping in `adapter/inbound/web/*_dto.rs` and `*_mapper.rs`.
- Put concrete runtime/client/repository code in `adapter/outbound/`.
- Keep framework, serialization, filesystem, ONNX, tokenizer, and HTTP concerns out of domain objects.
- Keep concrete adapter APIs out of application methods. Application calls traits; adapters implement traits.
- Keep `Arc` out of adapter structs when a generic owned dependency works. Use `Arc` in the composition root only when dependencies must be shared across multiple constructed objects.
- Prefer generic adapter structs such as `Gateway<R>` over `Gateway<Arc<R>>`. The caller may still pass a cloneable concrete service or a boxed trait object when needed.
- In repository modules, keep `mod.rs` as wiring/export only. Put implementation logic in explicit files such as `menu_repository.rs`, `reservation_repository.rs`, or `business_info_repository.rs`.
- Keep SQL row structs close to the repository query that owns them. Name projections precisely, for example `MenuItemSummaryRow` instead of `MenuItemRow` when the struct is not a full table mirror.

When a behavior appears to need multiple layers, split it explicitly:

- adapter: retrieve or produce raw boundary data;
- application: validate, orchestrate, decode, and decide;
- domain: enforce business invariants and represent business state.

## Feature Responsibilities

### Conversation

- Owns session lifecycle, workflow state, slot filling, conversation policy, and deterministic replies.
- Calls NLU through an analyzer port; it must not call ONNX runtime or tokenizer code directly.
- Owns chatbot-facing restaurant capabilities: menu reads, business-info reads, opening-hours reads, reservation creation, lookup, and cancellation.
- May keep restaurant-specific outbound ports under `application/port/outbound/restaurant/` to make the responsibility visible without creating a separate feature boundary.
- Uses read projections optimized for deterministic replies. These models do not need to expose every SQL column or stable IDs unless the chatbot workflow needs them.

### NLU Engine

- Owns local NLP inference behavior: tagged input construction, artifact validation, tokenization boundary, ONNX execution, intent ranking, and BIO entity decoding.
- Application layer owns preprocessing, artifact validation, model-output decoding, and final `NluAnalysis` construction.
- `adapter/outbound/onnx_nlu_runtime.rs` owns only artifact loading, tokenizer integration, ONNX Runtime execution, and returning raw logits plus token metadata through the output port.
- Do not add keyword intent classifiers here or in conversation code.

### Back Office

- Owns client-facing restaurant setup and CRUD behavior.
- Uses models with stable IDs and complete editable fields.
- Has its own inbound HTTP routes, commands, results, use cases, and outbound repository ports.
- May use the same PostgreSQL tables as conversation, but it should not reuse conversation reply projections as form/edit models.

## Restaurant Absorption Migration Plan

The absorption has been completed. Public HTTP behavior and deterministic chatbot replies must stay stable after future changes.

1. Rename intent-facing restaurant ports inside `conversation` to read/write capabilities.

Examples: `RestaurantMenuGatewayPort`, `RestaurantBusinessInfoGatewayPort`, `RestaurantReservationGatewayPort`.

2. Move restaurant query/result types used only by conversation into `conversation/application/port/outbound/restaurant/`.

These stay projection-oriented and do not need to expose all database fields.

3. Move restaurant application policies used only by chatbot workflows into `conversation/application/service/restaurant/`.

Examples: reservation availability policy, reservation response formatting, menu response formatting.

4. Move PostgreSQL adapters used only by the chatbot into `conversation/adapter/outbound/postgres_restaurant/`.

Keep subfolders visible by responsibility: `menu`, `business_info`, `availability`, `reservation`.

5. Delete the conversation-to-restaurant gateway layer once conversation has direct outbound ports implemented by PostgreSQL adapters.

After this step, the flow becomes:

```text
conversation handler
  -> conversation restaurant outbound port
  -> conversation postgres_restaurant adapter
  -> PostgreSQL
```

6. Introduce `back_office` separately for client-facing CRUD.

This feature gets its own models with IDs and complete editable fields. Do not reuse conversation projections for CRUD forms.

## Conversation Do / Don't

- Do keep `HandleConversationUseCase` as a thin orchestration trunk.
- Do keep `ConversationProcessor` as a router that chooses one path for the turn.
- Do let workflow handlers own workflow-turn execution in the application layer.
- Do model state changes with explicit returned values such as `into_started_workflow`, `into_workflow_slot`, and `into_slot`.
- Do keep nested state ownership explicit: `Conversation` owns workflow replacement and `Workflow` owns slot replacement.
- Do keep typed backend concepts in enums rather than ad hoc strings when the labels are Rust-owned.

- Do not let `ConversationProcessor` execute the same workflow rules through multiple paths.
- Do not mutate a borrowed `Conversation` or nested `Workflow` from helper functions that are not the state owner.
- Do not expose deep mutable accessors when a returned updated value is sufficient.
- Do not mix workflow execution, informational intent handling, and i18n rendering responsibilities into one component.
- Do not put dynamic reply behavior into catalogs when a handler or domain object is the behavior owner.

## Naming Conventions

| Concept | Convention | Example |
|---------|------------|---------|
| Use case struct | `{Action}{Feature}UseCase` | `HandleConversationUseCase` |
| Inbound port trait | `{Action}{Feature}Port` | `HandleConversationPort` |
| Outbound port trait | `{Capability}Port` or `{Entity}RepositoryPort` | `NluModelRuntimePort`, `ConversationRepositoryPort` |
| Command | `{Action}{Feature}Command` | `AnalyzeTextCommand` |
| Result | `{Action}{Feature}Result` | `HandleConversationResult` |
| HTTP request DTO | `{Action}{Feature}Request` | `SendMessageRequest` |
| HTTP response DTO | `{Feature}Response` or `{Action}{Feature}Response` | `SendMessageResponse` |
| Mapper object/module | `{Feature}WebMapper` or `<action>_<feature>_mapper.rs` | `send_message_mapper.rs` |
| Route file | `routes.rs` | `adapter/inbound/web/routes.rs` |
| Integration test | `<feature>_routes_integration_test.rs` | `conversation_routes_integration_test.rs` |

## NLU Engine Flow

```text
conversation/adapter/outbound/NluEngineGateway
  -> AnalyzeTextPort inbound port
  -> AnalyzeTextUseCase
       build TaggedInput
       validate artifact contract
       call NluModelRuntimePort outbound port
       decode logits and BIO tags into NluAnalysis
  -> OnnxNluRuntime
       tokenize prepared tagged text
       run ONNX Runtime
       return raw logits, tokens, offsets
```

The ONNX adapter must not build `TaggedInput` and must not decode `NluAnalysis`.

## Testing Strategy

| Type | Location | Scope |
|------|----------|-------|
| Unit | `#[cfg(test)] mod tests` at the bottom of the same file | Domain invariants, use-case orchestration, mapper conversions, adapter delegation |
| Integration | `tests/` at crate root | HTTP request/response behavior through Axum |
| Architecture tests | `tests/architecture*.rs` when introduced | Layer dependency rules and forbidden imports |

Architecture-sensitive changes must test the layer that now owns the behavior. For example, if preprocessing moves from an adapter into a use case, add a use-case test that proves the adapter receives prepared input.

Executable architecture boundary checks live in:

- [tests/architecture_test.rs](/C:/Users/Stephyu/git/chatbot/corebot-backend/tests/architecture_test.rs): stable layer-access rules with `arch_test_core`
- [tests/architecture_source_rules_test.rs](/C:/Users/Stephyu/git/chatbot/corebot-backend/tests/architecture_source_rules_test.rs): source-level checks for forbidden imports across layers and features
