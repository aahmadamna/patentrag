// backend/src/main.rs

mod ingest;
mod chunker;
mod db;
mod embedder;

use std::env;
use ingest::extract_text_from_pdf;
use chunker::chunk_text;
use db::save_chunk;
use embedder::run_embedding_job;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("ingest") => {
            let pdf_path = args.next().expect("Missing PDF path");
            let patent_id = args.next().expect("Missing patent ID");

            println!("Ingesting PDF '{}' as patent ID '{}'", pdf_path, patent_id);

            // Connect to Postgres
            let database_url = env::var("DATABASE_URL")?;
            let pool = PgPool::connect(&database_url).await?;

            // 1. Extract all text
            let full_text = extract_text_from_pdf(&pdf_path)?;
            println!("✅ Extracted {} characters of text", full_text.len());

            // 2. Chunk the text
            let chunks = chunk_text(&full_text, 800, 200);
            println!("✅ Created {} chunks", chunks.len());

            // 3. Persist each chunk into the database
            for (idx, chunk_text) in chunks.iter().enumerate() {
                let chunk_id = format!("{}-{}", patent_id, idx);
                save_chunk(&pool, &patent_id, &chunk_id, chunk_text).await?;
            }
            println!("✅ Persisted all {} chunks", chunks.len());
        }

        Some("embed") => {
            println!("Starting embedding job…");

            // Connect to Postgres
            let database_url = env::var("DATABASE_URL")?;
            let pool = PgPool::connect(&database_url).await?;

            // Run the embedding pipeline
            run_embedding_job(&pool).await?;
            println!("✅ Embedding pass complete");
        }

        _ => {
            eprintln!("Usage:");
            eprintln!("  patentrag ingest <pdf_path> <patent_id>");
            eprintln!("  patentrag embed");
        }
    }

    Ok(())
}
