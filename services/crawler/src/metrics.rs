use prometheus::{Gauge, Counter, Histogram, register_gauge, register_counter, register_histogram, HistogramOpts};
use lazy_static::lazy_static;

lazy_static! {
    // Gauges
    pub static ref CRAWL_QUEUE_SIZE: Gauge = register_gauge!(
        "crawl_queue_size",
        "Current size of the crawl queue"
    ).unwrap();
    
    // Counters
    pub static ref PAGES_CRAWLED_TOTAL: Counter = register_counter!(
        "pages_crawled_total",
        "Total number of pages crawled"
    ).unwrap();
    
    pub static ref CRAWL_ERRORS_TOTAL: Counter = register_counter!(
        "crawl_errors_total",
        "Total number of crawl errors"
    ).unwrap();
    
    pub static ref CRAWL_CYCLES_COMPLETED_TOTAL: Counter = register_counter!(
        "crawl_cycles_completed_total",
        "Total number of completed crawl cycles"
    ).unwrap();
    
    // Histograms
    pub static ref CRAWL_DURATION_SECONDS: Histogram = register_histogram!(
        HistogramOpts::new(
            "crawl_duration_seconds",
            "Duration of crawl cycles in seconds"
        )
        .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
    ).unwrap();
}

// Direct metric operations
pub fn increment_pages_crawled() {
    PAGES_CRAWLED_TOTAL.inc();
}

pub fn increment_crawl_errors() {
    CRAWL_ERRORS_TOTAL.inc();
}

pub fn increment_crawl_cycles() {
    CRAWL_CYCLES_COMPLETED_TOTAL.inc();
}

pub fn set_queue_size(size: i64) {
    CRAWL_QUEUE_SIZE.set(size as f64);
}

pub fn observe_crawl_duration(duration: f64) {
    CRAWL_DURATION_SECONDS.observe(duration);
}
