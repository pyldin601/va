FROM rust:1.83-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    pkg-config \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN cargo build --release --locked -p va-voice

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

ENV VOSK_MODEL_PATH=/opt/vosk-model
ENV LD_LIBRARY_PATH=/usr/local/lib

RUN mkdir -p /opt/vosk-model \
    && curl -L -o /tmp/vosk-model.zip https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip \
    && unzip /tmp/vosk-model.zip -d /tmp \
    && mv /tmp/vosk-model-small-en-us-0.15/* /opt/vosk-model/ \
    && rm -rf /tmp/vosk-model.zip /tmp/vosk-model-small-en-us-0.15

COPY --from=builder /app/target/release/va-voice /usr/local/bin/va-voice
COPY --from=builder /usr/local/lib/libvosk* /usr/local/lib/

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/va-voice"]
