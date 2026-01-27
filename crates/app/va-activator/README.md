# va-activator

`va-activator` receives webhook POSTs, filters for an activation word, and returns the command found in the same
sentence.

## Configuration

Environment variables (loaded via `.env` if present):

- `ACTIVATION_WORDS` (required): comma-separated list of words that start listening (case-insensitive).
- `STOP_WORDS` (required): comma-separated list of words that stop listening.
- `BIND_ADDR` (optional): address to bind the HTTP server (default: `127.0.0.1:8090`).
- `WEBHOOK_URL` (required): downstream webhook URL that receives `{ "text": "..." }`.
- `RUST_LOG` (optional): `tracing` filter, e.g. `info`.

## Endpoints

- `GET /health` — returns `{ "status": "ok" }`.
- `POST /webhook` — accepts `{ "text": "..." }`.

## Webhook behavior

- Only requests that start with the activation word are accepted.
- The command is the text after the activation word in the same sentence.
- If any stop word appears in that command text, the request is treated as cancelled.

Response example:

```json
{
  "status": "accepted",
  "command": "set volume to twenty"
}
```

## Run locally

```bash
ACTIVATION_WORDS=va,assistant \
STOP_WORDS=done,cancel \
WEBHOOK_URL=http://127.0.0.1:8091/webhook \
cargo run -p va-activator
```
