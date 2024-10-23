use elasticsearch::{Elasticsearch, http::transport::Transport, IndexParts};
use anyhow::{anyhow, Result};
use crate::document_models::processed_doc;
use serde_json::json;

pub async fn get_elasticsearch_client() -> Result<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

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

    let response = client
        .index(IndexParts::IndexId("processed_docs", &processed_doc.processed_doc_webpage_id.to_string()))
        .body(body)
        .send()
        .await?;

    if !response.status_code().is_success() {
        return Err(anyhow!("Failed to store document in Elasticsearch"));
    }

    Ok(())
}
