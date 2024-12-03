use rust_stemmers::{Algorithm, Stemmer};
use stopwords::{Stopwords, Language as StopwordsLanguage, Spark};
use crate::document_models::{HtmlDocs, ProcessedDoc};
use chrono::Utc;
use crate::error::{IndexerError, IndexerResult};
use serde_json::json;

pub fn process_content(doc: &HtmlDocs) -> IndexerResult<ProcessedDoc> {
    // Primary content (most important for search)
    let primary_content = format!(
        "{} {}", 
        doc.title.as_deref().unwrap_or(""),
        doc.content_summary.as_deref().unwrap_or("")
    );

    // Meta content (supplementary search info)
    let meta_content = format!(
        "{} {} {}", 
        doc.meta_description.as_deref().unwrap_or(""),
        doc.meta_keywords.as_deref().unwrap_or(""),
        doc.meta_title.as_deref().unwrap_or("")
    );

    // Process primary content with higher weight
    let primary_words = process_text(&primary_content)?;
    // Process meta content with lower weight
    let meta_words = process_text(&meta_content)?;

    let primary_words_clone = primary_words.clone();
    let mut all_words = primary_words;
    all_words.extend(meta_words.iter().cloned().filter(|w| !primary_words_clone.contains(w)));

    Ok(ProcessedDoc {
        processed_doc_webpage_id: doc.id,
        processed_doc_title: doc.title.clone(),
        processed_doc_body: Some(all_words.join(" ")),
        processed_doc_indexed_at: Utc::now(),
        processed_doc_metadata: Some(json!({
            "domain": doc.domain,
            "meta_description": doc.meta_description,
            "meta_keywords": doc.meta_keywords
        })),
        processed_doc_content_summary: doc.content_summary.clone(),
        processed_doc_keywords: Some(all_words),
        processed_doc_page_rank: doc.page_rank
    })
}

fn process_text(text: &str) -> IndexerResult<Vec<String>> {
    let en_stemmer = Stemmer::create(Algorithm::English);
    let stopwords = match Spark::stopwords(StopwordsLanguage::English) {
        Some(sw) => sw,
        None => return Err(IndexerError::Other("Failed to load stopwords".to_string()))
    };

    let words: Vec<String> = text
        .to_lowercase()
        .split_whitespace()
        .filter(|word| !stopwords.contains(&word))
        .map(|word| en_stemmer.stem(word).to_string())
        .collect();
        
    Ok(words)
}
