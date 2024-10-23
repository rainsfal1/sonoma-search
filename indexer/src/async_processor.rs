use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Semaphore;
use sqlx::PgPool;
use elasticsearch::Elasticsearch;
use crate::postgre_functions::{fetch_unprocessed_docs, mark_as_processed};
use crate::elastic_search_storage::store_processed_document_in_es;
use crate::content_processing::process_content;
use log::{info, error};

pub async fn concurrent_process_docs(pool: PgPool, client: Arc<Elasticsearch>) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(10));
    let documents = fetch_unprocessed_docs(&pool, 10).await?;

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
                        error!("Error storing processed_doc {}: {}", doc.id, e);
                    } else {
                        if let Err(e) = mark_as_processed(&pool, doc.id).await {
                            error!("Error marking webpage {} as processed: {}", doc.id, e);
                        } else {
                            info!("Successfully processed webpage {}", doc.id);
                        }
                    }
                },
                Err(e) => {
                    error!("Error processing doc {}: {}", doc.id, e);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            error!("Error: {:?}", e);
        }
    }

    Ok(())
}
