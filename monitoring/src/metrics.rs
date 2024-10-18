use lazy_static::lazy_static;
use prometheus::{Registry, Counter, IntGauge, Histogram};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Crawler metrics
    pub static ref PAGES_CRAWLED: Counter = Counter::new("pages_crawled_total", "Total number of pages crawled").expect("metric can be created");
    pub static ref CRAWL_ERRORS: Counter = Counter::new("crawl_errors_total", "Total number of crawl errors").expect("metric can be created");
    pub static ref CRAWL_QUEUE_SIZE: IntGauge = IntGauge::new("crawl_queue_size", "Current size of the crawl queue").expect("metric can be created");
    pub static ref CRAWL_DURATION: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("crawl_duration_seconds", "Duration of crawl operations")
    ).expect("metric can be created");

    // You can add more metrics for indexer and searcher here as needed
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(PAGES_CRAWLED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(CRAWL_ERRORS.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(CRAWL_QUEUE_SIZE.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(CRAWL_DURATION.clone())).expect("collector can be registered");
    // Register any additional metrics here
}

pub fn increment_metric(metric: &str) {
    match metric {
        "pages_crawled" => PAGES_CRAWLED.inc(),
        "crawl_errors" => CRAWL_ERRORS.inc(),
        _ => println!("Unknown metric: {}", metric),
    }
}

pub fn set_gauge(metric: &str, value: f64) {
    match metric {
        "crawl_queue_size" => CRAWL_QUEUE_SIZE.set(value as i64),
        _ => println!("Unknown gauge metric: {}", metric),
    }
}

pub fn observe_histogram(metric: &str, value: f64) {
    match metric {
        "crawl_duration_seconds" => CRAWL_DURATION.observe(value),
        _ => println!("Unknown histogram metric: {}", metric),
    }
}