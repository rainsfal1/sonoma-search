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

/// Represents a parsed webpage with all its extracted information
/// 
/// The URL field is a critical component used across multiple services:
/// 1. Storage: Used as a unique identifier in the database to prevent duplicate crawls
/// 2. Link Processing: Used to build the web graph for PageRank calculations
/// 3. Crawler: Used to track visited pages and maintain the crawl frontier
/// 4. Integration: Required by ranker and indexer services for page identification
#[allow(dead_code)]  // Field is used by other services
pub struct ParsedWebpage {
    /// The original URL of the webpage. This field is essential for:
    /// - Database uniqueness constraints (UNIQUE in webpages table)
    /// - Link graph construction for PageRank
    /// - Deduplication in the crawler's visited set
    /// - Integration with storage, indexer, and ranker services
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
    let mut content = String::new();

    // Wikipedia-specific content extraction
    if let Ok(wiki_content) = Selector::parse("#mw-content-text") {
        if let Some(element) = parsed_html.select(&wiki_content).next() {
            // Remove navigation, tables, and other non-content elements
            if let Ok(remove_selector) = Selector::parse(".navbox, .vertical-navbox, .infobox, .sidebar, table, .mw-editsection, .reference, .error") {
                let mut html = element.html();
                for el in parsed_html.select(&remove_selector) {
                    html = html.replace(&el.html(), "");
                }
                let fragment = Html::parse_fragment(&html);
                content = fragment.root_element().text().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // If not Wikipedia or no content found, try other common content areas
    if content.is_empty() {
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
    }

    // If still no content, fall back to body
    if content.is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = parsed_html.select(&body_selector).next() {
                content = body.text().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // Clean up the content
    let content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    // Basic content cleaning
    let content = content
        .replace("[edit]", "")
        .replace("Jump to navigation", "")
        .replace("Jump to search", "");

    Some(content).filter(|s| !s.is_empty() && s.split_whitespace().count() > 10)
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn extract_metadata(document: &Html) -> (Option<String>, Option<String>, Option<String>, Option<Value>) {
    let mut meta_title = None;
    let mut meta_description = None;
    let mut meta_keywords = None;
    
    // Try to get Wikipedia-specific metadata
    if let Ok(sitename_selector) = Selector::parse("meta[property='og:site_name']") {
        if let Some(element) = document.select(&sitename_selector).next() {
            meta_title = element.value().attr("content").map(|s| s.to_string());
        }
    }

    // Get standard meta tags
    if let Ok(selector) = Selector::parse("meta[name='description'], meta[property='og:description']") {
        if let Some(element) = document.select(&selector).next() {
            meta_description = element.value().attr("content").map(|s| s.to_string());
        }
    }

    if let Ok(selector) = Selector::parse("meta[name='keywords']") {
        if let Some(element) = document.select(&selector).next() {
            meta_keywords = element.value().attr("content").map(|s| s.to_string());
        }
    }

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
