use chrono::Utc;
use sqlx::{postgres::{PgPool, PgPoolOptions}};
use crate::schema::{Webpage, Link};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid data error: {0}")]
    DataError(String),
}

pub struct PostgresStorage {
    pub pool: Arc<PgPool>,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(database_url)
            .await?;

        Ok(Self { pool: Arc::new(pool) })
    }

    pub async fn save_webpage(&self, webpage: &Webpage) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT INTO webpages (
                id, url, domain, title, content_summary, fetch_timestamp, 
                last_updated_timestamp, status, content_hash, metadata,
                meta_title, meta_description, meta_keywords,
                ranked, last_ranked_at, page_rank
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (url) DO UPDATE
            SET domain = EXCLUDED.domain,
                title = COALESCE(webpages.title, EXCLUDED.title),
                content_summary = COALESCE(webpages.content_summary, EXCLUDED.content_summary),
                last_updated_timestamp = EXCLUDED.last_updated_timestamp,
                status = COALESCE(webpages.status, EXCLUDED.status),
                content_hash = COALESCE(webpages.content_hash, EXCLUDED.content_hash),
                metadata = COALESCE(webpages.metadata, EXCLUDED.metadata),
                meta_title = COALESCE(webpages.meta_title, EXCLUDED.meta_title),
                meta_description = COALESCE(webpages.meta_description, EXCLUDED.meta_description),
                meta_keywords = COALESCE(webpages.meta_keywords, EXCLUDED.meta_keywords),
                ranked = COALESCE(webpages.ranked, EXCLUDED.ranked),
                last_ranked_at = COALESCE(webpages.last_ranked_at, EXCLUDED.last_ranked_at),
                page_rank = COALESCE(webpages.page_rank, EXCLUDED.page_rank)
            "#,
            webpage.id,
            webpage.url,
            webpage.domain,
            webpage.title,
            webpage.content_summary,
            webpage.fetch_timestamp,
            webpage.last_updated_timestamp,
            webpage.status,
            webpage.content_hash,
            webpage.metadata.as_ref().map(|m| serde_json::to_value(m).unwrap()),
            webpage.meta_title,
            webpage.meta_description,
            webpage.meta_keywords,
            webpage.ranked,
            webpage.last_ranked_at,
            webpage.page_rank
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_link(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        link: &Link
    ) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT INTO links (id, source_webpage_id, target_url, anchor_text)
            VALUES ($1, $2, $3, $4)
            "#,
            link.id,
            link.source_webpage_id,
            link.target_url,
            link.anchor_text,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get_webpage(&self, id: Uuid) -> Result<Option<Webpage>, StorageError> {
        let webpage = sqlx::query!(
            r#"
            SELECT id, url, domain, title, content_summary, fetch_timestamp, last_updated_timestamp, status, content_hash, metadata as "metadata: Value", meta_title, meta_description, meta_keywords, ranked, last_ranked_at, page_rank
            FROM webpages
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(|row| Webpage {
            id: row.id,
            url: row.url,
            domain: row.domain,
            title: row.title,
            content_summary: row.content_summary,
            fetch_timestamp: row.fetch_timestamp.unwrap_or_else(|| Utc::now()),
            last_updated_timestamp: row.last_updated_timestamp,
            status: row.status,
            content_hash: row.content_hash,
            metadata: row.metadata,
            links: Vec::new(),
            meta_title: row.meta_title,
            meta_description: row.meta_description,
            meta_keywords: row.meta_keywords,
            ranked: row.ranked.unwrap_or(false),
            last_ranked_at: row.last_ranked_at,
            page_rank: row.page_rank.unwrap_or(0.0),
        });

        Ok(webpage)
    }

    pub async fn search_webpages(&self, query: &str, limit: i64) -> Result<Vec<Webpage>, StorageError> {
        let webpages = sqlx::query!(
            r#"
            SELECT id, url, domain, title, content_summary, fetch_timestamp, 
                   last_updated_timestamp, status, content_hash, metadata as "metadata: Value", 
                   meta_title, meta_description, meta_keywords, 
                   ranked, last_ranked_at, page_rank
            FROM webpages
            WHERE to_tsvector('english', coalesce(title, '') || ' ' || 
                  coalesce(content_summary, '') || ' ' || coalesce(meta_title, '') || 
                  ' ' || coalesce(meta_description, '') || ' ' || 
                  coalesce(meta_keywords, '')) @@ plainto_tsquery('english', $1)
            ORDER BY 
                ts_rank(to_tsvector('english', coalesce(title, '') || ' ' || 
                       coalesce(content_summary, '') || ' ' || coalesce(meta_title, '') || 
                       ' ' || coalesce(meta_description, '') || ' ' || 
                       coalesce(meta_keywords, '')), 
                       plainto_tsquery('english', $1)) * (1 + page_rank) DESC
            LIMIT $2
            "#,
            query,
            limit
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(|row| Webpage {
            id: row.id,
            url: row.url,
            domain: row.domain,
            title: row.title,
            content_summary: row.content_summary,
            fetch_timestamp: row.fetch_timestamp.unwrap_or_else(|| Utc::now()),
            last_updated_timestamp: row.last_updated_timestamp,
            status: row.status,
            content_hash: row.content_hash,
            metadata: row.metadata,
            links: Vec::new(),
            meta_title: row.meta_title,
            meta_description: row.meta_description,
            meta_keywords: row.meta_keywords,
            ranked: row.ranked.unwrap_or(false),
            last_ranked_at: row.last_ranked_at,
            page_rank: row.page_rank.unwrap_or(0.0),
        })
        .collect();

        Ok(webpages)
    }

    pub async fn delete_webpage(&self, url: &str) -> Result<bool, StorageError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM webpages
            WHERE url = $1
            "#,
            url
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn search_by_domain(&self, domain: &str, limit: i64) -> Result<Vec<Webpage>, StorageError> {
        let webpages = sqlx::query!(
            r#"
            SELECT id, url, domain, title, content_summary, fetch_timestamp, 
                    last_updated_timestamp, status, content_hash, 
                    metadata as "metadata: Value", meta_title, 
                    meta_description, meta_keywords, 
                    ranked, last_ranked_at, page_rank
            FROM webpages
            WHERE domain = $1
            LIMIT $2
            "#,
            domain,
            limit
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(|row| Webpage {
            id: row.id,
            url: row.url,
            domain: row.domain,
            title: row.title,
            content_summary: row.content_summary,
            fetch_timestamp: row.fetch_timestamp.unwrap_or_else(|| Utc::now()),
            last_updated_timestamp: row.last_updated_timestamp,
            status: row.status,
            content_hash: row.content_hash,
            metadata: row.metadata,
            links: Vec::new(),
            meta_title: row.meta_title,
            meta_description: row.meta_description,
            meta_keywords: row.meta_keywords,
            ranked: row.ranked.unwrap_or(false),
            last_ranked_at: row.last_ranked_at,
            page_rank: row.page_rank.unwrap_or(0.0),
        })
        .collect();

        Ok(webpages)
    }

    pub async fn update_webpage_rank(&self, id: Uuid, page_rank: f64) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            UPDATE webpages
            SET page_rank = $1, ranked = TRUE, last_ranked_at = NOW()
            WHERE id = $2
            "#,
            page_rank,
            id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}