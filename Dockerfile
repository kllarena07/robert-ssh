FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY normal.png ./normal.png
COPY scared.png ./scared.png
RUN cargo build --release

FROM ubuntu:24.04
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/blockmove /app/blockmove
COPY --from=builder /app/normal.png /app/normal.png
COPY --from=builder /app/scared.png /app/scared.png
ENV SECRETS_LOCATION=/run/secret/authorized_keys/id_ed25519
EXPOSE 22
CMD ["./blockmove"]
