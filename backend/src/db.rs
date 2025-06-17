use sqlx::PgPool;

pub async fn save_chunk(
    pool: &PgPool,
    patent_id: &str,
    chunk_id: &str,
    text: &str
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO chunks (patent_id, chunk_id, text) VALUES ($1, $2, $3)",
        patent_id,
        chunk_id,
        text
    )
    .execute(pool)
    .await?;
    Ok(())
}
