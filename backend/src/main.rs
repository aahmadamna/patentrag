mod api;
mod search;
mod rag;
mod ingest;
mod chunker;
mod db;
mod embedder;
mod pdf_generator;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    http::StatusCode,
    response::{Json, Response},
    body::Body,
};
use axum::http::header::{CONTENT_TYPE, CONTENT_DISPOSITION};
use sqlx::PgPool;
use redis::aio::Connection;
use serde_json::json;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use axum_extra::extract::Multipart;
use std::fs::File as StdFile;
use std::io::Write;
use uuid::Uuid;
use tower_http::cors::{CorsLayer, Any};
use axum::extract::Path;

use api::{SearchPayload, SearchResult as ApiSearchResult, QueryPayload, QueryResponse, ChatSummary, Message, CreateChatRequest, AddMessageRequest};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Patentrag backend starting...");
    // Connect to Postgres
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pg_pool = PgPool::connect(&database_url).await?;

    // Connect to Redis
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_async_connection().await?;

    // Shared state
    let shared = Arc::new(AppState {
        pg_pool,
        redis: Mutex::new(conn),
    });

    // Router
    let app = Router::new()
        .route("/", get(root))
        .route("/search", post(handle_search))
        .route("/query", post(handle_query))
        .route("/upload_pdf", post(handle_upload_pdf))
        .route("/related-documents/:patent_id", get(find_related_documents))
        .route("/chats", post(create_chat).get(list_chats))
        .route("/chats/:chat_id/messages", get(list_messages).post(add_message))
        .route("/chats/:chat_id/summary-pdf", get(download_chat_summary))
        .route("/chats/:chat_id", axum::routing::delete(delete_chat))
        .route("/test-search", get(test_search))
        .route("/download-summary-pdf", post(download_smart_summary_pdf))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(shared);

    // Launch
    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8000".to_string())
    .parse::<u16>()
    .expect("PORT must be a valid u16");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("✅ Axum server is about to start...");
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "Patentrag API" }))
}

async fn test_search() -> Json<serde_json::Value> {
    println!("🔍 Test search endpoint called!");
    Json(json!({ "status": "test_search_working" }))
}

