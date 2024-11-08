use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use unicode_segmentation::UnicodeSegmentation;
use rust_stemmers::{Algorithm, Stemmer};
use stopwords::{Stopwords, Language as StopwordsLanguage, Spark};
use regex::Regex;
use crate::document_models::{HtmlDocs, ProcessedDoc};
use chrono::Utc;

pub fn process_content(doc: &HtmlDocs) -> Result<ProcessedDoc> {
    let mut parsed_text = String::new();
    let document = Html::parse_document(&doc.content_summary.as_deref().unwrap_or(""));

    let all_tag_selector = Selector::parse("p, h1, h2, h3, h4, h5, h6, span, div, li, ul")
        .map_err(|e| anyhow!("Invalid selector: {}", e))?;

    for i in document.select(&all_tag_selector) {
        let text = i.text().collect::<Vec<_>>().join(" ");
        if !text.trim().is_empty() {
            parsed_text.push_str(&text);
        }
    }

    let words: Vec<&str> = parsed_text.unicode_words().collect();

    let lower_words: Vec<String> = words
        .iter()
        .map(|w| w.to_lowercase())
        .collect();

    let stop_words = Spark::stopwords(StopwordsLanguage::English)
        .ok_or_else(|| anyhow!("Failed to load stop words"))?;

    let filter_words: Vec<&str> = lower_words
        .iter()
        .filter(|w| !stop_words.contains(&w.as_str()))
        .map(|w| w.as_str())
        .collect();

    let stemmer = Stemmer::create(Algorithm::English);
    let stem_words: Vec<String> = filter_words
        .iter()
        .map(|w| stemmer.stem(w).to_string())
        .collect();

    let rex = Regex::new(r"\W+").map_err(|e| anyhow!("Invalid regex: {}", e))?;
    let processed_tokens: Vec<String> = stem_words
        .iter()
        .map(|w| rex.replace_all(w, "").to_string())
        .filter(|w| !w.is_empty())
        .collect();

    Ok(ProcessedDoc {
        processed_doc_webpage_id: doc.id,
        processed_doc_title: doc.title.clone(),
        processed_doc_body: Some(processed_tokens.join(" ")),
        processed_doc_indexed_at: Utc::now(),
        processed_doc_metadata: None,
        processed_doc_content_summary: None,
        processed_doc_keywords: Some(processed_tokens.clone()),
    })
}
