//schema.rs
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Represents a webpage entry in the database.
#[derive(sqlx::FromRow)]
pub struct Webpage {
    pub id: Uuid,                            // Unique identifier (UUID)
    pub url: String,                         // Webpage URL (must be unique)
    pub title: Option<String>,               // Optional title of the webpage
    pub content: Option<String>,             // Optional extracted textual content
    pub html_content: Option<String>,        // Optional raw HTML content
    pub fetch_timestamp: DateTime<Utc>,      // Timestamp when webpage was fetched
    pub last_updated_timestamp: Option<DateTime<Utc>>, // Optional timestamp when last updated
    pub status: Option<i32>,                 // HTTP status code (e.g., 200, 404)
    pub content_hash: Option<String>,        // Optional content hash for detecting changes
    pub metadata: Option<Value>,             // Optional JSON metadata
}

/// Represents a link between webpages (source and target).
#[derive(sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,                              // Unique identifier (BIGSERIAL)
    pub source_webpage_id: Uuid,              // Foreign key referencing `webpages` table
    pub target_url: String,                   // Target URL
    pub anchor_text: Option<String>,          // Optional anchor text
    pub is_internal: Option<bool>,            // Whether the link is internal (true/false)
}

/// Creates the database schema (tables and indexes) if it does not already exist.
pub async fn create_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create `webpages` table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS webpages (
            id UUID PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            content TEXT,
            html_content TEXT,
            fetch_timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            last_updated_timestamp TIMESTAMPTZ,
            status INTEGER,
            content_hash TEXT,
            metadata JSONB
        )
        "#
    )
        .execute(pool)
        .await?;

    // Create index for URL to speed up lookups
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_webpages_url ON webpages(url)
        "#
    )
        .execute(pool)
        .await?;

    // Create full-text search index for title and content using GIN (Generalized Inverted Index)
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_webpages_content_gin ON webpages
        USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, '')))
        "#
    )
        .execute(pool)
        .await?;

    // Create GIN index for metadata (JSONB)
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_webpages_metadata ON webpages
        USING gin(metadata)
        "#
    )
        .execute(pool)
        .await?;

    // Create `links` table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS links (
            id BIGSERIAL PRIMARY KEY,
            source_webpage_id UUID REFERENCES webpages(id) ON DELETE CASCADE,
            target_url TEXT NOT NULL,
            anchor_text TEXT,
            is_internal BOOLEAN
        )
        "#
    )
        .execute(pool)
        .await?;

    // Index for faster lookup on `source_webpage_id`
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_links_source_webpage_id ON links(source_webpage_id)
        "#
    )
        .execute(pool)
        .await?;

    // Index for faster lookup on `target_url`
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_links_target_url ON links(target_url)
        "#
    )
        .execute(pool)
        .await?;

    Ok(())
}

/// Drops the existing database schema (removes `webpages` and `links` tables).
pub async fn drop_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Drop `links` table if it exists
    sqlx::query!("DROP TABLE IF EXISTS links")
        .execute(pool)
        .await?;

    // Drop `webpages` table if it exists
    sqlx::query!("DROP TABLE IF EXISTS webpages")
        .execute(pool)
        .await?;

    Ok(())
}
