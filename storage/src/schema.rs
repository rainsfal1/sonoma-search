use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webpage {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub content_summary: Option<String>,
    pub fetch_timestamp: DateTime<Utc>,
    pub last_updated_timestamp: Option<DateTime<Utc>>,
    pub status: Option<i32>,
    pub content_hash: Option<String>,
    pub metadata: Option<Value>,
    pub links: Vec<Link>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub source_webpage_id: Uuid,
    pub target_url: String,
    pub anchor_text: Option<String>,
}

impl Webpage {
    pub fn new(url: String) -> Result<Self, url::ParseError> {
        let parsed_url = Url::parse(&url)?;
        Ok(Self {
            id: Uuid::new_v4(),
            url: url.clone(),
            domain: parsed_url.domain().unwrap_or("").to_string(),
            title: None,
            content_summary: None,
            fetch_timestamp: Utc::now(),
            last_updated_timestamp: None,
            status: None,
            content_hash: None,
            metadata: None,
            links: Vec::new(),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        })
    }
}
