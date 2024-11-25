use crate::fetcher;
use crate::error::CrawlerError;
use crate::config::Config;
use crate::parser;
use crate::metrics;
use crate::summarizer;
use crate::robots::RobotsChecker;
use log::{info, warn, error, debug};
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use std::collections::VecDeque;
use reqwest::Client;
use url::Url;
use uuid::Uuid;
use storage::{PostgresStorage, Webpage, Link};

fn display_crawl_summary(visited_urls: &HashSet<String>, queue_size: usize) {
    println!("\nCrawler Status Summary:");
    println!("----------------------");
    println!("Pages Crawled: {}", visited_urls.len());
    println!("URLs in Queue: {}", queue_size);
    println!("----------------------\n");
}

pub struct Crawler {
    client: Client,
    config: Arc<Config>,
    storage: PostgresStorage,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    visited: Arc<Mutex<HashSet<String>>>,
    crawl_start_time: Option<Instant>,
    robots_checker: RobotsChecker,
}

impl Crawler {
    pub fn new(client: Client, config: Config, storage: PostgresStorage) -> Self {
        Crawler {
            robots_checker: RobotsChecker::new(client.clone()),
            client,
            config: Arc::new(config),
            storage,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            visited: Arc::new(Mutex::new(HashSet::new())),
            crawl_start_time: None,
        }
    }

    pub async fn crawl(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.crawl_start_time = Some(Instant::now());
        info!("Starting crawl process");
        
        // Initialize metrics for this cycle
        let mut pages_crawled = 0;
        let mut errors = 0;
        metrics::set_queue_size(0);
        
        // Process URLs until queue is empty or max pages reached
        info!("Adding {} seed URLs to queue", self.config.seed_urls.len());
        for seed_url in &self.config.seed_urls {
            let normalized_url = normalize_url(seed_url)?;
            info!("Adding seed URL to queue: {}", normalized_url);
            self.queue.lock().await.push_back((normalized_url, 0));
        }

        // Update queue size after adding seeds
        let initial_queue_size = self.queue.lock().await.len() as i64;
        metrics::set_queue_size(initial_queue_size);
        info!("Initial queue size: {}", initial_queue_size);

        info!("Starting main crawl loop");
        loop {
            // Collect a batch of URLs to process in parallel
            let mut batch = Vec::new();
            {
                let mut queue = self.queue.lock().await;
                while batch.len() < self.config.concurrent_requests && !queue.is_empty() {
                    if let Some((url, depth)) = queue.pop_front() {
                        if !self.visited.lock().await.contains(&url) {
                            batch.push((url, depth));
                        }
                    }
                }
            }

            if batch.is_empty() {
                info!("Queue is empty, crawling complete");
                break;
            }

            let queue_size = self.queue.lock().await.len();
            let visited_count = self.visited.lock().await.len();
            info!("Current queue size: {}, visited count: {}", queue_size, visited_count);
            metrics::set_queue_size(queue_size as i64);

            if visited_count >= self.config.max_pages {
                info!("Reached maximum pages limit of {}", self.config.max_pages);
                break;
            }

            display_crawl_summary(&self.visited.lock().await.clone(), queue_size);

            // Fetch pages in parallel
            let results = fetcher::fetch_pages_in_parallel(
                batch,
                &self.client,
                self.config.request_delay,
                self.config.concurrent_requests,
                &self.config.user_agent,
                self.config.max_content_size,
            ).await;

            // Process results
            for (url, depth, result) in results {
                match result {
                    Ok((content, status)) => {
                        // Check robots.txt before processing
                        let allowed = self.robots_checker.is_allowed(&url, &self.config.user_agent).await;
                        if !allowed {
                            info!("URL {} is disallowed by robots.txt", url);
                            continue;
                        }

                        info!("Successfully fetched {}", url);
                        // Mark as visited before processing to prevent duplicates
                        self.visited.lock().await.insert(url.clone());
                        
                        if let Err(e) = self.process_page(&url, &content, status.as_u16() as i32, depth).await {
                            error!("Error processing page {}: {}", url, e);
                            errors += 1;
                            metrics::increment_crawl_errors();
                        } else {
                            pages_crawled += 1;
                            metrics::increment_pages_crawled();
                        }
                    }
                    Err(e) => {
                        error!("Error fetching {}: {}", url, e);
                        errors += 1;
                        metrics::increment_crawl_errors();
                        self.visited.lock().await.insert(url);
                    }
                }
            }
        }

        // Record crawl duration and cycle completion
        if let Some(start_time) = self.crawl_start_time {
            let duration = start_time.elapsed().as_secs_f64();
            metrics::observe_crawl_duration(duration);
            info!("Crawl cycle completed in {:.2} seconds", duration);
            info!("Pages crawled: {}, Errors: {}", pages_crawled, errors);
            
            // Only increment cycle counter if we actually crawled some pages
            if pages_crawled > 0 {
                metrics::increment_crawl_cycles();
            }
        }

        Ok(())
    }

