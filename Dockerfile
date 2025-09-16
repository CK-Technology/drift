# Multi-stage build for optimized production image
FROM rust:1.75-bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /var/lib/drift drift

# Copy the binary from builder stage
COPY --from=builder /app/target/release/drift /usr/local/bin/drift

# Create necessary directories
RUN mkdir -p /var/lib/drift && \
    chown -R drift:drift /var/lib/drift

# Copy configuration files
COPY drift.toml /app/drift.toml

# Set user
USER drift

# Expose ports
EXPOSE 5000 5001

# Health check
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

# Set working directory
WORKDIR /var/lib/drift

# Start the application
CMD ["/usr/local/bin/drift", "--config", "/app/drift.toml"]