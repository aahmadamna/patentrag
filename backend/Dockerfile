# --- Build Stage ---
    FROM rust:slim as builder

    # Install build dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev build-essential ca-certificates curl \
        && rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    # Caching dependencies
    COPY Cargo.toml Cargo.lock ./
    RUN mkdir -p src && echo "fn main() {}" > src/main.rs
    RUN cargo build --release || true
    
    # Copy full source and build
    COPY . .
    RUN cargo build --release
    
    # --- Runtime Stage (NO APT!) ---
    FROM debian:bookworm-slim
    
    # Set working dir
    WORKDIR /app
    
    # Copy release binary
    COPY --from=builder /app/target/release/backend .
    
    # ✅ Copy system certs from builder (avoid apt-get entirely)
    COPY --from=builder /etc/ssl/certs /etc/ssl/certs
    
    ENV PORT=8000
    EXPOSE 8000
    
    CMD ["./backend"]
    