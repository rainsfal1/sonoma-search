use actix_web::{web, App, HttpServer, HttpResponse};
use prometheus::{Encoder, TextEncoder};
use tokio::time::{interval, Duration};
use log::{info, error};
use std::error::Error;
use std::path::PathBuf;
use env_logger::{Builder, Env};
use crate::error::CrawlerError;

mod config;
mod crawler;
mod fetcher;
mod parser;
mod robots;
mod summarizer;
mod metrics;
mod error;

use crate::config::Config;
use crate::crawler::Crawler;
use crate::fetcher::create_http_client;
use storage::PostgresStorage;

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(String::from_utf8(buffer).unwrap())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_config_file(&current_dir);
    let config = Config::from_file(config_path)?;
    let client = create_http_client()?;
    
    // Get database URL with better error handling
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| CrawlerError::EnvError(std::env::VarError::NotPresent))?;
    
    let storage = PostgresStorage::new(&database_url).await?;

    info!("Starting continuous crawl process");
    let mut crawler = Crawler::new(client, config, storage);
    let mut interval = interval(Duration::from_secs(300)); // 5 minute intervals
    
    loop {
        interval.tick().await;
        match crawler.crawl().await {
            Ok(_) => {
                info!("Crawl cycle completed");
                metrics::increment_crawl_cycles();
            }
            Err(e) => {
                error!("Crawl cycle failed: {}", e);
                metrics::increment_crawl_errors();
            }
        }
    }
}

fn find_config_file(current_dir: &PathBuf) -> PathBuf {
    let mut config_path = current_dir.join("crawler").join("config.toml");
    if config_path.exists() {
        return config_path;
    }

    config_path = current_dir.join("config.toml");
    if config_path.exists() {
        return config_path;
    }

    current_dir.join("crawler").join("config.toml") // Default path if not found
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Starting crawler service...");
    info!("Checking environment variables...");
    
    // Check DATABASE_URL before starting any services
    if std::env::var("DATABASE_URL").is_err() {
        error!("DATABASE_URL environment variable is not set. Please set it before running the crawler.");
        std::process::exit(1);
    }
    
    // Start metrics server in a separate task
    let metrics_server = tokio::spawn(
        HttpServer::new(|| {
            App::new()
                .route("/metrics", web::get().to(metrics))
        })
        .bind("127.0.0.1:9091")?
        .run()
    );

    info!("Metrics server started on http://127.0.0.1:9091/metrics");

    // Run the crawler directly (not in a separate task)
    if let Err(e) = run().await {
        error!("Crawler error: {}", e);
    }

    // Wait for metrics server to finish if crawler exits
    if let Err(e) = metrics_server.await? {
        error!("Metrics server error: {}", e);
    }

    Ok(())
}
