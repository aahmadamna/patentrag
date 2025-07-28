# --- Build Stage ---
    FROM rust:slim as builder

    # Accept build-time DATABASE_URL
    ARG DATABASE_URL
    ENV DATABASE_URL=${DATABASE_URL}
    
    # Optional: for offline builds (if you ever use sqlx prepare)
    # ENV SQLX_OFFLINE=true
    
    # Install build dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev build-essential ca-certificates curl \
        && rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    # Caching dependencies
    COPY Cargo.toml Cargo.lock ./
    RUN mkdir -p src && echo "fn main() {}" > src/main.rs
    RUN cargo build --release || true
    
    # Copy full source and build again
    COPY . .
    RUN cargo build --release
    
    # --- Runtime Stage ---
    FROM debian:bookworm-slim
    
    WORKDIR /app
    
    # Copy binary and certs
    COPY --from=builder /app/target/release/backend .
    COPY --from=builder /etc/ssl/certs /etc/ssl/certs
    
    ENV PORT=8000
    EXPOSE 8000
    
    CMD ["./backend"]
    