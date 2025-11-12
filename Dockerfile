# Dockerfile for bashrs - Bash/Makefile/Dockerfile Purification Tool
# Multi-stage build for minimal production image

# Build stage
FROM rust:1.83-alpine3.21 AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && rm -rf /var/cache/apk/*

# Create app directory
WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY rash/Cargo.toml ./rash/
COPY rash-runtime/Cargo.toml ./rash-runtime/
COPY rash-mcp/Cargo.toml ./rash-mcp/

# Copy source code
COPY rash/src ./rash/src
COPY rash-runtime/src ./rash-runtime/src
COPY rash-mcp/src ./rash-mcp/src

# Copy benchmarks (referenced in Cargo.toml)
COPY rash/benches ./rash/benches

# Build release binary with static linking
RUN cargo build --release --bin bashrs --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1000 bashrs && \
    adduser -D -u 1000 -G bashrs bashrs

# Copy binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/bashrs /usr/local/bin/bashrs

# Set ownership
RUN chown bashrs:bashrs /usr/local/bin/bashrs

# Switch to non-root user
USER bashrs

# Set working directory
WORKDIR /workspace

# Verify installation
RUN bashrs --version

# Default command
ENTRYPOINT ["bashrs"]
CMD ["--help"]