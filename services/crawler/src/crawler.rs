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
use tokio::time::Duration;
use std::collections::HashMap;
use urlencoding;

fn display_crawl_summary(visited_urls: &HashSet<String>, queue_size: usize) {
    println!("\nCrawler Status Summary:");
    println!("----------------------");
    println!("Pages Crawled: {}", visited_urls.len());
    println!("URLs in Queue: {}", queue_size);
    println!("----------------------\n");
}

#[derive(Debug)]
pub struct CrawlStatus {
    pub has_results: bool,
    pub existing_results_count: usize,
    pub suggested_queries: Vec<String>,
    pub message: String,
}

pub struct Crawler {
    robots_checker: RobotsChecker,
    client: Client,
    config: Arc<Config>,
    storage: PostgresStorage,
    queue: Arc<Mutex<VecDeque<(String, usize)>>>,
    visited: Arc<Mutex<HashSet<String>>>,
    crawl_start_time: Arc<Mutex<Option<Instant>>>,
    initialized: Arc<Mutex<bool>>,
    query_cache: Arc<Mutex<HashMap<String, HashSet<String>>>>,
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
            crawl_start_time: Arc::new(Mutex::new(None)),
            initialized: Arc::new(Mutex::new(false)),
            query_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize the crawler with seed URLs from config
    pub async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        info!("Initializing crawler with seed URLs...");
        for seed_url in &self.config.seed_urls {
            let normalized_url = normalize_url(seed_url)?;
            info!("Adding seed URL to queue: {}", normalized_url);
            self.queue.lock().await.push_back((normalized_url, 0));
        }

