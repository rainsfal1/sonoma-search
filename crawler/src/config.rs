// config.rs
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;
use std::error::Error;

#[derive(Debug, Deserialize, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "https://example.com")]
    pub start_url: String,

    #[structopt(short, long, default_value = "3")]
    pub max_depth: usize,

    #[structopt(short, long, default_value = "1000")]
    pub max_pages: usize,

    #[structopt(short, long, default_value = "10")]
    pub concurrent_requests: usize,

    #[structopt(short, long, default_value = "100")]
    pub delay_ms: u64,

    #[structopt(short, long, default_value = "MyCrawler/1.0")]
    pub user_agent: String,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error >> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::de::from_str(&contents)?;
        Ok(config)
    }
}