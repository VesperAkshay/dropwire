# Build stage
FROM rust:1-slim-bookworm AS builder

WORKDIR /usr/src/app
COPY . .

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install ca-certificates for any secure external connections if needed
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/dropwire /usr/local/bin/dropwire

# Expose TCP Stapling (9009) and WebSocket Signaling (9010)
EXPOSE 9009 9010

# Start DropWire in relay mode
ENTRYPOINT ["dropwire", "relay", "--bind", "0.0.0.0:9009", "--ws-bind", "0.0.0.0:9010"]
