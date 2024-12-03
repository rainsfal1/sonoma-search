use std::fmt;
use sqlx::Error as SqlxError;
use elasticsearch::Error as ElasticsearchError;

#[derive(Debug)]
pub enum IndexerError {
    Database(SqlxError),
    Elasticsearch(ElasticsearchError),
    Server(String),
    Processing(String),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    TaskJoin(tokio::task::JoinError),
    Retry(String),
    Other(String),
}

impl std::error::Error for IndexerError {}

impl fmt::Display for IndexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexerError::Database(e) => write!(f, "Database error: {}", e),
            IndexerError::Elasticsearch(e) => write!(f, "Elasticsearch error: {}", e),
            IndexerError::Server(e) => write!(f, "Server error: {}", e),
            IndexerError::Processing(e) => write!(f, "Processing error: {}", e),
            IndexerError::Io(e) => write!(f, "IO error: {}", e),
            IndexerError::Serialization(e) => write!(f, "JSON serialization error: {}", e),
            IndexerError::TaskJoin(e) => write!(f, "Task join error: {}", e),
            IndexerError::Retry(e) => write!(f, "Retry error: {}", e),
            IndexerError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<SqlxError> for IndexerError {
    fn from(error: SqlxError) -> Self {
        IndexerError::Database(error)
    }
}

impl From<ElasticsearchError> for IndexerError {
    fn from(error: ElasticsearchError) -> Self {
        IndexerError::Elasticsearch(error)
    }
}

impl From<std::io::Error> for IndexerError {
    fn from(error: std::io::Error) -> Self {
        IndexerError::Io(error)
    }
}

impl From<serde_json::Error> for IndexerError {
    fn from(error: serde_json::Error) -> Self {
        IndexerError::Serialization(error)
    }
}

impl From<tokio::task::JoinError> for IndexerError {
    fn from(error: tokio::task::JoinError) -> Self {
        IndexerError::TaskJoin(error)
    }
}

pub type IndexerResult<T> = Result<T, IndexerError>;