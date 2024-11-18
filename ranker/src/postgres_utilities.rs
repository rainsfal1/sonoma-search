use sqlx::postgres::PgPoolOptions;
use std::{collections, env};
use dotenv::dotenv;
use sqlx::{Error, PgPool};
use collections::HashMap;
use uuid::Uuid;

use crate::data_models::LinkStorage;

pub async fn connect_to_db() -> Result<PgPool, Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await?;

    println!("Connected to the database!");
    Ok(pool)
}

pub async fn fetch_links(pool: &PgPool) -> Result<Vec<LinkStorage>, Error> {
    let rows = sqlx::query_as::<_, LinkStorage>(
        "SELECT DISTINCT source_webpage_id, target_url
         FROM links
         WHERE target_url LIKE 'http%'
         AND source_webpage_id IS NOT NULL
         LIMIT 10"
    )
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_page_ranks(pool: &PgPool, ranks: &HashMap<Uuid, f64>) -> Result<(), Error> {
    for (id, score) in ranks {
        sqlx::query("UPDATE webpages SET page_rank = $1 WHERE id = $2")
            .bind(score)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
