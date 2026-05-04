use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::document::{DocStatus, Document};

pub async fn insert(
    pool: &PgPool,
    filename: &str,
    mime_type: &str,
    s3_key: &str,
    size_bytes: i64,
) -> AppResult<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (filename, mime_type, s3_key, size_bytes)
        VALUES ($1, $2, $3, $4)
        RETURNING *, 0::bigint AS pages_indexed
        "#,
    )
    .bind(filename)
    .bind(mime_type)
    .bind(s3_key)
    .bind(size_bytes)
    .fetch_one(pool)
    .await?;
    Ok(doc)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Document>> {
    let doc = sqlx::query_as::<_, Document>(
        r#"SELECT d.*, (SELECT COUNT(*) FROM page_indexes pi WHERE pi.document_id = d.id) AS pages_indexed
           FROM documents d WHERE d.id = $1"#,
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(doc)
}

pub async fn list_all(pool: &PgPool) -> AppResult<Vec<Document>> {
    let docs = sqlx::query_as::<_, Document>(
        r#"SELECT d.*, (SELECT COUNT(*) FROM page_indexes pi WHERE pi.document_id = d.id) AS pages_indexed
           FROM documents d ORDER BY d.created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(docs)
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: DocStatus,
    error_msg: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE documents
        SET status = $2, error_msg = $3, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(error_msg)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_page_count(pool: &PgPool, id: Uuid, page_count: i32) -> AppResult<()> {
    sqlx::query("UPDATE documents SET page_count = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(page_count)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn find_pending(pool: &PgPool) -> AppResult<Vec<Document>> {
    let docs = sqlx::query_as::<_, Document>(
        r#"SELECT d.*, (SELECT COUNT(*) FROM page_indexes pi WHERE pi.document_id = d.id) AS pages_indexed
           FROM documents d WHERE d.status = 'uploaded' ORDER BY d.created_at ASC LIMIT 5"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(docs)
}
