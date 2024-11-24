use elasticsearch::{
    Elasticsearch, 
    http::transport::Transport, 
    indices::{IndicesCreateParts, IndicesExistsParts},
    IndexParts,
    Error as EsError,
};
use crate::document_models::ProcessedDoc;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use crate::error::{IndexerError, IndexerResult};
use log::{info, debug, warn, error};
use std::env;

pub async fn get_elasticsearch_client() -> IndexerResult<Elasticsearch> {
    let elasticsearch_url = env::var("ELASTICSEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:9200".to_string());
        
    let transport = Transport::single_node(&elasticsearch_url)
        .map_err(|e| IndexerError::Elasticsearch(e))?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

pub async fn ensure_index_exists(client: &Elasticsearch) -> IndexerResult<()> {
    let index_name = "pages";

    info!("Checking if index '{}' exists", index_name);
    let exists = client
        .indices()
        .exists(IndicesExistsParts::Index(&[index_name]))
        .send()
        .await
        .map_err(|e| IndexerError::Elasticsearch(e))?
        .status_code()
        .is_success();

    if exists {
        info!("Index '{}' already exists", index_name);
    } else {
        info!("Creating index '{}'", index_name);
        // Define the index settings and mappings
        let body = json!({
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 1,
                "analysis": {
                    "analyzer": {
                        "default": {
                            "type": "standard"
                        }
                    }
                }
            },
            "mappings": {
                "properties": {
                    "webpage_id": { "type": "keyword" },
                    "title": { "type": "text", "analyzer": "standard" },
                    "body": { "type": "text", "analyzer": "standard" },
                    "indexed_at": { "type": "date" },
                    "metadata": { "type": "object" },
                    "content_summary": { "type": "text", "analyzer": "standard" },
                    "keywords": { "type": "keyword" },
                    "page_rank": { "type": "double" }
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

pub async fn store_processed_document_in_es(client: &Elasticsearch, doc: &ProcessedDoc) -> IndexerResult<()> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        match store_document(client, doc).await {
            Ok(_) => {
                debug!("Successfully stored document {}", doc.processed_doc_webpage_id);
                return Ok(());
            },
            Err(e) => {
                attempt += 1;
                if attempt == MAX_RETRIES {
                    error!("Failed to store document after {} attempts: {}", MAX_RETRIES, e);
                    return Err(IndexerError::Elasticsearch(e));
                }
                warn!("Retry {} for document {}", attempt, doc.processed_doc_webpage_id);
                sleep(Duration::from_secs(2u64.pow(attempt))).await;
            }
        }
    }
    Err(IndexerError::Retry(format!(
        "Max retries reached for document {}", 
        doc.processed_doc_webpage_id
    )))
}

async fn store_document(client: &Elasticsearch, doc: &ProcessedDoc) -> Result<(), EsError> {
    let response = client
        .index(IndexParts::Index("pages"))
        .body(json!({
            "webpage_id": doc.processed_doc_webpage_id,
            "title": doc.processed_doc_title,
            "body": doc.processed_doc_body,
            "indexed_at": doc.processed_doc_indexed_at,
            "metadata": doc.processed_doc_metadata,
            "content_summary": doc.processed_doc_content_summary,
            "keywords": doc.processed_doc_keywords,
            "page_rank": doc.processed_doc_page_rank
        }))
        .send()
        .await?;

    if !response.status_code().is_success() {
        let error_message = format!(
            "Elasticsearch request failed with status: {}. Document ID: {}",
            response.status_code(),
            doc.processed_doc_webpage_id
        );
        return Err(EsError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            error_message
        )));
    }
    Ok(())
}
