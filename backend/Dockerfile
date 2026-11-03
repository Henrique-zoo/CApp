# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --locked --bin backend \
    && cp target/release/backend /usr/local/bin/backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/backend /usr/local/bin/backend

USER 10001:10001
EXPOSE 3000

CMD ["/usr/local/bin/backend"]
