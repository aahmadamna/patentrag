# --- Build Stage ---
    FROM rust:slim as builder

    # Accept build-time DATABASE_URL
    ARG DATABASE_URL
    ENV DATABASE_URL=${DATABASE_URL}
    
    # Install build dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev build-essential ca-certificates curl \
        && rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    # Copy Cargo files first for dependency caching
    COPY Cargo.toml Cargo.lock ./
    
    # Create a dummy main.rs and build dependencies only
    RUN mkdir src && echo "fn main() {}" > src/main.rs
    RUN cargo build --release
    RUN rm src/main.rs
    
    # Now copy your actual source code
    COPY src ./src
    
    # Build the real application
    RUN cargo build --release
    
    # --- Runtime Stage ---
    FROM debian:bookworm-slim
    
    WORKDIR /app
    
    # Copy binary and certs
    COPY --from=builder /app/target/release/backend .
    COPY --from=builder /etc/ssl/certs /etc/ssl/certs
    
    CMD ["./backend"]