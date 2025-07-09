mod api;
mod search;
mod rag;
mod ingest;
mod chunker;
mod db;
mod embedder;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
};
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

use api::{SearchPayload, SearchResult as ApiSearchResult, QueryPayload, QueryResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(shared);

    // Launch
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("🚀 Listening on http://{}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "Patentrag API" }))
}

async fn handle_search(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchPayload>,
) -> Result<Json<Vec<ApiSearchResult>>, StatusCode> {
    let mut redis_conn = state.redis.lock().await;
    let results = search::run_search(
        &state.pg_pool,
        search::SearchRequest {
            query: payload.query,
            top_k: payload.top_k,
            patent_id: payload.patent_id,
        },
        &mut *redis_conn,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let chunks = crate::chunker::chunk_text(&text, 800, 200);
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

struct AppState {
    pg_pool: PgPool,
    redis: Mutex<Connection>,
}