    /// Determines if a page is worth storing based on various crawl-time metrics.
    /// This is different from the ranker's scoring - it's used to filter out low-quality
    /// pages during crawling to save storage space and processing time.
    /// 
    /// Scoring criteria:
    /// - Content length (30 points): Rewards pages with substantial content
    /// - Metadata completeness (20 points): Rewards pages with good metadata
    /// - Link structure (20 points): Rewards pages that are well-connected
    /// - Domain priority (30 points): Rewards pages from priority domains
    /// 
    /// Returns a score from 0-100. Pages below config.min_quality_score are not stored.
    fn calculate_crawl_quality_score(&self, page: &parser::ParsedWebpage) -> u32 {
        let mut score = 0;

        // Content length score (up to 30 points)
        // Rewards pages that have substantial textual content
        if let Some(ref content) = page.content {
            let word_count = content.split_whitespace().count();
            // 1000 words = 30 points, scaled linearly up to this point
            score += (word_count as f32 / 1000.0 * 30.0).min(30.0) as u32;
        }

        // Metadata completeness score (up to 20 points)
        // Rewards pages with good metadata for better indexing
        if page.title.is_some() { score += 5; }            // Basic page title
        if page.meta_description.is_some() { score += 5; } // SEO description
        if page.meta_keywords.is_some() { score += 5; }    // Topic keywords
        if page.metadata.is_some() { score += 5; }         // Additional metadata

        // Link structure score (up to 20 points)
        // Rewards pages that are well-connected in the web graph
        let link_count = page.links.len();
        // 50 links = 20 points, scaled linearly up to this point
        score += ((link_count as f32 / 50.0 * 20.0).min(20.0)) as u32;

        // Domain priority score (30 points)
        // Rewards pages from domains we specifically want to crawl
        if let Some(ref priority_domains) = self.config.priority_domains {
            if priority_domains.iter().any(|d| page.domain.contains(d)) {
                score += 30;
            }
        }

        score
    }

    async fn process_page(&self, url: &str, content: &str, status: i32, depth: usize) -> Result<(), Box<dyn Error>> {
        let parsed_page = parser::parse_webpage(content, url, status)?;
        
        // Calculate content quality score
        let quality_score = self.calculate_crawl_quality_score(&parsed_page);
        
        // Only process pages that meet the minimum quality threshold
        if quality_score < self.config.min_quality_score {
            debug!("Skipping low quality page: {} (score: {})", url, quality_score);
            return Ok(());
        }

        // Create webpage record
        let webpage = Webpage {
            id: Uuid::new_v4(),
            url: url.to_string(),
            domain: parsed_page.domain.clone(),
            title: parsed_page.title,
            content_summary: Some(summarizer::tfidf_summary(&parsed_page.content.unwrap_or_default(), 3)),
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
        info!("Saving webpage: {} (quality: {})", webpage.url, quality_score);
        if let Err(e) = self.storage.save_webpage(&webpage).await {
            warn!("Failed to save webpage {}: {}", webpage.url, e);
            return Ok(());
        }

        // Process links in smaller batches to avoid transaction timeouts
        let batch_size = 50;
        for links_chunk in parsed_page.links.chunks(batch_size) {
            let mut transaction = match self.storage.pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    warn!("Failed to start transaction for links: {}", e);
                    continue;
                }
            };

            let mut had_error = false;
            for link in links_chunk {
                if let Ok(normalized_url) = normalize_url(&link.target_url) {
                    // Check if we should follow this link
                    if !self.config.should_follow_link(&normalized_url, &parsed_page.domain) {
                        continue;
                    }

                    // Create target webpage record
                    let target_webpage = Webpage::new(normalized_url.clone())?;
                    
                    // Check if this is a priority domain
                    let is_priority = self.config.priority_domains.as_ref()
                        .map(|domains| domains.iter().any(|d| target_webpage.domain.contains(d)))
                        .unwrap_or(false);

                    let db_link = Link {
                        id: Uuid::new_v4(),
                        source_webpage_id: webpage.id,
                        target_url: normalized_url.clone(),
                        anchor_text: link.anchor_text.clone(),
                    };

                    if let Err(e) = self.storage.save_link(&mut transaction, &db_link).await {
                        warn!("Failed to save link {} -> {}: {}", webpage.url, link.target_url, e);
                        had_error = true;
                        break;
                    }

                    if depth < self.config.max_depth {
                        if !self.visited.lock().await.contains(&normalized_url) {
                            let mut queue = self.queue.lock().await;
                            if is_priority {
                                queue.push_front((normalized_url, depth + 1));
                            } else {
                                queue.push_back((normalized_url, depth + 1));
                            }
                        }
                    }
                }
            }

            // Only commit if no errors occurred
            if !had_error {
                if let Err(e) = transaction.commit().await {
                    warn!("Failed to commit links transaction: {}", e);
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
