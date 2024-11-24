use crate::fetcher;
use crate::error::CrawlerError;
use crate::robots::RobotsChecker;
use crate::config::Config;
use crate::parser;
use crate::metrics;
use crate::summarizer;
use log::{warn, info, debug};
use std::time::Instant;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use url::Url;
use uuid::Uuid;
use storage::{PostgresStorage, Webpage, Link};
use std::collections::{HashSet, VecDeque};

pub struct Crawler {
    client: Client,
    visited: Arc<Mutex<HashSet<String>>>,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    config: Arc<Config>,
    robots_checker: RobotsChecker,
    storage: Arc<PostgresStorage>,
    crawl_start_time: Option<Instant>,
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
            crawl_start_time: None,
        }
    }

    pub async fn crawl(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.crawl_start_time = Some(Instant::now());
        info!("Starting crawl process");
        debug!("Initial URL: {}", self.config.start_url);
        let start_url = normalize_url(&self.config.start_url)?;
        self.queue.lock().await.push_back((start_url, 0));

        while !self.queue.lock().await.is_empty() {
            let queue_size = self.queue.lock().await.len();
            metrics::set_queue_size(queue_size as i64);

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
                        info!("Crawled: {} (Status: {})", url, status);
                        metrics::increment_pages_crawled();
                        if let Err(e) = self.process_page(&url, &content, status.as_u16() as i32, depth).await {
                            warn!("Error processing URL {}: {}", url, e);
                            metrics::increment_crawl_errors();
                        }
                    }
                    Err(e) => {
                        warn!("Error fetching URL {}: {}", url, e);
                        metrics::increment_crawl_errors();
                    }
                }
            }

            if self.visited.lock().await.len() >= self.config.max_pages {
                break;
            }
        }

        if let Some(start_time) = self.crawl_start_time.take() {
            let duration = start_time.elapsed().as_secs_f64();
            metrics::observe_crawl_duration(duration);
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
        let parsed_page = parser::parse_webpage(content, url, status)?;
        
        // Create summary from the text content
        let summary = summarizer::tfidf_summary(&parsed_page.content.unwrap_or_default(), 3);
        
        // Create webpage record
        let webpage = Webpage {
            id: Uuid::new_v4(),
            url: url.to_string(),
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
            ranked: false,
            last_ranked_at: None,
            page_rank: 0.0,
        };

        // Store webpage
        info!("Saving webpage: {}", webpage.url);
        self.storage.save_webpage(&webpage).await?;
        
        // Process and store links
        info!("Processing {} links for webpage {}", parsed_page.links.len(), webpage.url);

        for link in parsed_page.links {
            if let Ok(normalized_url) = normalize_url(&link.target_url) {
                // Create a new transaction for each link to isolate failures
                let mut transaction = match self.storage.pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        warn!("Failed to start transaction: {}", e);
                        continue;
                    }
                };

                // First ensure target webpage exists
                let target_webpage = Webpage::new(normalized_url.clone())?;
                if let Err(e) = self.storage.save_webpage(&target_webpage).await {
                    warn!("Failed to save target webpage {}: {}", target_webpage.url, e);
                    continue;
                }

                let db_link = Link {
                    id: Uuid::new_v4(),
                    source_webpage_id: webpage.id,
                    target_url: normalized_url.clone(),
                    anchor_text: link.anchor_text,
                };

                if let Err(e) = self.storage.save_link(&mut transaction, &db_link).await {
                    warn!("Failed to save link {} -> {}: {}", webpage.url, normalized_url, e);
                    if let Err(e) = transaction.rollback().await {
                        warn!("Failed to rollback transaction: {}", e);
                    }
                    continue;
                }

                if let Err(e) = transaction.commit().await {
                    warn!("Failed to commit transaction: {}", e);
                    continue;
                }

                if depth < self.config.max_depth {
                    if !self.visited.lock().await.contains(&normalized_url) {
                        debug!("Queueing new URL for crawling: {}", normalized_url);
                        self.queue.lock().await.push_back((normalized_url, depth + 1));
                    }
                }
            }
        }
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
