use scraper::{Html, Selector};
use thiserror::Error;
use url::{Url, ParseError};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to extract URLs: {0}")]
    UrlExtractionError(String),
    #[error("Selector parse error: {0}")]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ParseError),
}

pub struct ParsedWebpage {
    pub url: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub html_content: String,
    pub fetch_timestamp: DateTime<Utc>,
    pub last_updated_timestamp: Option<DateTime<Utc>>,
    pub status: Option<i32>,
    pub content_hash: String,
    pub metadata: Option<Value>,
    pub links: Vec<ParsedLink>,
}

pub struct ParsedLink {
    pub target_url: String,
    pub anchor_text: Option<String>,
    pub is_internal: bool,
}

pub fn parse_webpage(html: &str, url: &str, status: i32) -> Result<ParsedWebpage, ParserError> {
    let parsed_html = Html::parse_document(html);
    let base_url = Url::parse(url)?;

    let title = extract_title(&parsed_html);
    let content = extract_content(&parsed_html);
    let links = extract_links(&parsed_html, &base_url)?;
    let content_hash = calculate_hash(html);
    let metadata = extract_metadata(&parsed_html);

    Ok(ParsedWebpage {
        url: url.to_string(),
        title,
        content,
        html_content: html.to_string(),
        fetch_timestamp: Utc::now(),
        last_updated_timestamp: None, // This would typically be set when updating an existing record
        status: Some(status),
        content_hash,
        metadata,
        links,
    })
}

fn extract_links(parsed_html: &Html, base_url: &Url) -> Result<Vec<ParsedLink>, ParserError> {
    let selector = Selector::parse("a[href]")?;
    let mut links = Vec::new();

    for element in parsed_html.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(resolved_url) = base_url.join(href) {
                let anchor_text = element.text().collect::<String>().trim().to_string();
                let is_internal = is_internal_link(base_url, &resolved_url);
                links.push(ParsedLink {
                    target_url: resolved_url.into(),
                    anchor_text: Some(anchor_text).filter(|s| !s.is_empty()),
                    is_internal,
                });
            }
        }
    }

    if links.is_empty() {
        Err(ParserError::UrlExtractionError("No valid URLs found".to_string()))
    } else {
        Ok(links)
    }
}

fn extract_title(parsed_html: &Html) -> Option<String> {
    parsed_html
        .select(&Selector::parse("title").unwrap())
        .next()
        .and_then(|title| Some(title.text().collect::<String>().trim().to_string()))
        .filter(|s| !s.is_empty())
}

fn extract_content(parsed_html: &Html) -> Option<String> {
    // More sophisticated content extraction
    let mut content = String::new();

    // Try to find main content area
    let main_selectors = [
        "article", "main", "#content", ".content", ".post-content",
        "[role='main']", ".entry-content", ".post", "#main-content",
    ];

    for selector in main_selectors.iter() {
        if let Ok(sel) = Selector::parse(selector) {
            if let Some(element) = parsed_html.select(&sel).next() {
                content = element.text().collect::<Vec<_>>().join(" ");
                break;
            }
        }
    }

    // If no main content area found, fall back to body
    if content.is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = parsed_html.select(&body_selector).next() {
                content = body.text().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // Remove extra whitespace and trim
    let content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    Some(content).filter(|s| !s.is_empty())
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn is_internal_link(base_url: &Url, link_url: &Url) -> bool {
    base_url.host() == link_url.host()
}

fn extract_metadata(parsed_html: &Html) -> Option<Value> {
    let mut metadata = serde_json::Map::new();

    // Extract Open Graph metadata
    let og_selector = Selector::parse("meta[property^='og:']").unwrap();
    for element in parsed_html.select(&og_selector) {
        if let (Some(property), Some(content)) = (element.value().attr("property"), element.value().attr("content")) {
            metadata.insert(property.to_string(), Value::String(content.to_string()));
        }
    }

    // Extract Twitter Card metadata
    let twitter_selector = Selector::parse("meta[name^='twitter:']").unwrap();
    for element in parsed_html.select(&twitter_selector) {
        if let (Some(name), Some(content)) = (element.value().attr("name"), element.value().attr("content")) {
            metadata.insert(name.to_string(), Value::String(content.to_string()));
        }
    }

    // Extract other relevant metadata (e.g., description, keywords)
    let meta_selector = Selector::parse("meta[name='description'], meta[name='keywords']").unwrap();
    for element in parsed_html.select(&meta_selector) {
        if let (Some(name), Some(content)) = (element.value().attr("name"), element.value().attr("content")) {
            metadata.insert(name.to_string(), Value::String(content.to_string()));
        }
    }

    if metadata.is_empty() {
        None
    } else {
        Some(Value::Object(metadata))
    }
}