        *initialized = true;
        Ok(())
    }

    pub async fn crawl_for_query(&self, query: &str, max_depth: usize, max_pages: usize) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.initialize().await?;

        // Reset visited set for new query
        {
            let mut visited = self.visited.lock().await;
            visited.clear();
        }

        let cache_hit = {
            let cache = self.query_cache.lock().await;
            cache.get(query).cloned()
        };

        if let Some(cached_urls) = cache_hit {
            info!("Found {} cached URLs for query: {}", cached_urls.len(), query);
            let mut queue = self.queue.lock().await;
            queue.clear(); // Clear existing queue
            for url in cached_urls {
                queue.push_back((url, 0));
            }
        } else {
            let relevant_urls = self.find_relevant_urls(query).await?;
            let mut cache = self.query_cache.lock().await;
            cache.insert(query.to_string(), relevant_urls.clone());
            
            let mut queue = self.queue.lock().await;
            queue.clear(); // Clear existing queue
            for url in relevant_urls {
                queue.push_back((url, 0));
            }
        }

        self.crawl_with_params(max_depth, max_pages).await
    }

    async fn find_relevant_urls(&self, query: &str) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        let mut relevant_urls = HashSet::new();
        let search_domains = vec![
            "stackoverflow.com",
            "github.com",
            "medium.com",
            "dev.to",
            "reddit.com/r/programming",
        ];

        for domain in search_domains {
            let url = format!("https://{}/search?q={}", domain, urlencoding::encode(query));
            relevant_urls.insert(url);
        }

        Ok(relevant_urls)
    }

    /// Crawl with specific parameters
    pub async fn crawl_with_params(&self, max_depth: usize, max_pages: usize) -> Result<(), Box<dyn Error + Send + Sync>> {
        *self.crawl_start_time.lock().await = Some(Instant::now());
        info!("Starting crawl process");
        
        let mut pages_crawled = 0;
        let mut errors = 0;
        metrics::set_queue_size(0);
        
        info!("Starting main crawl loop");
        loop {
            let mut batch = Vec::new();
            {
                let mut queue = self.queue.lock().await;
                while batch.len() < self.config.concurrent_requests && !queue.is_empty() {
                    if let Some((url, depth)) = queue.pop_front() {
                        if !self.visited.lock().await.contains(&url) {
                            if depth <= max_depth {
                                batch.push((url, depth));
                            }
                        }
                    }
                }
            }

            if batch.is_empty() {
                info!("Queue is empty, crawling complete");
                break;
            }

            let queue_size = self.queue.lock().await.len();
            let visited_count = pages_crawled;  // Use pages_crawled instead of visited set size
            info!("Current queue size: {}, visited count: {}", queue_size, visited_count);
            metrics::set_queue_size(queue_size as i64);

            if visited_count >= max_pages {
                info!("Reached maximum pages limit of {}", max_pages);
                break;
            }

            display_crawl_summary(&self.visited.lock().await.clone(), queue_size);

            let results = fetcher::fetch_pages_in_parallel(
                batch,
                &self.client,
                self.config.request_delay,
                self.config.concurrent_requests,
                &self.config.user_agent,
                self.config.max_content_size,
            ).await;

            for (url, depth, result) in results {
                match result {
                    Ok((content, status)) => {
                        let allowed = self.robots_checker.is_allowed(&url, &self.config.user_agent).await;
                        if !allowed {
                            info!("URL {} is disallowed by robots.txt", url);
                            continue;
                        }

                        info!("Successfully fetched {}", url);
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

        if let Some(start_time) = *self.crawl_start_time.lock().await {
            let duration = start_time.elapsed().as_secs_f64();
            metrics::observe_crawl_duration(duration);
            info!("Crawl cycle completed in {:.2} seconds", duration);
            info!("Pages crawled: {}, Errors: {}", pages_crawled, errors);
            
            if pages_crawled > 0 {
                metrics::increment_crawl_cycles();
            }
        }

        Ok(())
    }

    async fn process_page(&self, url: &str, content: &str, status: i32, depth: usize) -> Result<(), Box<dyn Error>> {
        let parsed_page = parser::parse_webpage(content, url, status)?;
        
        let quality_score = self.calculate_crawl_quality_score(&parsed_page);
        
        if quality_score < self.config.min_quality_score {
            debug!("Skipping low quality page: {} (score: {})", url, quality_score);
            return Ok(());
        }

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

        info!("Saving webpage: {} (quality: {})", webpage.url, quality_score);
        if let Err(e) = self.storage.save_webpage(&webpage).await {
            warn!("Failed to save webpage {}: {}", webpage.url, e);
            return Ok(());
        }

        let batch_size = 50;
        for links_chunk in parsed_page.links.chunks(batch_size) {
            let mut retry_count = 0;
            let max_retries = 3;
            
            loop {
                let mut transaction = match self.storage.pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        warn!("Failed to start transaction for links: {}", e);
                        if retry_count >= max_retries {
                            break;
                        }
                        retry_count += 1;
                        tokio::time::sleep(Duration::from_millis(100 * retry_count as u64)).await;
                        continue;
                    }
                };

                let mut had_error = false;
                for link in links_chunk {
                    if let Ok(normalized_url) = normalize_url(&link.target_url) {
                        if !self.config.should_follow_link(&normalized_url, &parsed_page.domain) {
                            continue;
                        }

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
                            let mut queue = self.queue.lock().await;
                            let is_priority = self.config.priority_domains.as_ref()
                                .map(|domains| domains.iter().any(|d| normalized_url.contains(d)))
                                .unwrap_or(false);

                            if is_priority {
                                queue.push_front((normalized_url, depth + 1));
                            } else {
                                queue.push_back((normalized_url, depth + 1));
                            }
                        }
                    }
                }

                if !had_error {
                    if let Err(e) = transaction.commit().await {
                        warn!("Failed to commit links transaction: {}", e);
                        if retry_count >= max_retries {
                            break;
                        }
                        retry_count += 1;
                        tokio::time::sleep(Duration::from_millis(100 * retry_count as u64)).await;
                        continue;
                    }
                    break;
                } else {
                    if retry_count >= max_retries {
                        break;
                    }
                    retry_count += 1;
                    tokio::time::sleep(Duration::from_millis(100 * retry_count as u64)).await;
                }
            }
        }

        Ok(())
    }

    fn calculate_crawl_quality_score(&self, page: &parser::ParsedWebpage) -> u32 {
        let mut score = 0;

        if let Some(ref content) = page.content {
            let word_count = content.split_whitespace().count();
            score += (word_count as f32 / 1000.0 * 30.0).min(30.0) as u32;
        }

        if page.title.is_some() { score += 5; }            
        if page.meta_description.is_some() { score += 5; } 
        if page.meta_keywords.is_some() { score += 5; }    
        if page.metadata.is_some() { score += 5; }         

        let link_count = page.links.len();
        score += ((link_count as f32 / 50.0 * 20.0).min(20.0)) as u32;

        if let Some(ref priority_domains) = self.config.priority_domains {
            if priority_domains.iter().any(|d| page.domain.contains(d)) {
                score += 30;
            }
        }

        score
    }

    /// Check if we have any existing results for a query
    pub async fn check_existing_results(&self, query: &str) -> Result<CrawlStatus, Box<dyn Error + Send + Sync>> {
        // Limit to 10 results for quick checking
        let results = self.storage.search_webpages(query, 10).await?;
        let count = results.len();
        
        if count == 0 {
            // Generate suggested queries by splitting and combining query terms
            let mut suggested = Vec::new();
            let terms: Vec<&str> = query.split_whitespace().collect();
            
            if terms.len() > 1 {
                // Suggest individual terms
                for term in terms.iter() {
                    suggested.push(term.to_string());
                }
                
                // Suggest pairs of terms if we have more than 2 terms
                for i in 0..terms.len()-1 {
                    suggested.push(format!("{} {}", terms[i], terms[i+1]));
                }
            }

            Ok(CrawlStatus {
                has_results: false,
                existing_results_count: 0,
                suggested_queries: suggested,
                message: format!("No existing results found for query: {}. Would you like to trigger a fresh crawl?", query)
            })
        } else {
            Ok(CrawlStatus {
                has_results: true,
                existing_results_count: count,
                suggested_queries: vec![],
                message: format!("Found {} existing results for query: {}", count, query)
            })
        }
    }
}

pub fn normalize_url(url: &str) -> Result<String, CrawlerError> {
    let parsed = Url::parse(url).map_err(|e| CrawlerError::UrlNormalizationError(e.to_string()))?;
    let mut normalized = parsed.clone();

    if (parsed.scheme() == "http" && parsed.port() == Some(80)) ||
       (parsed.scheme() == "https" && parsed.port() == Some(443)) {
        normalized.set_port(None).ok();
    }

    let path = normalized.path().to_string();
    let trimmed_path = path.trim_end_matches('/');
    normalized.set_path(trimmed_path);

    if let Some(query) = normalized.query() {
        let mut params: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();
        params.sort_by(|a, b| a.0.cmp(&b.0));
        normalized.set_query(Some(&url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish()));
    }

    Ok(normalized.to_string().to_lowercase())
}
