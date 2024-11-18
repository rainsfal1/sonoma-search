mod config;
mod crawler;
mod fetcher;
mod parser;
mod robots;
mod summarizer;
mod bloom;

use std::error::Error;
use tokio::main;
use dotenv::dotenv;
use crate::config::Config;
use crate::crawler::Crawler;
use crate::fetcher::create_http_client;
use storage::PostgresStorage;
use env_logger::Env;
use std::path::PathBuf;

#[main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Load environment variables from the .env file
    dotenv().ok();

    match run().await {
        Ok(_) => println!("Crawling completed successfully."),
        Err(e) => eprintln!("Error occurred: {}\n{:?}", e, e),
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    // Print current working directory
    let current_dir = std::env::current_dir()?;

    // Load the configuration for the crawler
    let config_path = find_config_file(&current_dir);
    let config = Config::from_file(config_path)?;

    // Create the HTTP client
    let client = create_http_client()?;

    // Fetch storage URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize the PostgresStorage instance asynchronously
    let storage = PostgresStorage::new(&database_url).await?;

    // Pass the client, config, and storage to the Crawler
    let crawler = Crawler::new(client, config, storage);

    // Start the crawl process
    crawler.crawl().await?;

    Ok(())
}

fn find_config_file(current_dir: &PathBuf) -> PathBuf {
    let mut config_path = current_dir.join("crawler").join("config.toml");
    if config_path.exists() {
        return config_path;
    }

    config_path = current_dir.join("config.toml");
    if config_path.exists() {
        return config_path;
    }

    current_dir.join("crawler").join("config.toml") // Default path if not found
}
