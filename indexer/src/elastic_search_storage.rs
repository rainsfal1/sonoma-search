use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts};
use anyhow::{anyhow, Result};
use crate::document_models::processed_doc;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

pub async fn get_elasticsearch_client() -> Result<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

const MAX_RETRIES: u32 = 3;

pub async fn store_processed_document_in_es(client: &Elasticsearch, processed_doc: &processed_doc) -> Result<()> {
    let body = json!({
        "webpage_id": processed_doc.processed_doc_webpage_id.to_string(),
        "title": processed_doc.processed_doc_title,
        "body": processed_doc.processed_doc_body,
        "indexed_at": processed_doc.processed_doc_indexed_at,
        "metadata": processed_doc.processed_doc_metadata,
        "content_summary": processed_doc.processed_doc_content_summary,
        "keywords": processed_doc.processed_doc_keywords,
    });

    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;

        match client
            .index(IndexParts::IndexId("processed_docs", &processed_doc.processed_doc_webpage_id.to_string()))
            .body(body.clone())  // clone body to retry with the same content
            .send()
            .await
        {
            Ok(response) if response.status_code().is_success() => return Ok(()),
            Ok(response) => {
                eprintln!("Attempt {}: Failed to store document in Elasticsearch. HTTP status: {}", attempt, response.status_code());
            },
            Err(e) => {
                eprintln!("Attempt {}: Network error storing document in Elasticsearch: {:?}", attempt, e);
            }
        }

        if attempt == MAX_RETRIES {
            return Err(anyhow!("Failed to store document in Elasticsearch after {} attempts", MAX_RETRIES));
        }

        // Exponential backoff with some random jitter
        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(anyhow!("Max retries reached for storing document in Elasticsearch"))
}
