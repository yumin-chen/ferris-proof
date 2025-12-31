# Multi-stage build for optimized Docker image
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

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

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false ferrisproof

# Copy binary from builder stage
COPY --from=builder /app/target/release/ferris-proof /usr/local/bin/ferris-proof

# Set permissions
RUN chmod +x /usr/local/bin/ferris-proof

# Switch to non-root user
USER ferrisproof

# Set working directory
WORKDIR /workspace

# Default command
ENTRYPOINT ["ferris-proof"]
CMD ["--help"]