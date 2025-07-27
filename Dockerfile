# Use a minimal Rust image
FROM rust:slim

# --- 🔧 Allow DATABASE_URL to be passed at build time ---
ARG DATABASE_URL
ENV DATABASE_URL=postgresql://postgres:PgKXMHuLveRLYMhrTrYtoTVnLZfemARG@postgres.railway.internal:5432/railway


# --- 📦 Install system dependencies (for openssl etc) ---
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# --- 📁 Set working directory ---
WORKDIR /app

# --- 🧠 Cache dependencies first ---
COPY backend/Cargo.toml backend/Cargo.lock ./backend/

# Create dummy main.rs to cache dependencies
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cd backend && cargo build --release || true

# --- 💻 Copy full source and rebuild ---
COPY backend ./backend
RUN cd backend && cargo build --release
RUN ls -lh backend/target/release

# ✅ Make sure binary is executable (and named correctly)
RUN chmod +x backend/target/release/backend

# 🌐 Expose the port your app listens on
EXPOSE 8000

# ✅ Temporarily output contents of release folder and don't run app yet
CMD ["sh", "-c", "ls -lh backend/target/release && echo 'Done listing' && sleep 3600"]
