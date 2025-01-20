FROM rust:1.84.0 as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install dependencies required for running the binary
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy over the compiled binary
COPY --from=builder /usr/src/app/target/release/friendship_service /usr/local/bin/friendship_service

EXPOSE 8080

CMD ["friendship_service"]
