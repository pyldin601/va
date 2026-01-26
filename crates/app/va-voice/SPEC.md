# va-voice — Specification

## Purpose

`va-voice` captures audio from the system default microphone, transcribes speech with Vosk, and posts finalized
transcripts to a webhook.

## Inputs

- Microphone audio (system default input device).
- Environment variables:
  - `VOSK_MODEL_PATH` (required): filesystem path to the Vosk model directory.
  - `WEBHOOK_URL` (required): URL to `POST` recognized text to.
  - `WEBHOOK_QUEUE_SIZE` (optional): bounded queue size for webhook delivery (default: `128`).
  - `RUST_LOG` (optional): `tracing` filter, e.g. `info`.

## Outputs

- HTTP `POST` request to `WEBHOOK_URL` for each finalized transcript.
- JSON payload:

```json
{
  "text": "recognized text"
}
```

## Behavior

- Uses the device default sample rate and channel count provided by `cpal`.
- Downmixes to mono by taking channel 0.
- Emits only finalized Vosk results; partial results are ignored.
- Logs each finalized transcript at `info` level (`recognized: <text>`).
- Webhook delivery is performed on a dedicated thread.
- Backpressure is handled with a bounded queue:
  - If the queue is full, the transcript is dropped and a warning is logged.

## Error handling

- Missing or invalid environment variables cause startup failure with a clear error message.
- Audio and decoding errors are logged via `tracing`.
- Webhook failures are logged but do not terminate the process.

## Non-goals

- No HTTP server or health endpoint.
- No activation word detection or command parsing.
- No persistence, retries, or replay.
