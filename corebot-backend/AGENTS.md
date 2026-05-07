# Rust Backend — Agent Instructions

## Language and Style

- All code, comments, identifiers, docs, and commit messages must be in English.
- Use `snake_case` for files, modules, variables, and functions.
- Use `PascalCase` for structs, enums, and traits.
- Use `SCREAMING_SNAKE_CASE` for constants.
- No abbreviations (`uc`, `cmd`, `repo`) unless universally known.
- No `.unwrap()` in production paths — use `unwrap_or_else` with a descriptive message.
- One concept per file. File name must describe its content without opening it.

## Architecture

- Follow hexagonal (ports and adapters) architecture under `src/core/<feature>/`.
- `application/` owns commands, use cases, and port traits. No framework imports allowed.
- `adapter/input/` owns inbound adapters (HTTP, future gRPC, CLI, ...).
- `adapter/output/` owns outbound adapters (repositories, NLP client, ...).
- Domain structs must NOT derive `Serialize`/`Deserialize` — only DTOs do.
- Adapters depend on port traits, never on use case structs directly.

## Naming Conventions

| Concept               | Convention                     | Example                          |
|-----------------------|--------------------------------|----------------------------------|
| File: command/result  | `<feature>_command.rs`         | `conversation_command.rs`        |
| File: use case        | `<feature>_usecase.rs`         | `conversation_usecase.rs`        |
| File: port traits     | `<feature>_trait.rs`           | `conversation_trait.rs`          |
| File: HTTP DTOs       | `<action>_<feature>_dto.rs`    | `send_message_dto.rs`            |
| File: mapper          | `<action>_<feature>_mapper.rs` | `send_message_mapper.rs`         |
| Struct: use case      | `{Action}{Feature}UseCase`     | `HandleConversationUseCase`      |
| Trait: input port     | `{Action}{Feature}`            | `HandleConversation`             |
| Struct: command       | `{Action}{Feature}Command`     | `HandleConversationCommand`      |
| Struct: result        | `{Action}{Feature}Result`      | `HandleConversationResult`       |
| Struct: HTTP request  | `{Action}{Feature}Request`     | `SendMessageRequest`             |
| Struct: HTTP response | `{Action}{Feature}Response`    | `SendMessageResponse`            |
| Integration test file | `<feature>_routes_integration_test.rs` | `conversation_routes_integration_test.rs` |

## File Structure Rules

```
src/core/<feature>/
├── adapter/
│   ├── input/
│   │   └── web/
│   │       ├── routes.rs                  ← Route definitions + handler functions
│   │       ├── <action>_<feature>_dto.rs  ← HTTP request/response structs
│   │       └── <action>_<feature>_mapper.rs ← From<> impls: DTO ↔ Command/Result
│   └── output/                            ← (future: repositories, external clients)
└── application/
    ├── <feature>_command.rs               ← Command and Result structs
    ├── <feature>_usecase.rs               ← Use case implementation
    └── port/
        ├── input/
        │   └── <feature>_trait.rs         ← Inbound port traits
        └── output/                        ← (future: outbound port traits)
```

## Testing Rules

- Unit tests live in `#[cfg(test)] mod tests` **at the bottom** of the same file.
- Integration tests live in `tests/` at the crate root.
- Use helper functions (`make_command`, `make_server`) to eliminate test boilerplate.
- Do not test private internals; test through the public interface.
- Every use case must have tests for: happy path, missing fields, generated vs provided IDs.
- Every `From<>` mapper impl must have a dedicated test.
- Integration tests must cover: 200 happy path, 415 missing Content-Type, 422 missing required fields.

## Chatbot Rules

- The Rust backend owns conversation state, session lifecycle, and reply generation.
- The Python NLP API owns intent classification and entity extraction.
- Sessions are in-memory for v1 and must not be treated as durable storage.

