mod api;
mod search;
mod rag;

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
        .with_state(shared);

    // Launch
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

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
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(QueryResponse { answer }))
}

struct AppState {
    pg_pool: PgPool,
    redis: Mutex<Connection>,
}