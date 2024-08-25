use scraper::{Html, Selector};
use std::collections::HashSet;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to extract URLs: {0}")]
    UrlExtractionError(String),
    #[error("Selector parse error: {0}")]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

pub fn extract_links_from_html(html: &str, base_url: &str) -> Result<HashSet<String>, ParserError> {
    let parsed_html = Html::parse_document(html);
    let selector = Selector::parse("a[href]")?;

    // Assume base_url is already a normalized and valid URL from fetcher.rs
    let base_url = Url::parse(base_url)?;
    let mut links = HashSet::new();

    for element in parsed_html.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(resolved_url) = base_url.join(href) {
                links.insert(resolved_url.into());
            }
        }
    }

    if links.is_empty() {
        Err(ParserError::UrlExtractionError("No valid URLs found".to_string()))
    } else {
        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links_from_html() {
        let base_url = "https://example.com";
        let html = r#"
            <html>
                <body>
                    <a href="/page1">Link 1</a>
                    <a href="https://example.com/page2">Link 2</a>
                    <a href="https://other.com/page3">Link 3</a>
                </body>
            </html>
        "#;

        let links = extract_links_from_html(html, base_url).unwrap();

        assert!(links.contains("https://example.com/page1"));
        assert!(links.contains("https://example.com/page2"));
        assert!(links.contains("https://other.com/page3"));
    }

    #[test]
    fn test_extract_links_from_html_with_invalid_html() {
        let base_url = "https://example.com";
        let html = "<html><body><a href=\"/page1\">Link 1</a></body></html";

        let result = extract_links_from_html(html, base_url);
        assert!(result.is_ok()); // The HTML parser is quite forgiving
    }

    #[test]
    fn test_extract_links_from_html_with_invalid_url() {
        let base_url = "not-a-valid-url";
        let html = r#"<a href="/page1">Link 1</a>"#;

        let result = extract_links_from_html(html, base_url);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParserError::UrlParseError(_)));
    }

    #[test]
    fn test_extract_links_from_html_with_no_links() {
        let base_url = "https://example.com";
        let html = "<html><body><p>No links here</p></body></html>";

        let result = extract_links_from_html(html, base_url);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParserError::UrlExtractionError(_)));
    }
}