use std::sync::Arc;
use tokio::sync::Semaphore;
use sqlx::PgPool;
use elasticsearch::Elasticsearch;
use crate::db_indexer::{fetch_unprocessed_docs, mark_as_processed};
use crate::elastic_search_storage::store_processed_document_in_es;
use crate::content_processing::process_content;
use log::{info, error, debug};
use uuid::Uuid;
use crate::error::{IndexerResult};
use crate::metrics::MetricsClient;
use std::time::Instant;

const BATCH_SIZE: i64 = 10;
const MAX_CONCURRENT_REQUESTS: usize = 2;
const PROCESS_DELAY_MS: u64 = 100;

pub async fn concurrent_process_docs(
    pool: PgPool, 
    client: Arc<Elasticsearch>,
    metrics: &Arc<MetricsClient>
) -> IndexerResult<usize> {
    let start_time = Instant::now();
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    
    let documents = match fetch_unprocessed_docs(&pool, BATCH_SIZE).await {
        Ok(docs) => {
            metrics.set_queue_size(docs.len() as i64);
            docs
        },
        Err(e) => {
            metrics.increment_index_errors();
            error!("Failed to fetch unprocessed documents: {}", e);
            return Err(e);
        }
    };

    if documents.is_empty() {
        metrics.set_queue_size(0);
        info!("No documents available");
        return Ok(0);
    }

    let doc_count = documents.len();
    let metrics_for_histogram = Arc::clone(metrics);
    let mut handles = vec![];

    for doc in documents {
        let pool = pool.clone();
        let sem = semaphore.clone();
        let client = client.clone();
        let metrics_for_task = Arc::clone(&metrics);
        let doc_uuid = Uuid::from_bytes(*doc.id.as_bytes());
        let doc_start_time = Instant::now();

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
                        metrics_for_task.increment_index_errors();
                    } else {
                        if let Err(e) = mark_as_processed(&pool, doc_uuid).await {
                            error!("Error marking webpage {} as processed: {}", doc_uuid, e);
                            metrics_for_task.increment_index_errors();
                        } else {
                            metrics_for_task.increment_docs_processed();
                            metrics_for_task.observe_processing_duration(doc_start_time.elapsed().as_secs_f64());
                            debug!("Successfully processed doc {}", doc_uuid);
                        }
                    }
                },
                Err(e) => {
                    error!("Error processing doc {}: {}", doc_uuid, e);
                    metrics_for_task.increment_index_errors();
                }
            }
            
            // Add delay between documents
            tokio::time::sleep(tokio::time::Duration::from_millis(PROCESS_DELAY_MS)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            error!("Task join error: {}", e);
        }
    }

    metrics_for_histogram.observe_index_duration(start_time.elapsed().as_secs_f64());
    Ok(doc_count)
}