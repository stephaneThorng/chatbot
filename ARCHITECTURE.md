# Restaurant Chatbot Architecture

## Overview

The backend is a Kotlin/Ktor service that orchestrates restaurant chat conversations. It receives messages over HTTP, resolves or creates an in-memory session, calls the Python NLP API when available, updates conversation state, and returns a deterministic bot reply.

The v1 architecture is intentionally single-instance and in-memory. Ports are defined so session storage, restaurant knowledge, reservation inventory, and NLP integration can be replaced later without rewriting the conversation flow.

## Package Structure

The chat feature follows a hexagonal layout under `core/chat`:

- `domain`: pure conversation, intent, NLP, workflow, session, knowledge, and reservation models.
- `application`: use cases, conversation coordination, intent routing, state dispatch, workflow engine, and outbound ports.
- `adapter/in/web`: Ktor routes, HTTP DTOs, and web mappers.
- `adapter/out`: in-memory repositories and the HTTP NLP client.

Application packages may be grouped by concept. For example:

- `application.intent.catalog`
- `application.intent.decision`
- `application.intent.handler`
- `application.workflow`
- `domain.intent`
- `domain.workflow`

## Main Flow

1. Client sends `POST /api/v1/chat/messages` with `message` and optional `sessionId`.
2. Ktor maps the request DTO to `HandleConversationCommand`.
3. `HandleConversationUseCase` loads or creates a `ConversationSession`.
4. `ConversationCoordinator` extracts backend-owned conversation signals.
5. Standalone greetings, thanks, or farewells are handled without NLP.
6. Business text is sent to `NlpAnalyzer` with the restaurant domain and session context.
7. `IntentDecisionEngine` combines ranked NLP intents, utterance signals, intent policies, session state, and topic memory.
8. `ConversationStateDispatcher` delegates to either `IdleStateHandler` or `WorkflowStateHandler`.
9. The selected `IntentHandler` owns the deterministic reply for its business intent.
10. Workflow intents advance through `WorkflowEngine` and generic workflow rules.
11. The updated session is saved with a refreshed sliding TTL.
12. Ktor returns session metadata, reply text, handled business intent, conversation act, state, slots, and completion status.

## Intent Handling

The Python NLP API owns restaurant business intent classification:

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

The Kotlin backend owns simple conversation acts:

- `greeting`
- `thanks`
- `farewell`

Conversation acts are API metadata, not workflow state. Standalone acts bypass NLP. Mixed messages such as `Hello, I want a reservation` are stripped before NLP and the reply is prefixed deterministically.

Each business intent is handled by an `IntentHandler`. Intent metadata such as category, clarification support, workflow allowance, and topic continuation support lives in `IntentPolicy` and is exposed through `IntentCatalog`.

## Intent Decision

The backend treats NLP as evidence, not as an unconditional routing decision. `IntentDecisionEngine` can:

- accept a business intent
- ask a deterministic clarification question
- return `unknown`

The decision uses the primary intent, ranked intent candidates, utterance kind, confidence, candidate margin, entity support, session topic memory, pending clarification state, and active workflow ownership. The backend does not classify business intents with local keyword rules. Missing `utterance` metadata is treated as `unknown` so old or unusable NLP responses fail safely.

## Workflow Model

Conversation state is coarse:

- `IDLE`
- `WORKFLOW`

Detailed workflow progress is stored in `ConversationSession.currentWorkflow`. A workflow is owned by an `IntentName` and contains ordered generic requirements.

Reservation creation and modification use these requirements:

- `name`
- `date`
- `time`
- `people`
- `confirmation`

Reservation cancellation uses a `confirmation` requirement.

Each requirement owns its validation and transformation through a value type. Examples include person name parsing, relative date normalization, reservation time validation, party size validation, and yes/no confirmation parsing.

## Workflow Interruption

Informational intents can be answered during an active workflow. The informational `IntentHandler` produces the primary reply, then the active workflow can be enriched in the background using `ProcessingMode.BACKGROUND_ENRICHMENT`.

Workflow cancellation is a backend-owned workflow command, not a business intent shortcut. A standalone `cancel`, `stop`, `abort`, or `never mind` aborts the current workflow only when a cancellable workflow is active. From `IDLE`, `reservation_cancel` remains a normal business intent for cancelling a confirmed reservation snapshot.

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

The NLP response contains:

- `intent`: primary business intent candidate
- `intents`: ranked business intent candidates
- `utterance`: utterance kind such as `business_query`, `small_talk`, `vague_follow_up`, `frustration`, or `out_of_domain`
- `entities`: raw spans plus canonical values, resolution metadata, and normalization status
- `warnings`: non-fatal analysis warnings

The NLP adapter is an anti-corruption layer. It maps Python wire names into Kotlin domain types such as `IntentName`, `SlotName`, `NlpAnalysis`, ranked intents, utterance signals, and normalized entities. Workflow filling prefers canonical entity values and can fall back to raw values when needed.

If the NLP API fails or returns unusable output, the backend keeps the session alive and falls back to deterministic clarification or unknown/help replies.

## Future WhatsApp Adapter

The current HTTP endpoint is the first inbound adapter. A future WhatsApp Business adapter should translate WhatsApp events into the same application command and reuse the existing use case, session repository port, and response model.
