use elasticsearch::{Elasticsearch, SearchParts, http::transport::Transport};
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;
use crate::errors::{SearchError, SearchResult};
use log::{info, error, debug};

pub fn initialize_client(_url: &str) -> SearchResult<Elasticsearch> {
    let es_url = std::env::var("ELASTICSEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:9200".to_string());
    
    debug!("Connecting to Elasticsearch at {}", es_url);
    
    match Transport::single_node(&es_url) {
        Ok(transport) => {
            info!("Successfully connected to Elasticsearch");
            Ok(Elasticsearch::new(transport))
        },
        Err(e) => {
            error!("Failed to connect to Elasticsearch at {}: {}", es_url, e);
            Err(SearchError::Processing(format!(
                "Failed to connect to Elasticsearch: {}. Please ensure Elasticsearch is running at {}",
                e, es_url
            )))
        }
    }
}

pub async fn fetch_bm25_scores(
    client: &Elasticsearch,
    query: &str,
) -> SearchResult<HashMap<Uuid, f64>> {
    let mut bm25_scores = HashMap::new();
    let mut from = 0;
    let size = 100;

    debug!("Executing Elasticsearch query: {}", query);
    
    // First, let's check if the index exists and has documents
    let count_response = client
        .count(elasticsearch::CountParts::Index(&["processed_docs"]))
        .send()
        .await?;
    
    let count_body: Value = count_response.json().await?;
    info!("Total documents in index: {}", count_body["count"]);

    loop {
        let query_body = json!({
            "from": from,
            "size": size,
            "_source": ["webpage_id"],
            "query": {
                "bool": {
                    "should": [
                        {
                            "match": {
                                "title": {
                                    "query": query,
                                    "boost": 3.0
                                }
                            }
                        },
                        {
                            "match": {
                                "content_summary": {
                                    "query": query,
                                    "boost": 2.0
                                }
                            }
                        },
                        {
                            "match": {
                                "body": {
                                    "query": query,
                                    "boost": 1.0
                                }
                            }
                        },
                        {
                            "match_phrase": {
                                "title": {
                                    "query": query,
                                    "boost": 4.0
                                }
                            }
                        }
                    ],
                    "minimum_should_match": 1
                }
            }
        });

        debug!("Search query body: {}", query_body);
        
        let response = client
            .search(SearchParts::Index(&["processed_docs"]))
            .body(query_body)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        debug!("Search response: {}", response_body);

        if let Some(hits) = response_body["hits"]["hits"].as_array() {
            if hits.is_empty() {
                break;
            }

            for hit in hits {
                if let (Some(id_str), Some(score)) = (
                    hit["_source"]["webpage_id"].as_str(),
                    hit["_score"].as_f64()
                ) {
                    if let Ok(doc_id) = Uuid::parse_str(id_str) {
                        debug!("Found document: {} with score: {}", id_str, score);
                        bm25_scores.insert(doc_id, score);
                    } else {
                        error!("Invalid UUID format: {}", id_str);
                    }
                }
            }
        }

        from += size;
        if from >= 1000 {
            break;
        }
    }

    info!("Found {} matching documents", bm25_scores.len());
    Ok(bm25_scores)
}
