use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Semaphore;
use sqlx::PgPool;
use elasticsearch::Elasticsearch;
use crate::db_indexer::{fetch_unprocessed_docs, mark_as_processed};
use crate::elastic_search_storage::store_processed_document_in_es;
use crate::content_processing::process_content;
use log::{info, error};
use uuid::Uuid;
use crate::error::{IndexerError, IndexerResult};

pub async fn concurrent_process_docs(pool: PgPool, client: Arc<Elasticsearch>) -> IndexerResult<()> {
    let semaphore = Arc::new(Semaphore::new(10));
    let documents = fetch_unprocessed_docs(&pool, 10)
        .await
        .map_err(|e| IndexerError::Database(e))?;

    if documents.is_empty() {
        info!("No documents available");
        return Ok(());
    }

    info!("Processing {} documents", documents.len());

    let mut handles = vec![];

    for doc in documents {
        let pool = pool.clone();
        let sem = semaphore.clone();
        let client = client.clone();

        let doc_uuid = Uuid::from_bytes(*doc.id.as_bytes());

        let handle = tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    error!("Failed to acquire permit: {}", e);
                    return;
                }
            };

            match process_content(&doc) {
                Ok(processed_doc) => {
                    if let Err(e) = store_processed_document_in_es(&client, &processed_doc).await {
                        error!("Error storing processed_doc {}: {}", doc_uuid, e);
                    } else {
                        if let Err(e) = mark_as_processed(&pool, doc_uuid).await {
                            error!("Error marking webpage {} as processed: {}", doc_uuid, e);
                        } else {
                            info!("Successfully processed webpage {}", doc_uuid);
                        }
                    }
                },
                Err(e) => {
                    error!("Error processing doc {}: {}", doc_uuid, e);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            error!("Task join error: {}", e);
            return Err(IndexerError::TaskJoin(e));
        }
    }

    Ok(())
}