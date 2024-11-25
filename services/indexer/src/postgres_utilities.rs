use sqlx::postgres::{PgPool, PgPoolOptions};
use std::error::Error;

pub async fn create_pool(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn check_connection(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    if row.0 != 1 {
        return Err("Database connection check failed".into());
    }
    
    Ok(())
}
