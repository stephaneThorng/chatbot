# corebot-backend — Architecture

## Overview

`corebot-backend` is the Rust migration target of the Kotlin chatbot backend.
It uses **Axum** for HTTP routing and follows **hexagonal (ports and adapters)** architecture.
Features are self-contained under `src/core/<feature>/`.

## Module Structure

```
src/
├── lib.rs                              Public crate root (exposes core modules)
├── main.rs                             Entry point — wires adapters, starts server
└── core/
    └── conversation/                   Feature: conversation handling
        ├── mod.rs
        ├── adapter/
        │   ├── mod.rs
        │   └── input/
        │       ├── mod.rs
        │       └── web/                Inbound HTTP adapter (Axum)
        │           ├── mod.rs
        │           ├── routes.rs             Route definitions + handler functions
        │           ├── send_message_dto.rs        HTTP request/response structs
        │           └── send_message_mapper.rs     From<> impls: DTO ↔ Command/Result
        └── application/
            ├── mod.rs
            ├── conversation_command.rs        Command + Result domain structs
            ├── conversation_usecase.rs        Use case implementation
            └── port/
                ├── mod.rs
                ├── input/
                │   ├── mod.rs
                │   └── conversation_trait.rs  Inbound port traits
                └── output/                    (future: NLP client, session repository)

tests/
└── conversation_routes_integration_test.rs   Integration tests (HTTP level)
```

## Hexagonal Architecture

```
         ┌────────────────────────────────────────┐
         │  Port: HandleConversation (trait)       │  ← application/port/input/
         │  Protocol-agnostic inbound contract     │
         └───────────────────┬────────────────────┘
                             │ implemented by
         ┌───────────────────▼────────────────────┐
         │  HandleConversationUseCase              │  ← application/conversation_usecase.rs
         │  Owns: session_id resolution, reply     │
         └───────────────────┬────────────────────┘
                             │ called via trait by
         ┌───────────────────┼──────────────────────────────┐
         │                   │                              │
    ┌────▼────────┐   ┌──────▼──────┐             ┌────────▼──────┐
    │ HTTP/Axum   │   │ gRPC        │             │  CLI          │
    │ routes.rs   │   │ (future)    │             │  (future)     │
    └─────────────┘   └─────────────┘             └───────────────┘
      adapter/input/web/
```

**Rule:** Adapters depend on the port **trait**, never on the use case struct directly.

## Dependency Rules

| Layer              | May depend on              | Must NOT import                      |
|--------------------|----------------------------|--------------------------------------|
| Domain structs     | nothing                    | framework crates, serde, axum        |
| `application/`     | domain only                | axum, serde, adapter modules         |
| `adapter/input/`   | application port traits    | use case structs directly            |
| `adapter/output/`  | application port traits    | adapter/input modules                |

## Request Flow

```
POST /api/v1/conversation/send_message
        │
        ▼
routes.rs           Deserializes JSON → SendMessageRequest
        │
        ▼
send_message_mapper.rs   From<SendMessageRequest> → HandleConversationCommand
        │
        ▼
HandleConversation trait  handle_message(command)
        │
        ▼
HandleConversationUseCase  Resolves/generates session_id, returns HandleConversationResult
        │
        ▼
send_message_mapper.rs   From<HandleConversationResult> → SendMessageResponse
        │
        ▼
routes.rs           Serializes JSON → HTTP 200
```

## Testing Strategy

| Type        | Location                               | Scope                                        |
|-------------|----------------------------------------|----------------------------------------------|
| Unit        | `#[cfg(test)]` inside each `.rs` file  | Use case logic, mapper `From<>` conversions  |
| Integration | `tests/`                               | Full HTTP request → response via `axum-test` |

### Current test coverage

| Test file                                    | Tests |
|----------------------------------------------|-------|
| `conversation_usecase.rs`                    | 4     |
| `send_message_mapper.rs`                     | 3     |
| `conversation_routes_integration_test.rs`    | 4     |

## Future Adapters

| Module                              | Purpose                               |
|-------------------------------------|---------------------------------------|
| `adapter/output/nlp_client.rs`      | HTTP client to Python NLP API         |
| `adapter/output/session_repository.rs` | In-memory session store            |
| `adapter/input/grpc/`               | gRPC inbound adapter (same port trait)|
| `adapter/input/whatsapp/`           | WhatsApp Business inbound adapter     |

## Dependencies

| Crate       | Role                        |
|-------------|-----------------------------|
| `axum`      | HTTP routing and handlers   |
| `tokio`     | Async runtime               |
| `serde`     | JSON serialization (DTOs)   |
| `uuid`      | Session ID generation       |
| `axum-test` | Integration test HTTP client (dev only) |

