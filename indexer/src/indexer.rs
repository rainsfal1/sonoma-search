use tantivy::{Index, IndexWriter, Document as TantivyDocument, Term};
use tantivy::schema::{Schema, Field};
use tantivy::query::QueryParser;
use tokio::sync::mpsc;
use futures::StreamExt;
use rayon::prelude::*;
use crate::{config::Config, schema::create_schema, document::Document, batch_processor::BatchProcessor};
use thiserror::Error;
use crate::storage::IndexStorage;
use std::sync::Arc;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Document processing error: {0}")]
    DocumentError(#[from] crate::document::DocumentError),
}

pub struct Indexer {
    index: Index,
    writer: IndexWriter,
    schema: Schema,
    title_field: Field,
    content_field: Field,
    url_field: Field,
    batch_processor: BatchProcessor,
}

impl Indexer {
    pub fn new(config: Config, index_storage: IndexStorage) -> Result<Self, IndexerError> {
        // ... (initialization code remains the same)
        todo!()
    }

    pub async fn run(&mut self) -> Result<(), IndexerError> {
        let (tx, mut rx) = mpsc::channel(100);

        // Use tokio for I/O-bound document fetching
        tokio::spawn(fetch_documents(tx));

        while let Some(batch) = rx.recv().await {
            // Use Rayon for CPU-bound parallel processing of documents
            let processed_batch: Vec<Document> = batch.par_iter()
                .map(|doc| self.batch_processor.process_document(doc))
                .collect::<Result<Vec<_>, _>>()?;

            self.index_batch(processed_batch).await?;
        }

        self.writer.commit()?;
        Ok(())
    }

    async fn index_batch(&mut self, batch: Vec<Document>) -> Result<(), IndexerError> {
        for doc in batch {
            let mut tantivy_doc = TantivyDocument::new();
            tantivy_doc.add_text(self.title_field, &doc.title);
            tantivy_doc.add_text(self.content_field, &doc.content);
            tantivy_doc.add_text(self.url_field, &doc.url);

            self.writer.add_document(tantivy_doc)?;
        }
        Ok(())
    }
    pub fn search(&self, query: &str) -> Result<Vec<TantivyDocument>, IndexerError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);
        let query = query_parser.parse_query(query)?;

        let top_docs = searcher.search(&query, &tantivy::collector::TopDocs::with_limit(10))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            results.push(retrieved_doc);
        }

        Ok(results)
    }
}


async fn fetch_documents(tx: mpsc::sender<Vec<Document>>) -> Result<(), IndexerError> {
    // Implement document fetching logic here
    // This could involve reading from a database, crawling websites, etc.
    // Send batches of documents through the channel
    Ok(())
}