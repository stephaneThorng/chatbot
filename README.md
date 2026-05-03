# Restaurant Chatbot

This repository contains a restaurant chatbot prototype with two services:

- `backend/chatbot`: Kotlin/Ktor backend that owns sessions, conversation state, restaurant data, and replies.
- `nlp-api`: Python NLP API that analyzes user text and returns intent plus extracted entities.

The v1 backend exposes a single HTTP endpoint for chat messages and keeps all state in memory. It is designed for local development and a single running backend instance.

## Backend Scope

The Python NLP API owns restaurant business intents:

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

The Kotlin backend owns conversation acts such as `greeting`, `thanks`, and `farewell`. It strips simple polite prefixes or suffixes before calling the NLP API, while preserving the detected act in the chat response metadata.

Reservation intents are handled with an explicit finite state machine. Informational intents are handled through deterministic dataset-backed handlers.

The backend uses a feature-level hexagonal structure under `core/chat`:

- `domain` for pure conversation, intent, workflow, NLP, session, restaurant knowledge, and reservation models.
- `application` for use cases, intent decisions, state handling, workflow progression, and outbound ports.
- `adapter` for Ktor HTTP, in-memory repositories, and the HTTP NLP client.

## Chat API

`POST /api/v1/chat/messages`

Request:

```json
{
  "message": "I want to book a table",
  "sessionId": null
}
```

Response:

```json
{
  "sessionId": "generated-session-id",
  "reply": "What name should I use for the reservation?",
  "intent": "reservation_create",
  "conversationAct": null,
  "state": "WORKFLOW",
  "slots": {},
  "missingSlots": ["name", "date", "time", "people"],
  "completed": false
}
```

When `sessionId` is omitted, the backend creates a new session. Sessions use a sliding in-memory TTL of about 30 minutes.

## Running Locally

Start the Kotlin backend:

```powershell
cd backend/chatbot
.\gradlew.bat run
```

The backend defaults to:

- Backend URL: `http://localhost:8080`
- NLP API URL: `http://localhost:8000`

The backend has deterministic local fallback behavior when the NLP API is unavailable, but the intended full flow is to run both services.

## Terminal Chat

Start the backend in one terminal:

```powershell
cd backend/chatbot
.\gradlew.bat run
```

Start the terminal chat client in another terminal:

```powershell
cd backend/chatbot
.\gradlew.bat chatCli
```

The CLI sends each message to `POST /api/v1/chat/messages`, stores the returned `sessionId`, and prints the bot reply with intent, conversation act, and state metadata. Use `/reset` to start a new session and `/exit` to quit.

To target another backend URL:

```powershell
.\gradlew.bat chatCli -PchatbotApiUrl=http://localhost:8080/api/v1/chat/messages
```

## Testing

Run backend tests:

```powershell
cd backend/chatbot
.\gradlew.bat test
```
