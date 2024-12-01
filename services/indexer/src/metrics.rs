use reqwest::Client;
use prometheus::{Registry, Counter, Histogram, IntGauge, HistogramOpts, register_int_gauge, register_counter, register_histogram};
use lazy_static::lazy_static;

lazy_static! {
    // Gauges
    pub static ref PROCESSING_QUEUE_SIZE: IntGauge = register_int_gauge!("indexer_queue_size", "Current size of the indexing queue").expect("Failed to create queue size gauge");
    
    pub static ref ELASTICSEARCH_DOCS_COUNT: IntGauge = register_int_gauge!("indexer_docs_count", "Current number of documents in Elasticsearch").expect("Failed to create docs count gauge");
    
    // Counters
    pub static ref DOCS_PROCESSED_TOTAL: Counter = register_counter!("indexer_docs_processed_total", "Total number of documents processed").expect("Failed to create docs processed counter");
    
    pub static ref INDEX_ERRORS_TOTAL: Counter = register_counter!("indexer_errors_total", "Total number of indexing errors").expect("Failed to create index errors counter");
    
    pub static ref INDEX_CYCLES_COMPLETED_TOTAL: Counter = register_counter!("indexer_cycles_total", "Total number of completed indexing cycles").expect("Failed to create index cycles counter");
    
    // Histograms
    pub static ref INDEX_DURATION_SECONDS: Histogram = register_histogram!("indexer_duration_seconds", "Duration of indexing operations in seconds").expect("Failed to create index duration histogram");
    
    pub static ref DOCUMENT_PROCESSING_DURATION_SECONDS: Histogram = register_histogram!("indexer_doc_processing_duration_seconds", "Duration of individual document processing in seconds").expect("Failed to create document processing duration histogram");
}

#[derive(Clone)]
pub struct MetricsClient {
    _client: Client,
    _base_url: String,
}

impl MetricsClient {
    pub fn new(base_url: String) -> Self {
        Self {
            _client: Client::new(),
            _base_url: base_url,
        }
    }

    pub fn increment_docs_processed(&self) {
        DOCS_PROCESSED_TOTAL.inc();
    }

    pub fn increment_index_errors(&self) {
        INDEX_ERRORS_TOTAL.inc();
    }

    // Used during index cycle completion
    pub fn increment_index_cycles(&self) {
        INDEX_CYCLES_COMPLETED_TOTAL.inc();
    }

    pub fn observe_processing_duration(&self, duration_secs: f64) {
        DOCUMENT_PROCESSING_DURATION_SECONDS.observe(duration_secs);
    }

    pub fn observe_index_duration(&self, duration_secs: f64) {
        INDEX_DURATION_SECONDS.observe(duration_secs);
    }

    pub fn set_queue_size(&self, size: i64) {
        PROCESSING_QUEUE_SIZE.set(size);
    }

    pub fn get_queue_size(&self) -> i64 {
        PROCESSING_QUEUE_SIZE.get()
    }

    // Used during Elasticsearch sync
    pub fn set_elasticsearch_docs_count(&self, count: i64) {
        ELASTICSEARCH_DOCS_COUNT.set(count);
    }
}