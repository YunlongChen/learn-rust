# Domain Agent Dockerfile
# Multi-stage build for optimized image size

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
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
RUN cargo build --release --bin domain_agent

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r agent && useradd -r -g agent agent

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/domain_agent /app/domain_agent

# Copy entrypoint script
COPY <<'EOF' /app/entrypoint.sh
#!/bin/bash
set -e

# If config file mounted at /app/agent.toml, use it
if [ -f "/app/agent.toml" ]; then
    exec /app/domain_agent --config /app/agent.toml "$@"
else
    exec /app/domain_agent "$@"
fi
EOF
RUN chmod +x /app/entrypoint.sh

# Create config directory
RUN mkdir -p /app && chown -R agent:agent /app

# Switch to non-root user
USER agent

# Health check - verify agent starts and connects
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -x domain_agent || exit 1

# Run the application
ENTRYPOINT ["/app/entrypoint.sh"]
