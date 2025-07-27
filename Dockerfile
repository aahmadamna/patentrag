# Use the latest stable Rust image
FROM rust:slim

# Install system dependencies required for building OpenSSL and other crates
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy only Cargo manifest files to cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./backend/

# Create dummy main.rs to build and cache dependencies
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cd backend && cargo build --release || true

# Copy actual source code
COPY backend ./backend

# Build the full app
RUN cd backend && cargo build --release

# ✅ Make sure the binary is executable
RUN chmod +x backend/target/release/backend

# Expose the port your app listens on (update if not 8000)
EXPOSE 8000

# ✅ Run the binary
RUN chmod +x backend/target/release/patentrag
CMD ["./backend/target/release/patentrag"]
