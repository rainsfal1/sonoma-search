//robots.rs

use reqwest::Client;
use robotxt::Robots;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use log::{info, warn};

pub struct RobotsChecker {
    client: Client,
    cache: Arc<Mutex<HashMap<String, Robots>>>,
}

impl RobotsChecker {
    pub fn new(client: Client) -> Self {
        RobotsChecker {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn is_allowed(&self, url: &str, user_agent: &str) -> bool {
        let parsed_url = match Url::parse(url) {
            Ok(url) => url,
            Err(e) => {
                warn!("Failed to parse URL {}: {}", url, e);
                return false
            },
        };

        let robots_url = match (parsed_url.scheme(), parsed_url.host_str()) {
            (scheme, Some(host)) => format!("{}://{}/robots.txt", scheme, host),
            _ => {
                warn!("Could not construct robots.txt URL for {}", url);
                return true
            },
        };

        info!("Checking robots.txt at: {}", robots_url);
        let robots = {
            let mut cache = self.cache.lock().await;
            if !cache.contains_key(&robots_url) {
                info!("Fetching robots.txt from {}", robots_url);
                let robots_content = match self.client.get(&robots_url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            info!("Successfully fetched robots.txt from {}", robots_url);
                            response.text().await.unwrap_or_default()
                        } else {
                            warn!("Failed to fetch robots.txt from {} (status: {})", robots_url, response.status());
                            String::new() // Allow crawling if robots.txt returns error
                        }
                    },
                    Err(e) => {
                        warn!("Error fetching robots.txt from {}: {}", robots_url, e);
                        String::new() // Allow crawling if robots.txt can't be fetched
                    }
                };
                let robots_file = Robots::from_bytes(robots_content.as_bytes(), user_agent);
                cache.insert(robots_url.clone(), robots_file);
            } else {
                info!("Using cached robots.txt for {}", robots_url);
            }
            cache.get(&robots_url).unwrap().clone()
        };

        let allowed = robots.is_relative_allowed(parsed_url.path());
        info!("URL {} is {} by robots.txt", url, if allowed { "allowed" } else { "disallowed" });
        allowed
    }
}