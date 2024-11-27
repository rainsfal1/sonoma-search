use reqwest::Client;
use prometheus::{Registry, Gauge, Counter, Histogram, HistogramOpts, register_gauge_with_registry, register_counter_with_registry, register_histogram_with_registry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref PROCESSING_QUEUE_SIZE: Gauge = register_gauge_with_registry!(
        "processing_queue_size",
        "Current size of the indexing queue",
        REGISTRY
    ).unwrap();
    
    pub static ref ELASTICSEARCH_DOCS_COUNT: Gauge = register_gauge_with_registry!(
        "elasticsearch_docs_count",
        "Current number of documents in Elasticsearch",
        REGISTRY
    ).unwrap();
    
    pub static ref DOCS_PROCESSED_TOTAL: Counter = register_counter_with_registry!(
        "docs_processed_total",
        "Total number of documents processed",
        REGISTRY
    ).unwrap();
    
    pub static ref INDEX_ERRORS_TOTAL: Counter = register_counter_with_registry!(
        "index_errors_total",
        "Total number of indexing errors",
        REGISTRY
    ).unwrap();
    
    pub static ref INDEX_CYCLES_COMPLETED_TOTAL: Counter = register_counter_with_registry!(
        "index_cycles_completed_total",
        "Total number of completed indexing cycles",
        REGISTRY
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
        PROCESSING_QUEUE_SIZE.set(size as f64);
    }

    pub fn get_queue_size(&self) -> f64 {
        PROCESSING_QUEUE_SIZE.get()
    }

    // Used during Elasticsearch sync
    pub fn set_elasticsearch_docs_count(&self, count: i64) {
        ELASTICSEARCH_DOCS_COUNT.set(count as f64);
    }
}