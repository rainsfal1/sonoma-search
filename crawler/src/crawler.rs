use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use crate::config::Config;
use crate::fetcher;
use crate::parser;
use crate::storage;
use std::error::Error;

pub struct Crawler {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    config: Arc<Config>,
}

impl Crawler {
    pub fn new(client: Client, config: Config) -> Self {
        Crawler {
            client,
            visited: Arc::new(Mutex::new(HashSet::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            config: Arc::new(config),
        }
    }

    pub async fn crawl(&self) -> Result<(), Box<dyn Error>> {
        let start_url = fetcher::normalize_url(&self.config.start_url)?;
        self.queue.lock().await.push_back((start_url, 0));

        while !self.queue.lock().await.is_empty() {
            let mut urls_to_fetch = Vec::new();
            {
                let mut queue = self.queue.lock().await;
                while let Some(url) = queue.pop_front() {
                    if !self.visited.lock().await.contains(&url.0) {
                        urls_to_fetch.push(url);
                        if urls_to_fetch.len() >= self.config.concurrent_requests {
                            break;
                        }
                    }
                }
            }

            if urls_to_fetch.is_empty() {
                break;
            }

            let fetched_pages = fetcher::fetch_pages_in_parallel(
                urls_to_fetch,
                &self.client,
                self.config.delay_ms,
                self.config.concurrent_requests,
            ).await;

            for (url, depth, result) in fetched_pages {
                self.visited.lock().await.insert(url.clone());

                match result {
                    Ok(content) => {
                        println!("Crawled: {}", url);
                        if let Err(e) = storage::save_content(&url, &content).await {
                            eprintln!("Failed to save content for {}: {:?}", url, e);
                        }

                        if depth < self.config.max_depth {
                            if let Ok(links) = parser::extract_links_from_html(&content, &url) {
                                for link in links {
                                    if let Ok(normalized_link) = fetcher::normalize_url(&link) {
                                        if !self.visited.lock().await.contains(&normalized_link) {
                                            self.queue.lock().await.push_back((normalized_link, depth + 1));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to fetch page {}: {:?}", url, e),
                }
            }

            if self.visited.lock().await.len() >= self.config.max_pages {
                break;
            }
        }

        Ok(())
    }
}