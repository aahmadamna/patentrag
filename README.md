# PatentRAG

A Retrieval-Augmented Patent Q&A & Summarization Engine with Cache-Augmented Generation

PatentRAG+CAG is an AI-powered patent intelligence assistant that transforms days of manual prior-art hunting, infringement analysis, and freedom-to-operate (FTO) scanning into a few clicks.

## Features

- **RAG (Retrieval-Augmented Generation)**: Natural language questions about patents
- **CAG (Cache-Augmented Generation)**: Cached results for faster responses
- **Executive Summaries**: One-click patent overviews
- **PDF Upload & Processing**: Upload and analyze patent PDFs
- **Chat History**: Save and manage conversations

## Local Development Setup

### Prerequisites

- Rust (for backend)
- Node.js (for frontend)
- PostgreSQL
- Redis
- OpenAI API key

### Backend Setup

1. Navigate to the backend directory:
   ```bash
   cd backend
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Set environment variables:
   ```bash
   export DATABASE_URL="postgresql://username:password@localhost:5432/patentrag"
   export REDIS_URL="redis://localhost:6379"
   export OPENAI_API_KEY="your_openai_api_key"
   ```

4. Run the backend:
   ```bash
   cargo run
   ```

### Frontend Setup

1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Set environment variables:
   ```bash
   export NEXT_PUBLIC_API_URL="http://localhost:8000"
   ```

4. Run the frontend:
   ```bash
   npm run dev
   ```

### Database Setup

1. Create a PostgreSQL database named `patentrag`
2. Run the SQL files in the `data/` directory to set up the schema
3. Start Redis server

## Usage

1. Start both backend and frontend servers
2. Open http://localhost:3000 in your browser
3. Upload a patent PDF
4. Ask questions about the patent
5. View chat history and summaries
