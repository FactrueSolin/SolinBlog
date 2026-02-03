# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY front ./front

RUN cargo build --release --bin SolinBlog

FROM debian:bookworm-slim AS runtime

ENV WEB_HOST=0.0.0.0
ENV WEB_PORT=3002

WORKDIR /app

RUN useradd --create-home --uid 10001 --shell /usr/sbin/nologin appuser \
    && mkdir -p /app/data /app/front \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/SolinBlog /app/solinblog
COPY --from=builder /app/front /app/front

USER appuser

EXPOSE 3002

ENTRYPOINT ["/app/solinblog"]
