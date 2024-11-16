use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts, indices::IndicesCreateParts, Error as EsError};
use crate::document_models::ProcessedDoc;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use crate::error::{IndexerError, IndexerResult};

pub async fn get_elasticsearch_client() -> IndexerResult<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")
        .map_err(|e| IndexerError::Elasticsearch(e))?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

pub async fn ensure_index_exists(client: &Elasticsearch) -> IndexerResult<()> {
    let index_name = "processed_docs";

    // Check if the index exists
    let exists = client
        .indices()
        .exists(elasticsearch::indices::IndicesExistsParts::Index(&[index_name]))
        .send()
        .await
        .map_err(|e| IndexerError::Elasticsearch(e))?;

    if exists {
        println!("Index '{}' already exists", index_name);
    } else {
        // Define the index settings and mappings
        let body = json!({
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 1
            },
            "mappings": {
                "properties": {
                    "webpage_id": { "type": "keyword" },
                    "title": { "type": "text" },
                    "body": { "type": "text" },
                    "indexed_at": { "type": "date" },
                    "metadata": { "type": "object" },
                    "content_summary": { "type": "text" },
                    "keywords": { "type": "keyword" }
                }
            }
        });

        // Create the index
        let response = client
            .indices()
            .create(IndicesCreateParts::Index(index_name))
            .body(body)
            .send()
            .await?;

        if response.status_code().is_success() {
            println!("Index '{}' created successfully", index_name);
        } else {
            eprintln!("Failed to create index '{}'", index_name);
        }
    }

    Ok(())
}

const MAX_RETRIES: u32 = 3;

pub async fn store_processed_document_in_es(client: &Elasticsearch, processed_doc: &ProcessedDoc) -> IndexerResult<()> {
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
            return Err(IndexerError::Elasticsearch(
                format!("Failed to store document after {} attempts", MAX_RETRIES).into()
            ));
        }

        // Exponential backoff with some random jitter
        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(IndexerError::Elasticsearch(
        format!("Max retries reached for storing document in Elasticsearch").into()
    ))
}
