FROM rust:1-bookworm

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests

RUN cargo fetch

CMD ["cargo", "run"]
