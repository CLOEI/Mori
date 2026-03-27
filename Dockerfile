# Web build stage
FROM oven/bun:1 AS web-builder

WORKDIR /app

COPY web/package.json web/bun.lock* ./web/
RUN cd web && bun install --frozen-lockfile

COPY web ./web
RUN cd web && bun run build:release

# Rust build stage
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build the actual project
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/Mori ./Mori
COPY --from=web-builder /app/dist ./dist
COPY items.dat ./items.dat

EXPOSE 3000

CMD ["./Mori"]
