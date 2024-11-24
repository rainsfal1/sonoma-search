use std::env;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use log::{info, error, debug};
use sqlx::postgres::PgPoolOptions;
use elastic_search_storage::{get_elasticsearch_client};
use async_processor::concurrent_process_docs;
use env_logger::Env;
use crate::error::IndexerError;
use metrics::{MetricsClient, REGISTRY};
use tokio::time::interval;
use actix_web::{web, App, HttpServer, HttpResponse};
use prometheus::{Encoder, TextEncoder};

mod db_indexer;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod error;
mod metrics;

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(buffer)
}

#[tokio::main]
async fn main() -> Result<(), IndexerError> {
    // Initialize logger with custom levels
    env_logger::Builder::from_env(
        Env::default()
            .filter_or("RUST_LOG", "warn,indexer=info,elasticsearch=warn")
    ).init();
    
    info!("Starting indexer process");
    dotenv().ok();
    
    debug!("Loading environment variables");
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    info!("Connecting to database");
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|e| {
            error!("Database connection error: {}", e);
            IndexerError::Database(e)
        })?;

    debug!("Testing database connection");
    sqlx::query("SELECT 1").execute(&pool).await
        .map_err(|e| {
            error!("Database connection test failed: {}", e);
            IndexerError::Database(e)
        })?;

    info!("Successfully connected to database");
    info!("Initializing Elasticsearch client");
    let es_client = Arc::new(get_elasticsearch_client().await?);
    
    elastic_search_storage::ensure_index_exists(&es_client).await?;

    let metrics_url = env::var("METRICS_URL")
        .unwrap_or_else(|_| "http://localhost:9091".to_string());
    
    let metrics_client = Arc::new(MetricsClient::new(metrics_url));

    // Start metrics server
    let metrics_port = env::var("METRICS_PORT")
        .unwrap_or_else(|_| "9091".to_string())
        .parse::<u16>()
        .expect("METRICS_PORT must be a valid port number");

    info!("Starting metrics server on port {}", metrics_port);
    let metrics_server = HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(metrics))
    })
    .bind(("0.0.0.0", metrics_port))
    .expect("Failed to bind metrics server")
    .run();

    // Spawn metrics server
    tokio::spawn(metrics_server);
    
    info!("Starting document processing loop");
    let mut interval = interval(Duration::from_secs(30)); // 30 second intervals
    
    loop {
        interval.tick().await;  // Wait for next interval
        let client_clone = Arc::clone(&es_client);
        let metrics_clone = Arc::clone(&metrics_client);
        
        match concurrent_process_docs(pool.clone(), client_clone, &metrics_clone).await {
            Ok(processed_count) => {
                if processed_count == 0 {
                    debug!("No documents to process, waiting for next interval");
                } else {
                    info!("Successfully processed {} documents", processed_count);
                }
            }
            Err(e) => {
                error!("Error processing documents: {}", e);
            }
        }
    }
}