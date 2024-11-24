use reqwest::{Client, ClientBuilder, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::task;
use tokio::time::sleep;
use log::{error, warn};
use reqwest::header::{ACCEPT_ENCODING, USER_AGENT};
use rand::Rng;
use crate::error::CrawlerError;

const MAX_RETRIES: usize = 3;  // Number of retry attempts allowed

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
    let mut retries = 0;
    let mut delay = 1000; // Start with a 1-second delay

    loop {
        match client.get(url)
            .header(USER_AGENT, user_agent)
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    warn!("Received non-success status code {} for URL {}", status, url);
                    return Err(CrawlerError::StatusError(status));
                }
                match response.text().await {
                    Ok(content) => {
                        return Ok((content, status));
                    }
                    Err(e) => {
                        error!("Failed to get response text for URL {}: {}", url, e);
                        return Err(CrawlerError::ResponseError(e));
                    }
                }
            }
            Err(e) => {
                error!("Failed to send request for URL {}: {}", url, e);
                if retries >= MAX_RETRIES {
                    return Err(CrawlerError::MaxRetriesReached);
                }
                retries += 1;
                let jitter = rand::thread_rng().gen_range(0..=200);
                sleep(Duration::from_millis(delay + jitter)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

// Function to fetch multiple pages in parallel
pub async fn fetch_pages_in_parallel(
    urls: Vec<(String, usize)>,
    client: &Client,
    delay_ms: u64,
    max_concurrent_requests: usize,
    user_agent: &str
) -> Vec<(String, usize, Result<(String, StatusCode), CrawlerError>)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut tasks = vec![];

    for (url, depth) in urls {
        let client = client.clone();
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let results = Arc::clone(&results);
        let user_agent = user_agent.to_string();

        let task = task::spawn(async move {
            let _permit = permit;
            let result = fetch_page(&url, &client, &user_agent).await;
            results.lock().await.push((url.clone(), depth, result));
            sleep(Duration::from_millis(delay_ms)).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    Arc::try_unwrap(results).unwrap().into_inner()
}
