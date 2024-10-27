use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use crate::document_models::html_Docs;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

const MAX_RETRIES: u32 = 3;
pub async fn fetch_unprocessed_docs(pool: &PgPool, limit: i64) -> Result<Vec<html_Docs>> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match sqlx::query_as!(html_Docs,
            r#"
            SELECT id, url, content, html_content, title
            FROM webpages
            WHERE processed = FALSE
            LIMIT $1
            "#,
            limit
        ).fetch_all(pool).await
        {
            Ok(records) => return Ok(records),
            Err(e) => {
                eprintln!("Attempt {}: Failed to fetch unprocessed documents from PostgreSQL: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(anyhow!("Failed to fetch unprocessed documents after {} attempts", MAX_RETRIES));
        }

        // Exponential backoff with some random jitter
        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(anyhow!("Max retries reached for fetching documents from PostgreSQL"))
}


pub async fn mark_as_processed(pool: &PgPool, doc_id: Uuid) -> Result<()> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match sqlx::query!(
            r#"
            UPDATE webpages
            SET processed = TRUE
            WHERE id = $1
            "#,
            doc_id
        )
            .execute(pool)
            .await
        {
            Ok(_) => return Ok(()), // Successful update
            Err(e) => {
                eprintln!("Attempt {}: Failed to mark document as processed in PostgreSQL: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(anyhow!("Failed to mark document as processed after {} attempts", MAX_RETRIES));
        }

        // Exponential backoff with jitter
        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(anyhow!("Max retries reached for marking document as processed in PostgreSQL"))
}
