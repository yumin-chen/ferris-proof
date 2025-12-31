# Multi-stage build for optimized container image (Podman-optimized)
FROM docker.io/library/rust:1.75-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY ferris-proof-cli/Cargo.toml ./ferris-proof-cli/
COPY ferris-proof-core/Cargo.toml ./ferris-proof-core/
COPY ferris-proof-config/Cargo.toml ./ferris-proof-config/
COPY ferris-proof-plugins/Cargo.toml ./ferris-proof-plugins/

# Create dummy source files to cache dependencies
RUN mkdir -p ferris-proof-cli/src ferris-proof-core/src ferris-proof-config/src ferris-proof-plugins/src && \
    echo "fn main() {}" > ferris-proof-cli/src/main.rs && \
    echo "// dummy" > ferris-proof-cli/src/lib.rs && \
    echo "// dummy" > ferris-proof-core/src/lib.rs && \
    echo "// dummy" > ferris-proof-config/src/lib.rs && \
    echo "// dummy" > ferris-proof-plugins/src/lib.rs

# Build dependencies
RUN cargo build --release && rm -rf src target/release/deps/ferris_proof*

# Copy source code
COPY . .

# Build application
RUN cargo build --release --bin ferris-proof

# Runtime stage - Debian slim with security optimizations
FROM docker.io/library/debian:bookworm-slim

# Install runtime dependencies and security updates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && apt-get upgrade -y \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user with specific UID/GID for Podman compatibility
RUN groupadd -g 1001 ferrisproof && \
    useradd -r -u 1001 -g ferrisproof -s /bin/false -d /workspace ferrisproof

# Copy binary from builder stage
COPY --from=builder /app/target/release/ferris-proof /usr/local/bin/ferris-proof

# Create workspace directory with proper ownership
RUN mkdir -p /workspace && \
    chown ferrisproof:ferrisproof /workspace && \
    chmod +x /usr/local/bin/ferris-proof

# Health check for container monitoring
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ferris-proof --version || exit 1

# OCI labels for better container management
LABEL org.opencontainers.image.title="FerrisProof" \
      org.opencontainers.image.description="Multi-layer correctness pipeline for Rust applications" \
      org.opencontainers.image.vendor="FerrisProof Contributors" \
      org.opencontainers.image.licenses="CC0-1.0" \
      org.opencontainers.image.source="https://github.com/ferris-proof/ferris-proof" \
      org.opencontainers.image.documentation="https://github.com/ferris-proof/ferris-proof/blob/main/README.md"

# Switch to non-root user
USER ferrisproof

# Set working directory
WORKDIR /workspace

# Default command
ENTRYPOINT ["ferris-proof"]
CMD ["--help"]