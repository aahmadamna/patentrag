// backend/src/db.rs

use sqlx::PgPool;
use uuid::Uuid;
use time::PrimitiveDateTime;

/// Inserts a new text chunk into the `chunks` table.
pub async fn save_chunk(
    pool: &PgPool,
    patent_id: &str,
    chunk_id: &str,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO chunks (patent_id, chunk_id, text)
        VALUES ($1, $2, $3)
        "#,
        patent_id,
        chunk_id,
        text
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetches all chunks whose `embedding` is still NULL.
pub async fn fetch_unembedded_chunks(
    pool: &PgPool,
) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT chunk_id, patent_id, text
        FROM chunks
        WHERE embedding IS NULL
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| (r.chunk_id, r.patent_id, r.text))
        .collect())
}

/// Updates the `embedding` column for a given chunk.
/// Uses `sqlx::query()` with `.bind()` so we can pass a `Vec<f32>`.
pub async fn save_embedding(
    pool: &PgPool,
    chunk_id: &str,
    embedding: &[f32],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chunks
        SET embedding = $1
        WHERE chunk_id = $2
        "#,
    )
    .bind(embedding)
    .bind(chunk_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Create a new chat and return its id
pub async fn create_chat(pool: &PgPool, title: &str) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO chats (id, title) VALUES ($1, $2)"#,
        id,
        title
    )
    .execute(pool)
    .await?;
    Ok(id)
}

/// List all chats
pub async fn list_chats(pool: &PgPool) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    let rows = sqlx::query!(r#"SELECT id, title FROM chats ORDER BY created_at DESC"#)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| (r.id, r.title.unwrap_or_default())).collect())
}

/// Add a message to a chat
pub async fn add_message(
    pool: &PgPool,
    chat_id: Uuid,
    sender: &str,
    content: &str,
    msg_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO messages (id, chat_id, sender, content, type) VALUES ($1, $2, $3, $4, $5)"#,
        id,
        chat_id,
        sender,
        content,
        msg_type
    )
    .execute(pool)
    .await?;
    Ok(id)
}

/// List all messages for a chat
pub async fn list_messages(pool: &PgPool, chat_id: Uuid) -> Result<Vec<(Uuid, String, String, String, String)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id, sender, content, type, created_at FROM messages WHERE chat_id = $1 ORDER BY created_at ASC"#,
        chat_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| (
        r.id,
        r.sender.unwrap_or_default(),
        r.content.unwrap_or_default(),
        r.r#type.unwrap_or_default(),
        r.created_at.map(|dt: PrimitiveDateTime| dt.to_string()).unwrap_or_default()
    )).collect())
}

/// Delete a chat and all its messages
pub async fn delete_chat(pool: &PgPool, chat_id: Uuid) -> Result<(), sqlx::Error> {
    // Delete messages first (foreign key constraint)
    sqlx::query!("DELETE FROM messages WHERE chat_id = $1", chat_id)
        .execute(pool)
        .await?;
    // Delete the chat
    sqlx::query!("DELETE FROM chats WHERE id = $1", chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
