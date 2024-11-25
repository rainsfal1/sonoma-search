use scraper::{Html, Selector, ElementRef};
use thiserror::Error;
use url::{Url, ParseError};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use log::warn;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] ParseError),

    #[error("Failed to parse selector: {0}")]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),
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
    let links = extract_links(&document, &parsed_url, true)?;
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

pub fn extract_links(parsed_html: &Html, base_url: &Url, respect_nofollow: bool) -> Result<Vec<ParsedLink>, ParserError> {
    let selector = Selector::parse("a[href]").map_err(|e| ParserError::SelectorParseError(e))?;
    let mut links = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    for element in parsed_html.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            // Check for nofollow if configured
            if respect_nofollow {
                if let Some(rel) = element.value().attr("rel") {
                    if rel.contains("nofollow") {
                        continue;
                    }
                }
            }
            
            // Resolve relative URLs
            match base_url.join(href) {
                Ok(absolute_url) => {
                    let url_str = absolute_url.to_string();
                    if !seen_urls.contains(&url_str) {
                        seen_urls.insert(url_str.clone());
                        
                        let anchor_text = element.text()
                            .collect::<String>()
                            .trim()
                            .to_string();
                        
                        let final_text = match (anchor_text.is_empty(), element.value().attr("title")) {
                            (false, _) => anchor_text,
                            (true, Some(title)) => title.to_string(),
                            (true, None) => String::new()
                        };

                        links.push(ParsedLink {
                            target_url: url_str,
                            anchor_text: Some(final_text).filter(|s| !s.is_empty()),
                        });
                    }
                }
                Err(e) => warn!("Failed to resolve URL '{}': {}", href, e),
            }
        }
    }

    Ok(links)
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

    // Priority areas that typically contain main content
    let content_selectors = [
        // Main content containers
        "article", "main", "[role='main']", ".main-content", "#main-content",
        // Blog and article specific
        ".post-content", ".article-content", ".entry-content", ".content",
        // Documentation specific
        ".documentation", ".docs-content", ".wiki-content",
        // Generic content areas
        "#content", ".content-area", "[itemprop='articleBody']",
        // Fallback to any large text container
        ".text-content", ".body-content"
    ];

    // Elements that usually contain noise
    let noise_selector = Selector::parse(concat!(
        "header, footer, nav, aside, .sidebar, .comments, .advertisement,",
        ".share-buttons, .social-media, .related-posts, .recommended,",
        ".navigation, .menu, .search, .popup, .modal, script, style,",
        "[role='complementary'], [role='banner'], [role='contentinfo'],",
        ".cookie-notice, .newsletter-signup, .subscription-box,",
        "#comments, .comments-area, .widget-area"
    )).unwrap_or_else(|_| Selector::parse("script, style").unwrap());

    // First try to find main content areas
    for selector in content_selectors.iter() {
        if let Ok(sel) = Selector::parse(selector) {
            if let Some(element) = parsed_html.select(&sel).next() {
                // Remove noise elements
                let mut html = element.html();
                for el in parsed_html.select(&noise_selector) {
                    html = html.replace(&el.html(), "");
                }
                
                // Parse the cleaned HTML
                let fragment = Html::parse_fragment(&html);
                let text = fragment.root_element()
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ");
                
                // If we found substantial content, use it
                if text.split_whitespace().count() > 50 {
                    content = text;
                    break;
                }
            }
        }
    }

    // If no content found in priority areas, try extracting from body
    // but with smarter content detection
    if content.is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = parsed_html.select(&body_selector).next() {
                // Remove noise first
                let mut html = body.html();
                for el in parsed_html.select(&noise_selector) {
                    html = html.replace(&el.html(), "");
                }
                
                let fragment = Html::parse_fragment(&html);
                
                // Find paragraphs and other text-heavy elements
                let text_selectors = [
                    "p", "article", "section", "div > p",
                    "[class*='text']", "[class*='content']",
                    "h1 ~ p", "h2 ~ p", "h3 ~ p"
                ];
                
                let mut text_blocks = Vec::new();
                
                for selector in text_selectors.iter() {
                    if let Ok(sel) = Selector::parse(selector) {
                        for element in fragment.select(&sel) {
                            let text = element.text().collect::<String>();
                            // Only include blocks with substantial text
                            if text.split_whitespace().count() > 20 {
                                text_blocks.push(text);
                            }
                        }
                    }
                }
                
                content = text_blocks.join(" ");
            }
        }
    }

    // Clean up the content
    let content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace() && c != '.' && c != ',' && c != '?' && c != '!', " ")
        .replace("  ", " ")
        .trim()
        .to_string();

    if content.split_whitespace().count() < 30 {
        None
    } else {
        Some(content)
    }
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn extract_meta_content<'a>(element: &'a ElementRef<'a>) -> Option<&'a str> {
    element
        .value()
        .attr("content")
        .or_else(|| element.text().next())
}

fn extract_metadata(document: &Html) -> (Option<String>, Option<String>, Option<String>, Option<Value>) {
    let meta_selectors = [
        ("meta[name='title'], meta[property='og:title']", "content"),
        ("meta[name='description'], meta[property='og:description']", "content"),
        ("meta[name='keywords']", "content"),
    ];

    let mut title = None;
    let mut description = None;
    let mut keywords = None;

    for (selector_str, _attr) in meta_selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let content = extract_meta_content(&element)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                match *selector_str {
                    s if s.contains("title") => title = title.or(content),
                    s if s.contains("description") => description = description.or(content),
                    s if s.contains("keywords") => keywords = keywords.or(content),
                    _ => {}
                }
            }
        }
    }

    let other_metadata = extract_other_metadata(document);

    (title, description, keywords, other_metadata)
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
