use thiserror::Error;
use reqwest::StatusCode;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("HTTP request failed with status: {0}")]
    StatusError(StatusCode),

    #[error("Failed to normalize URL: {0}")]
    UrlNormalizationError(String),

    #[error("Max retries reached")]
    MaxRetriesReached,

    #[error("Response error: {0}")]
    ResponseError(reqwest::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Content too large: {0} bytes")]
    ContentTooLarge(u64),
}