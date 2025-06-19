// backend/src/search.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::env;

/// Structure of the OpenAI embedding response
#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

/// CLI request parameters for a search
pub struct SearchRequest {
    pub query: String,
    pub top_k: i64,
}

/// One search result entry
#[derive(Serialize)]
pub struct SearchResult {
    pub patent_id: String,
    pub chunk_id: String,
    pub snippet: String,
    pub distance: f64,
}

/// Call OpenAI to embed the user query text
async fn embed_query(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let body = serde_json::json!({
        "input": text,
        "model": "text-embedding-ada-002"
    });

    let resp = client
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .json::<EmbeddingResponse>()
        .await?;

    Ok(resp.data
        .into_iter()
        .next()
        .expect("Expected at least one embedding")
        .embedding)
}

/// Run a nearest-neighbor search in Postgres using pgvector
pub async fn run_search(
    pool: &PgPool,
    req: SearchRequest,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // 1) Embed the query
    let q_emb = embed_query(&req.query).await?;

    // 2) Execute a vector distance search, binding the Vec<f32> directly
    let rows = sqlx::query(
        r#"
        SELECT patent_id, chunk_id, text AS snippet,
               embedding <-> ($1::vector) AS distance
        FROM chunks
        ORDER BY embedding <-> ($1::vector)
        LIMIT $2
        "#,
    )
    .bind(&q_emb)         // bind the query embedding
    .bind(req.top_k)      // bind the limit
    .fetch_all(pool)
    .await?;

    // 3) Map each row into our SearchResult struct
    let results = rows
        .into_iter()
        .map(|row| SearchResult {
            patent_id: row.get("patent_id"),
            chunk_id: row.get("chunk_id"),
            snippet: row.get("snippet"),
            distance: row.get("distance"),
        })
        .collect();

    Ok(results)
}
