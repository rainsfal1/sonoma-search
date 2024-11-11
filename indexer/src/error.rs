use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Elasticsearch error: {0}")]
    Elasticsearch(#[from] elasticsearch::Error),

    #[error("Content processing error: {0}")]
    ContentProcessing(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Retry error: {0}")]
    Retry(String),
}

pub type IndexerResult<T> = Result<T, IndexerError>; 