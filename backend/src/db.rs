// backend/src/db.rs

use sqlx::PgPool;

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
