# Use official Rust image
FROM rust:1.72-slim

# Set working directory inside container
WORKDIR /app

# Copy just the Cargo manifest files first
COPY backend/Cargo.toml backend/Cargo.lock ./backend/

# Dummy main.rs to cache dependencies
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cd backend && cargo build --release || true

# Copy actual source code
COPY backend ./backend

# Build the real app
RUN cd backend && cargo build --release

# Expose port (change if needed)
EXPOSE 8000

# Run the compiled binary
CMD ["./backend/target/release/patentrag"]
