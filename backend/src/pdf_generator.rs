use std::error::Error;
use chrono::Utc;
use crate::api::{Message, ChatSummary};

pub struct ChatSummaryData {
    pub chat: ChatSummary,
    pub messages: Vec<Message>,
    pub relevant_chunks: Vec<String>,
    pub patent_id: Option<String>,
}

pub fn generate_chat_summary_pdf(data: &ChatSummaryData) -> Result<Vec<u8>, Box<dyn Error>> {
    // For now, generate a simple text-based summary that can be easily converted to PDF
    let mut summary = String::new();
    
    // Title
    summary.push_str("=== CHAT SUMMARY REPORT ===\n\n");
    summary.push_str(&format!("Chat: {}\n", data.chat.title));
    summary.push_str(&format!("Generated: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    if let Some(patent_id) = &data.patent_id {
        summary.push_str(&format!("Patent ID: {}\n", patent_id));
    }
    
    summary.push_str("\n=== CONVERSATION HISTORY ===\n\n");
    
    // Add conversation history
    for message in &data.messages {
        let sender = if message.sender == "user" { "You" } else { "AI Assistant" };
        summary.push_str(&format!("{}: {}\n\n", sender, message.content));
    }
    
    // Add relevant chunks if any
    if !data.relevant_chunks.is_empty() {
        summary.push_str("=== RELEVANT CHUNKS FROM PATENT ===\n\n");
        for (i, chunk) in data.relevant_chunks.iter().enumerate() {
            summary.push_str(&format!("Chunk {}:\n{}\n\n", i + 1, chunk));
        }
    }
    
    // Convert to bytes (this will be a simple text file for now)
    Ok(summary.into_bytes())
} 