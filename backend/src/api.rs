use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchPayload {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: i64,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub patent_id: String,
    pub chunk_id: String,
    pub snippet: String,
    pub distance: f64,
}

#[derive(Deserialize)]
pub struct QueryPayload {
    pub question: String,
    #[serde(default = "default_top_k")]
    pub top_k: i64,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub answer: String,
}

fn default_top_k() -> i64 { 5 }
