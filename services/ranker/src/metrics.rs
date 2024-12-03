use prometheus::{Counter, Histogram, IntGauge, Gauge, register_int_gauge, register_gauge, register_counter, register_histogram};
use lazy_static::lazy_static;
use reqwest::Client;
use std::error::Error;
use log::{error, info};

lazy_static! {
    // Gauges
    pub static ref PAGES_TO_RANK: IntGauge = register_int_gauge!(
        "ranker_pages_to_rank",
        "Number of pages to be ranked"
    ).expect("Failed to create pages to rank gauge");
    
    pub static ref GRAPH_SIZE: IntGauge = register_int_gauge!(
        "ranker_graph_size",
        "Current size of the web graph"
    ).expect("Failed to create graph size gauge");
    
    pub static ref AVERAGE_PAGE_RANK: Gauge = register_gauge!(
        "ranker_average_page_rank",
        "Average PageRank across all pages"
    ).expect("Failed to create average page rank gauge");
    
    // Counters
    pub static ref RANK_CALCULATION_COMPLETED_TOTAL: Counter = register_counter!(
        "ranker_calculation_completed_total",
        "Total number of completed rank calculations"
    ).expect("Failed to create rank calculations counter");
    
    pub static ref RANK_ERRORS_TOTAL: Counter = register_counter!(
        "ranker_errors_total",
        "Total number of ranking errors"
    ).expect("Failed to create rank errors counter");
    
    pub static ref RANK_CYCLES_COMPLETED_TOTAL: Counter = register_counter!(
        "ranker_cycles_total",
        "Total number of completed ranking cycles"
    ).expect("Failed to create rank cycles counter");
    
    // Histograms
    pub static ref RANK_CALCULATION_DURATION_SECONDS: Histogram = register_histogram!(
        "ranker_calculation_duration_seconds",
        "Duration of rank calculations in seconds"
    ).expect("Failed to create rank calculation duration histogram");
    
    pub static ref RANK_ITERATION_DURATION_SECONDS: Histogram = register_histogram!(
        "ranker_iteration_duration_seconds",
        "Duration of individual PageRank iterations in seconds"
    ).expect("Failed to create rank iteration duration histogram");
    
    pub static ref RANK_CONVERGENCE_ITERATIONS: Histogram = register_histogram!(
        prometheus::HistogramOpts::new(
            "ranker_convergence_iterations",
            "Number of iterations needed for PageRank convergence"
        ).buckets(vec![5.0, 10.0, 20.0, 30.0, 50.0])
    ).expect("Failed to create rank convergence iterations histogram");
}

pub fn init_metrics() {
    info!("All metrics registered successfully");
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