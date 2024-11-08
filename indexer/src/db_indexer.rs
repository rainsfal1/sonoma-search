use sqlx::PgPool;
use uuid::Uuid;
use sqlx::types::Uuid as SqlxUuid;
use anyhow::{Result, anyhow};
use crate::document_models::HtmlDocs;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

// Create a database-specific struct that matches our table
#[derive(sqlx::FromRow)]
struct DbHtmlDoc {
    id: SqlxUuid,
    url: String,
    content_summary: Option<String>,
    title: Option<String>,
}

const MAX_RETRIES: u32 = 3;
pub async fn fetch_unprocessed_docs(pool: &PgPool, limit: i64) -> Result<Vec<HtmlDocs>> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match sqlx::query_as!(
            DbHtmlDoc,
            r#"
            SELECT id, url, content_summary, title
            FROM webpages
            WHERE processed = FALSE
            LIMIT $1
            "#,
            limit
        )
            .fetch_all(pool)
            .await
        {
            Ok(db_docs) => {
                // Convert database docs to application docs
                return Ok(db_docs
                    .into_iter()
                    .map(|db_doc| HtmlDocs {
                        id: Uuid::from_bytes(*db_doc.id.as_bytes()),
                        url: db_doc.url,
                        content_summary: db_doc.content_summary,
                        title: db_doc.title,
                    })
                    .collect());
            }
            Err(e) => {
                eprintln!("Attempt {}: Failed to fetch documents: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(anyhow!("Failed to fetch documents after {} attempts", MAX_RETRIES));
        }

        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(anyhow!("Max retries reached"))
}

pub async fn mark_as_processed(pool: &PgPool, doc_id: Uuid) -> Result<()> {
    let sqlx_uuid = SqlxUuid::from_bytes(doc_id.into_bytes());
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match sqlx::query!(
            r#"
            UPDATE webpages
            SET processed = TRUE
            WHERE id = $1
            "#,
            sqlx_uuid
        )
            .execute(pool)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                eprintln!("Attempt {}: Failed to mark document as processed: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(anyhow!("Failed to mark document as processed after {} attempts", MAX_RETRIES));
        }

        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(anyhow!("Max retries reached"))
}