use scraper::{Html, Selector};
use thiserror::Error;
use url::{Url, ParseError};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

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
    pub domain: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub fetch_timestamp: DateTime<Utc>,
    pub last_updated_timestamp: Option<DateTime<Utc>>,
    pub status: Option<i32>,
    pub content_hash: String,
    pub metadata: Option<Value>,
    pub links: Vec<ParsedLink>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
}

pub struct ParsedLink {
    pub target_url: String,
    pub anchor_text: Option<String>,
}

pub fn parse_webpage(html: &str, url: &str, status: i32) -> Result<ParsedWebpage, ParserError> {
    let document = Html::parse_document(html);
    let parsed_url = Url::parse(url)?;
    let domain = parsed_url.domain().unwrap_or("").to_string();

    let title = extract_title(&document);
    let content = extract_content(&document); // This is still full content, to be summarized later
    let links = extract_links(&document, &parsed_url)?;
    let content_hash = calculate_hash(html);
    let (meta_title, meta_description, meta_keywords, other_metadata) = extract_metadata(&document);

    Ok(ParsedWebpage {
        url: url.to_string(),
        domain,
        title,
        content, // This will be summarized in the crawler
        fetch_timestamp: Utc::now(),
        last_updated_timestamp: None,
        status: Some(status),
        content_hash,
        metadata: other_metadata,
        links,
        meta_title,
        meta_description,
        meta_keywords,
    })
}

fn extract_links(parsed_html: &Html, base_url: &Url) -> Result<Vec<ParsedLink>, ParserError> {
    let selector = Selector::parse("a[href]")?;
    let mut links = Vec::new();

    for element in parsed_html.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(resolved_url) = base_url.join(href) {
                let anchor_text = element.text().collect::<String>().trim().to_string();
                links.push(ParsedLink {
                    target_url: resolved_url.into(),
                    anchor_text: Some(anchor_text).filter(|s| !s.is_empty()),
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
        .take(1000) 
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

fn extract_metadata(document: &Html) -> (Option<String>, Option<String>, Option<String>, Option<Value>) {
    let meta_title = document.select(&Selector::parse("meta[property='og:title']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .or_else(|| document.select(&Selector::parse("title").unwrap()).next().map(|el| el.inner_html()));

    let meta_description = document.select(&Selector::parse("meta[name='description']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let meta_keywords = document.select(&Selector::parse("meta[name='keywords']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let other_metadata = extract_other_metadata(document);

    (meta_title, meta_description, meta_keywords, other_metadata)
}

fn extract_other_metadata(document: &Html) -> Option<Value> {
    let mut metadata = json!({});
    if let Some(lang) = document.select(&Selector::parse("html[lang]").unwrap()).next() {
        if let Some(lang_value) = lang.value().attr("lang") {
            metadata["language"] = json!(lang_value);
        }
    }
    // Add more metadata extraction as needed
    if metadata.as_object().unwrap().is_empty() {
        None
    } else {
        Some(metadata)
    }
}

