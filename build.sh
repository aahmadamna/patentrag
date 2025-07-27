#!/bin/bash

echo "🚀 Building PatentRAG backend..."

# Change to backend directory
cd backend

# Install dependencies and build
echo "📦 Installing dependencies..."
cargo build --release

echo "✅ Build complete!" 