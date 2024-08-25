use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use std::error::Error;
use url::Url;
use crate::fetcher::CrawlerError;
use crate::robots::RobotsChecker;
use crate::config::Config;
use crate::fetcher;
use crate::parser;
use crate::storage;


pub struct Crawler {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    config: Arc<Config>,
    robots_checker: RobotsChecker,
}



impl Crawler {
    pub fn new(client: Client, config: Config) -> Self {
        let robots_checker = RobotsChecker::new(client.clone());
        Crawler {
            client,
            visited: Arc::new(Mutex::new(HashSet::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            config: Arc::new(config),
            robots_checker,
        }
    }

    pub async fn crawl(&self) -> Result<(), Box<dyn Error>> {
        let start_url = normalize_url(&self.config.start_url)?;
        self.queue.lock().await.push_back((start_url, 0));

        while !self.queue.lock().await.is_empty() {
            let mut urls_to_fetch = Vec::new();
            {
                let mut queue = self.queue.lock().await;
                while let Some((url, depth)) = queue.pop_front() {
                    if !self.visited.lock().await.contains(&url) {
                        let is_allowed = self.robots_checker.is_allowed(&url, &self.config.user_agent).await;
                        if is_allowed {
                            urls_to_fetch.push((url, depth));
                        }
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
                &self.config.user_agent
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
                                    if let Ok(normalized_link) = normalize_url(&link) {
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


// Function to normalize a given URL
pub fn normalize_url(url: &str) -> Result<String, CrawlerError> {
    let mut parsed_url = Url::parse(url).map_err(|e| CrawlerError::UrlNormalizationError(e.to_string()))?;
    parsed_url.set_fragment(None);
    Ok(parsed_url.as_str().trim_end_matches('/').to_lowercase())
}