mod postgre_functions;
mod elastic_search_storage;
mod content_processing;
mod document_models;
mod async_processor;
mod searcher;

use std::env;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use log::error;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use postgre_functions::fetch_unprocessed_docs;
use elastic_search_storage::{get_elasticsearch_client, store_processed_document_in_es};
use content_processing::process_content;
use document_models::{html_Docs, processed_doc};
use async_processor::concurrent_process_docs;
use searcher::search_documents;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let database_URL = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");


    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&database_URL)
        .await?;

    // Initialize Elasticsearch client
    let es_client = Arc::new(get_elasticsearch_client().await?);

    // Example of calling the search function (you can remove this if you want)
    if let Err(e) = search_documents(&es_client, "your_keyword").await {
        error!("Search error: {}", e);
    }
    
    loop {
        let client_clone = Arc::clone(&es_client); // Cloning the Arc, not the client itself
        if let Err(e) = concurrent_process_docs(pool.clone(), client_clone).await {
            error!("Error processing docs: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}