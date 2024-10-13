mod config;
mod crawler;
mod fetcher;
mod parser;
mod robots;
mod summarizer;

use std::error::Error;
use tokio::main;
use dotenv::dotenv;
use crate::config::Config;
use crate::crawler::Crawler;
use crate::fetcher::create_http_client;
use storage::PostgresStorage;
use env_logger::Env;

#[main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Load environment variables from the .env file
    dotenv().ok();

    match run().await {
        Ok(_) => println!("Crawling completed successfully."),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    // Load the configuration for the crawler
    let config = Config::from_file("crawler/config.toml")?;

    // Create the HTTP client
    let client = create_http_client()?;

    // Fetch database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize the PostgresStorage instance asynchronously
    let storage = PostgresStorage::new(&database_url).await?;

    // Pass the client, config, and storage to the Crawler
    let crawler = Crawler::new(client, config, storage);

    // Start the crawl process
    crawler.crawl().await?;

    Ok(())
}
