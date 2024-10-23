use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;
use anyhow::Result;

pub async fn search_documents(client: &Elasticsearch, keyword: &str) -> Result<()> {
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

    let response_body = response.json::<serde_json::Value>().await?;
    println!("Search results: {:?}", response_body);

    Ok(())
}
