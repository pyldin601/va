# va-activator — Specification

## Purpose

`va-activator` listens for webhook requests, detects an activation word, buffers subsequent text, and stops when a
stop word is received.

## Inputs

- HTTP `POST /webhook` with JSON body:

```json
{
  "text": "recognized text"
}
```

- Environment variables:
  - `ACTIVATION_WORD` (required)
  - `STOP_WORDS` (required, comma-separated)
  - `BIND_ADDR` (optional, default `127.0.0.1:8090`)
  - `RUST_LOG` (optional)

## Outputs

- JSON responses describing the current state:

```json
{
  "status": "ignored | listening | capturing | stopped",
  "command": "... or null"
}
```

## Behavior

- Text is normalized by trimming and converting to lowercase.
- If idle and the text starts with the activation word, listening begins.
- While listening, text is buffered until a stop word is received.
- On stop word, listening resets and the buffered command is returned.
- If stop word is received immediately, the command is null.

## Endpoints

- `GET /health` for status.
- `POST /webhook` for text ingestion.

## Non-goals

- No persistence, retries, or queuing.
- No streaming responses.
- No multi-session or per-client state.
