//document.rs

use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Document processing error: {0}")]
    ProcessingError(Cow<'static, str>),  // Use Cow for static and dynamic strings
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document<'a> {
    pub id: Cow<'a, str>,
    pub title: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub url: Cow<'a, str>,
}

impl<'a> Document<'a> {
    pub fn new(id: &'a str, title: &'a str, content: &'a str, url: &'a str) -> Self {
        Document {
            id: Cow::Borrowed(id),
            title: Cow::Borrowed(title),
            content: Cow::Borrowed(content),
            url: Cow::Borrowed(url),
        }
    }
}


// Example: Parallel processing using rayon (adjust according to your program flow)
fn process_documents_parallel(docs: Vec<Document>) -> Result<(), DocumentError> {
    docs.par_iter().try_for_each(|doc| {
        // Process each document in parallel
        // Custom logic goes here
        Ok(())
    })
}