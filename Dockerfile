# --- Stage 1: Build the Rust Binary ---
FROM rust:1.82-slim AS builder
WORKDIR /app

# Copy manifest files first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code and build the real binary
COPY src ./src
RUN cargo build --release

# --- Stage 2: Runtime Environment ---
FROM debian:bookworm-slim
WORKDIR /app

# Install system dependencies (OpenSSL & certs for Solana RPC HTTPS/WSS calls)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder stage
COPY --from=builder /app/target/release/blackgold-server /app/blackgold-server

# Expose port (Railway dynamically injects $PORT, but 3000 is our default)
EXPOSE 3000

# Run the server binary
CMD ["./blackgold-server"]