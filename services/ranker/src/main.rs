use std::env;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use log::{info, error};
use sqlx::postgres::PgPoolOptions;
use tokio::time::interval;
use env_logger::Env;
use actix_web::{web, App, HttpServer, HttpResponse};
use prometheus::{Encoder, TextEncoder};

mod page_rank;
mod postgres_utilities;
mod display_rank;
mod data_models;
mod metrics;

use crate::metrics::MetricsClient;

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(String::from_utf8(buffer).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(
        Env::default()
            .filter_or("RUST_LOG", "info,ranker=info")
    ).init();
    
    info!("Starting ranker service...");
    
    // Initialize metrics
    info!("Initializing metrics...");
    metrics::init_metrics();
    info!("Metrics initialized successfully");
    
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    info!("Connecting to database");
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(30))
        .connect_lazy(&database_url)
        .expect("Failed to create pool");
    
    let metrics_url = env::var("METRICS_URL")
        .unwrap_or_else(|_| "http://localhost:9093".to_string()); // Port 9093 for ranker
    
    let metrics_client = Arc::new(MetricsClient::new(metrics_url.clone()));
    
    // Start metrics server in a separate task
    let _metrics_server = tokio::spawn(
        HttpServer::new(|| {
            App::new()
                .route("/metrics", web::get().to(metrics))
        })
        .bind("0.0.0.0:9093")?
        .run()
    );

    info!("Metrics server started on http://127.0.0.1:9093/metrics");
    
    info!("Starting ranking process");
    let mut interval = interval(Duration::from_secs(300)); // 5 minute intervals
    
    loop {
        interval.tick().await;
        let timer = metrics::Timer::new();
        
        let rows = match postgres_utilities::fetch_links(&pool).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("Error fetching links: {}", e);
                metrics_client.increment("rank_errors").await?;
                continue;
            }
        };
        
        let (page_links, unique_pages) = page_rank::prepare_page_links(rows);
        metrics::PAGES_TO_RANK.set(unique_pages.len() as i64);
        
        // Set graph size metric (total number of links)
        let total_links: i64 = page_links.values().map(|links| links.len() as i64).sum();
        metrics::GRAPH_SIZE.set(total_links);
        
        let page_ranks = page_rank::calculate_page_rank(&page_links, 0.85, 100);
        
        match postgres_utilities::update_page_ranks(&pool, &page_ranks).await {
            Ok(_) => {
                info!("Successfully ranked {} pages", unique_pages.len());
                display_rank::display_rank_info(&page_ranks);
                metrics_client.increment("rank_cycles_completed").await?;
                metrics::RANK_CALCULATION_COMPLETED_TOTAL.inc();
            }
            Err(e) => {
                error!("Error updating ranks: {}", e);
                metrics_client.increment("rank_errors").await?;
            }
        }
        
        metrics::RANK_CALCULATION_DURATION_SECONDS.observe(timer.elapsed_secs());
        metrics_client.observe_histogram("rank_calculation_duration_seconds", timer.elapsed_secs()).await?;
    }
}
