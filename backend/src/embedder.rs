use sqlx::PgPool;
use crate::db::{fetch_unembedded_chunks, save_embedding};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    input: &'a str,
    model: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

pub async fn run_embedding_job(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load API key
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");

    let client = Client::new();
    let chunks = fetch_unembedded_chunks(pool).await?;
    println!("Embedding {} chunks...", chunks.len());

    for (idx, (chunk_id, _patent_id, text)) in chunks.iter().enumerate() {
        // 2. Call OpenAI
        let req_body = EmbeddingRequest {
            input: text,
            model: "text-embedding-ada-002",
        };
        let resp = client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&api_key)
            .json(&req_body)
            .send()
            .await?
            .json::<EmbeddingResponse>()
            .await?;

        let embedding = &resp.data[0].embedding;
        // 3. Save back to DB
        save_embedding(pool, chunk_id, embedding).await?;
        println!("  ({}/{}) chunk {} embedded", idx+1, chunks.len(), chunk_id);
    }

    println!("✅ All chunks embedded.");
    Ok(())
}
