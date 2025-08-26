# Use the official Rust image as build environment
FROM rust:1.75-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members
COPY vy-core/ ./vy-core/
COPY vy-web/ ./vy-web/
COPY vy-cli/ ./vy-cli/
COPY vy/ ./vy/

# Build the release binary
RUN cargo build --release --bin vy-web

# Runtime stage - use minimal debian image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create a non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser

# Set the working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/vy-web /app/vy-web

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the port (Cloud Run will set PORT environment variable)
EXPOSE 8080

# Set default environment variables for Cloud Run
ENV HOST=0.0.0.0
ENV PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/health || exit 1

# Run the binary
CMD ["./vy-web"]
