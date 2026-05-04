use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "doc_status", rename_all = "lowercase")]
pub enum DocStatus {
    Uploaded,
    Processing,
    Indexed,
    Failed,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub s3_key: String,
    pub size_bytes: i64,
    pub status: DocStatus,
    pub page_count: Option<i32>,
    pub pages_indexed: Option<i64>,
    pub error_msg: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PageIndex {
    pub id: Uuid,
    pub document_id: Uuid,
    pub page_num: i32,
    pub content: String,
    pub tree_index: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DocumentIndex {
    pub id: Uuid,
    pub document_id: Uuid,
    pub root_index: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
