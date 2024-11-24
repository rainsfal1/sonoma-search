use reqwest::Client;
use std::error::Error;
use log::error;
use prometheus::{Registry, Gauge, Counter, Histogram, register_gauge, register_counter, register_histogram, HistogramOpts};
use lazy_static::lazy_static;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    
    // Gauges
    static ref PAGES_TO_RANK: Gauge = register_gauge!(
        "pages_to_rank",
        "Number of pages to be ranked"
    ).unwrap();
    
    static ref GRAPH_SIZE: Gauge = register_gauge!(
        "graph_size",
        "Current size of the web graph"
    ).unwrap();
    
    static ref AVERAGE_PAGE_RANK: Gauge = register_gauge!(
        "average_page_rank",
        "Average PageRank across all pages"
    ).unwrap();
    
    // Counters
    static ref RANK_CALCULATION_COMPLETED_TOTAL: Counter = register_counter!(
        "rank_calculation_completed_total",
        "Total number of completed rank calculations"
    ).unwrap();
    
    static ref RANK_ERRORS_TOTAL: Counter = register_counter!(
        "rank_errors_total",
        "Total number of ranking errors"
    ).unwrap();
    
    static ref RANK_CYCLES_COMPLETED_TOTAL: Counter = register_counter!(
        "rank_cycles_completed_total",
        "Total number of completed ranking cycles"
    ).unwrap();
    
    // Histograms
    static ref RANK_CALCULATION_DURATION_SECONDS: Histogram = register_histogram!(
        HistogramOpts::new(
            "rank_calculation_duration_seconds",
            "Duration of rank calculations in seconds"
        )
        .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0])
    ).unwrap();
    
    static ref RANK_ITERATION_DURATION_SECONDS: Histogram = register_histogram!(
        HistogramOpts::new(
            "rank_iteration_duration_seconds",
            "Duration of individual PageRank iterations in seconds"
        )
        .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0])
    ).unwrap();
    
    static ref RANK_CONVERGENCE_ITERATIONS: Histogram = register_histogram!(
        HistogramOpts::new(
            "rank_convergence_iterations",
            "Number of iterations needed for PageRank convergence"
        )
        .buckets(vec![5.0, 10.0, 20.0, 30.0, 50.0])
    ).unwrap();
}

pub struct MetricsClient {
    _client: Client,
    _base_url: String,
}

impl MetricsClient {
    pub fn new(base_url: String) -> Self {
        MetricsClient {
            _client: Client::new(),
            _base_url: base_url,
        }
    }

    pub async fn increment(&self, metric: &str) -> Result<(), Box<dyn Error>> {
        match metric {
            "rank_calculation_completed" => RANK_CALCULATION_COMPLETED_TOTAL.inc(),
            "rank_errors" => RANK_ERRORS_TOTAL.inc(),
            "rank_cycles_completed" => RANK_CYCLES_COMPLETED_TOTAL.inc(),
            _ => error!("Unknown counter metric: {}", metric),
        }
        Ok(())
    }

    pub async fn observe_histogram(&self, metric: &str, value: f64) -> Result<(), Box<dyn Error>> {
        match metric {
            "rank_calculation_duration_seconds" => RANK_CALCULATION_DURATION_SECONDS.observe(value),
            "rank_iteration_duration_seconds" => RANK_ITERATION_DURATION_SECONDS.observe(value),
            "rank_convergence_iterations" => RANK_CONVERGENCE_ITERATIONS.observe(value),
            _ => error!("Unknown histogram metric: {}", metric),
        }
        Ok(())
    }
}

pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}