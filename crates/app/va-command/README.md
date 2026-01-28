# va-command

Receives webhook POSTs with `{ "text": "..." }`, sends a prompt to Ollama, logs the model response to stderr, and
responds with a short echo.

## Configuration

Environment variables (loaded via `.env` if present):

- `BIND_ADDR` (optional): address to bind the HTTP server (default: `127.0.0.1:8092`).
- `OLLAMA_BASE_URL` (optional): Ollama API base URL (default: `http://localhost:11434`).
- `OLLAMA_MODEL` (optional): Ollama model name (default: `gemma3n`).

## Endpoints

- `GET /health` — returns `{ "status": "ok" }`.
- `POST /webhook` — accepts `{ "text": "..." }`.

## Run locally

```bash
OLLAMA_BASE_URL=http://localhost:11434 \
OLLAMA_MODEL=gemma3n \
cargo run -p va-command
```
