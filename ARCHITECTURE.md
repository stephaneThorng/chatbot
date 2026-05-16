# Restaurant Chatbot Architecture

## Overview

The active backend is the Rust service in `corebot-backend`. It orchestrates restaurant chat conversations, keeps state in memory for v1, calls the local NLU engine, and returns deterministic replies over HTTP.

The repository also contains `model_training`, which produces the ONNX model artifacts consumed by the Rust runtime. Training and runtime are separate responsibilities.

## Repository Structure

- `corebot-backend`: production backend code
- `model_training`: Python training and ONNX export
- `scripts`: helper scripts; some still reflect the removed Kotlin setup and are not the architecture source of truth

## Backend Architecture

The Rust backend follows feature-based hexagonal architecture under `corebot-backend/src/core/<feature>/`.

- `conversation`: session lifecycle, workflow state, slot filling, reply generation
- `nlu_engine`: local NLU inference orchestration and decoding
- `restaurant`: transitional restaurant data and reservation capability, currently backed by PostgreSQL
- `configuration` or `back_office`: future client-facing restaurant configuration and CRUD

Each feature is split into:

- `domain`: pure business state, value objects, invariants
- `application`: use cases, application services, inbound and outbound ports
- `adapter`: HTTP, gateways, repositories, external runtimes, and other concrete integrations

## Main Flow

1. Client sends `POST /api/v1/conversation/send_message`.
2. Axum maps the request DTO to the conversation command.
3. `HandleConversationUseCase` loads or creates the in-memory conversation state.
4. Conversation logic determines the current workflow/state behavior.
5. Business text is analyzed through the NLU analyzer port.
6. `nlu_engine` builds tagged input, validates artifacts, runs ONNX inference through its runtime port, and decodes the result into `NluAnalysis`.
7. The conversation use case updates state and returns a deterministic reply.
8. Axum maps the result back to the HTTP response DTO.

## Layer Boundaries

- Domain must not import `serde`, Axum, ONNX runtime, tokenizer APIs, or adapter modules.
- Application may depend on domain and ports only.
- Input adapters may depend on input ports and DTO/mapper code only.
- Output adapters implement output ports and may use concrete infrastructure libraries.
- Cross-feature access should go through ports or stable domain contracts, not through another feature's concrete adapter.
- The target direction is for `conversation` to own chatbot-facing restaurant reads and reservation workflows, while client-facing CRUD lives in `configuration` or `back_office`.

## NLU Runtime Contract

`model_training` exports:

- `model.onnx`
- `tokenizer.json`
- `label_maps.json`
- `onnx_contract.json`

`corebot-backend` consumes those files through `COREBOT_NLU_ONNX_DIR`.

The application layer in `nlu_engine` owns:

- tagged input construction
- artifact validation
- result decoding into `NluAnalysis`

The ONNX adapter owns:

- artifact loading
- tokenizer calls
- ONNX Runtime execution
- returning raw logits and token metadata
