use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::services::s3;
use crate::state::AppState;

/// POST /api/documents - upload a document
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<Value>)> {
    let mut file_data: Option<(String, String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field
                .file_name()
                .unwrap_or("unknown")
                .to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            file_data = Some((filename, content_type, bytes.to_vec()));
        }
    }

    let (filename, content_type, bytes) =
        file_data.ok_or_else(|| AppError::BadRequest("no file field in upload".into()))?;

    let s3_key = format!("documents/{}/{}", Uuid::new_v4(), filename);
    let size_bytes = bytes.len() as i64;

    // Upload to DigitalOcean Spaces
    s3::upload_bytes(&state.bucket, &s3_key, &bytes, &content_type).await?;

    // Insert metadata
    let doc = db::documents::insert(&state.db, &filename, &content_type, &s3_key, size_bytes).await?;

    tracing::info!(doc_id = %doc.id, filename = %doc.filename, "document uploaded");

    Ok((StatusCode::CREATED, Json(json!(doc))))
}

/// GET /api/documents - list all documents
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let docs = db::documents::list_all(&state.db).await?;
    Ok(Json(json!(docs)))
}

/// GET /api/documents/:id - get single document
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let doc = db::documents::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;
    Ok(Json(json!(doc)))
}

/// DELETE /api/documents/:id - delete document + S3 object
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let doc = db::documents::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;

    // Delete from S3
    s3::delete_object(&state.bucket, &doc.s3_key).await?;

    // Delete from DB (cascades to page_indexes + document_indexes)
    db::documents::delete(&state.db, id).await?;

    tracing::info!(doc_id = %id, "document deleted");
    Ok(StatusCode::NO_CONTENT)
}
