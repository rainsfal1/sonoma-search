use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlDocs {
    pub id: Uuid,
    pub url: String,
    pub content_summary: Option<String>,
    pub title: Option<String>,
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
}