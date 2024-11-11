mod db_indexer;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod searcher;
mod error;

use std::env;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use log::{error};
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
// use postgre_functions::fetch_unprocessed_docs;
use elastic_search_storage::{get_elasticsearch_client, ensure_index_exists}; // store_processed_document_in_es
// use content_processing::process_content;
// use document_models::{html_Docs, processed_doc};
use async_processor::concurrent_process_docs;
use searcher::search_documents;
use crate::error::{IndexerError, IndexerResult};

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

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            IndexerError::Database(e)
        })?;

    // Test the connection with a simple query
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => println!("Database connection test successful"),
        Err(e) => {
            eprintln!("Database connection test failed: {}", e);
            return Err(e.into());
        }
    }

    println!("Successfully connected to database");

    // Initialize Elasticsearch client
    let es_client = Arc::new(get_elasticsearch_client().await?);

    // Ensure index exists in Elasticsearch
    // ensure_index_exists(&es_client).await.map_err(|e| {
    //     eprintln!("Error ensuring index exists: {}", e);
    //     e
    // })?;

    // Example of calling the search function (you can remove this if you want)
    if let Err(e) = search_documents(&es_client, "w3schools").await {
        error!("Search error: {}", e);
    }
    
    // Add graceful shutdown handling
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    
    // Handle Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await {
            let _ = shutdown_tx_clone.send(());
        }
    });

    loop {
        let client_clone = Arc::clone(&es_client);
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                if let Err(e) = concurrent_process_docs(pool.clone(), client_clone).await {
                    error!("Error processing docs: {}", e);
                }
            }
            _ = shutdown_rx => {
                println!("Shutting down gracefully...");
                break;
            }
        }
    }

    Ok(())
}