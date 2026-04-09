# Web build stage
FROM --platform=$BUILDPLATFORM oven/bun:1 AS web-builder

WORKDIR /app

COPY web/package.json web/bun.lock* ./web/
RUN cd web && bun install --frozen-lockfile

COPY web ./web
RUN cd web && bun run build:release

# Rust build stage
FROM --platform=$BUILDPLATFORM rust:1.88-slim-bookworm AS builder

# Copy xx scripts to help with cross-compilation
COPY --from=tonistiigi/xx / /

RUN apt-get update && apt-get install -y \
    clang \
    lld \
    pkg-config \
    git \
    cmake \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETPLATFORM

# Install cross-compilation dependencies for the target
RUN xx-apt-get update && xx-apt-get install -y \
    binutils \
    gcc \
    g++ \
    libc6-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Use xx-cargo for cross-compilation
RUN xx-cargo build --release
RUN rm -rf src

# Build the actual project
COPY src ./src
RUN touch src/main.rs && \
    xx-cargo build --release && \
    cp target/$(xx-cargo --print-target-triple)/release/Mori /app/Mori && \
    xx-verify /app/Mori

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/Mori ./Mori
COPY --from=web-builder /app/dist ./dist
COPY items.dat ./items.dat

EXPOSE 3000

CMD ["./Mori"]
