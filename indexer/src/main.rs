// main.rs

use std::error::Error;
use tokio::main;
use crate::config::Config;
use crate::indexer::Indexer;

mod config;
mod document;
mod indexer;
mod schema;
mod batch_processor;
mod tokenizer;
mod storage;

#[main]
async fn main() {
    match run().await {
        Ok(_) => println!("Indexing completed successfully."),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    let mut indexer = Indexer::new(config)?;
    indexer.run().await?;
    Ok(())
}
