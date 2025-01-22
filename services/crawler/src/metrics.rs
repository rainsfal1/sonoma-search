use prometheus::{Counter, Histogram, IntGauge, Opts, HistogramOpts};
use lazy_static::lazy_static;
use std::sync::Once;

static METRICS_INIT: Once = Once::new();

lazy_static! {
    // Gauges
    pub static ref QUEUE_SIZE: IntGauge = IntGauge::with_opts(
        Opts::new("crawler_queue_size", "Current number of URLs in the crawler queue")
    )
    .expect("Failed to create queue size gauge");
    
    // Counters
    pub static ref PAGES_CRAWLED: Counter = Counter::with_opts(
        Opts::new("crawler_pages_crawled", "Total number of pages crawled")
    )
    .expect("Failed to create pages crawled counter");
    
    pub static ref CRAWL_ERRORS: Counter = Counter::with_opts(
        Opts::new("crawler_errors_total", "Total number of crawl errors")
    )
    .expect("Failed to create crawl errors counter");
    
    pub static ref CRAWL_CYCLES: Counter = Counter::with_opts(
        Opts::new("crawler_cycles_total", "Total number of completed crawl cycles")
    )
    .expect("Failed to create crawl cycles counter");
    
    // Histograms
    pub static ref CRAWL_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("crawler_duration_seconds", "Duration of crawl cycles in seconds")
    )
    .expect("Failed to create crawl duration histogram");
}

pub fn init_metrics() {
    METRICS_INIT.call_once(|| {
        // Only try to register metrics once
        let registry = prometheus::default_registry();
        
        // Register metrics, ignoring AlreadyReg errors
        let _ = registry.register(Box::new(QUEUE_SIZE.clone()));
        let _ = registry.register(Box::new(PAGES_CRAWLED.clone()));
        let _ = registry.register(Box::new(CRAWL_ERRORS.clone()));
        let _ = registry.register(Box::new(CRAWL_CYCLES.clone()));
        let _ = registry.register(Box::new(CRAWL_DURATION.clone()));
    });
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
