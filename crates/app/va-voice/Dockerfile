FROM rust:1.93-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

ARG VOSK_VERSION=0.3.45
ENV VOSK_VERSION=$VOSK_VERSION

RUN wget https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-$(arch)-$VOSK_VERSION.zip \
    && unzip vosk-linux-$(arch)-$VOSK_VERSION.zip \
    && rm vosk-linux-$(arch)-$VOSK_VERSION.zip \
    && cp vosk-linux-*-$VOSK_VERSION/libvosk.so /usr/local/lib/

COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN cargo build --release --locked -p va-voice

FROM rust:1.93-bookworm AS runner

WORKDIR /

RUN apt-get update && apt-get install -y --no-install-recommends \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

ARG VOSK_MODEL=vosk-model-small-en-us-0.15
ENV VOSK_MODEL=$VOSK_MODEL

RUN wget https://alphacephei.com/vosk/models/$VOSK_MODEL.zip \
    && unzip $VOSK_MODEL.zip \
    && rm $VOSK_MODEL.zip

ENV VOSK_MODEL_PATH=/$VOSK_MODEL

COPY --from=builder /usr/local/lib/libvosk.so /usr/local/lib/libvosk.so
COPY --from=builder /app/target/release/va-voice /va-voice

RUN ldconfig

CMD ["/va-voice"]
