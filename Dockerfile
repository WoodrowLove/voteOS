# VoteOS — Multi-stage Docker build
# Produces a minimal runtime image with the VoteOS binary.
#
# Build:  docker build -t voteos .
# Run:    docker run -p 3100:3100 -v voteos-data:/data voteos

# --- Build stage ---
FROM rust:1.77-slim AS builder

WORKDIR /build

# Copy dependency manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Copy the AxiaSystem Rust Bridge (required dependency)
# In production, this would be a registry dependency instead of a local path.
# For now, we copy it alongside the build context.
COPY ../AxiaSystem-Rust-Bridge /build/AxiaSystem-Rust-Bridge

# Copy source
COPY src/ src/
COPY config/ config/
COPY tests/ tests/

# Build release binary
RUN cargo build --release --bin voteos

# --- Runtime stage ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash voteos

WORKDIR /app

# Copy binary
COPY --from=builder /build/target/release/voteos /app/voteos

# Copy default config
COPY config/voteos.toml /app/config/voteos.toml

# Create data directory
RUN mkdir -p /data && chown voteos:voteos /data

# Switch to non-root
USER voteos

# Expose API port
EXPOSE 3100

# Persistence volume
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3100/health || exit 1

# Default command
CMD ["/app/voteos", "/app/config/voteos.toml"]
