use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;
use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedDoc {
    pub processed_doc_webpage_id: Uuid,
    pub processed_doc_title: Option<String>,
    pub processed_doc_body: Option<String>,
    pub processed_doc_indexed_at: DateTime<Utc>,
    pub processed_doc_metadata: Option<Value>,
    pub processed_doc_content_summary: Option<String>,
    pub processed_doc_keywords: Option<Vec<String>>,
}

pub async fn print_search_results(client: &Elasticsearch, keyword: &str) -> Result<()> {
    let response = client.search(SearchParts::Index(&["processed_docs"]))
        .body(json!({
            "query": {
                "match": {
                    "body": keyword
                }
            }
        }))
        .send()
        .await?;

    let response_body = response.json::<Value>().await?;

    // Check if there are hits in the response
    if let Some(hits) = response_body["hits"]["hits"].as_array() {
        for hit in hits {
            // Deserialize the _source field into a ProcessedDoc struct
            let doc: ProcessedDoc = serde_json::from_value(hit["_source"].clone())?;

            // Access the _score field for BM25 scoring
            let score = hit["_score"].as_f64().unwrap_or(0.0);

            // Print the document along with its score
            println!("Document: {:?}, Score: {}", doc, score);
        }
    } else {
        println!("No documents found for keyword: {}", keyword);
    }

    Ok(())
}
