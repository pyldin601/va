# PLAN — va (Voice Assistant)

## Goal

Provide a simple, local speech-to-webhook service. The system listens to the default microphone, transcribes with
Vosk, and posts finalized text to a configured webhook.

## Current pipeline

Microphone → Vosk ASR → finalized transcript → HTTP POST webhook

## Service: va-voice

- Capture audio from the system default input device
- Feed audio into Vosk for speech recognition
- Emit only finalized recognition results
- POST each finalized phrase to `WEBHOOK_URL` as JSON:

```json
{
  "text": "recognized text"
}
```

## Inputs

- `VOSK_MODEL_PATH` — path to the Vosk model directory
- `WEBHOOK_URL` — webhook to receive recognized text

## Out of scope

- Activation words or command parsing
- HTTP server / SSE streams
- AI agent planning or tool execution
- Persistence, retries, or queuing
