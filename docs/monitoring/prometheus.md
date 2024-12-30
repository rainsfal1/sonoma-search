# Prometheus Setup

This document describes the Prometheus setup and configuration for monitoring the Sonoma Search engine.

## Configuration

### Main Configuration

```yaml
# /etc/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # API Gateway
  - job_name: 'api-gateway'
    static_configs:
      - targets: ['api-gateway:9090']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Crawler Service
  - job_name: 'crawler'
    static_configs:
      - targets: ['crawler:9091']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Indexer Service
  - job_name: 'indexer'
    static_configs:
      - targets: ['indexer:9092']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Ranker Service
  - job_name: 'ranker'
    static_configs:
      - targets: ['ranker:9093']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Storage Service
  - job_name: 'storage'
    static_configs:
      - targets: ['storage:9094']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Infrastructure
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 30s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s

  - job_name: 'elasticsearch'
    static_configs:
      - targets: ['elasticsearch-exporter:9114']
    metrics_path: /metrics
    scrape_interval: 30s
```

### Alert Rules

```yaml
# /etc/prometheus/rules/service_alerts.yml
groups:
  - name: service_alerts
    rules:
      # Service Health
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.instance }} down"
          description: "Service {{ $labels.instance }} has been down for more than 1 minute"

      # Error Rate
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate on {{ $labels.instance }}"
          description: "Error rate is above 10% for more than 2 minutes"

      # Latency
      - alert: HighLatency
        expr: http_request_duration_seconds > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.instance }}"
          description: "Request latency is above 1 second for more than 5 minutes"

      # Resource Usage
      - alert: HighCPUUsage
        expr: container_cpu_usage_seconds_total > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is above 80% for more than 10 minutes"
```

## Recording Rules

```yaml
# /etc/prometheus/rules/recording_rules.yml
groups:
  - name: service_metrics
    rules:
      # Request Rate
      - record: job:request_rate:5m
        expr: rate(http_requests_total[5m])

      # Error Rate
      - record: job:error_rate:5m
        expr: rate(http_requests_total{status=~"5.."}[5m])

      # Latency
      - record: job:latency:5m
        expr: rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])
```

## Service Discovery

### Static Configuration
```yaml
scrape_configs:
  - job_name: 'services'
    file_sd_configs:
      - files:
        - '/etc/prometheus/targets/*.yml'
        refresh_interval: 5m
```

### Dynamic Configuration
```yaml
scrape_configs:
  - job_name: 'docker'
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: [__meta_docker_container_name]
        regex: '/(.*)'
        target_label: container_name
```

## Storage Configuration

```yaml
storage:
  tsdb:
    path: /prometheus
    retention.time: 15d
    retention.size: 50GB
    wal-compression: true
```

## Security Configuration

```yaml
basic_auth_users:
  admin: $ADMIN_PASSWORD_HASH

tls_server_config:
  cert_file: /etc/prometheus/certs/prometheus.crt
  key_file: /etc/prometheus/certs/prometheus.key
```

## Docker Configuration

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.45.0
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus:/etc/prometheus
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=15d'
      - '--web.enable-lifecycle'
    restart: unless-stopped

volumes:
  prometheus_data:
```

## Exporters Configuration

### Node Exporter
```yaml
  node-exporter:
    image: prom/node-exporter:v1.6.1
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--path.rootfs=/rootfs'
```

### Postgres Exporter
```yaml
  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:v0.12.1
    ports:
      - "9187:9187"
    environment:
      DATA_SOURCE_NAME: "postgresql://user:password@postgres:5432/sonoma?sslmode=disable"
```

### Elasticsearch Exporter
```yaml
  elasticsearch-exporter:
    image: prometheuscommunity/elasticsearch-exporter:v1.5.0
    ports:
      - "9114:9114"
    command:
      - '--es.uri=http://elasticsearch:9200'
```

## Management

### Hot Reload
```bash
# Reload configuration
curl -X POST http://localhost:9090/-/reload

# Check configuration
curl -X POST http://localhost:9090/-/reload?validate=true
```

### Backup
```bash
# Snapshot current data
curl -X POST http://localhost:9090/api/v1/admin/tsdb/snapshot

# Copy snapshot
cp -r /prometheus/snapshots/* /backup/
```

### Maintenance
```bash
# Clean tombstones
curl -X POST http://localhost:9090/api/v1/admin/tsdb/clean_tombstones

# Delete series
curl -X POST http://localhost:9090/api/v1/admin/tsdb/delete_series
``` 