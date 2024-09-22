mod document;
mod tokenizer;
mod index_writer;
mod schema;
mod config;
mod error;

use document::DocumentProcessor;
use index_writer::IndexWriter;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables from the .env file
    dotenv().ok();

    let index_writer = IndexWriter::new().unwrap();
    let doc_processor = DocumentProcessor::new();

    // Example usage: Batch process documents and add them to the index
    let documents = doc_processor.load_documents().await.unwrap();
    index_writer.index_documents(documents).unwrap();
}
