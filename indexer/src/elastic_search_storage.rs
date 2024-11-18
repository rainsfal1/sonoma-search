<<<<<<< Updated upstream
use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts, indices::IndicesCreateParts, Error as EsError};
=======
use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts, indices::IndicesCreateParts};
>>>>>>> Stashed changes
use crate::document_models::ProcessedDoc;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use crate::error::{IndexerError, IndexerResult};

pub async fn get_elasticsearch_client() -> IndexerResult<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")
        .map_err(|e| IndexerError::Elasticsearch(e))?;
    

pub async fn get_elasticsearch_client() -> IndexerResult<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")
        .map_err(|e| IndexerError::Elasticsearch(e.into()))?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

pub async fn ensure_index_exists(client: &Elasticsearch) -> IndexerResult<()> {
    let index_name = "processed_docs";

    // Check if the index exists
    let response = client
        .indices()
        .exists(elasticsearch::indices::IndicesExistsParts::Index(&[index_name]))
        .send()
        .await
        .map_err(|e| IndexerError::Elasticsearch(e))?;

    // Check the response status code to determine if the index exists
    if response.status_code().is_success() {
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
                    "indexed_at": { "type": "date" },
                    "metadata": { "type": "object" },
                    "content_summary": { "type": "text" }
                }
            }
        });

        // Create the index
        let create_response = client
            .indices()
            .create(IndicesCreateParts::Index(index_name))
            .body(body)
            .send()
            .await?;

        if create_response.status_code().is_success() {
            println!("Index '{}' created successfully", index_name);
        } else {
            return Err(IndexerError::IndexCreationFailed(format!(
                "Failed to create index '{}'", index_name
            )));
        }
    }

    Ok(())
}

const MAX_RETRIES: u32 = 3;

// Inside your store_processed_document_in_es function
pub async fn store_processed_document_in_es(client: &Elasticsearch, processed_doc: &ProcessedDoc) -> IndexerResult<()> {
    let body = json!({
        "webpage_id": processed_doc.processed_doc_webpage_id.to_string(),
        "title": processed_doc.processed_doc_title,
        "indexed_at": processed_doc.processed_doc_indexed_at,
        "metadata": processed_doc.processed_doc_metadata,
        "content_summary": processed_doc.processed_doc_content_summary,
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
<<<<<<< Updated upstream
            return Err(IndexerError::Elasticsearch(
                format!("Failed to store document after {} attempts", MAX_RETRIES).into()
            ));
=======
            return Err(IndexerError::GenericError(format!("Failed to store document after {} attempts", MAX_RETRIES))); // Use GenericError
>>>>>>> Stashed changes
        }

        // Exponential backoff with some random jitter
        let backoff = Duration::from_millis(2u64.pow(attempt) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

<<<<<<< Updated upstream
    Err(IndexerError::Elasticsearch(
        format!("Max retries reached for storing document in Elasticsearch").into()
    ))
=======
    Err(IndexerError::GenericError("Max retries reached for storing document in Elasticsearch".to_string())) // Use GenericError
>>>>>>> Stashed changes
}
