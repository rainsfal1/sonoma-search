mod db_indexer;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod error;

use std::env;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use log::{error};
use elastic_search_storage::{get_elasticsearch_client, ensure_index_exists};
use async_processor::concurrent_process_docs;
use crate::error::{IndexerError, IndexerResult}; // Add the custom error type
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> IndexerResult<()> {
    dotenv().ok();
    env_logger::init();

    // Print current working directory to verify .env location
    println!("Current directory: {:?}", std::env::current_dir()?);

    // Print all environment variables (be careful with sensitive data)
    for (key, value) in env::vars() {
        if key == "DATABASE_URL" {
            println!("Found DATABASE_URL in environment");
            println!("URL structure: {}", value.split("://").next().unwrap_or("invalid"));
        }
    }

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("postgres://postgres:ptcl12345@localhost:5432/postgres"));

    println!("Attempting to connect to database...");
    println!("Using connection string structure: postgres://user:pass@host:port/dbname");

    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&database_url)
        .await
        .map_err(|e| IndexerError::Database(e))?;

    // Test the connection with a simple query
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            IndexerError::Database(e)
        })?;

    println!("Successfully connected to database");

    // Initialize Elasticsearch client
    let es_client = Arc::new(get_elasticsearch_client().await?);

    // Ensure index exists in Elasticsearch
    ensure_index_exists(&es_client).await.map_err(|e| {
        eprintln!("Error ensuring index exists: {}", e);
        e
    })?;

    loop {
        let client_clone = Arc::clone(&es_client); // Cloning the Arc, not the client itself
        if let Err(e) = concurrent_process_docs(pool.clone(), client_clone).await {
            error!("Error processing docs: {}", e);
            return Err(e);  // Return error if document processing fails
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// export DATABASE_URL="postgres://postgres:ptcl12345@localhost:5432/postgres?connect_timeout=5"
