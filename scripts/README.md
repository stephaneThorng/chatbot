# Chatbot Test Scripts

This directory still contains helper scripts from the earlier Kotlin-based setup. The Kotlin backend has been removed, so this directory is not the source of truth for the current backend architecture.

## Current Status

- `install.*` and `launch.*` still reference the removed Kotlin backend and need a Rust rewrite before they can be used as-is.
- `train.*` remains conceptually aligned with `model_training`, but should also be reviewed before relying on it operationally.

## Source of Truth

Use these documents instead:

- repository rules: [AGENTS.md](/C:/Users/steph/git/chatbot/AGENTS.md)
- backend rules: [corebot-backend/AGENTS.md](/C:/Users/steph/git/chatbot/corebot-backend/AGENTS.md)
- backend architecture: [corebot-backend/ARCHITECTURE.md](/C:/Users/steph/git/chatbot/corebot-backend/ARCHITECTURE.md)

## Current Manual Workflow

Run the Rust backend:

```powershell
cd corebot-backend
cargo run
```

Run backend tests:

```powershell
cd corebot-backend
cargo test
```

Run model training tests:

```powershell
cd model_training
python -m pytest tests
```
