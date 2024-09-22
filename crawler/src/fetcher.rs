use reqwest::{Client, ClientBuilder, StatusCode};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::task;
use tokio::time::sleep;
use thiserror::Error;
use rand::Rng;

const MAX_RETRIES: usize = 3;  // Number of retry attempts allowed

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

// Function to create an HTTP client with predefined settings
pub fn create_http_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(10)) // time out duration, if request takes more than 10 seconds then abort it
        .gzip(true) //makes the client will decompress gzip-compressed responses it receives from the server.
        .pool_max_idle_per_host(10) // allows the HTTP client to maintain up to 10 idle connections per host.
        // This means that the client can reuse these connections for new requests to the same host
        .build()
}



// Function to fetch a page with retries and exponential backoff
pub async fn fetch_page(url: &str, client: &Client, user_agent: &str) -> Result<(String, StatusCode), CrawlerError> {
    let mut attempt = 0;
    while attempt < MAX_RETRIES {
        attempt += 1;
        match client.get(url)
            .header("User-Agent", user_agent)
            .send()
            .await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let content = response.text().await.map_err(CrawlerError::RequestError)?;
                    return Ok((content, status));
                } else {
                    eprintln!("Attempt {}: Failed to fetch {}: HTTP {}", attempt, url, status);
                    if attempt == MAX_RETRIES {
                        return Err(CrawlerError::StatusError(status));
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

        let backoff = Duration::from_millis(2u64.pow(attempt as u32) * 100 + rand::thread_rng().gen_range(0..100));
        sleep(backoff).await;
    }

    Err(CrawlerError::MaxRetriesReached)
}

// Function to fetch multiple pages in parallel
pub async fn fetch_pages_in_parallel(
    urls: Vec<(String, usize)>, // URL, depth (how deep crawler is in crawling process)

    client: &Client,
    delay_ms: u64,
    max_concurrent_requests: usize,
    user_agent: &str
) -> Vec<(String, usize, Result<(String, StatusCode), CrawlerError>)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));
    /*
    Reference Count in Arc<T>: When you clone an Arc, the reference count is increased,
    and when an Arc goes out of scope, the reference count is decreased. Since this reference
     count is shared between multiple threads, incrementing and decrementing it needs to happen
     atomically. Otherwise, if two threads try to modify the count at the same time, the count
     could become incorrect, leading to memory safety issues (like premature deallocation of memory).
    */

    /*
    Atomicity: An atomic operation either happens completely or not at all. There's no in-between state
     that another thread can observe. For example, if you increment a shared counter using an atomic operation,
      no other thread can see the counter in an inconsistent or intermediate state.
    */

    let visited_urls = Arc::new(Mutex::new(HashSet::new()));
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut tasks = vec![];

    for (url, depth) in urls {
        let client = client.clone();
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let visited_urls = Arc::clone(&visited_urls);
        let results = Arc::clone(&results);
        let user_agent = user_agent.to_string();

        let task = task::spawn(async move {
            let mut visited = visited_urls.lock().await;

            if !visited.contains(&url) {
                visited.insert(url.clone());
                drop(visited);

                let _permit = permit;
                let result = fetch_page(&url, &client, &user_agent).await;
                results.lock().await.push((url.clone(), depth, result));
                sleep(Duration::from_millis(delay_ms)).await;
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    Arc::try_unwrap(results).unwrap().into_inner()
}
