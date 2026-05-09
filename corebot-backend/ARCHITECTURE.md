# corebot-backend Architecture

## Overview

`corebot-backend` is the Rust backend for the chatbot. It uses Axum for HTTP routing and follows feature-based hexagonal architecture under `src/core/<feature>/`.

The main architectural goal is strict separation of concerns:

- domain code owns business concepts and invariants;
- application code owns use-case orchestration and ports;
- adapters own concrete protocols, runtimes, clients, repositories, and DTOs.

## Module Structure

```text
src/
├── lib.rs
├── main.rs
└── core/
    ├── conversation/
    │   ├── domain/
    │   ├── application/
    │   │   └── port/
    │   │       ├── input/
    │   │       └── output/
    │   └── adapter/
    │       ├── input/web/
    │       └── output/
    ├── nlu_engine/
    │   ├── domain/
    │   ├── application/
    │   │   └── port/
    │   │       ├── input/
    │   │       └── output/
    │   └── adapter/output/
    └── restaurant/
        ├── application/port/input/
        └── adapter/input/
```

## Layer Rules

| Layer | May depend on | Must not import |
|-------|---------------|-----------------|
| `domain/` | Rust standard library and pure domain modules | `serde`, `axum`, `ort`, `tokenizers`, adapters, application services |
| `application/` | domain types, application services, input/output port traits | Axum, HTTP DTOs, adapter modules, concrete repositories/clients/runtimes |
| `application/port/input/` | command/result/domain types | adapters, concrete use-case construction |
| `application/port/output/` | domain types and boundary data types needed by use cases | concrete adapter implementations |
| `adapter/input/` | input ports, commands/results, DTOs, mappers, framework crates | domain policy, persistence/runtime implementation details |
| `adapter/output/` | output ports, domain types, concrete infrastructure crates | inbound adapters, use-case orchestration internals |

Dependency direction must point inward:

```text
adapter/input  ─┐
                ├─> application ports/use cases ─> domain
adapter/output ─┘
```

Cross-feature dependency must use a port. A feature must not reach into another feature's adapter or concrete use case.

The production composition root lives in `src/main.rs`. Inbound HTTP adapter modules expose route construction that accepts already-built use-case dependencies instead of instantiating concrete output adapters directly.

## Method and Object Placement

- Put business state and invariants in `domain/`.
- Put application commands/results in `application/*_command.rs` unless the feature has a justified split.
- Put use-case coordination in `application/*_usecase.rs`.
- Put reusable application transformations or decoders in `application/<service>.rs`.
- Put inbound protocol mapping in `adapter/input/web/*_dto.rs` and `*_mapper.rs`.
- Put concrete runtime/client/repository code in `adapter/output/`.
- Keep framework, serialization, filesystem, ONNX, tokenizer, and HTTP concerns out of domain objects.
- Keep concrete adapter APIs out of application methods. Application calls traits; adapters implement traits.

When a behavior appears to need multiple layers, split it explicitly:

- adapter: retrieve or produce raw boundary data;
- application: validate, orchestrate, decode, and decide;
- domain: enforce business invariants and represent business state.

## Naming Conventions

| Concept | Convention | Example |
|---------|------------|---------|
| Use case struct | `{Action}{Feature}UseCase` | `HandleConversationUseCase` |
| Input port trait | `{Action}{Feature}` | `HandleConversation` |
| Output port trait | `{Capability}` or `{Entity}Repository` | `NluModelRuntime`, `ConversationRepository` |
| Command | `{Action}{Feature}Command` | `AnalyzeTextCommand` |
| Result | `{Action}{Feature}Result` | `HandleConversationResult` |
| HTTP request DTO | `{Action}{Feature}Request` | `SendMessageRequest` |
| HTTP response DTO | `{Feature}Response` or `{Action}{Feature}Response` | `SendMessageResponse` |
| Mapper object/module | `{Feature}WebMapper` or `<action>_<feature>_mapper.rs` | `send_message_mapper.rs` |
| Route file | `routes.rs` | `adapter/input/web/routes.rs` |
| Integration test | `<feature>_routes_integration_test.rs` | `conversation_routes_integration_test.rs` |

## Feature Responsibilities

### Conversation

- Owns session lifecycle, workflow state, slot filling, conversation policy, and deterministic replies.
- Calls NLU through an analyzer port; it must not call ONNX runtime or tokenizer code directly.
- May use restaurant data through a domain gateway port.

### NLU Engine

- Owns local NLP inference behavior: tagged input construction, artifact validation, tokenization boundary, ONNX execution, intent ranking, and BIO entity decoding.
- Application layer owns preprocessing, artifact validation, model-output decoding, and final `NluAnalysis` construction.
- `adapter/output/onnx_nlu_runtime.rs` owns only artifact loading, tokenizer integration, ONNX Runtime execution, and returning raw logits plus token metadata through the output port.
- Do not add keyword intent classifiers here or in conversation code.

### Restaurant

- Owns static restaurant business data for v1.
- Remains in-memory unless a persistence port is explicitly introduced.

## NLU Engine Flow

```text
conversation/adapter/output/NluEngineGateway
  -> AnalyzeText input port
  -> AnalyzeTextUseCase
       build TaggedInput
       validate artifact contract
       call NluModelRuntime output port
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

- [tests/architecture_test.rs](/C:/Users/steph/git/chatbot/corebot-backend/tests/architecture_test.rs): stable layer-access rules with `arch_test_core`
- [tests/architecture_source_rules_test.rs](/C:/Users/steph/git/chatbot/corebot-backend/tests/architecture_source_rules_test.rs): source-level checks for forbidden imports across layers and features

## Rust Architecture Tooling

Useful crates/tools for architecture checks:

- `arch-lint`: AST-based architecture linter that can run in `cargo test` with `arch_lint::check!()` and configuration in `arch-lint.toml`.
- `arch_test_core` / `cargo-archtest-cli`: rule-based architecture tests for module/layer access rules such as `MayNotAccess`, `MayOnlyAccess`, and cyclic dependency checks. This repository currently uses `arch_test_core` for layer-access rules.
- `dep_graph_rs`: internal module dependency graph visualization from `use crate::...` statements. Useful for audits, not a hard warning system by itself.
- `cargo-deny`: dependency graph policy checks for third-party crates, licenses, advisories, and duplicate/banned dependencies. It does not enforce internal hexagonal layers.

Recommended next step: add an architecture test with `arch_test_core` or `arch-lint` once the layer rules are stable enough to enforce in CI.
