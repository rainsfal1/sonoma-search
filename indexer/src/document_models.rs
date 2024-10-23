use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct html_Docs {
    pub id: Uuid,
    pub url: String,
    pub content: String,
    pub html_content: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct processed_doc {
    pub processed_doc_webpage_id: Uuid,
    pub processed_doc_title: Option<String>,
    pub processed_doc_body: Option<String>,
    pub processed_doc_indexed_at: DateTime<Utc>,
    pub processed_doc_metadata: Option<serde_json::Value>,
    pub processed_doc_content_summary: Option<String>,
    pub processed_doc_keywords: Option<Vec<String>>,
}
