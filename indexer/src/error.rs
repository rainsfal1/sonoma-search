use thiserror::Error;
use elasticsearch::Error as ElasticsearchError;
use std::io;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Elasticsearch error: {0}")]
    Elasticsearch(#[from] ElasticsearchError),

    #[error("Content processing error: {0}")]
    ContentProcessing(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("UUID conversion error: {0}")]
    UuidConversion(#[from] uuid::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Retry error: {0}")]
    Retry(String),

    #[error("Generic error: {0}")]
    GenericError(String),

    #[error("Failed to create Elasticsearch index: {0}")]
    IndexCreationFailed(String),
}

pub type IndexerResult<T> = Result<T, IndexerError>;
