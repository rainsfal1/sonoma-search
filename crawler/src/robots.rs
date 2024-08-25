use reqwest::Client;
use robotxt::Robots;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

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
            Err(_) => return false, // Invalid URL, assume it's not allowed
        };

        let robots_url = match (parsed_url.scheme(), parsed_url.host_str()) {
            (scheme, Some(host)) => format!("{}://{}/robots.txt", scheme, host),
            _ => return true, // If we can't construct a robots.txt URL, assume it's allowed
        };

        let robots = {
            let mut cache = self.cache.lock().await;
            if !cache.contains_key(&robots_url) {
                let robots_content = match self.client.get(&robots_url).send().await {
                    Ok(response) => response.text().await.unwrap_or_default(),
                    Err(_) => String::new(),
                };
                let robots_file = Robots::from_bytes(robots_content.as_bytes(), user_agent);
                cache.insert(robots_url.clone(), robots_file);
            }
            cache.get(&robots_url).unwrap().clone()
        };

        robots.is_relative_allowed(parsed_url.path())
    }
}