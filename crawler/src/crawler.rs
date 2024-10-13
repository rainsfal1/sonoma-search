use crate::fetcher::{self, CrawlerError};
use crate::robots::RobotsChecker;
use crate::config::Config;
use crate::parser;
use crate::summarizer;
use log::{error, warn, info, debug};

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use std::error::Error;
use url::Url;
use uuid::{Uuid};
use storage::{PostgresStorage, Webpage, Link};

pub struct Crawler {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    config: Arc<Config>,
    robots_checker: RobotsChecker,
    storage: Arc<PostgresStorage>,
}

impl Crawler {
    pub fn new(client: Client, config: Config, storage: PostgresStorage) -> Self {
        let robots_checker = RobotsChecker::new(client.clone());
        Crawler {
            client,
            visited: Arc::new(Mutex::new(HashSet::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            config: Arc::new(config),
            robots_checker,
            storage: Arc::new(storage),
        }
    }

    pub async fn crawl(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting crawl process");
        debug!("Initial URL: {}", self.config.start_url);
        let start_url = normalize_url(&self.config.start_url)?;
        self.queue.lock().await.push_back((start_url, 0));

        while !self.queue.lock().await.is_empty() {
            let urls_to_fetch = self.get_urls_to_fetch().await;

            let fetched_pages = fetcher::fetch_pages_in_parallel(
                urls_to_fetch,
                &self.client,
                self.config.delay_ms,
                self.config.concurrent_requests,
                &self.config.user_agent,
            ).await;

            for (url, depth, result) in fetched_pages {
                self.visited.lock().await.insert(url.clone());

                match result {
                    Ok((content, status)) => {
                        println!("Crawled: {} (Status: {})", url, status);
                        if let Err(e) = self.process_page(&url, &content, status.as_u16() as i32, depth).await {
                            warn!("Error processing URL {}: {}", url, e);
                        }
                    }
                    Err(e) => error!("Failed to fetch page {}: {:?}", url, e),
                }
            }

            if self.visited.lock().await.len() >= self.config.max_pages {
                break;
            }
        }

        Ok(())
    }

    async fn get_urls_to_fetch(&self) -> Vec<(String, usize)> {
        debug!("Fetching new URLs to crawl");
        let mut urls_to_fetch = Vec::new();
        let mut queue = self.queue.lock().await;
        let visited = self.visited.lock().await;

        while let Some((url, depth)) = queue.pop_front() {
            if !visited.contains(&url) {
                let is_allowed = self.robots_checker.is_allowed(&url, &self.config.user_agent).await;
                if is_allowed {
                    urls_to_fetch.push((url, depth));
                }
                if urls_to_fetch.len() >= self.config.concurrent_requests {
                    break;
                }
            }
        }
        urls_to_fetch
    }

    async fn process_page(&self, url: &str, content: &str, status: i32, depth: usize) -> Result<(), Box<dyn Error>> {
        debug!("Processing page: {} (status: {}, depth: {})", url, status, depth);
        let parsed_page = parser::parse_webpage(content, url, status)?;
        let summary = summarizer::tfidf_summary(&parsed_page.content.unwrap_or_default(), 3);

        let webpage = Webpage {
            id: Uuid::new_v4(),
            url: parsed_page.url.clone(),
            domain: parsed_page.domain,
            title: parsed_page.title,
            content_summary: Some(summary),
            fetch_timestamp: parsed_page.fetch_timestamp,
            last_updated_timestamp: parsed_page.last_updated_timestamp,
            status: parsed_page.status,
            content_hash: Some(parsed_page.content_hash),
            metadata: parsed_page.metadata,
            links: Vec::new(),
            meta_title: parsed_page.meta_title,
            meta_description: parsed_page.meta_description,
            meta_keywords: parsed_page.meta_keywords,
        };

        self.storage.save_webpage(&webpage).await?;
        self.process_links(&webpage, parsed_page.links, depth).await?;

        Ok(())
    }

    async fn process_links(&self, webpage: &Webpage, links: Vec<parser::ParsedLink>, depth: usize) -> Result<(), Box<dyn Error>> {
        let mut transaction = self.storage.pool.begin().await?;

        for link in links {
            let db_link = Link {
                id: Uuid::new_v4(),
                source_webpage_id: webpage.id,
                target_url: link.target_url.clone(),
                anchor_text: link.anchor_text,
            };

            self.storage.save_link(&mut transaction, &db_link).await?;

            if depth < self.config.max_depth {
                if let Ok(normalized_link) = normalize_url(&link.target_url) {
                    if !self.visited.lock().await.contains(&normalized_link) {
                        self.queue.lock().await.push_back((normalized_link, depth + 1));
                    }
                }
            }
        }

        transaction.commit().await?;
        Ok(())
    }
}

pub fn normalize_url(url: &str) -> Result<String, CrawlerError> {
    let parsed = Url::parse(url).map_err(|e| CrawlerError::UrlNormalizationError(e.to_string()))?;
    let mut normalized = parsed.clone();

    // Remove default ports
    if (parsed.scheme() == "http" && parsed.port() == Some(80)) ||
       (parsed.scheme() == "https" && parsed.port() == Some(443)) {
        normalized.set_port(None).ok();
    }

    // Remove trailing slash
    let path = normalized.path().to_string();
    let trimmed_path = path.trim_end_matches('/');
    normalized.set_path(trimmed_path);

    // Sort query parameters
    if let Some(query) = normalized.query() {
        let mut params: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();
        params.sort_by(|a, b| a.0.cmp(&b.0));
        normalized.set_query(Some(&url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish()));
    }

    // Convert to lowercase
    Ok(normalized.to_string().to_lowercase())
}
