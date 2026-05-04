use std::sync::Arc;

use s3::Bucket;
use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::rag_agent::RagAgent;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub bucket: Arc<Box<Bucket>>,
    pub config: Arc<AppConfig>,
    pub rag: Arc<RagAgent>,
}
