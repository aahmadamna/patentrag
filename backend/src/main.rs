mod ingest;
mod chunker;
mod db;
mod embedder;
mod search;
mod rag;


use std::env;
use sqlx::PgPool;
use ingest::extract_text_from_pdf;
use chunker::chunk_text;
use db::save_chunk;
use embedder::run_embedding_job;
use search::{run_search, SearchRequest};
use rag::run_query;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    //connect to redis 
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let client = redis::Client::open(redis_url)?;
    let mut redis_conn = client.get_async_connection().await?;


    match args.next().as_deref() {
        Some("ingest") => {
            let pdf_path = args.next().expect("Missing PDF path");
            let patent_id = args.next().expect("Missing patent ID");

            println!("Ingesting PDF '{}' as patent ID '{}'", pdf_path, patent_id);

            // Connect to Postgres
            let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set");
            let pool = PgPool::connect(&database_url).await?;

            // 1. Extract all text
            let full_text = extract_text_from_pdf(&pdf_path)?;
            println!("Extracted {} characters of text", full_text.len());

            // 2. Chunk the text
            let chunks = chunk_text(&full_text, 800, 200);
            println!("Created {} chunks", chunks.len());

            // 3. Persist each chunk into the database
            for (idx, chunk_text) in chunks.iter().enumerate() {
                let chunk_id = format!("{}-{}", patent_id, idx);
                save_chunk(&pool, &patent_id, &chunk_id, chunk_text).await?;
            }
            println!("Persisted all {} chunks", chunks.len());
        }

        Some("embed") => {
            println!("Starting embedding job…");

            // Connect to Postgres
            let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set");
            let pool = PgPool::connect(&database_url).await?;

            // Run the embedding pipeline
            run_embedding_job(&pool).await?;
            println!("Embedding pass complete");
        }

        Some("search") => {
            let query = args.next().expect("Usage: patentrag search <query> [top_k]");
            let top_k = args.next()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(5);

            // Connect to Postgres
            let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set");
            let pool = PgPool::connect(&database_url).await?;

            // Run the search
            let results = run_search(&pool, SearchRequest { query, top_k }, &mut redis_conn).await?;
            for res in results {
                println!(
                    "{} | {} | {:.4}\n{}\n---",
                    res.patent_id, res.chunk_id, res.distance, res.snippet
                );
            }
        }

        Some("query") => {
            let question = args.next().expect("Usage: patentrag query <question> [top_k]");
            let top_k    = args.next().and_then(|s| s.parse().ok()).unwrap_or(5);
        
            let database_url = env::var("DATABASE_URL")?;
            let pool = PgPool::connect(&database_url).await?;
        
            println!("💬 Query: {}", question);
            let answer = run_query(&pool, &question, top_k, &mut redis_conn).await?;
            println!("\n💡 Answer:\n{}", answer);
        }

        _ => {
            eprintln!("Usage:");
            eprintln!("  patentrag ingest <pdf_path> <patent_id>");
            eprintln!("  patentrag embed");
            eprintln!("  patentrag search <query> [top_k]");
        }
    }

    Ok(())
}
