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

FROM alpine:latest AS actual
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build --chown=nooqie:nooqie ./target/x86_64-unknown-linux-musl/release/nooqie /app/nooqie
RUN apk add yt-dlp
USER nooqie:nooqie
ENV DISCORD_TOKEN YOURTOKENHERE
ENV OLLAMA_POST_URL "http://0.0.0.0/api/generate"
ENV OLLAMA_MODEL "llama2-uncensored"
ENV RUST_LOG none,nooqie=debug
ENTRYPOINT ["./app/nooqie"]
