use prometheus::{Counter, Histogram, IntGauge, register_counter, register_histogram, register_int_gauge};
use lazy_static::lazy_static;

lazy_static! {
    // Gauges
    pub static ref QUEUE_SIZE: IntGauge = register_int_gauge!(
        "crawler_queue_size",
        "Current number of URLs in the crawler queue"
    ).expect("Failed to create queue size gauge");
    
    // Counters
    pub static ref PAGES_CRAWLED: Counter = register_counter!(
        "crawler_pages_crawled",
        "Total number of pages crawled"
    ).expect("Failed to create pages crawled counter");
    
    pub static ref CRAWL_ERRORS: Counter = register_counter!(
        "crawler_errors_total", 
        "Total number of crawl errors"
    ).expect("Failed to create crawl errors counter");
    
    pub static ref CRAWL_CYCLES: Counter = register_counter!(
        "crawler_cycles_total",
        "Total number of completed crawl cycles"
    ).expect("Failed to create crawl cycles counter");
    
    // Histograms
    pub static ref CRAWL_DURATION: Histogram = register_histogram!(
        "crawler_duration_seconds",
        "Duration of crawl cycles in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).expect("Failed to create crawl duration histogram");
}

// Direct metric operations
pub fn increment_pages_crawled() {
    PAGES_CRAWLED.inc();
}

pub fn increment_crawl_errors() {
    CRAWL_ERRORS.inc();
}

pub fn increment_crawl_cycles() {
    CRAWL_CYCLES.inc();
}

pub fn set_queue_size(size: i64) {
    QUEUE_SIZE.set(size);
}

pub fn observe_crawl_duration(duration: f64) {
    CRAWL_DURATION.observe(duration);
}
