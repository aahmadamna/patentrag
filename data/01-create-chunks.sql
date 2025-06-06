CREATE TABLE IF NOT EXISTS chunks (
  patent_id TEXT NOT NULL,
  chunk_id  TEXT PRIMARY KEY,
  text      TEXT NOT NULL
);
