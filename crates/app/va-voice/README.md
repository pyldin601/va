# va-voice

**va-voice** captures audio from the default microphone, transcribes speech with Vosk, and posts finalized text
to a webhook as JSON.

## Configuration

Environment variables (loaded via `.env` if present):

- `VOSK_MODEL_PATH` (required): filesystem path to the Vosk model directory.
- `WEBHOOK_URL` (required): URL to `POST` recognized text to.
- `WEBHOOK_QUEUE_SIZE` (optional): bounded queue size for webhook deliveries (default: 128).

## Webhook payload

```json
{
  "text": "recognized text"
}
```

## Run locally

```bash
VOSK_MODEL_PATH=/path/to/vosk-model \
WEBHOOK_URL=http://localhost:8080/voice \
cargo run -p va-voice
```
