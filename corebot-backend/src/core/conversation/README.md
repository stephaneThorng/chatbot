# Conversation Core

## Session Lifecycle

`HandleConversationUseCase` receives a user message and an optional `session_id`.
If the session exists, the stored `Conversation` is reused. If it does not exist,
the use case creates a new `Conversation` with the domain injected at bootstrap.

For v1, sessions are stored in memory. They are conversation state, not durable
business records.

## State Machine

Conversation state transitions live in the domain FSM:

- `Idle`: no active workflow.
- `Workflow`: one active workflow collecting generic slot requirements.
- `WF_CHOICE`: NLU task context used when all required workflow slots are filled
  and the bot expects a binary confirmation.

The FSM receives a `ConversationEvent` built from NLU output and returns a
`ConversationTransition` with the updated conversation state plus a typed
`ConversationEffect`.

Core transitions:

- Idle + workflow intent starts the matching workflow.
- Active workflow + slot entities fills slots through catalog mappings.
- Ready workflow derives `WF_CHOICE`.
- `affirmative` completes the active workflow.
- `negative` cancels the active workflow.
- `cancel` is a built-in interrupt that only cancels an active workflow.

## Intent, Task, Workflow, Catalog

These are separate concepts:

- `intent`: model label predicted for an utterance.
- `workflow`: deterministic conversation behavior started by configured intents.
- `task`: NLU context tag passed while a workflow step is active.
- `catalog`: Rust-built domain reference containing intent policies, workflow
  slots, entity-to-slot mappings, NLU task tags, and i18n keys.

The catalog remains Rust-built in v1. Restaurant currently has
`reservation_create` and `reservation_cancel` workflows. Hotel intentionally has
an empty catalog until that domain is defined.

## Domain Package Layout

The conversation domain is organized by responsibility:

- `model`: stateful domain objects and value objects, such as `Conversation`,
  `ConversationId`, `Workflow`, `SlotBag`, and `DomainType`.
- `catalog`: Rust-built intent and workflow metadata.
- `fsm`: deterministic conversation state transitions and typed effects.

Compatibility re-exports are kept from `domain::conversation`,
`domain::intent`, and related paths to avoid forcing unrelated code changes.

## Language Detection

Language detection happens only when a conversation is created. Existing sessions
preserve the language stored in `Conversation.lang`.

The use case depends on `LanguageDetectorPort`; the current outbound adapter uses
`langdetect-rs` and normalizes unsupported detections to English.

## Reply Rendering

The domain FSM never calls `rust-i18n` or domain gateways. It emits typed effects
such as slot prompts, confirmation prompts, workflow completions, static intent
responses, and dynamic domain responses.

`ConversationReplyRenderer` resolves those effects in the application layer:

- i18n keys are rendered with `rust-i18n`.
- informational intent effects are delegated to registered `IntentHandler`
  implementations when a handler exists.
- dynamic domain responses inside handlers use application ports such as
  `DomainGatewayPort`.

Intent handlers isolate "what to do for this intent" from both the FSM and the
use case. `OpeningHoursIntentHandler` is the first concrete handler and handles
`ask_opening_hours`.

## Use Case Role

`HandleConversationUseCase` is intentionally thin orchestration:

1. Load or create the conversation.
2. Detect language for new sessions only.
3. Build the catalog for the conversation domain.
4. Ask the FSM for the current NLU context.
5. Call the NLU gateway.
6. Apply the FSM event.
7. Render the returned effect.
8. Save the conversation.
9. Return `session_id` and `reply`.

It does not own workflow transition rules, slot filling, confirmation behavior,
language detection internals, i18n formatting, or domain gateway reply logic.

## V1 Limitations

- Sessions are in-memory and not durable.
- Catalogs are built in Rust, not external YAML or database data.
- Restaurant is wired in bootstrap as `DomainType::Restaurant`.
- Dynamic domain replies are minimal and deterministic.
- The Rust backend does not implement keyword intent classification; NLU labels
  must come from the NLU engine.
