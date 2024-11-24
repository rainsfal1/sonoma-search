mod errors;
mod postgres_storage;
mod bm25_es_searcher;

use std::collections::HashMap;
use dotenv::dotenv;
use log::{info, error, debug};
use uuid::Uuid;
use serde_json::Value;

use crate::postgres_storage::{connect_to_db, avg_score};
use crate::bm25_es_searcher::{initialize_client, fetch_bm25_scores};
use crate::errors::SearchError;

#[tokio::main]
async fn main() -> Result<(), SearchError> {
    // Initialize logger with debug level for development
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("Starting search process");
    dotenv().ok();
    
    debug!("Initializing database connection");
    let pool = match connect_to_db().await {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        },
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(e);
        }
    };

    info!("Initializing Elasticsearch client");
    let client = match initialize_client("http://localhost:9200") {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to initialize Elasticsearch client");
            println!("\nError: {}", e);
            println!("Please ensure Elasticsearch is running and accessible.");
            return Err(e);
        }
    };

    println!("\nEnter your search query:");
    let mut query = String::new();
    std::io::stdin().read_line(&mut query)?;
    let query = query.trim();

    if query.is_empty() {
        error!("Empty search query provided");
        return Err(SearchError::InvalidInput("Query cannot be empty".into()));
    }

    info!("Processing search query: {}", query);
    let bm25_scores: HashMap<Uuid, f64> = fetch_bm25_scores(&client, query).await?;
    
    info!("Found {} matching documents", bm25_scores.len());
    if bm25_scores.is_empty() {
        println!("No results found for your query.");
        debug!("Checking if documents exist in Elasticsearch...");
        
        // Add a check to verify documents in ES
        let response = client
            .count(elasticsearch::CountParts::Index(&["processed_docs"]))
            .send()
            .await?;
        
        let body: Value = response.json().await?;
        let doc_count = body["count"].as_u64().unwrap_or(0);
        info!("Total documents in Elasticsearch: {}", doc_count);
        
        return Ok(());
    }

    info!("Fetching document details from database");
    let results = avg_score(&pool, &bm25_scores).await?;

    println!("\nSearch Results ({} found):", results.len());
    println!("======================");

    // Sort results by score in descending order
    let mut sorted_results: Vec<_> = results.into_iter().collect();
    sorted_results.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap());

    // Display top results with better formatting
    for (doc_id, (url, score)) in sorted_results.iter().take(10) {
        println!("\n📄 Document: {}", url);
        println!("   Score: {:.4}", score);
        println!("   ID: {}", doc_id);
        println!("   ----------------------");
    }

    if sorted_results.len() > 10 {
        println!("\n... and {} more results", sorted_results.len() - 10);
    }

    Ok(())
}

