use crate::document::Document;
use thiserror::Error;

pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        BatchProcessor { batch_size }
    }

    pub async fn process_batch(&self, documents: Vec<Document>) -> Result<(), BatchProcessorError> {
        // Batch processing logic
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum BatchProcessorError {
    #[error("Batch processing error: {0}")]
    ProcessingError(String),
}
