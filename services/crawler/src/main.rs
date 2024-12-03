use actix_web::{web, App, HttpServer, HttpResponse};
use prometheus::{Encoder, TextEncoder, Registry};
use log::{info, error};
use std::error::Error;
use std::path::PathBuf;
use env_logger::{Builder, Env};
use crate::error::CrawlerError;
use crate::config::Config;
use crate::crawler::Crawler;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::fetcher::create_http_client;
use storage::PostgresStorage;
use dotenv;

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(String::from_utf8(buffer).unwrap())
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

mod api;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file
    dotenv::dotenv().ok();
    
    // Initialize logger
    Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Starting crawler service...");
    info!("Checking environment variables...");
    
    // Initialize metrics
    info!("Initializing metrics...");
    prometheus::default_registry()
        .register(Box::new(crate::metrics::QUEUE_SIZE.clone()))
        .expect("Failed to register queue size metric");
    prometheus::default_registry()
        .register(Box::new(crate::metrics::PAGES_CRAWLED.clone()))
        .expect("Failed to register pages crawled metric");
    prometheus::default_registry()
        .register(Box::new(crate::metrics::CRAWL_ERRORS.clone()))
        .expect("Failed to register crawl errors metric");
    prometheus::default_registry()
        .register(Box::new(crate::metrics::CRAWL_CYCLES.clone()))
        .expect("Failed to register crawl cycles metric");
    prometheus::default_registry()
        .register(Box::new(crate::metrics::CRAWL_DURATION.clone()))
        .expect("Failed to register crawl duration metric");
    info!("Metrics initialized successfully");
    
    // Check DATABASE_URL before starting any services
    if std::env::var("DATABASE_URL").is_err() {
        error!("DATABASE_URL environment variable is not set. Please set it before running the crawler.");
        std::process::exit(1);
    }

    let current_dir = std::env::current_dir()?;
    let config_path = find_config_file(&current_dir);
    let config = Config::from_file(config_path)?;
    let client = create_http_client()?;
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| CrawlerError::EnvError(std::env::VarError::NotPresent))?;
    let storage = PostgresStorage::new(&database_url).await?;
    
    // Create crawler instance and wrap it in Arc<Mutex>
    let crawler = Arc::new(Mutex::new(Crawler::new(client, config.clone(), storage)));
    let crawler_data = web::Data::new(crawler.clone());
    
    // Start initial crawl in background
    let crawler_clone = crawler.clone();
    tokio::spawn(async move {
        info!("Starting initial crawl with seed URLs...");
        let crawler_guard = crawler_clone.lock().await;
        if let Err(e) = crawler_guard.initialize().await {
            error!("Failed to initialize crawler: {}", e);
            return;
        }
        
        if let Err(e) = crawler_guard.crawl_with_params(config.max_depth, config.max_pages).await {
            error!("Initial crawl failed: {}", e);
        }
        info!("Initial crawl completed");
    });
    
    // Start API server
    let api_server = HttpServer::new(move || {
        App::new()
            .app_data(crawler_data.clone())
            .service(api::crawl)
            .service(api::get_job_status)
    })
    .bind("0.0.0.0:8000")?
    .run();
    
    info!("API server started on http://0.0.0.0:8000");
    
    // Start metrics server
    let metrics_server = HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(metrics))
    })
    .bind("0.0.0.0:9091")?
    .run();
    
    info!("Metrics server started on http://0.0.0.0:9091/metrics");

    // Run both servers
    tokio::try_join!(api_server, metrics_server)?;

    Ok(())
}

mod config;
mod crawler;
mod fetcher;
mod parser;
mod robots;
mod summarizer;
mod metrics;
mod error;
