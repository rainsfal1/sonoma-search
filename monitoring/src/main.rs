mod metrics;
mod server;

use actix_web::{App, HttpServer};
use log::{info, error};
use std::env;

const DEFAULT_PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Register metrics
    metrics::register_metrics();

    // Get port from environment variable or use default
    let port = env::var("MONITORING_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let addr = format!("0.0.0.0:{}", port);

    info!("Starting monitoring server on {}", addr);

    HttpServer::new(|| {
        App::new()
            .configure(server::configure_routes)
    })
    .bind(&addr)?
    .run()
    .await
    .map_err(|e| {
        error!("Server error: {}", e);
        e
    })
}
