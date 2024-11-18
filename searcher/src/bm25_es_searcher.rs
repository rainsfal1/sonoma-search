use elasticsearch::{Elasticsearch, SearchParts, http::transport::Transport};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub fn initialize_client(url: &str) -> Result<Elasticsearch, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let es_url = std::env::var("ELASTICSEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:9200".to_string());
    
    let transport = Transport::single_node(&es_url)?;
    Ok(Elasticsearch::new(transport))
}

pub async fn fetch_bm25_scores(
    client: &Elasticsearch,
    query: &str,
) -> Result<HashMap<Uuid, f64>, Box<dyn std::error::Error>> {
    let mut bm25_scores: HashMap<Uuid, f64> = HashMap::new();
    let mut from = 0;
    let size = 100;

    loop {
        let response = client
            .search(SearchParts::Index(&["processed_docs"]))
            .body(serde_json::json!({
                "query": {
                    "multi_match": {
                        "query": query,
                        "fields": ["title^2", "content_summary", "body"]
                    }
                },
                "fields": ["_id", "_score"],
                "from": from,
                "size": size
            }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            eprintln!("Failed to query Elasticsearch: {}", response.status_code());
            return Err("Elasticsearch query failed.".into());
        }

        let body: Value = response.json().await?;
        if let Some(docs) = body["hits"]["hits"].as_array() {
            if docs.is_empty() {
                break;
            }

            for doc in docs {
                if let Some(id_str) = doc["_id"].as_str() {
                    if let Ok(doc_id) = Uuid::parse_str(id_str) {
                        if let Some(score) = doc["_score"].as_f64() {
                            bm25_scores.insert(doc_id, score as f64);
                        } else {
                            eprintln!("Missing BM25 score for document with ID: {}", id_str);
                        }
                    } else {
                        eprintln!("Failed to parse document ID: {}", id_str);
                    }
                }
            }
        } else {
            break;
        }

        from += size;
    }

    Ok(bm25_scores)
}
