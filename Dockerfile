FROM rust:1.75 as builder

WORKDIR /app

# Copy Cargo files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create dummy main.rs for dependency building
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove dummy and copy real source
RUN rm src/main.rs
COPY backend/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/backend /app/backend

# Expose port
EXPOSE 8000

# Start the application
CMD ["./backend"] 