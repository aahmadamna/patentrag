#!/bin/bash

# Check if required environment variables are set
if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL not set. Starting in demo mode..."
    export DATABASE_URL="postgresql://demo:demo@localhost:5432/demo"
fi

if [ -z "$REDIS_URL" ]; then
    echo "⚠️  REDIS_URL not set. Using default Redis..."
    export REDIS_URL="redis://127.0.0.1:6379"
fi

if [ -z "$OPENAI_API_KEY" ]; then
    echo "⚠️  OPENAI_API_KEY not set. Some features may not work..."
fi

echo "🚀 Starting PatentRAG backend..."
echo "📊 Database: $DATABASE_URL"
echo "🔴 Redis: $REDIS_URL"

# Start the application
exec cargo run --release 