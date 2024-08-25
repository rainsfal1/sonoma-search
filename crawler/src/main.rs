// main.rs

mod config;
mod crawler;
mod fetcher;
mod parser;
mod storage;
mod robots;

use config::Config;
use crawler::Crawler;
use fetcher::create_http_client;
use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_file("crawler/config.toml")?;
    let client = create_http_client()?;

    let crawler = Crawler::new(client, config);
    crawler.crawl().await?;

    Ok(())
}
