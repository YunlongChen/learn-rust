# Domain STUN Server Dockerfile
# Multi-stage build for optimized image size

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs for dependency caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release 2>/dev/null || true

# Remove dummy source
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build release binary
RUN cargo build --release --bin domain-stun

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r stun && useradd -r -g stun stun

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/domain-stun /app/domain-stun

# Create data directory
RUN mkdir -p /data && chown -R stun:stun /data

# Create templates directory and copy templates
RUN mkdir -p /app/templates && chown -R stun:stun /app/templates
COPY templates /app/templates

# Switch to non-root user
USER stun

# Expose ports
# STUN/TURN UDP: 3478
# HTTP API TCP: 3479
EXPOSE 3478/udp 3479/tcp

# Volume for persistent data
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3479/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV SERVER_PORT=3479
ENV STUN_PORT=3478
ENV TURN_PORT=3478

# Run the application
ENTRYPOINT ["/app/domain-stun"]
