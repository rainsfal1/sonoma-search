use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;
use std::error::Error;
use std::path::Path;

/// Struct `Config` represents the configuration options for a web crawler.
/// It holds parameters like the start URL, crawl depth, number of pages to fetch,
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
#[derive(Debug, Deserialize, StructOpt)]
pub struct Config {
    /// The starting URL for the web crawler.
    /// This is the entry point for crawling and is provided either via
    /// the command line or from a configuration file.
    /// Default value: "https://example.com"
    #[structopt(short, long, default_value = "https://example.com")]
    pub start_url: String,

    /// The maximum depth of the crawl. Depth refers to how far the crawler
    /// will follow links from the starting page.
    /// Depth 0 means only the start_url will be fetched.
    /// Each increment in depth allows the crawler to go one level deeper into linked pages.
    /// Default value: 3
    #[structopt(short, long, default_value = "3")]
    pub max_depth: usize,

    /// The maximum number of pages that the crawler will attempt to crawl.
    /// Once this limit is reached, the crawler will stop.
    /// Default value: 1000
    #[structopt(short, long, default_value = "1000")]
    pub max_pages: usize,

    /// Specifies the maximum number of concurrent HTTP requests that can be made at once.
    /// This is useful for limiting resource usage and for politeness in web crawling,
    /// ensuring that the crawler does not overwhelm servers with too many requests at the same time.
    /// Default value: 10
    #[structopt(short, long, default_value = "10")]
    pub concurrent_requests: usize,

    /// Specifies the delay in milliseconds between consecutive HTTP requests.
    /// This is another way to implement crawling politeness by preventing the crawler
    /// from hammering a server with too many requests in rapid succession.
    /// Default value: 100 ms
    #[structopt(short, long, default_value = "100")]
    pub delay_ms: u64,

    /// The User-Agent string that will be sent in the HTTP requests made by the crawler.
    /// This helps servers identify the crawler, and it’s a good practice to provide a clear and honest User-Agent.
    /// Default value: "MyCrawler/1.0"
    #[structopt(short, long, default_value = "MyCrawler/1.0")]
    pub user_agent: String,
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
        let path = path.as_ref();

        let mut file = File::open(path)?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
