use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
}

/// POST /api/query - ask a question against the knowledge base
pub async fn query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> AppResult<Json<serde_json::Value>> {
    tracing::info!(question = %req.question, "query received");

    let response = state
        .rag
        .query(&req.question)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!(response)))
}
