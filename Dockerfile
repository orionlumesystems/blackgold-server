# --- Stage 1: Build the Rust Binary ---
FROM rust:1.82-slim AS builder
WORKDIR /app

# Install required system packages for compiling Solana SDK and TLS dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    clang \
    && rm -rf /var/lib/apt/lists/*

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

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder stage
COPY --from=builder /app/target/release/blackgold-server /app/blackgold-server

EXPOSE 3000

CMD ["./blackgold-server"]