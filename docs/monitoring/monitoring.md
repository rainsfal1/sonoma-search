# Monitoring and Observability

This document describes the monitoring and observability setup for the Sonoma Search engine.

## Overview

The monitoring stack consists of:
- Prometheus: Metrics collection and storage
- Grafana: Visualization and alerting
- Service metrics endpoints
- Log aggregation
- Health checks
- Tracing

## Prometheus Setup

### Configuration
```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'api-gateway'
    static_configs:
      - targets: ['api-gateway:8000']
    metrics_path: '/metrics'

  - job_name: 'searcher'
    static_configs:
      - targets: ['searcher:8001']
    metrics_path: '/metrics'

  - job_name: 'ranker'
    static_configs:
      - targets: ['ranker:8002']
    metrics_path: '/metrics'

  - job_name: 'indexer'
    static_configs:
      - targets: ['indexer:8003']
    metrics_path: '/metrics'

  - job_name: 'crawler'
    static_configs:
      - targets: ['crawler:8004']
    metrics_path: '/metrics'

  - job_name: 'storage'
    static_configs:
      - targets: ['storage:8005']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'elasticsearch'
    static_configs:
      - targets: ['elasticsearch-exporter:9114']

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### Alert Rules
```yaml
# prometheus/rules/alerts.yml
groups:
  - name: service_alerts
    rules:
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "Service has been down for more than 1 minute"

      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate on {{ $labels.job }}"
          description: "Error rate is above 10% for 5 minutes"

      - alert: HighLatency
        expr: http_request_duration_seconds{quantile="0.9"} > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.job }}"
          description: "90th percentile latency is above 1s for 5 minutes"
```

## Grafana Dashboards

### Overview Dashboard
```json
{
  "title": "Sonoma Search Overview",
  "panels": [
    {
      "title": "Service Status",
      "type": "stat",
      "targets": [
        {
          "expr": "up",
          "legendFormat": "{{job}}"
        }
      ]
    },
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total[5m])",
          "legendFormat": "{{job}}"
        }
      ]
    },
    {
      "title": "Error Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
          "legendFormat": "{{job}}"
        }
      ]
    },
    {
      "title": "Latency",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])",
          "legendFormat": "{{job}}"
        }
      ]
    }
  ]
}
```

### Service-Specific Metrics

#### Searcher Dashboard
```json
{
  "title": "Searcher Service",
  "panels": [
    {
      "title": "Search Requests",
      "targets": [
        {
          "expr": "rate(search_requests_total[5m])",
          "legendFormat": "requests/sec"
        }
      ]
    },
    {
      "title": "Search Latency",
      "targets": [
        {
          "expr": "rate(search_duration_seconds_sum[5m]) / rate(search_duration_seconds_count[5m])",
          "legendFormat": "avg latency"
        }
      ]
    },
    {
      "title": "Cache Hit Rate",
      "targets": [
        {
          "expr": "rate(search_cache_hits_total[5m]) / rate(search_requests_total[5m])",
          "legendFormat": "hit rate"
        }
      ]
    }
  ]
}
```

#### Crawler Dashboard
```json
{
  "title": "Crawler Service",
  "panels": [
    {
      "title": "Pages Crawled",
      "targets": [
        {
          "expr": "rate(pages_crawled_total[5m])",
          "legendFormat": "pages/sec"
        }
      ]
    },
    {
      "title": "Crawl Queue Size",
      "targets": [
        {
          "expr": "crawler_queue_size",
          "legendFormat": "queue size"
        }
      ]
    },
    {
      "title": "Crawl Errors",
      "targets": [
        {
          "expr": "rate(crawler_errors_total[5m])",
          "legendFormat": "{{error_type}}"
        }
      ]
    }
  ]
}
```

## Service Metrics

### Common Metrics
```rust
// HTTP metrics
http_requests_total{method, path, status}
http_request_duration_seconds{method, path}
http_response_size_bytes{method, path}

// System metrics
process_cpu_seconds_total
process_resident_memory_bytes
process_open_fds
process_max_fds

// Runtime metrics
rust_gc_collections_total
rust_gc_collection_duration_seconds
rust_memory_allocated_bytes
```

### Service-Specific Metrics

#### Searcher
```rust
// Search metrics
search_requests_total
search_duration_seconds
search_results_total
search_cache_hits_total
search_cache_misses_total
search_errors_total{error_type}

