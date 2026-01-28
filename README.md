# va

Minimal voice-assistant services. Four binaries form a pipeline:

- `va-voice` captures mic audio, transcribes via Vosk, and POSTs `{ "text": "..." }` to a webhook.
- `va-activator` receives webhook POSTs, filters for activation/stop words, and forwards accepted commands to a downstream webhook.
- `va-command` receives commands, sends a prompt to Ollama, logs the model response to stderr, and replies with an echo.
- `va-actions` lists supported commands and executes tool calls (currently server time/date).

Typical flow: `va-voice` → `va-activator` → `va-command` → `va-actions`.
