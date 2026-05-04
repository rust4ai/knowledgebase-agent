use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::document::{DocumentIndex, PageIndex};

pub async fn insert_page(
    pool: &PgPool,
    document_id: Uuid,
    page_num: i32,
    content: &str,
    tree_index: &serde_json::Value,
) -> AppResult<PageIndex> {
    let page = sqlx::query_as::<_, PageIndex>(
        r#"
        INSERT INTO page_indexes (document_id, page_num, content, tree_index)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (document_id, page_num) DO UPDATE
        SET content = EXCLUDED.content, tree_index = EXCLUDED.tree_index
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(page_num)
    .bind(content)
    .bind(tree_index)
    .fetch_one(pool)
    .await?;
    Ok(page)
}

pub async fn insert_document_index(
    pool: &PgPool,
    document_id: Uuid,
    root_index: &serde_json::Value,
) -> AppResult<DocumentIndex> {
    let idx = sqlx::query_as::<_, DocumentIndex>(
        r#"
        INSERT INTO document_indexes (document_id, root_index)
        VALUES ($1, $2)
        ON CONFLICT (document_id) DO UPDATE
        SET root_index = EXCLUDED.root_index
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(root_index)
    .fetch_one(pool)
    .await?;
    Ok(idx)
}

pub async fn get_pages_for_document(
    pool: &PgPool,
    document_id: Uuid,
) -> AppResult<Vec<PageIndex>> {
    let pages = sqlx::query_as::<_, PageIndex>(
        "SELECT * FROM page_indexes WHERE document_id = $1 ORDER BY page_num",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;
    Ok(pages)
}

pub async fn get_page(
    pool: &PgPool,
    document_id: Uuid,
    page_num: i32,
) -> AppResult<Option<PageIndex>> {
    let page = sqlx::query_as::<_, PageIndex>(
        "SELECT * FROM page_indexes WHERE document_id = $1 AND page_num = $2",
    )
    .bind(document_id)
    .bind(page_num)
    .fetch_optional(pool)
    .await?;
    Ok(page)
}

pub async fn get_document_index(
    pool: &PgPool,
    document_id: Uuid,
) -> AppResult<Option<DocumentIndex>> {
    let idx = sqlx::query_as::<_, DocumentIndex>(
        "SELECT * FROM document_indexes WHERE document_id = $1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?;
    Ok(idx)
}

pub async fn get_all_document_indexes(pool: &PgPool) -> AppResult<Vec<DocumentIndex>> {
    let indexes = sqlx::query_as::<_, DocumentIndex>(
        "SELECT * FROM document_indexes ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(indexes)
}
