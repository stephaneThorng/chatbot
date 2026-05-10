# Conversation Core

## Session Lifecycle

`HandleConversationUseCase` receives a user message and an optional `session_id`.
If the session exists, the stored `Conversation` is reused. If it does not exist,
the use case creates a new `Conversation` with the domain injected at bootstrap.

For v1, sessions are stored in memory. They are conversation state, not durable
business records.

## Conversation State

Conversation state is modeled in the domain with:

- `Idle`: no active workflow.
- `Workflow`: one active workflow collecting generic slot requirements.
- `WF_CHOICE`: NLU task context used when all required workflow slots are filled
  and the bot expects a binary confirmation.

The current runtime execution path for workflow turns is the workflow handler
selected by `ConversationProcessor`. Workflow task derivation lives on
`Conversation`.

Core transitions:

- Idle + workflow intent starts the matching workflow.
- Active workflow + slot entities fills slots through handler-owned policy mappings.
- Ready workflow derives `WF_CHOICE`.
- `affirmative` completes the active workflow.
- `negative` cancels the active workflow.
- `cancel` is a built-in interrupt that only cancels an active workflow.
- Informational intents are not persistent workflow state and are handled while
  the conversation is idle by the application `ConversationProcessor`.

## Intent, Task, Workflow, Catalog

These are separate concepts:

- `intent`: model label predicted for an utterance.
- `workflow`: deterministic conversation behavior started by configured intents.
- `task`: NLU context tag passed while a workflow step is active.
- `handler policy`: Rust-built intent reference exposed by each `IntentHandler`,
  including category, workflow slots, entity-to-slot mappings, NLU task tags, and
  workflow prompt/completion keys.
- `catalog`: shared text catalog for system/fallback i18n keys.

Policies remain Rust-built in v1. Restaurant currently has
`reservation_create` and `reservation_cancel` workflow handlers. Hotel
intentionally registers no handlers until that domain is defined.

## Domain Package Layout

The conversation domain is organized by responsibility:

- `model`: stateful domain objects and value objects, such as `Conversation`,
  `ConversationId`, `Workflow`, `SlotBag`, and `DomainType`.
- `catalog`: shared system text metadata.
- `fsm`: deterministic conversation state transitions and typed effects.

Compatibility re-exports are kept from `domain::conversation`,
`domain::intent`, and related paths to avoid forcing unrelated code changes.

## Language Detection

Language detection happens only when a conversation is created. Existing sessions
preserve the language stored in `Conversation.lang`.

The use case depends on `LanguageDetectorPort`; the current outbound adapter uses
`langdetect-rs` and normalizes unsupported detections to English.

## Conversation Processing

`ConversationProcessor` is the application-level decision point after NLU:

- active workflows and workflow-starting intents are delegated to the matching
  workflow handler;
- informational intents are delegated to stateless `IntentHandler`
  implementations;
- static conversational replies such as greeting, thanks, and goodbye are handled
  by static reply handlers;
- unknown labels fall back to deterministic system text.

Intent handlers isolate immediate "what to do for this intent" behavior from
the use case. They receive the raw message, language, intent, and NER entities.
They do not mutate `Conversation` and do not create pending clarification
state. If required NER is missing, the handler returns a direct message asking
the user to reformulate; the conversation remains `Idle`.

## Do / Don't

- Do keep `HandleConversationUseCase` as a readable orchestration trunk.
- Do keep `ConversationProcessor` as a router that picks one path for the turn.
- Do keep workflow behavior in workflow handlers and domain-owned `with_*`
  methods.
- Do return updated values from owners such as `Conversation::with_workflow_slot`
  and `Workflow::with_slot`.
- Do keep application-only intermediate structs out of the domain package.

- Do not mutate borrowed external state deeply from application helpers.
- Do not split the same workflow execution logic between processor, use cases,
  and handlers.
- Do not use raw strings for backend-owned slot, intent, or entity checks when a
  typed enum exists.
- Do not use catalogs as a second execution layer. Shared metadata is fine;
  runtime behavior belongs elsewhere.

## Use Case Role

`HandleConversationUseCase` is intentionally thin orchestration:

1. Load or create the conversation.
2. Detect language for new sessions only.
3. Build the shared text catalog for the conversation domain.
4. Ask the processor for the active workflow NLU task.
5. Call the NLU gateway.
6. Delegate the NLU result to `ConversationProcessor`.
7. Processor routes to a workflow handler or an idle intent handler and returns
   the reply.
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
