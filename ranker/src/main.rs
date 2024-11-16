mod page_rank;
mod postgres_utilities;
mod display_rank;
mod data_models;

use postgres_utilities::{connect_to_db, fetch_links, update_page_ranks};
use page_rank::{prepare_page_links, calculate_page_rank};
use display_rank::display_rank_info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = connect_to_db().await?;
    let rows = fetch_links(&pool).await?;
    let (page_links, unique_pages) = prepare_page_links(rows);

    println!("Total unique pages: {}", unique_pages.len());
    println!("Pages with outgoing links: {}", page_links.len());

    let damping_factor = 0.85;
    let iterations = 10;
    let ranks = calculate_page_rank(&page_links, damping_factor, iterations);

    display_rank_info(&ranks);

    if let Err(e) = update_page_ranks(&pool, &ranks).await {
        eprintln!("Error updating ranks: {}", e);
    }

    Ok(())
}
