# va-activator — Specification

## Purpose

`va-activator` listens for webhook requests, detects an activation word, and extracts the command from the same
sentence.

## Inputs

- HTTP `POST /webhook` with JSON body:

```json
{
  "text": "recognized text"
}
```

- Environment variables:
  - `ACTIVATION_WORDS` (required, comma-separated)
  - `STOP_WORDS` (required, comma-separated)
  - `BIND_ADDR` (optional, default `127.0.0.1:8090`)
  - `WEBHOOK_URL` (required)
  - `RUST_LOG` (optional)

## Outputs

- JSON responses describing the current state:

```json
{
  "status": "ignored | stopped | accepted | error",
  "command": "... or null"
}
```

## Behavior

- Text is normalized by trimming and converting to lowercase.
- If the text starts with any activation word, the command is the text after it.
- If any stop word appears in the command text, the request is treated as cancelled.
- If the command text is empty, the request is ignored.
- If the command is accepted, it is forwarded to `WEBHOOK_URL` as `{ "text": "..." }`.

## Endpoints

- `GET /health` for status.
- `POST /webhook` for text ingestion.

## Non-goals

- No persistence, retries, or queuing.
- No streaming responses.
- No multi-session or per-client state.
