# Restaurant Chatbot Architecture

## Overview

The backend is a Kotlin/Ktor service that orchestrates restaurant chat conversations. It receives messages over HTTP, resolves or creates a session, calls the Python NLP API when available, updates conversation state, and returns a deterministic bot reply.

The v1 architecture is intentionally single-instance and in-memory. Ports are still defined so session storage, restaurant knowledge, inventory, and NLP integration can be replaced later without rewriting the orchestration logic.

## Main Flow

1. Client sends `POST /api/v1/chat/messages` with `message` and optional `sessionId`.
2. Ktor maps the request DTO to `HandleChatMessageCommand`.
3. `HandleChatMessageUseCase` loads or creates a `ConversationSession`.
4. `HandleChatMessageUseCase` delegates message handling to `ChatMessageOrchestrator`.
5. The orchestrator detects simple conversation acts and strips them from mixed business messages.
6. The orchestrator calls `NlpAnalyzer` with business message text, restaurant domain, and conversation context.
7. `IntentResolver` resolves the business intent from NLP output.
8. `ChatStateMachine` dispatches by `ConversationSession.state`.
9. The selected state handler treats the resolved intent as an input event.
10. Informational intents read from the restaurant dataset through `ReplyComposer`.
11. Workflow states advance generic requirements through `ReservationWorkflowService`.
12. The updated session is saved with a refreshed sliding TTL.
13. Ktor returns session metadata, reply text, business intent, conversation act, state, slots, and completion status.

## Intent Handling

The backend mirrors the NLP API business intent names:

- `reservation_create`
- `reservation_modify`
- `reservation_cancel`
- `reservation_status`
- `menu_request`
- `opening_hours`
- `location_request`
- `pricing_request`
- `contact_request`
- `unknown`

The backend separately detects these conversation acts:

- `greeting`
- `thanks`
- `farewell`

Conversation acts are API metadata, not workflow state. Standalone acts bypass NLP. Mixed messages such as `Hello, I want a reservation` are stripped before NLP and the reply is adjusted deterministically.

Reservation intents use an explicit finite state machine. Informational intents are lightweight handlers that can run during an active reservation flow without losing the reservation state.

## Application Services

The chat use case stays thin and delegates specialized behavior:

- `ChatMessageOrchestrator` routes one message across preprocessing, NLP, informational replies, and workflows.
- `ChatStateMachine` dispatches processing by the current `ConversationState`.
- `IdleStateHandler` handles non-workflow messages and workflow starts.
- `WorkflowStateHandler` handles active workflows while still allowing informational intent switches.
- `ConversationActPreprocessor` owns greeting, thanks, and farewell stripping.
- `IntentResolver` owns NLP confidence handling and active workflow fallback.
- `ReservationWorkflowService` owns reservation workflow definitions, requirement filling, confirmation, cancellation, and availability checks.
- `ReplyComposer` owns deterministic response text.

## Workflow FSM

`ConversationSession.currentWorkflow` stores the active workflow separately from generic conversation metadata. The session state is intentionally coarse:

- `IDLE`
- `RESERVATION_CREATION`
- `RESERVATION_MODIFICATION`
- `RESERVATION_CANCELLATION`

Detailed progress is represented by generic workflow requirements rather than by additional states. Reservation creation and modification use `name`, `date`, `time`, `people`, and `confirmation` requirements. Reservation cancellation uses a `confirmation` requirement.

Each requirement has a value type that owns validation and transformation. Examples:

- `PersonNameRequirementType` validates length and name shape.
- `DateRequirementType` resolves values such as `tomorrow`, `Friday`, or `July 7` against the current date.
- `TimeRequirementType` normalizes values such as `7pm` or `19h00`.
- `PartySizeRequirementType` validates party size against v1 inventory constraints.
- `ConfirmationRequirementType` validates yes/no answers.

The state machine dispatches by `ConversationSession.state`, then the active workflow requirements decide what is missing. If no requirement is missing, the workflow can complete and the session returns to `IDLE`.

Active workflows can be cancelled explicitly. For example, `reservation_cancel` during `RESERVATION_CREATION` aborts the in-progress workflow. From `IDLE`, `reservation_cancel` starts a cancellation workflow for the confirmed reservation snapshot.

## In-Memory Data

The backend has two in-memory data categories:

- Conversation sessions keyed by backend-issued `sessionId`.
- Static restaurant knowledge: profile, address, contact details, opening hours, menu items, price ranges, and mock booking inventory rules.

Session data expires with a sliding TTL. Static restaurant data is loaded at application startup and is not admin-editable in v1.

## External NLP API

The backend calls the Python NLP API at `POST /analyze` with:

- `text`
- `domain`
- `context.current_intent`
- `context.previous_intent`
- `context.slots_filled`
- `context.required_slots`

If the NLP API fails, returns an unusable result, or is unavailable, the backend keeps the session alive and falls back to clarification. The backend does not classify business intents with local keywords.

## Future WhatsApp Adapter

The current HTTP endpoint is the first inbound adapter. A future WhatsApp Business adapter should translate WhatsApp events into the same application command and reuse the existing use case, session repository port, and response model.
