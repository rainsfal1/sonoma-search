use sqlx::PgPool;
use uuid::Uuid;
use sqlx::types::Uuid as SqlxUuid;
use crate::error::{IndexerError, IndexerResult};
use crate::document_models::HtmlDocs;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use sqlx::types::JsonValue;
use chrono::{DateTime, Utc};

// Create a database-specific struct that matches our table
#[derive(sqlx::FromRow)]
struct DbHtmlDoc {
    id: SqlxUuid,
    url: String,
    domain: String,
    content_summary: Option<String>,
    title: Option<String>,
    meta_title: Option<String>,
    meta_description: Option<String>,
    meta_keywords: Option<String>,
    metadata: Option<JsonValue>,
    page_rank: Option<f64>,
    ranked: Option<bool>,
    last_ranked_at: Option<DateTime<Utc>>
}

const MAX_RETRIES: u32 = 3;
pub async fn fetch_unprocessed_docs(pool: &PgPool, limit: i64) -> IndexerResult<Vec<HtmlDocs>> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match sqlx::query_as!(
            DbHtmlDoc,
            r#"
            SELECT 
                id, url, domain, title, content_summary, 
                meta_title, meta_description, meta_keywords, 
                metadata, page_rank, 
                COALESCE(ranked, false) as ranked,
                last_ranked_at
            FROM webpages
            WHERE processed = FALSE AND ranked = TRUE
            ORDER BY page_rank DESC
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
                        domain: db_doc.domain,
                        content_summary: db_doc.content_summary,
                        title: db_doc.title,
                        meta_title: db_doc.meta_title,
                        meta_description: db_doc.meta_description,
                        meta_keywords: db_doc.meta_keywords,
                        metadata: db_doc.metadata,
                        page_rank: db_doc.page_rank.unwrap_or(0.0),
                        ranked: db_doc.ranked.unwrap_or(false),
                        last_ranked_at: db_doc.last_ranked_at,
                        content_hash: String::new(),
                        fetch_timestamp: Utc::now(),
                        last_updated_timestamp: None,
                        status: "pending".to_string(),
                        links: Vec::new()
                    })
                    .collect());
            }
            Err(e) => {
                eprintln!("Attempt {}: Failed to fetch documents: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(IndexerError::Retry(
                format!("Failed to fetch documents after {} attempts", MAX_RETRIES)
            ));
        }

        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(IndexerError::Retry("Max retries reached".to_string()))
}

pub async fn mark_as_processed(pool: &PgPool, doc_id: Uuid) -> IndexerResult<()> {
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
            return Err(IndexerError::Retry(
                format!("Failed to mark document as processed after {} attempts", MAX_RETRIES)
            ));
        }

        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(IndexerError::Retry("Max retries reached".to_string()))
}