// config.rs
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;
use std::error::Error;

#[derive(Debug, Deserialize, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "batch_process")]
    pub batch_process_mode: String,

    #[structopt(short, long, default_value = "10")]
    pub batch_size: usize,

    #[structopt(short, long, default_value = "10")]
    pub concurrent_workers: usize,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::de::from_str(&contents)?;
        Ok(config)
    }
}
