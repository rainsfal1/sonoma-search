use reqwest::Client;
use prometheus::{Registry, Gauge, Counter, Histogram, HistogramOpts, register_gauge, register_counter, register_histogram_with_registry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref PROCESSING_QUEUE_SIZE: Gauge = register_gauge!(
        "processing_queue_size",
        "Current size of the indexing queue"
    ).unwrap();
    
    pub static ref ELASTICSEARCH_DOCS_COUNT: Gauge = register_gauge!(
        "elasticsearch_docs_count",
        "Current number of documents in Elasticsearch"
    ).unwrap();
    
    pub static ref DOCS_PROCESSED_TOTAL: Counter = register_counter!(
        "docs_processed_total",
        "Total number of documents processed"
    ).unwrap();
    
    pub static ref INDEX_ERRORS_TOTAL: Counter = register_counter!(
        "index_errors_total",
        "Total number of indexing errors"
    ).unwrap();
    
    pub static ref INDEX_CYCLES_COMPLETED_TOTAL: Counter = register_counter!(
        "index_cycles_completed_total",
        "Total number of completed indexing cycles"
    ).unwrap();
    
    pub static ref INDEX_DURATION_SECONDS: Histogram = register_histogram_with_registry!(
        HistogramOpts::new(
            "index_duration_seconds",
            "Duration of indexing operations in seconds"
        )
        .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]),
        REGISTRY
    ).unwrap();
    
    pub static ref DOCUMENT_PROCESSING_DURATION_SECONDS: Histogram = register_histogram_with_registry!(
        HistogramOpts::new(
            "document_processing_duration_seconds",
            "Duration of individual document processing in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0]),
        REGISTRY
    ).unwrap();
}

#[derive(Clone)]
#[allow(dead_code)]  // Fields used internally by reqwest client
pub struct MetricsClient {
    base_url: String,
    client: Client,
}

impl MetricsClient {
    pub fn new(base_url: String) -> Self {
        MetricsClient {
            base_url,
            client: Client::new(),
        }
    }

    pub fn increment_docs_processed(&self) {
        DOCS_PROCESSED_TOTAL.inc();
    }

    pub fn increment_index_errors(&self) {
        INDEX_ERRORS_TOTAL.inc();
    }

    #[allow(dead_code)]  // Used during index cycle completion
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
        PROCESSING_QUEUE_SIZE.set(size as f64);
    }

    pub fn get_queue_size(&self) -> f64 {
        PROCESSING_QUEUE_SIZE.get()
    }

    #[allow(dead_code)]  // Used during Elasticsearch sync
    pub fn set_elasticsearch_docs_count(&self, count: i64) {
        ELASTICSEARCH_DOCS_COUNT.set(count as f64);
    }
}