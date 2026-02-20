# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY front ./front
COPY public ./public

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    cargo build --release --bin server

FROM debian:bookworm-slim AS runtime

ENV WEB_HOST=0.0.0.0
ENV WEB_PORT=3002

WORKDIR /app

RUN useradd --create-home --uid 10001 --shell /usr/sbin/nologin appuser \
    && mkdir -p /app/data /app/front /app/public \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/server /app/solinblog
COPY --from=builder /app/front /app/front
COPY --from=builder /app/public /app/public

USER appuser

EXPOSE 3002

ENTRYPOINT ["/app/solinblog"]
