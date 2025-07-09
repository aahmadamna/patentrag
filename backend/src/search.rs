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
    pub patent_id: Option<String>,
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

/// Run a keyword-based search in Postgres as a fallback
async fn run_keyword_search(
    pool: &PgPool,
    req: &SearchRequest,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    println!("🔍 Falling back to keyword search for: '{}'", req.query);
    
    let rows = if let Some(ref patent_id) = req.patent_id {
        sqlx::query(
            r#"
            SELECT patent_id, chunk_id, text AS snippet, 0.0 AS distance
            FROM chunks
            WHERE patent_id = $1 AND text ILIKE $2
            ORDER BY chunk_id
            LIMIT $3
            "#,
        )
        .bind(patent_id)
        .bind(&format!("%{}%", req.query))
        .bind(req.top_k)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT patent_id, chunk_id, text AS snippet, 0.0 AS distance
            FROM chunks
            WHERE text ILIKE $1
            ORDER BY chunk_id
            LIMIT $2
            "#,
        )
        .bind(&format!("%{}%", req.query))
        .bind(req.top_k)
        .fetch_all(pool)
        .await?
    };

    println!("📊 Found {} rows from keyword search", rows.len());

    let results: Vec<SearchResult> = rows
        .into_iter()
        .map(|row| SearchResult {
            patent_id: row.get("patent_id"),
            chunk_id: row.get("chunk_id"),
            snippet: row.get("snippet"),
            distance: row.get("distance"),
        })
        .collect();

    println!("✅ Returning {} keyword search results", results.len());
    Ok(results)
}

/// Run a nearest-neighbor search in Postgres using pgvector
pub async fn run_search(
    pool: &PgPool,
    req: SearchRequest,
    redis_conn: &mut Connection,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    println!("🔍 Starting search for query: '{}'", req.query);
    println!("🔍 Patent ID: {:?}, Top K: {}", req.patent_id, req.top_k);
    
    // 1) Embed the query (with caching)
    let q_emb = embed_query(&req.query, redis_conn).await?;
    println!("✅ Query embedded successfully, embedding length: {}", q_emb.len());

    // 2) Execute a vector distance search, binding the Vec<f32> directly
    let rows = if let Some(ref patent_id) = req.patent_id {
        println!("🔍 Searching for patent_id: {}", patent_id);
        sqlx::query(
            r#"
            SELECT patent_id, chunk_id, text AS snippet,
                   embedding <-> ($1::vector) AS distance
            FROM chunks
            WHERE patent_id = $2 AND embedding IS NOT NULL
            ORDER BY embedding <-> ($1::vector)
            LIMIT $3
            "#,
        )
        .bind(&q_emb)
        .bind(patent_id)
        .bind(req.top_k)
        .fetch_all(pool)
        .await?
    } else {
        println!("🔍 Searching across all patents");
        sqlx::query(
            r#"
            SELECT patent_id, chunk_id, text AS snippet,
                   embedding <-> ($1::vector) AS distance
            FROM chunks
            WHERE embedding IS NOT NULL
            ORDER BY embedding <-> ($1::vector)
            LIMIT $2
            "#,
        )
        .bind(&q_emb)
        .bind(req.top_k)
        .fetch_all(pool)
        .await?
    };

    println!("📊 Found {} rows from vector search", rows.len());

    // 3) Map each row into our SearchResult struct
    let mut results = rows
        .into_iter()
        .map(|row| SearchResult {
            patent_id: row.get("patent_id"),
            chunk_id: row.get("chunk_id"),
            snippet: row.get("snippet"),
            distance: row.get("distance"),
        })
        .collect::<Vec<_>>();

    // 4) If vector search returned no results, try keyword search as fallback
    if results.is_empty() {
        println!("⚠️  No results from vector search, trying keyword search...");
        results = run_keyword_search(pool, &req).await?;
    }

    println!("✅ Returning {} search results", results.len());
    Ok(results)
}
