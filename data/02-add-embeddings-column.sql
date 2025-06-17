-- data/02-add-embeddings-column.sql

-- Add a 1536-dimensional vector column for embeddings
ALTER TABLE chunks
ADD COLUMN IF NOT EXISTS embedding vector(1536);
