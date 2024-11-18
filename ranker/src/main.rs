mod page_rank;
mod postgres_utilities;
mod display_rank;
mod data_models;

use postgres_utilities::{connect_to_db, fetch_links, update_page_ranks};
use page_rank::{prepare_page_links, calculate_page_rank};
use display_rank::display_rank_info;
use log::{info, error, debug};
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with default INFO level
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Starting PageRank calculation process");
    debug!("Initializing database connection");
    
    let pool = connect_to_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        e
    })?;

    info!("Fetching link data from database");
    let rows = fetch_links(&pool).await.map_err(|e| {
        error!("Failed to fetch links: {}", e);
        e
    })?;

    debug!("Preparing page links for PageRank calculation");
    let (page_links, unique_pages) = prepare_page_links(rows);

    info!("PageRank calculation statistics:");
    info!("Total unique pages: {}", unique_pages.len());
    info!("Pages with outgoing links: {}", page_links.len());

    let damping_factor = 0.85;
    let iterations = 10;
    
    info!("Starting PageRank calculation with damping factor {} and {} iterations", 
          damping_factor, iterations);
    let ranks = calculate_page_rank(&page_links, damping_factor, iterations);

    debug!("Displaying rank information");
    display_rank_info(&ranks);

    info!("Updating PageRank scores in database");
    if let Err(e) = update_page_ranks(&pool, &ranks).await {
        error!("Failed to update PageRank scores: {}", e);
        return Err(e.into());
    }

    info!("PageRank calculation completed successfully");
    Ok(())
}