// Elasticsearch metrics
elasticsearch_requests_total
elasticsearch_request_duration_seconds
elasticsearch_errors_total{error_type}
```

#### Crawler
```rust
// Crawl metrics
pages_crawled_total
crawler_queue_size
crawler_errors_total{error_type}
crawler_retry_count
crawler_request_duration_seconds
crawler_robots_txt_cache_size
```

## Health Checks

### Endpoint Configuration
```rust
// Health check response format
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
    dependencies: Vec<DependencyHealth>,
}

#[derive(Serialize)]
struct DependencyHealth {
    name: String,
    status: String,
    latency: f64,
}

// Implementation
async fn health_check() -> impl Responder {
    let mut health = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime(),
        dependencies: vec![],
    };

    // Check dependencies
    if let Err(_) = check_database().await {
        health.status = "degraded".to_string();
    }

    if let Err(_) = check_elasticsearch().await {
        health.status = "degraded".to_string();
    }

    HttpResponse::Ok().json(health)
}
```

### Monitoring Script
```bash
#!/bin/bash
# health-check.sh

services=(
    "api-gateway:8000"
    "searcher:8001"
    "ranker:8002"
    "indexer:8003"
    "crawler:8004"
    "storage:8005"
)

check_service() {
    local service=$1
    local response=$(curl -s -o /dev/null -w "%{http_code}" http://$service/health)
    if [ "$response" == "200" ]; then
        echo "✅ $service: OK"
    else
        echo "❌ $service: FAIL ($response)"
    fi
}

for service in "${services[@]}"; do
    check_service $service
done
```

## Log Aggregation

### Logging Configuration
```yaml
# vector/vector.toml
[sources.docker]
type = "docker_logs"
include_containers = ["sonoma-*"]

[transforms.parse_json]
type = "json_parser"
inputs = ["docker"]
field = "message"

[transforms.add_metadata]
type = "add_fields"
inputs = ["parse_json"]
fields.environment = "production"
fields.datacenter = "us-west"

[sinks.elasticsearch]
type = "elasticsearch"
inputs = ["add_metadata"]
endpoint = "http://elasticsearch:9200"
index = "logs-%Y-%m-%d"
```

### Log Format
```rust
// Log format
#[derive(Debug, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    service: String,
    message: String,
    context: HashMap<String, Value>,
    trace_id: Option<String>,
    span_id: Option<String>,
}

// Logging macro
macro_rules! log {
    ($level:expr, $message:expr, $context:expr) => {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: $level.to_string(),
            service: env!("CARGO_PKG_NAME").to_string(),
            message: $message.to_string(),
            context: $context,
            trace_id: opentelemetry::trace::current_trace_id(),
            span_id: opentelemetry::trace::current_span_id(),
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
    };
}
```

## Alerting

### Alert Manager Configuration
```yaml
# alertmanager/alertmanager.yml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'job']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'slack'

receivers:
  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX'
        channel: '#alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: "{{ range .Alerts }}*Alert:* {{ .Annotations.summary }}\n*Description:* {{ .Annotations.description }}\n{{ end }}"

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['job']
```

## Tracing

### OpenTelemetry Configuration
```rust
// Tracing setup
fn init_tracer() -> Result<sdktrace::Tracer> {
    let exporter = opentelemetry_jaeger::new_pipeline()
        .with_agent_endpoint("jaeger:6831")
        .with_service_name("sonoma-search")
        .install_batch(opentelemetry::runtime::Tokio)?;

    Ok(exporter)
}

// Trace example
async fn search(query: String) -> Result<Vec<SearchResult>> {
    let tracer = global::tracer("search");
    let span = tracer.start("search_request");
    let _guard = span.enter();

    span.set_attribute(KeyValue::new("query", query.clone()));

    let results = search_internal(query).await?;
    span.set_attribute(KeyValue::new("result_count", results.len() as i64));

    Ok(results)
}
```

## Performance Monitoring

### Resource Usage
```bash
#!/bin/bash
# monitor-resources.sh

while true; do
    echo "=== $(date) ==="
    echo "CPU Usage:"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"
    echo "Memory Usage:"
    free -h
    echo "Disk Usage:"
    df -h
    sleep 60
done
```

### Performance Metrics
```rust
// Custom metrics for performance monitoring
lazy_static! {
    static ref SEARCH_LATENCY: Histogram = register_histogram!(
        "search_latency_seconds",
        "Search request latency in seconds",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();

    static ref SEARCH_ERRORS: Counter = register_counter!(
        "search_errors_total",
        "Total number of search errors"
    ).unwrap();

    static ref ACTIVE_REQUESTS: Gauge = register_gauge!(
        "active_requests",
        "Number of currently active requests"
    ).unwrap();
}
``` 