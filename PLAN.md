# PLAN — va (Voice Assistant) MVP

## Goal

Build a minimal, streaming, voice-driven system composed of four independent services. Each service has a single responsibility and communicates via HTTP streaming.

The system continuously listens to audio, converts speech to text, detects commands, uses AI to plan actions, and executes those actions through explicit tools.

---

## High-level pipeline

Microphone\
→ ASR (Vosk)\
→ text stream\
→ activation + command buffering\
→ command stream\
→ AI reasoning\
→ tool execution

---

## Services overview

1. **va-voice**\
   Audio input + speech recognition → text stream

2. **va-activator**\
   Activation word detection + command buffering → command stream

3. **va-agent**\
   AI reasoning + tool planning

4. **va-tools**\
   Tool execution (integrations, side effects)

---

## Service 1: va-voice

### Responsibility

- Capture audio from microphone
- Convert audio to required format (16 kHz, mono, PCM16)
- Decode speech using Vosk
- Stream recognized text events to consumers

### Endpoint

**GET **`` — streaming endpoint (SSE, `text/event-stream`)

### Stream events

- `partial` — intermediate recognition result
- `final` — finalized phrase
- `error` — decoding or audio error

### Event payload (minimal)

```json
{
  "ts_ms": 123456,
  "text": "recognized text",
  "kind": "partial | final | error"
}
```

### Behavior

- Runs continuously
- No activation logic
- No command semantics
- Emits text as soon as it is decoded

### Other endpoints

- `GET /health`

### MVP done when

- Speaking into the microphone produces `final` text events on `/text`

---

## Service 2: va-activator

### Responsibility

- Connect to `va-voice /text`
- Consume **final** text events
- Detect activation word
- Accumulate command text
- Detect command end or cancellation
- Stream finalized commands

### State machine

- `IDLE` — waiting for activation word
- `CAPTURING` — accumulating command text
- `READY` — command finalized and emitted

### Control words (MVP)

- Activation word: `va`
- Stop word (end command): `done`
- Cancel word (abort command): `cancel`

### Command completion rules

A command is finalized when:

- Stop word is detected, OR
- No new `final` text arrives for N seconds

A command is cancelled when:

- Cancel word is detected during `CAPTURING`

### Endpoint

**GET **`` — streaming endpoint (SSE)

### Stream events

#### Command

```text
event: command
data: {
  "id": "uuid",
  "text": "set volume to twenty",
  "ts_ms": 123456
}
```

#### Cancel (optional)

```text
event: cancel
data: {
  "id": "uuid",
  "reason": "cancel_word"
}
```

### Behavior

- Commands are not persisted
- One consumer (va-agent) in MVP
- If consumer disconnects, commands are lost (acceptable for MVP)

### Other endpoints

- `GET /health`

### MVP done when

- Saying: “va set volume to twenty done”
- Emits exactly one `command` event with text `set volume to twenty`

---

## Service 3: va-agent

### Responsibility

- Consume command stream from `va-activator`
- For each command:
  - Send text to AI (LLM)
  - Receive structured tool calls
  - Validate tool calls (schema + allowlist)
  - Dispatch tool calls to executor
- Aggregate execution results

### Input

- `GET /command` (SSE)

### Output

- `POST /execute` to va-tools
- Internal logs/events for observability

### Tool call format (MVP)

Each tool call includes:

```json
{
  "id": "uuid",
  "name": "tool.name",
  "args": {},
  "risk": "low | medium | high",
  "reason": "why this tool is needed",
  "requires_confirmation": false
}
```

Any tool call failing validation is rejected.

### Behavior

- One command = one independent AI cycle
- No buffering of commands
- Minimal backpressure handling (ignore new commands if busy)

### MVP done when

- Command triggers AI call
- AI returns at least one valid tool call
- Tool call is sent to va-tools

---

## Service 4: va-tools

### Responsibility

- Execute real-world actions based on tool calls
- Enforce security and safety boundaries
- Return execution results

This service is the **final authority** on what actions are allowed.

### Endpoint

**POST **``

Request:

```json
{
  "calls": [ ToolCall ]
}
```

Response:

```json
{
  "results": [ ToolResult ]
}
```

### ToolResult (MVP)

```json
{
  "id": "uuid",
  "ok": true,
  "data": {},
  "error": null
}
```

### Security policy (MVP)

- Allowlist tool names
- Validate arguments
- Risk handling:
  - `low`: auto-execute
  - `medium`: execute with logging
  - `high`: reject (confirmation added later)

### Initial tools (recommended)

- `weather.get_forecast`
- `music.play`
- `music.pause`
- `music.next`
- `system.say` (future TTS)

### MVP done when

- Executor accepts tool calls
- Executes at least one stub tool
- Returns structured results

---

## Streaming decisions (final)

- Text stream: **SSE**
- Command stream: **SSE**
- Tool execution: **HTTP POST**
- No persistence
- No queues
- No retries
- Single consumer per stream

---

## Development phases

### Phase 0 — Workspace

- Cargo workspace
- Service crates
- PLAN.md

### Phase 1 — va-voice skeleton

- `/health`
- `/text` SSE with mock events

### Phase 2 — Vosk integration

- Decode audio buffers
- Emit `partial` and `final` events

### Phase 3 — Microphone input

- Real microphone capture
- End-to-end speech → `/text`

### Phase 4 — va-activator

- State machine
- `/command` SSE

### Phase 5 — va-agent

- Command stream consumer
- AI integration (can be mocked)
- Tool call validation

### Phase 6 — va-tools

- `/execute`
- Stub tools
- Security boundaries

---

## Non-goals (for MVP)

- Persistence
- Multi-consumer streams
- Authentication
- High-risk actions
- Full natural language dialogue

---

## Open questions (non-blocking)

- Supported languages
- Silence timeout values
- Confirmation UX for high-risk tools
- Persistence / replay (future)

