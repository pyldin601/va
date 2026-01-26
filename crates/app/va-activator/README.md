# va-activator

`va-activator` receives webhook POSTs, waits for an activation word, then buffers subsequent text until a stop word
is received.

## Configuration

Environment variables (loaded via `.env` if present):

- `ACTIVATION_WORD` (required): word that starts listening (case-insensitive).
- `STOP_WORDS` (required): comma-separated list of words that stop listening.
- `BIND_ADDR` (optional): address to bind the HTTP server (default: `127.0.0.1:8090`).
- `RUST_LOG` (optional): `tracing` filter, e.g. `info`.

## Endpoints

- `GET /health` — returns `{ "status": "ok" }`.
- `POST /webhook` — accepts `{ "text": "..." }`.

## Webhook behavior

- If not listening, only requests that start with the activation word are accepted.
- Once activated, subsequent text is buffered.
- When a stop word is received, the buffer is returned as `command` and listening resets.

Response example:

```json
{
  "status": "stopped",
  "command": "set volume to twenty"
}
```

## Run locally

```bash
ACTIVATION_WORD=va \
STOP_WORDS=done,cancel \
cargo run -p va-activator
```
