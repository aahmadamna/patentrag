// backend/src/search.rs

use std::error::Error;
use std::env;

use redis::aio::Connection;
use redis::AsyncCommands;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hex;

use sqlx::{PgPool, Row};

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

/// Embed the user query text, using Redis to cache embeddings
async fn embed_query(
    text: &str,
    redis_conn: &mut Connection,
) -> Result<Vec<f32>, Box<dyn Error>> {
    // 1) Compute cache key
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hex::encode(hasher.finalize());
    let cache_key = format!("embed:{}", hash);

    // 2) Try to fetch from Redis
    if let Ok(cached_json) = redis_conn.get::<_, String>(cache_key.clone()).await {
        let vec: Vec<f32> = serde_json::from_str(&cached_json)?;
        return Ok(vec);
    }

    // 3) Call OpenAI embeddings API
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
    let embedding = resp.data
        .into_iter()
        .next()
        .ok_or("No embedding returned")?
        .embedding;

    // 4) Cache the result for 24h
    let serialized = serde_json::to_string(&embedding)?;
    let ttl_seconds = 86_400;
    redis_conn
    .set_ex::<String, String, ()>(cache_key, serialized, ttl_seconds)
    .await?;


    // 5) Return the embedding
    Ok(embedding)
}

/// Run a nearest-neighbor search in Postgres using pgvector
pub async fn run_search(
    pool: &PgPool,
    req: SearchRequest,
    redis_conn: &mut Connection,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    // 1) Embed the query (with caching)
    let q_emb = embed_query(&req.query, redis_conn).await?;

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
    .bind(&q_emb)
    .bind(req.top_k)
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
