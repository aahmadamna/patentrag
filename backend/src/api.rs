use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SearchPayload {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: i64,
    pub patent_id: Option<String>,
    #[serde(default = "default_search_mode")]
    pub search_mode: String,
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
    pub patent_id: Option<String>,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub answer: String,
}

#[derive(Serialize)]
pub struct ChatSummary {
    pub id: Uuid,
    pub title: String,
}

#[derive(Serialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub msg_type: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateChatRequest {
    pub title: String,
}

#[derive(Deserialize)]
pub struct AddMessageRequest {
    pub sender: String,
    pub content: String,
    pub msg_type: String,
}

fn default_top_k() -> i64 { 5 }

fn default_search_mode() -> String { "semantic".to_string() }
