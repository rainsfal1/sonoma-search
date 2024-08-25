// fetcher.rs

use reqwest::{Client, ClientBuilder, StatusCode};
use std::time::Duration;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};
use thiserror::Error;
use tokio::task;
use url::Url;
use tokio::time::sleep;
use rand::Rng;

// number of retry attempts allowed
const MAX_RETRIES: usize = 3;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("HTTP request failed with status: {0}")]
    StatusError(StatusCode),
    #[error("Failed to normalize URL: {0}")]
    UrlNormalizationError(String),
    #[error("Max retries reached")]
    MaxRetriesReached,
}

pub fn create_http_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .gzip(true)
        .pool_max_idle_per_host(10)
        .build()
}

pub fn normalize_url(url: &str) -> Result<String, CrawlerError> {
    let mut parsed_url = Url::parse(url).map_err(|e| CrawlerError::UrlNormalizationError(e.to_string()))?;
    parsed_url.set_fragment(None);
    Ok(parsed_url.as_str().trim_end_matches('/').to_lowercase())
}

pub async fn fetch_page(url: &str, client: &Client) -> Result<String, CrawlerError> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;
        match client.get(url)
            .header("User-Agent", "MyCrawler/1.0")
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    return match response.text().await {
                        Ok(body) => Ok(body),
                        Err(e) => Err(CrawlerError::RequestError(e)),
                    };
                } else {
                    eprintln!("Attempt {}: Failed to fetch {}: HTTP {}", attempt, url, response.status());
                    if attempt == MAX_RETRIES {
                        return Err(CrawlerError::StatusError(response.status()));
                    }
                }
            }
            Err(e) => {
                eprintln!("Attempt {}: Network error fetching {}: {:?}", attempt, url, e);
                if attempt == MAX_RETRIES {
                    return Err(CrawlerError::RequestError(e));
                }
            }
        }

        // this is exponential backoff with some jitter to avoid multiple retries at the same time
        let backoff = Duration::from_millis(2u64.pow(attempt as u32) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(CrawlerError::MaxRetriesReached)
}

pub async fn fetch_pages_in_parallel(
    urls: Vec<(String, usize)>,
    client: &Client,
    delay_ms: u64,
    max_concurrent_requests: usize
) -> Vec<(String, usize, Result<String, CrawlerError>)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));
    let visited_urls = Arc::new(Mutex::new(HashSet::new()));
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut tasks = vec![];

    for (url, depth) in urls {
        let client = client.clone();
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let visited_urls = Arc::clone(&visited_urls);
        let results = Arc::clone(&results);

        let task = task::spawn(async move {
            let mut visited = visited_urls.lock().await;

            if !visited.contains(&url) {
                visited.insert(url.clone());
                drop(visited);

                let _permit = permit;
                let result = fetch_page(&url, &client).await;
                results.lock().await.push((url.clone(), depth, result));
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    Arc::try_unwrap(results).unwrap().into_inner()
}