# ZQLite Container for Drift Registry
FROM alpine:latest

# Install dependencies
RUN apk add --no-cache \
    wget \
    curl \
    ca-certificates

# Install Zig
RUN wget https://ziglang.org/download/0.15.0/zig-linux-x86_64-0.15.0.tar.xz \
    && tar -xf zig-linux-x86_64-0.15.0.tar.xz \
    && mv zig-linux-x86_64-0.15.0 /opt/zig \
    && ln -s /opt/zig/zig /usr/local/bin/zig \
    && rm zig-linux-x86_64-0.15.0.tar.xz

# Create zqlite user
RUN adduser -D -s /bin/sh zqlite

# Set up working directory
WORKDIR /app

# Copy zqlite source (will be mounted from archive)
COPY . .

# Build zqlite
RUN zig build -Doptimize=ReleaseFast

# Create data directory
RUN mkdir -p /var/lib/zqlite && chown zqlite:zqlite /var/lib/zqlite

# Switch to zqlite user
USER zqlite

# Expose port
EXPOSE 5433

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ./zig-out/bin/zqlite ping || exit 1

# Default command
CMD ["./zig-out/bin/zqlite", "server", "--port", "5433", "--data-dir", "/var/lib/zqlite"]