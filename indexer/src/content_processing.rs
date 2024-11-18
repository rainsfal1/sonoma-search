use scraper::{Html, Selector};
use unicode_segmentation::UnicodeSegmentation;
use rust_stemmers::{Algorithm, Stemmer};
use stopwords::{Stopwords, Language as StopwordsLanguage, Spark};
use regex::Regex;
use crate::document_models::{HtmlDocs, ProcessedDoc};
use chrono::Utc;
use crate::error::{IndexerError, IndexerResult};
use serde_json::json;

pub fn process_content(doc: &HtmlDocs) -> IndexerResult<ProcessedDoc> {
    let combined_text = format!(
        "{} {} {} {} {}", 
        doc.title.as_deref().unwrap_or(""),
        doc.meta_title.as_deref().unwrap_or(""),
        doc.meta_description.as_deref().unwrap_or(""),
        doc.meta_keywords.as_deref().unwrap_or(""),
        doc.content_summary.as_deref().unwrap_or("")
    );

    let words: Vec<&str> = combined_text.unicode_words().collect();
    let lower_words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();

    let stop_words = Spark::stopwords(StopwordsLanguage::English)?;
    let filtered_words: Vec<&str> = lower_words.iter()
        .filter(|w| !stop_words.contains(&w.as_str()))
        .map(|w| w.as_str())
        .collect();

    let stemmer = Stemmer::create(Algorithm::English);
    let stemmed_words: Vec<String> = filtered_words.iter()
        .map(|w| stemmer.stem(w).to_string())
        .collect();

    let metadata = json!({
        "domain": doc.domain,
        "meta_title": doc.meta_title,
        "meta_description": doc.meta_description,
        "meta_keywords": doc.meta_keywords,
        "processed_keywords": stemmed_words.clone()
    });

    Ok(ProcessedDoc {
        processed_doc_webpage_id: doc.id,
        processed_doc_title: doc.title.clone(),
        processed_doc_body: Some(stemmed_words.join(" ")),
        processed_doc_indexed_at: Utc::now(),
        processed_doc_metadata: Some(metadata),
        processed_doc_content_summary: doc.content_summary.clone(),
        processed_doc_keywords: Some(stemmed_words),
    })
}
