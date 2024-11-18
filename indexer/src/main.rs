mod db_indexer;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod searcher;

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
use elasticsearch::Elasticsearch;
use crate::searcher::print_search_results;

#[tokio::main]
async fn main() -> Result<()> {
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
        .unwrap_or_else(|_| String::from("postgres://rainsfall:262912@localhost:5432/search_engine"));
        
    println!("Attempting to connect to database...");
    println!("Using connection string structure: postgres://user:pass@host:port/dbname");

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            eprintln!("Please ensure PostgreSQL is running and the connection details are correct");
            e
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

    // // Example of calling the search function (you can remove this if you want)
    if let Err(e) = print_search_results(&es_client, "w3schools").await {
        error!("Search error: {}", e);
    }
    // let client = Elasticsearch::default();
    // let keyword = "w3schools";
    //
    // print_search_results(&client, keyword).await?;
    //
    loop {
        let client_clone = Arc::clone(&es_client); // Cloning the Arc, not the client itself
        if let Err(e) = concurrent_process_docs(pool.clone(), client_clone).await {
            error!("Error processing docs: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// export DATABASE_URL="postgres://postgres:ptcl12345@localhost/postgres"
