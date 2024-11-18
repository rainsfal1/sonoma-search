use scraper::{Html, Selector};
use unicode_segmentation::UnicodeSegmentation;
use rust_stemmers::{Algorithm, Stemmer};
use stopwords::{Stopwords, Language as StopwordsLanguage, Spark};
use crate::document_models::{HtmlDocs, ProcessedDoc};
use chrono::Utc;
use crate::error::{IndexerError, IndexerResult};
use reqwest::Client; // Use the async client
use serde::{Deserialize, Serialize};
use tokio::task; // Import tokio::task to spawn blocking tasks


// Define structure for API payload
#[derive(Serialize)]
struct LemmatizeRequest {
    tokens: Vec<String>,
}

// Define structure for API response
#[derive(Deserialize)]
struct LemmatizeResponse {
    lemmatized_tokens: Vec<String>,
}

pub async fn lemmatize_tokens(tokens: Vec<String>) -> IndexerResult<Vec<String>> {
    let client = Client::new(); // Using async client
    let url = "http://localhost:8000/lemmatize";

    let response = client
        .post(url)
        .json(&LemmatizeRequest { tokens })
        .send()
        .await // Now this works because `send` is asynchronous
        .map_err(|e| IndexerError::ContentProcessing(format!("Request error: {}", e)))?
        .json::<LemmatizeResponse>()
        .await // This is also async now
        .map_err(|e| IndexerError::ContentProcessing(format!("Response parsing error: {}", e)))?;

    Ok(response.lemmatized_tokens)
}

pub async fn process_content(doc: &HtmlDocs) -> IndexerResult<ProcessedDoc> {
    // Use block_in_place for synchronous operations
    let parsed_text = task::block_in_place(|| {
        let document = Html::parse_document(&doc.content_summary.as_deref().unwrap_or(""));

        // Handle the selector parsing error without using `?`
        let all_tag_selector = Selector::parse("p, h1, h2, h3, h4, h5, h6, span, div, li, ul")
            .map_err(|e| IndexerError::ContentProcessing(format!("Invalid selector: {}", e)))?;

        let mut parsed_text = String::new();
        for i in document.select(&all_tag_selector) {
            let text = i.text().collect::<Vec<_>>().join(" ");
            if !text.trim().is_empty() {
                parsed_text.push_str(&text);
            }
        }

        // Return Ok with the parsed text
        Ok::<String, IndexerError>(parsed_text) // Specify the return type
    })?; // The `?` operator is applied here to propagate any errors

    // Continue with the rest of your processing logic...
    let words: Vec<&str> = parsed_text.unicode_words().collect();
    let lower_words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();

    let stop_words = Spark::stopwords(StopwordsLanguage::English)
        .ok_or_else(|| IndexerError::ContentProcessing("Failed to load stop words".to_string()))?;

    let filtered_words: Vec<&str> = lower_words
        .iter()
        .filter(|w| !stop_words.contains(&w.as_str()))
        .map(|w| w.as_str())
        .collect();

    let stemmer = Stemmer::create(Algorithm::English);
    let stemmed_tokens: Vec<String> = filtered_words
        .iter()
        .map(|w| stemmer.stem(w).to_string())
        .collect();

    // Await the async lemmatize_tokens call here
    let lemmatized_tokens = lemmatize_tokens(stemmed_tokens).await?;


    Ok(ProcessedDoc {
        processed_doc_webpage_id: doc.id,
        processed_doc_title: doc.title.clone(),
        processed_doc_indexed_at: Utc::now(),
        processed_doc_metadata: None,
        processed_doc_content_summary: Some(lemmatized_tokens.join(" ")),
    })
}