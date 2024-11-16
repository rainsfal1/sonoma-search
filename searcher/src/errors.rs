use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("BM25 score processing error: {0}")]
    ScoreProcessingError(String),
}
