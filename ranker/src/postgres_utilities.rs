use sqlx::postgres::PgPoolOptions;
use std::{collections, time::Duration};
use dotenv::dotenv;
use sqlx::{Error, PgPool};
use collections::HashMap;
use uuid::Uuid;
use log::{info, error, debug};

use crate::data_models::LinkStorage;

pub async fn connect_to_db() -> Result<PgPool, Error> {
    let current_dir = std::env::current_dir().unwrap_or_default();
    
    // Navigate to project root (parent of ranker directory)
    let root_dir = current_dir.parent().unwrap_or(&current_dir);
    
    // Set working directory to project root for .env loading
    if let Err(e) = std::env::set_current_dir(root_dir) {
        error!("Failed to set working directory: {}", e);
    }

    // Clear any existing env vars that might interfere
    std::env::remove_var("DATABASE_URL");
    
    // Check if .env file exists and load it
    debug!("Checking for .env file in: {:?}", root_dir.join(".env"));
    if root_dir.join(".env").exists() {
        debug!(".env file found");
        dotenv().ok();
    } else {
        error!(".env file not found");
    }
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    debug!("Using database URL: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await?;

    info!("Successfully connected to database");
    Ok(pool)
}

pub async fn fetch_links(pool: &PgPool) -> Result<Vec<LinkStorage>, Error> {
    debug!("Fetching links from database...");
    let rows = sqlx::query_as::<_, LinkStorage>(
        "SELECT DISTINCT source_webpage_id, target_url
         FROM links
         WHERE target_url LIKE 'http%'
         AND source_webpage_id IS NOT NULL"
    )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch links: {}", e);
            e
        })?;
    
    info!("Successfully fetched {} links", rows.len());
    Ok(rows)
}

pub async fn update_page_ranks(pool: &PgPool, ranks: &HashMap<Uuid, f64>) -> Result<(), Error> {
    for (id, score) in ranks {
        sqlx::query("UPDATE webpages SET page_rank = $1 WHERE id = $2")
            .bind(score)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
