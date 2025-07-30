FROM rust:1.75-alpine AS builder

# Install system dependencies for building
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    git \
    build-base

# Set working directory
WORKDIR /workspace

# Copy source code
COPY . .

# Build Ellex transpiler in release mode
WORKDIR /workspace/crates
RUN cargo build --release --bin ellex_cli

# Create minimal runtime image
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    bash \
    curl

# Copy binary from builder
COPY --from=builder /workspace/crates/target/release/ellex_cli /usr/local/bin/ellex

# Create benchmark wrapper
RUN cat > /usr/local/bin/ellex-benchmark << 'EOF'
#!/bin/bash
exec /usr/local/bin/ellex "$@"
EOF

RUN chmod +x /usr/local/bin/ellex-benchmark

# Create results directory
RUN mkdir -p /results/ellex

# Add benchmark user
RUN adduser -D -s /bin/bash benchmark
USER benchmark

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/ellex --version || exit 1

WORKDIR /workspace

CMD ["/usr/local/bin/ellex", "--version"]