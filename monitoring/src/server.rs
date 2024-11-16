use actix_web::{web, HttpResponse, Responder, Result};
use prometheus::{Encoder, TextEncoder};
use serde::Deserialize;
use log::error;
use crate::metrics;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/metrics").route(web::get().to(metrics_handler)))
       .service(web::resource("/increment").route(web::post().to(increment_metric)))
       .service(web::resource("/gauge").route(web::post().to(set_gauge)))
       .service(web::resource("/histogram").route(web::post().to(observe_histogram)))
       .service(web::resource("/health").route(web::get().to(health_check)));
}

async fn metrics_handler() -> Result<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = metrics::REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).map_err(|e| {
        error!("Failed to encode metrics: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to encode metrics")
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(String::from_utf8(buffer).map_err(|e| {
            error!("Failed to convert metrics to UTF-8: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to process metrics")
        })?))
}

#[derive(Deserialize)]
struct IncrementMetric {
    metric: String,
}

async fn increment_metric(metric: web::Json<IncrementMetric>) -> Result<HttpResponse> {
    metrics::increment_metric(&metric.metric);
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
struct GaugeMetric {
    metric: String,
    value: f64,
}

async fn set_gauge(gauge: web::Json<GaugeMetric>) -> Result<HttpResponse> {
    metrics::set_gauge(&gauge.metric, gauge.value);
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
struct HistogramMetric {
    metric: String,
    value: f64,
}

async fn observe_histogram(histogram: web::Json<HistogramMetric>) -> Result<HttpResponse> {
    metrics::observe_histogram(&histogram.metric, histogram.value);
    Ok(HttpResponse::Ok().finish())
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Monitoring service is running")
}
