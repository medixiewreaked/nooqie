FROM rust:1.77.0-slim as build
RUN rustup target add x86_64-unknown-linux-musl && apt update && apt install -y musl-tools musl-dev libssl-dev pkg-config && update-ca-certificates
COPY ./nooqie/src ./src
COPY ./nooqie/Cargo.lock .
COPY ./nooqie/Cargo.toml .
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "nooqie"

RUN cargo build --target x86_64-unknown-linux-musl --release
