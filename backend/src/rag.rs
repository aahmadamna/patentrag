use std::error::Error;
use std::env;

use redis::aio::Connection;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::Value;
use crate::search::{run_search, SearchRequest, SearchResult};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    answer: String,
}

/// Wraps your question + retrieved snippets into a chat completion,
/// with Redis-backed caching for Cache-Augmented Generation.
pub async fn run_query(
    pool: &PgPool,
    question: &str,
    top_k: i64,
    redis_conn: &mut Connection,
    patent_id: Option<String>,
) -> Result<String, Box<dyn Error>> {
    // 1) Retrieve the top-K chunks via your existing cached run_search
    let results: Vec<SearchResult> = run_search(
        pool,
        SearchRequest { query: question.to_string(), top_k, patent_id },
        redis_conn,                     
    )
    .await?;

    // 2) Compute a stable cache key over question + context text
    let mut hasher = Sha256::new();
    hasher.update(question.as_bytes());
    for chunk in &results {
        hasher.update(chunk.snippet.as_bytes());
    }
    let cache_key = format!("cag:{}", hex::encode(hasher.finalize()));

    // 3) If we have a cached answer, return it
    if let Ok(cached_json) = redis_conn.get::<_, String>(&cache_key).await {
        if let Ok(entry) = serde_json::from_str::<CacheEntry>(&cached_json) {
            return Ok(entry.answer);
        }
    }

    // 4) Assemble your RAG prompt exactly as before
    let mut prompt = format!(
        "You are a patent expert. Answer using ONLY the context. Cite each point like [1], [2].\n\nQuestion: {}\n\nContext:\n",
        question
    );
    for (i, chunk) in results.iter().enumerate() {
        prompt.push_str(&format!(
            "[{}] ({}-{}): {}\n\n",
            i + 1,
            chunk.patent_id,
            chunk.chunk_id.split('-').last().unwrap_or(""),
            chunk.snippet
        ));
    }

    // 5) Call the Chat API
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            { "role": "system", "content": "You're a precise, citation-driven patent assistant." },
            { "role": "user",   "content": prompt }
        ]
    });
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    // 6) Extract the answer text
    let answer = resp["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid API response")?
        .to_string();

    // 7) Cache the answer for 24 hours
    let entry = CacheEntry { answer: answer.clone() };
    let serialized = serde_json::to_string(&entry)?;
    let ttl_secs = 60 * 60 * 24;
    let _: () = redis_conn.set_ex(&cache_key, serialized, ttl_secs).await?;

    Ok(answer)
}
