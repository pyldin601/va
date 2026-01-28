FROM rust:1.93-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN cargo build --release --locked -p va-command

FROM debian:bookworm-slim AS runner

WORKDIR /

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV BIND_ADDR=0.0.0.0:8092

COPY --from=builder /app/target/release/va-command /va-command

CMD ["/va-command"]
