# corebot-backend -- Architecture

## Overview

`corebot-backend` is the Rust migration target of the Kotlin chatbot backend.
It uses **Axum** for HTTP routing and follows **hexagonal (ports and adapters)** architecture.
Features are self-contained under `src/core/<feature>/`.

## Module Structure

```
src/
├── lib.rs                              Public crate root (exposes core modules)
├── main.rs                             Entry point - wires adapters, starts server
└── core/
    ├── conversation/                   Feature: conversation orchestration
    │   ├── adapter/
    │   │   ├── input/
    │   │   │   └── web/                Inbound HTTP adapter (Axum)
    │   │   │       ├── routes.rs             Route definitions + handler functions
    │   │   │       ├── send_message_dto.rs        HTTP request/response structs
    │   │   │       └── send_message_mapper.rs     From<> impls: DTO <-> Command/Result
    │   │   └── output/
    │   │       └── restaurant_domain_gateway.rs   Bridges conversation -> RestaurantPort
    │   └── application/
    │       ├── conversation_command.rs        Command + Result domain structs
    │       ├── conversation_usecase.rs        Use case: session resolution + dispatch
    │       └── port/
    │           ├── input/
    │           │   └── conversation_trait.rs  Inbound port (HandleConversation)
    │           └── output/
    │               └── domain_gateway_trait.rs  Outbound port (DomainGateway)
    │
    └── restaurant/                     Feature: restaurant domain data
        ├── adapter/
        │   └── input/
        │       └── restaurant_adapter.rs   Stub impl of RestaurantPort
        └── application/
            └── port/
                └── input/
                    └── restaurant_trait.rs   RestaurantPort trait

tests/
└── conversation_routes_integration_test.rs   Integration tests (HTTP level)
```

## Hexagonal Architecture

```
HTTP request
     |
     v
[conversation/adapter/input/web/routes.rs]
     |  uses
     v
[HandleConversation trait]              <- conversation/application/port/input/
     |  implemented by
     v
[HandleConversationUseCase]             <- conversation/application/
     |  calls via
     v
[DomainGateway trait]                   <- conversation/application/port/output/
     |  implemented by
     v
[RestaurantDomainGateway]               <- conversation/adapter/output/
     |  calls via
     v
[RestaurantPort trait]                  <- restaurant/application/port/input/
     |  implemented by
     v
[RestaurantAdapter]                     <- restaurant/adapter/input/
     |
     v
  (data source - in-memory stub for v1)
```

**Key rule:** Each layer communicates only via port traits, never via concrete structs across boundaries.

## Adding a New Domain (e.g. Hotel)

1. Create `core/hotel/application/port/input/hotel_trait.rs` with `trait HotelPort`
2. Create `core/hotel/adapter/input/hotel_adapter.rs` implementing `HotelPort`
3. Create `core/conversation/adapter/output/hotel_domain_gateway.rs` implementing `DomainGateway` via `HotelPort`
4. Wire `HotelDomainGateway` in `routes.rs` or via `main.rs` depending on runtime domain

Zero changes needed in `conversation_usecase.rs` or any port trait.

## Adding a New Data Method (e.g. get_menu)

1. Add `fn get_menu(&self) -> Menu` to `DomainGateway` trait
2. Add `fn get_menu(&self) -> Menu` to `RestaurantPort` trait
3. Implement in `RestaurantAdapter` (return stub or real data)
4. Implement in `RestaurantDomainGateway` (delegate to restaurant port)
5. Call `self.domain_gateway.get_menu()` in the use case when intent routing resolves to `menu`

## Dependency Rules

| Layer                          | May depend on              | Must NOT import                      |
|-------------------------------|----------------------------|--------------------------------------|
| Domain structs                 | nothing                    | framework crates, serde, axum        |
| `application/`                 | domain + port traits only  | axum, serde, adapter modules         |
| `adapter/input/`               | application port traits    | use case structs directly            |
| `adapter/output/`              | application port traits    | adapter/input modules                |
| `restaurant/adapter/input/`    | restaurant port trait only | conversation modules                 |

## Request Flow

```
POST /api/v1/conversation/send_message
  -> routes.rs            deserialize JSON -> SendMessageRequest
  -> send_message_mapper  From<SendMessageRequest> -> HandleConversationCommand
  -> HandleConversation   handle_message(command)
  -> HandleConversationUseCase
       resolve session_id
       call domain_gateway.get_opening_hours()
  -> RestaurantDomainGateway
       delegate to restaurant.get_opening_hours()
  -> RestaurantAdapter    return "Not implemented yet"
  <- HandleConversationResult { session_id, reply }
  -> send_message_mapper  From<HandleConversationResult> -> SendMessageResponse
  <- HTTP 200 JSON
```

## Testing Strategy

| Type        | Location                               | Scope                                             |
|-------------|----------------------------------------|---------------------------------------------------|
| Unit        | `#[cfg(test)]` inside each `.rs` file  | Use case logic, mapper conversions, gateway delegation |
| Integration | `tests/`                               | Full HTTP request -> response via `axum-test`     |

### Current test coverage

| Test file                                          | Tests |
|----------------------------------------------------|-------|
| `conversation_usecase.rs`                          | 4     |
| `send_message_mapper.rs`                           | 3     |
| `restaurant_domain_gateway.rs`                     | 1     |
| `restaurant_adapter.rs`                            | 1     |
| `conversation_routes_integration_test.rs`          | 4     |

## Future Work

| Item                                    | Location                                    |
|-----------------------------------------|---------------------------------------------|
| Real restaurant data (menu, hours, ...) | `restaurant/adapter/input/restaurant_adapter.rs` |
| NLP intent routing                      | `conversation/application/conversation_usecase.rs` |
| Session management                      | `conversation/application/` + session port  |
| Hotel domain                            | `core/hotel/`                               |
| NLP client (outbound)                   | `conversation/adapter/output/nlp_client.rs` |

## Dependencies

| Crate       | Role                                          |
|-------------|-----------------------------------------------|
| `axum`      | HTTP routing and handlers                     |
| `tokio`     | Async runtime                                 |
| `serde`     | JSON serialization (DTOs only)                |
| `uuid`      | Session ID generation                         |
| `axum-test` | Integration test HTTP client (dev only)       |

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

