# Multi-stage build for Reverse Shell System
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY reverse-shell-server/Cargo.toml ./reverse-shell-server/
COPY reverse-shell-client/Cargo.toml ./reverse-shell-client/
COPY reverse-shell-cli/Cargo.toml ./reverse-shell-cli/

# Copy source code
COPY reverse-shell-server/src ./reverse-shell-server/src
COPY reverse-shell-client/src ./reverse-shell-client/src
COPY reverse-shell-cli/src ./reverse-shell-cli/src

# Build the applications
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Create app directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/reverse-shell-server /usr/local/bin/
COPY --from=builder /app/target/release/reverse-shell-client /usr/local/bin/
COPY --from=builder /app/target/release/reverse-shell-cli /usr/local/bin/

# Copy additional files
COPY README.md LICENSE ./

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose ports
EXPOSE 8080 8081

# Default command (server)
CMD ["reverse-shell-server", "--client-port", "8080", "--admin-port", "8081", "--log-level", "info"]

# Labels
LABEL org.opencontainers.image.title="Reverse Shell System"
LABEL org.opencontainers.image.description="Secure WebSocket-based reverse shell with authentication"
LABEL org.opencontainers.image.source="https://github.com/chaudhryfaisal/open-hands-poc"
LABEL org.opencontainers.image.licenses="MIT"