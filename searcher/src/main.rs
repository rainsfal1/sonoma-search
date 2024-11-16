mod errors;
mod postgres_storage;
mod bm25_es_searcher;

use crate::postgres_storage::{connect_to_db, avg_score};
use crate::bm25_es_searcher::{initialize_client, fetch_bm25_scores};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Database connection
    let pool = connect_to_db().await?;
    println!("Connected to the database!");

    // Initialize Elasticsearch client
    let client = initialize_client("http://localhost:9200")?;
    println!("Connected to Elasticsearch!");

    // Get user query input
    println!("Enter your query: ");
    let mut query = String::new();
    std::io::stdin().read_line(&mut query)?;
    let query = query.trim();

    if query.is_empty() {
        eprintln!("Query cannot be empty!");
        return Err("Empty query provided.".into());
    }

    // Fetch BM25 scores from Elasticsearch
    let bm25_scores: HashMap<Uuid, f64> = fetch_bm25_scores(&client, query).await?;
    println!("\nTotal documents retrieved: {}", bm25_scores.len());

    // Calculate average scores and URLs
    let average_scores = avg_score(&pool, &bm25_scores).await?;

    // Display results
    for (doc_id, (url, avg_score)) in average_scores {
        println!("Document ID: {}, URL: {}, Average Score: {}", doc_id, url, avg_score);
    }

    Ok(())
}

