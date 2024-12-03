use sqlx::{Error, PgPool};
use std::collections::HashMap;
use uuid::Uuid;
use log::{info, error, debug};

use crate::data_models::LinkStorage;

pub async fn fetch_links(pool: &PgPool) -> Result<Vec<LinkStorage>, Error> {
    debug!("Fetching links from database...");
    
    // Using the existing compound index idx_links_source_target
    let rows = sqlx::query_as::<_, LinkStorage>(
        "SELECT source_webpage_id, target_url
         FROM links
         WHERE source_webpage_id IS NOT NULL 
           AND target_url LIKE 'http%'
         ORDER BY source_webpage_id, target_url"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch links: {}", e);
        e
    })?;
    
    info!("Successfully fetched {} links", rows.len());
    Ok(rows)
}

pub async fn update_page_ranks(pool: &PgPool, ranks: &HashMap<Uuid, f64>) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    
    for (id, score) in ranks {
        sqlx::query!(
            "UPDATE webpages 
             SET page_rank = $1, ranked = TRUE, last_ranked_at = NOW() 
             WHERE id = $2",
            score,
            id
        )
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(())
}
