// use std::sync::Arc;
// use scraper::{Html, Selector};
// use unicode_segmentation::UnicodeSegmentation;
// use rust_stemmers::{Algorithm, Stemmer};
// use stopwords::{Stopwords, Language as StopwordsLanguage, Spark};
// use regex::Regex;
// use tokio::sync::Semaphore;
// use tokio::task;
// use tokio_postgres;
// use std::env;
// use std::time::Duration;
// use sqlx::postgres::PgPoolOptions;
// use sqlx::PgPool;
// use dotenv::dotenv;
// use anyhow::{anyhow, Result};
// use chrono::{DateTime, Utc};
// use log::{info, error};
// use env_logger;
// use serde::{Deserialize,Serialize};
// use uuid::Uuid;
// use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts};
// use serde_json::json;
//
// // Struct representing a webpage from the database
// #[derive(Debug, Serialize, Deserialize)]
// struct html_Docs{
//     id: Uuid,
//     url: String,
//     content: String,
//     html_content: String,
//     title: Option<String>,
// }
//
// // Struct representing a processed document
// #[derive(Debug, Serialize, Deserialize)]
// struct processed_doc{
//     processed_doc_webpage_id: Uuid,
//     processed_doc_title: Option<String>,
//     processed_doc_body: Option<String>,
//     processed_doc_indexed_at: DateTime<Utc>,
//     processed_doc_metadata: Option<serde_json::Value>,
//     processed_doc_content_summary: Option<String>,
//     processed_doc_keywords: Option<Vec<String>>,
// }
//
// // Function to fetch unprocessed documents
// async fn fetch_unprocessed_docs(pool: &PgPool, limit: i64) -> Result<Vec<html_Docs>> {
//     let records = sqlx::query_as!(html_Docs,
//     r#"
//     SELECT id,url,content,html_content,title
//     FROM webpages
//     WHERE processed = FALSE
//     LIMIT $1
//     "#,
//     limit
//     ).fetch_all(pool).await?;
//
//     Ok(records)
// }
//
// // Function to initialize Elasticsearch client
// async fn get_elasticsearch_client() -> Result<Elasticsearch> {
//     let transport = Transport::single_node("http://localhost:9200")?; // Adjust URL as necessary
//     let client = Elasticsearch::new(transport);
//     Ok(client)
// }
//
// // Function to store processed document in Elasticsearch
// async fn store_processed_document_in_es(client: &Elasticsearch, processed_doc: &processed_doc) -> Result<()> {
//     let body = json!({
//         "webpage_id": processed_doc.processed_doc_webpage_id.to_string(),
//         "title": processed_doc.processed_doc_title,
//         "body": processed_doc.processed_doc_body,
//         "indexed_at": processed_doc.processed_doc_indexed_at,
//         "metadata": processed_doc.processed_doc_metadata,
//         "content_summary": processed_doc.processed_doc_content_summary,
//         "keywords": processed_doc.processed_doc_keywords,
//     });
//
//     let response = client
//         .index(IndexParts::IndexId("processed_docs", &processed_doc.processed_doc_webpage_id.to_string()))
//         .body(body)
//         .send()
//         .await?;
//
//     if !response.status_code().is_success() {
//         Err(anyhow!("Failed to store document in Elasticsearch"))?;
//     }
//
// <<<<<<< Updated upstream
// async fn fetch_documents(tx: mpsc::sender<Vec<Document>>) -> Result<(), IndexerError> {
//     // Implement document fetching logic here
//     // This could involve reading from a storage, crawling websites, etc.
//     // Send batches of documents through the channel
// =======
// >>>>>>> Stashed changes
//     Ok(())
// }
//
// // Function to process the content of a document
// fn process_content(doc: &html_Docs) -> Result<processed_doc>{
//
//     let mut parsed_text = String::new();
//     let document = Html::parse_document(&doc.html_content);
//     let p_tag_selector = Selector::parse("p").map_err(|e| anyhow!("Invalid selector: {}", e))?;
//
//     for i in document.select(&p_tag_selector){
//         parsed_text.push_str(&i.text().collect::<Vec<_>>().join(" "));
//     }
//
//     let words: Vec<&str> = parsed_text.unicode_words().collect();
//
//
//     let lower_words: Vec<String> = words
//         .iter()
//         .map(|w| w.to_lowercase())
//         .collect();
//
//
//     let stop_words = Spark::stopwords(StopwordsLanguage::English)
//         .ok_or_else(||anyhow!("Failed to load stop words"))?;
//
//
//     let filter_words: Vec<&str> = lower_words
//         .iter()
//         .filter(|w| !stop_words.contains(&w.as_str()))
//         .map(|w| w.as_str())
//         .collect();
//
//
//     let stemmer = Stemmer::create(Algorithm::English);
//     let stem_words: Vec<String> = filter_words
//         .iter()
//         .map(|w| stemmer.stem(w).to_string())
//         .collect();
//
//
//     let rex = Regex::new(r"\W+").map_err(|e| anyhow!("Invalid regex: {}", e))?;
//     let processed_tokens: Vec<String> = stem_words
//         .iter()
//         .map(|w| rex.replace_all(w, "").to_string())
//         .filter(|w| !w.is_empty())
//         .collect();
//
//     Ok(processed_doc {
//         processed_doc_webpage_id: doc.id,
//         processed_doc_title: doc.title.clone(),
//         processed_doc_body: Some(processed_tokens.join(" ")),
//         processed_doc_indexed_at: Utc::now(),
//         processed_doc_metadata: None, // Can be filled with actual metadata
//         processed_doc_content_summary: None,      // Can implement summarization
//         processed_doc_keywords: Some(processed_tokens.clone()), // Example, can extract keywords differently
//     })
// }
//
// async fn mark_as_processed(pool: &PgPool, doc_id: Uuid) -> Result<()> {
//     sqlx::query!(
//         r#"
//         UPDATE webpages
//         SET processed = TRUE
//         WHERE id = $1
//         "#,
//         doc_id
//     )
//         .execute(&pool)
//         .await?;
//
//     Ok(())
// }
//
// // Modify concurrent processing to store in Elasticsearch
// async fn concurrent_process_docs(pool: PgPool, client: Arc<Elasticsearch>) -> Result<()> {
//     let semaphore = Arc::new(Semaphore::new(10));
//     let documents = fetch_unprocessed_docs(&pool, 10).await?;
//
//     if documents.is_empty() {
//         info!("No documents available");
//         return Ok(());
//     }
//
//     info!("Processing {} documents", documents.len());
//
//     let mut handles = vec![];
//
//     for doc in documents {
//         let pool = pool.clone();
//         let sem = semaphore.clone();
//         let client = client.clone();
//
//         let handle = tokio::spawn(async move {
//             let _permit = match sem.acquire().await {
//                 Ok(permit) => permit,
//                 Err(e) => {
//                     error!("Failed to acquire permit: {}", e);
//                     return;
//                 }
//             };
//
//             match process_content(&doc) {
//                 Ok(processed_doc) => {
//                     // Store processed document in Elasticsearch
//                     if let Err(e) = store_processed_document_in_es(&client, &processed_doc).await {
//                         error!("Error storing processed_doc {}: {}", doc.id, e);
//                     } else {
//                         // Mark as processed in PostgreSQL
//                         if let Err(e) = mark_as_processed(&pool, doc.id).await {
//                             error!("Error marking webpage {} as processed: {}", doc.id, e);
//                         } else {
//                             info!("Successfully processed webpage {}", doc.id);
//                         }
//                     }
//                 },
//                 Err(e) => {
//                     error!("Error processing doc {}: {}", doc.id, e);
//                 }
//             }
//         });
//         handles.push(handle);
//     }
//
//     for handle in handles {
//         if let Err(e) = handle.await {
//             error!("Error: {:?}", e);
//         }
//     }
//
//     Ok(())
// }
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     dotenv().ok();
//     env_logger::init();
//     let database_URL = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
//
//     let pool = PgPoolOptions::new()
//         .max_connections(15)
//         .connect(&database_URL)
//         .await?;
//
//     // Initialize Elasticsearch client
//     let es_client = Arc::new(get_elasticsearch_client().await?);
//
//     loop {
//         let client_clone = Arc::clone(&es_client); // Cloning the Arc, not the client itself
//         if let Err(e) = concurrent_process_docs(pool.clone(), client_clone).await {
//             error!("Error processing docs: {}", e);
//         }
//
//         tokio::time::sleep(Duration::from_secs(5)).await;
//     }
// }