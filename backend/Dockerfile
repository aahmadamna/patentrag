# ---- Build Stage ----
    FROM rust:1.73-slim AS builder

    # Install build dependencies
    RUN apt-get update && \
        apt-get install -y pkg-config libssl-dev build-essential ca-certificates curl && \
        rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    # Copy manifests and build deps first (for caching)
    COPY Cargo.toml Cargo.lock ./
    RUN mkdir -p src && echo "fn main() {}" > src/main.rs
    RUN cargo build --release || true
    
    # Copy actual source and rebuild
    COPY . .
    RUN cargo build --release
    
    # ---- Runtime Stage ----
    FROM debian:bookworm-slim
    
    RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    # Copy release binary
    COPY --from=builder /app/target/release/backend .
    
    # Railway listens on $PORT
    ENV PORT=8000
    EXPOSE 8000
    
    CMD ["./backend"]
    