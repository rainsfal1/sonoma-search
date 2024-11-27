use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dotenv::dotenv;
use log::{info, error, debug};
use sqlx::postgres::PgPoolOptions;
use elastic_search_storage::{get_elasticsearch_client, get_elasticsearch_doc_count};
use async_processor::concurrent_process_docs;
use env_logger::Env;
use crate::error::IndexerError;
use metrics::{MetricsClient, REGISTRY};
use tokio::time::interval;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use prometheus::{Encoder, TextEncoder};

mod db_indexer;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod error;
mod metrics;

async fn metrics() -> ActixResult<HttpResponse> {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&REGISTRY.gather(), &mut buffer)
        .map_err(|e| {
            error!("Failed to encode metrics: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to encode metrics")
        })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer))
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
        .unwrap_or_else(|_| "9092".to_string())
        .parse::<u16>()
        .expect("METRICS_PORT must be a valid port number");

    info!("Starting metrics server on port {}", metrics_port);
    let metrics_server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .route("/metrics", web::get().to(metrics))
    })
    .bind(("0.0.0.0", metrics_port))
    .map_err(|e| {
        error!("Failed to bind metrics server: {}", e);
        IndexerError::Server(e.to_string())
    })?
    .run();

    // Spawn metrics server
    tokio::spawn(metrics_server);
    
    info!("Starting document processing loop");
    let mut interval = interval(Duration::from_secs(30)); // 30 second intervals
    
    loop {
        interval.tick().await;  // Wait for next interval
        let client_clone = Arc::clone(&es_client);
        let metrics_clone = Arc::clone(&metrics_client);
        let pool_clone = pool.clone();
        
        let start_time = Instant::now();
        
        match concurrent_process_docs(pool_clone, client_clone, &metrics_clone).await {
            Ok(processed) => {
                metrics_clone.increment_docs_processed();
                metrics_clone.observe_index_duration(start_time.elapsed().as_secs_f64());
                if processed > 0 {
                    info!("Processed {} documents", processed);
                }
            }
            Err(e) => {
                error!("Error processing documents: {}", e);
                metrics_clone.increment_index_errors();
            }
        }
        
        // Update Elasticsearch document count
        if let Ok(count) = get_elasticsearch_doc_count(&es_client).await {
            metrics_clone.set_elasticsearch_docs_count(count);
        }
        
        metrics_clone.increment_index_cycles();
    }
}