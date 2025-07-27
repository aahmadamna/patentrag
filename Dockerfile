# Use a minimal Rust image
FROM rust:slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests and cache deps
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cd backend && cargo build --release || true

# Copy full source and build
COPY backend ./backend
RUN cd backend && cargo build --release

# Final stage: runtime
FROM debian:bookworm-slim

# Install runtime deps
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/backend/target/release/backend /app/backend
RUN chmod +x /app/backend

EXPOSE 8000

# Run app
CMD ["./backend"]
