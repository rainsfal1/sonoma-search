//model.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webpage {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub html_content: Option<String>,
    pub fetch_timestamp: DateTime<Utc>,
    pub last_updated_timestamp: Option<DateTime<Utc>>,
    pub status: Option<i32>,
    pub content_hash: Option<String>,
    pub metadata: Option<Value>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub source_webpage_id: Uuid,
    pub target_url: String,
    pub anchor_text: Option<String>,
    pub is_internal: Option<bool>,
}

impl Webpage {
    pub fn new(url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            title: None,
            content: None,
            html_content: None,
            fetch_timestamp: Utc::now(),
            last_updated_timestamp: None,
            status: None,
            content_hash: None,
            metadata: None,
            links: Vec::new(),
        }
    }
}