use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;
use crate::error::{IndexerError, IndexerResult};
use crate::document_models::ProcessedDoc;

pub async fn search_documents(client: &Elasticsearch, keyword: &str) -> IndexerResult<Vec<ProcessedDoc>> {
    let response = client.search(SearchParts::Index(&["processed_docs"]))
        .body(json!({
            "query": {
                "multi_match": {
                    "query": keyword,
                    "fields": ["title^2", "body", "content_summary"]
                }
            },
            "size": 10
        }))
        .send()
        .await
        .map_err(|e| IndexerError::Search(e.to_string()))?;

    let response_body = response.json::<serde_json::Value>().await
        .map_err(|e| IndexerError::Serialization(e))?;

    // Parse the response into ProcessedDoc structs
    let hits = response_body["hits"]["hits"]
        .as_array()
        .ok_or_else(|| IndexerError::Search("Invalid response format".to_string()))?;

    let docs = hits
        .iter()
        .filter_map(|hit| serde_json::from_value(hit["_source"].clone()).ok())
        .collect();

    Ok(docs)
}