async fn handle_search(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchPayload>,
) -> Result<Json<Vec<ApiSearchResult>>, StatusCode> {
    println!("🔍 handle_search called with mode: {}", payload.search_mode);
    let mut redis_conn = state.redis.lock().await;
    let results = search::run_search(
        &state.pg_pool,
        search::SearchRequest {
            query: payload.query,
            top_k: payload.top_k,
            patent_id: payload.patent_id,
            search_mode: payload.search_mode,
        },
        &mut *redis_conn,
    )
    .await
    .map_err(|e| {
        println!("❌ Search error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let api_results = results
        .into_iter()
        .map(|r| ApiSearchResult {
            patent_id: r.patent_id,
            chunk_id: r.chunk_id,
            snippet: r.snippet,
            distance: r.distance,
        })
        .collect();
    Ok(Json(api_results))
}

async fn handle_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryPayload>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let mut redis_conn = state.redis.lock().await;
    let answer = rag::run_query(
        &state.pg_pool,
        &payload.question,
        payload.top_k,
        &mut *redis_conn,
        payload.patent_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(QueryResponse { answer }))
}

async fn handle_upload_pdf(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("📄 PDF upload request received");
    
    // Only accept one file field named 'file'
    let mut pdf_path = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        println!("Received field: {:?}", field.name());
        if field.name() == Some("file") {
            let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "uploaded.pdf".to_string());
            println!("📁 File name: {}", file_name);
            let save_path = format!("../data/{}", file_name);
            println!("💾 Saving to: {}", save_path);
            
            let data = field.bytes().await.map_err(|e| {
                println!("❌ Error reading file bytes: {:?}", e);
                StatusCode::BAD_REQUEST
            })?;
            println!("📊 File size: {} bytes", data.len());
            
            let mut file = StdFile::create(&save_path).map_err(|e| {
                println!("❌ Error creating file: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            file.write_all(&data).map_err(|e| {
                println!("❌ Error writing file: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            pdf_path = Some(save_path);
            println!("✅ File saved successfully");
            break;
        }
    }
    let pdf_path = pdf_path.ok_or_else(|| {
        println!("❌ No file field found in request");
        StatusCode::BAD_REQUEST
    })?;

    // Extract text from PDF
    println!("📖 Extracting text from PDF...");
    let text = match crate::ingest::extract_text_from_pdf(&pdf_path) {
        Ok(t) => {
            println!("✅ Text extracted, length: {} characters", t.len());
            t
        },
        Err(e) => {
            println!("❌ Error extracting text: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Chunk the text
    println!("✂️ Chunking text...");
    let chunks = crate::chunker::chunk_text(&text, 2000, 400);
    println!("📦 Created {} chunks", chunks.len());
    let patent_id = Uuid::new_v4().to_string();
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}-{}", patent_id, i);
        if let Err(e) = crate::db::save_chunk(&state.pg_pool, &patent_id, &chunk_id, chunk).await {
            println!("❌ Error saving chunk {}: {:?}", i, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    println!("💾 All chunks saved to database");

    // Optionally, trigger embedding job here or let it run in background
    crate::embedder::run_embedding_job(&state.pg_pool).await.ok();

    println!("🎉 PDF upload completed successfully");
    Ok(Json(json!({ "status": "ok", "patent_id": patent_id })))
}

// --- Chat Endpoints ---

async fn create_chat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateChatRequest>,
) -> Result<Json<ChatSummary>, StatusCode> {
    let id = crate::db::create_chat(&state.pg_pool, &payload.title).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ChatSummary { id, title: payload.title }))
}

async fn list_chats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ChatSummary>>, StatusCode> {
    let chats = crate::db::list_chats(&state.pg_pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(chats.into_iter().map(|(id, title)| ChatSummary { id, title }).collect()))
}

async fn add_message(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(chat_id): axum::extract::Path<Uuid>,
    Json(payload): Json<AddMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    let msg_id = crate::db::add_message(
        &state.pg_pool,
        chat_id,
        &payload.sender,
        &payload.content,
        &payload.msg_type,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Fetch created_at for the new message
    let messages = crate::db::list_messages(&state.pg_pool, chat_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let msg = messages.into_iter().find(|(id, _, _, _, _)| *id == msg_id).unwrap();
    Ok(Json(Message {
        id: msg.0,
        sender: msg.1,
        content: msg.2,
        msg_type: msg.3,
        created_at: msg.4,
    }))
}

async fn list_messages(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(chat_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let messages = crate::db::list_messages(&state.pg_pool, chat_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(messages.into_iter().map(|(id, sender, content, msg_type, created_at)| Message {
        id, sender, content, msg_type, created_at
    }).collect()))
}

async fn download_chat_summary(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(chat_id): axum::extract::Path<Uuid>,
) -> Result<Response<Body>, StatusCode> {
    // Get chat details
    let chats = crate::db::list_chats(&state.pg_pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let chat = chats.into_iter().find(|(id, _)| *id == chat_id).ok_or(StatusCode::NOT_FOUND)?;
    let chat_summary = api::ChatSummary { id: chat.0, title: chat.1 };
    
    // Get messages
    let messages_data = crate::db::list_messages(&state.pg_pool, chat_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let messages: Vec<api::Message> = messages_data.into_iter().map(|(id, sender, content, msg_type, created_at)| {
        api::Message { id, sender, content, msg_type, created_at }
    }).collect();
    
    // Extract relevant chunks from search result messages
    let mut relevant_chunks = Vec::new();
    for message in &messages {
        if message.msg_type == "search_results" && message.content.starts_with("Search results:") {
            let chunks_text = message.content.replace("Search results:\n", "");
            let chunks: Vec<&str> = chunks_text.split("\n\n").collect();
            relevant_chunks.extend(chunks.iter().map(|s| s.to_string()));
        }
    }
    
    // Extract patent_id from messages if available
    let patent_id = messages.iter()
        .find(|m| m.msg_type == "question" || m.msg_type == "search")
        .and_then(|_| Some("patent_id".to_string())); // You might want to store patent_id in messages
    
    let summary_data = pdf_generator::ChatSummaryData {
        chat: chat_summary,
        messages,
        relevant_chunks,
        patent_id,
    };
    
    // Generate PDF
    let pdf_bytes = pdf_generator::generate_chat_summary_pdf(&summary_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create response with text file headers
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/plain")
        .header(CONTENT_DISPOSITION, format!("attachment; filename=\"chat-summary-{}.txt\"", chat_id))
        .body(Body::from(pdf_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}

async fn delete_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<Uuid>,
) -> StatusCode {
    match crate::db::delete_chat(&state.pg_pool, chat_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn find_related_documents(
    State(state): State<Arc<AppState>>,
    Path(patent_id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let mut redis_conn = state.redis.lock().await;
    
    // Get a representative chunk from the current patent
    let representative_chunk = sqlx::query!(
        r#"
        SELECT text FROM chunks 
        WHERE patent_id = $1 AND embedding IS NOT NULL 
        LIMIT 1
        "#,
        patent_id
    )
    .fetch_optional(&state.pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if let Some(chunk) = representative_chunk {
        // Use the chunk text to find similar documents
        let results = search::run_search(
            &state.pg_pool,
            search::SearchRequest {
                query: chunk.text,
                top_k: 5,
                patent_id: None, // Search across all patents
                search_mode: "semantic".to_string(),
            },
            &mut *redis_conn,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Filter out the current patent and format results
        let related_docs = results
            .into_iter()
            .filter(|r| r.patent_id != patent_id)
            .map(|r| serde_json::json!({
                "patent_id": r.patent_id,
                "snippet": r.snippet,
                "similarity": 1.0 - r.distance
            }))
            .collect();
        
        Ok(Json(related_docs))
    } else {
        Ok(Json(vec![]))
    }
}

async fn download_smart_summary_pdf(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response<Body>, StatusCode> {
    let patent_id = payload["patent_id"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let summary_text = payload["summary"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let filename = payload["filename"].as_str().unwrap_or("patent");

    // Generate summary PDF content
    let mut summary_content = String::new();
    summary_content.push_str("=== SMART SUMMARY REPORT ===\n\n");
    summary_content.push_str(&format!("Patent: {}\n", filename));
    summary_content.push_str(&format!("Patent ID: {}\n", patent_id));
    summary_content.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    summary_content.push_str("\n=== SUMMARY ===\n\n");
    summary_content.push_str(summary_text);

    // Create response with text file headers
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/plain")
        .header(CONTENT_DISPOSITION, format!("attachment; filename=\"smart-summary-{}.txt\"", filename))
        .body(Body::from(summary_content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

struct AppState {
    pg_pool: PgPool,
    redis: Mutex<Connection>,
}