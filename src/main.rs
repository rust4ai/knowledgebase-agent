use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod config;
mod controllers;
mod db;
mod error;
mod models;
mod services;
mod state;

use config::AppConfig;
use services::rag_agent::RagAgent;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = AppConfig::from_env();

    // Database
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    tracing::info!("connected to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    tracing::info!("migrations complete");

    // S3 (DigitalOcean Spaces)
    let bucket = services::s3::create_bucket(&config).expect("failed to create S3 bucket client");

    // RAG Agent (metalcraft + rig)
    let rag = RagAgent::new(&config, db.clone()).expect("failed to create RAG agent");

    let state = AppState {
        db: db.clone(),
        bucket: Arc::new(bucket),
        config: Arc::new(config.clone()),
        rag: Arc::new(rag),
    };

    // Background indexer
    let indexer_state = state.clone();
    tokio::spawn(async move {
        services::indexer::run_indexer_loop(indexer_state).await;
    });

    // Routes
    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/documents", post(controllers::documents::upload))
        .route("/api/documents", get(controllers::documents::list))
        .route("/api/documents/{id}", get(controllers::documents::get))
        .route(
            "/api/documents/{id}",
            delete(controllers::documents::delete),
        )
        .route("/api/query", post(controllers::query::query))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
