use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlDocs {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub content_summary: Option<String>,
    pub title: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub metadata: Option<JsonValue>,
    pub page_rank: f64,
    pub content_hash: String,
    pub fetch_timestamp: DateTime<Utc>,
    pub last_updated_timestamp: Option<DateTime<Utc>>,
    pub status: String,
    pub links: Vec<String>,
    pub ranked: bool,
    pub last_ranked_at: Option<DateTime<Utc>>,
}

impl HtmlDocs {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedDoc {
    pub processed_doc_webpage_id: Uuid,
    pub processed_doc_title: Option<String>,
    pub processed_doc_body: Option<String>,
    pub processed_doc_indexed_at: DateTime<Utc>,
    pub processed_doc_metadata: Option<serde_json::Value>,
    pub processed_doc_content_summary: Option<String>,
    pub processed_doc_keywords: Option<Vec<String>>,
    pub processed_doc_page_rank: f64
}