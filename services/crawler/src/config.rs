use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;
use std::error::Error;
use std::path::Path;
use url::Url;

/// Struct `Config` represents the configuration options for a web crawler.
/// It holds parameters like the start URLs, crawl depth, number of pages to fetch,
/// and other settings related to concurrency and rate limiting.
///
/// The struct uses two main functionalities:
/// 1. **Serde** for deserializing TOML file configurations.
/// 2. **StructOpt** for parsing command-line arguments into structured configuration.
///
/// Users can provide these configurations via:
/// - Command-line arguments
/// - A TOML configuration file that can be loaded and parsed into the `Config` struct.
///
/// If no configuration is provided via the command-line, default values are used.
#[derive(Debug, Clone, Deserialize, StructOpt)]
pub struct Config {
    /// URLs to start crawling from
    #[structopt(long, use_delimiter = true, default_value = "https://docs.rs,https://rust-lang.org,https://crates.io,https://blog.rust-lang.org,https://github.com/rust-lang")]
    pub seed_urls: Vec<String>,

    /// User agent string to identify the crawler
    #[structopt(long, default_value = "MyCrawler/1.0 (+https://github.com/yourusername/search-engine)")]
    pub user_agent: String,

    /// Maximum depth to crawl
    #[structopt(short, long, default_value = "3")]
    pub max_depth: usize,

    /// Maximum number of pages to crawl
    #[structopt(short, long, default_value = "1000")]
    pub max_pages: usize,

    /// Number of concurrent requests
    #[structopt(short, long, default_value = "8")]
    pub concurrent_requests: usize,

    /// Delay between requests in milliseconds
    #[structopt(long, default_value = "500")]
    pub request_delay: u64,

    /// Maximum size of page content to fetch (in bytes)
    #[structopt(long, default_value = "5242880")]  // 5MB
    pub max_content_size: usize,

    /// Minimum quality score for a page to be stored
    #[structopt(long, default_value = "40")]
    pub min_quality_score: u32,

    /// Priority domains to focus on (comma-separated)
    #[structopt(long, use_delimiter = true)]
    pub priority_domains: Option<Vec<String>>,

    /// List of allowed domains (empty means all domains are allowed)
    #[structopt(long, use_delimiter = true)]
    pub allowed_domains: Option<Vec<String>>,

    /// List of blocked domains
    #[structopt(long, use_delimiter = true)]
    pub blocked_domains: Option<Vec<String>>,
}

impl Config {
    /// The `from_file` function allows you to load a configuration from a file.
    ///
    /// This function reads a file containing configuration parameters in TOML format
    /// and deserializes it into a `Config` struct. The primary use case for this method
    /// is to allow the user to provide configurations via a file, instead of relying on
    /// command-line arguments alone.
    ///
    /// ### Parameters:
    /// - `file_path`: A string slice that specifies the path to the configuration file.
    ///
    /// ### Returns:
    /// - `Ok(Config)` on success, containing the deserialized configuration.
    /// - `Err(Box<dyn Error>)` if an error occurs during file reading or TOML parsing.
    ///
    /// ### Steps:
    /// 1. The function opens the specified file. If the file cannot be opened, an error is returned.
    /// 2. The contents of the file are read into a string.
    /// 3. The string is then parsed from TOML format into the `Config` struct using Serde's TOML deserialization.
    /// 4. If successful, the `Config` struct is returned. Otherwise, any encountered error is propagated.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }

    /// Checks if a link should be followed based on the configuration.
    /// This checks if the URL is within the allowed domains and not in blocked domains.
    pub fn should_follow_link(&self, url: &str, _source_domain: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            let target_domain = parsed_url.host_str().unwrap_or("").to_string();

            // Check blocked domains
            if let Some(ref blocked) = self.blocked_domains {
                if blocked.iter().any(|d| target_domain.contains(d)) {
                    return false;
                }
            }

            // If we have allowed domains, check if the target domain is allowed
            if let Some(ref allowed) = self.allowed_domains {
                return allowed.iter().any(|d| target_domain.contains(d));
            }
        }
        true
    }
}
