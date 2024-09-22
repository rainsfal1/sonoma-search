use sqlx::{postgres::{PgPool, PgPoolOptions}};
use crate::models::{Webpage, Link};
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
        let mut transaction = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO webpages (id, url, title, content, html_content, fetch_timestamp, last_updated_timestamp, status, content_hash, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (url) DO UPDATE
            SET title = $3, content = $4, html_content = $5, last_updated_timestamp = $7, status = $8, content_hash = $9, metadata = $10
            "#,
            webpage.id,
            webpage.url,
            webpage.title,
            webpage.content,
            webpage.html_content,
            webpage.fetch_timestamp,
            webpage.last_updated_timestamp,
            webpage.status.map(|s| s.to_string()),
            webpage.content_hash,
            webpage.metadata.as_ref().map(|m| serde_json::to_value(m).unwrap()),
        )
            .execute(&mut *transaction)
            .await?;

        for link in &webpage.links {
            self.save_link(&mut transaction, link).await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn save_link(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, link: &Link) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT INTO links (id, source_webpage_id, target_url, anchor_text, is_internal)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET source_webpage_id = $2, target_url = $3, anchor_text = $4, is_internal = $5
            "#,
            link.id as Uuid,
            link.source_webpage_id,
            link.target_url,
            link.anchor_text,
            link.is_internal,
        )
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn get_webpage(&self, url: &str) -> Result<Option<Webpage>, StorageError> {
        let webpage: Option<Webpage> = sqlx::query!(
        r#"
        SELECT id, url, title, content, html_content, fetch_timestamp, last_updated_timestamp, status, content_hash, metadata as "metadata: Value"
        FROM webpages
        WHERE url = $1
        "#,
        url
    )
            .fetch_optional(&*self.pool)
            .await?
            .map(|row| Webpage {
                id: row.id,
                url: row.url,
                title: row.title,
                content: row.content,
                html_content: row.html_content,
                fetch_timestamp: row.fetch_timestamp,
                last_updated_timestamp: row.last_updated_timestamp,
                status: row.status.and_then(|s| s.parse().ok()),
                content_hash: row.content_hash,
                metadata: row.metadata,
                links: Vec::new(),
            });

        if let Some(mut webpage) = webpage {
            webpage.links = self.get_links_for_webpage(webpage.id).await?;
            Ok(Some(webpage))
        } else {
            Ok(None)
        }
    }


    async fn get_links_for_webpage(&self, webpage_id: Uuid) -> Result<Vec<Link>, StorageError> {
        let links = sqlx::query_as!(
            Link,
            r#"
            SELECT id as "id: Uuid", source_webpage_id, target_url, anchor_text, is_internal
            FROM links
            WHERE source_webpage_id = $1
            "#,
            webpage_id
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(links)
    }

    // Initialize with an empty vector

    pub async fn search_webpages(&self, query: &str, limit: i64) -> Result<Vec<Webpage>, StorageError> {
        let webpages: Vec<Webpage> = sqlx::query!(
        r#"
        SELECT id, url, title, content, html_content, fetch_timestamp, last_updated_timestamp, status, content_hash, metadata as "metadata: Value"
        FROM webpages
        WHERE to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, '')) @@ plainto_tsquery('english', $1)
        ORDER BY ts_rank(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, '')), plainto_tsquery('english', $1)) DESC
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
                title: row.title,
                content: row.content,
                html_content: row.html_content,
                fetch_timestamp: row.fetch_timestamp,
                last_updated_timestamp: row.last_updated_timestamp,
                status: row.status.and_then(|s| s.parse().ok()),
                content_hash: row.content_hash,
                metadata: row.metadata,
                links: Vec::new(), // Initialize with an empty vector
            })
            .collect();

        let mut result: Vec<Webpage> = Vec::new();
        for mut webpage in webpages {
            webpage.links = self.get_links_for_webpage(webpage.id).await?;
            result.push(webpage);
        }

        Ok(result)
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
}