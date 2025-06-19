// backend/src/rag.rs

use std::error::Error;
use std::env;

use redis::aio::Connection;
use reqwest::Client;
use serde_json::Value;
use crate::search::{run_search, SearchRequest, SearchResult};
use sqlx::PgPool;

/// Wraps your question + retrieved snippets into a chat completion.
pub async fn run_query(
    pool: &PgPool,
    question: &str,
    top_k: i64,
    redis_conn: &mut Connection,      // <- take Redis conn
) -> Result<String, Box<dyn Error>> {
    // 1) Retrieve the top-K chunks via cached search
    let results: Vec<SearchResult> = run_search(
        pool,
        SearchRequest { query: question.to_string(), top_k },
        redis_conn,                     // <- pass it here
    )
    .await?;

    // 2) Assemble the prompt
    let mut prompt = format!(
        "You are a patent expert. Answer using ONLY the context. Cite each point like [1], [2].\n\nQuestion: {}\n\nContext:\n",
        question
    );
    for (i, chunk) in results.iter().enumerate() {
        prompt.push_str(&format!(
            "[{}] ({}-{}): {}\n\n",
            i + 1,
            chunk.patent_id,
            &chunk.chunk_id.split('-').last().unwrap_or(""),
            chunk.snippet
        ));
    }

    // 3) Call the Chat API
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            { "role": "system", "content": "You’re a precise, citation-driven patent assistant." },
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

    // 4) Extract and return the answer text
    let answer = resp["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid API response")?
        .to_string();

    Ok(answer)
}
