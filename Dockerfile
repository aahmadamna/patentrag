# Use latest Rust image
FROM rust:slim

# Install system dependencies (OpenSSL and pkg-config)
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY backend/Cargo.toml backend/Cargo.lock ./backend/

# Create dummy main.rs to warm up build cache
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cd backend && cargo build --release || true

# Copy full source
COPY backend ./backend

# Build real application
RUN cd backend && cargo build --release

# Expose the port your app uses
EXPOSE 8000

# Run the binary (change if needed)
CMD ["./backend/target/release"]
