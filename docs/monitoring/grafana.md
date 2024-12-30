# Grafana Setup

This document describes the Grafana setup and dashboard configuration for visualizing metrics from the Sonoma Search engine.

## Configuration

### Main Configuration

```ini
# /etc/grafana/grafana.ini
[server]
http_port = 3001
domain = localhost
root_url = %(protocol)s://%(domain)s:%(http_port)s/

[security]
admin_user = admin
admin_password = admin
disable_gravatar = true
cookie_secure = true

[auth]
disable_login_form = false
oauth_auto_login = false

[users]
allow_sign_up = false
auto_assign_org = true
auto_assign_org_role = Viewer
```

### Data Sources

```yaml
# /etc/grafana/provisioning/datasources/prometheus.yml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "15s"
```

## Dashboard Categories

### 1. Overview Dashboard
```json
{
  "title": "System Overview",
  "panels": [
    {
      "title": "Service Health",
      "type": "stat",
      "targets": [
        {
          "expr": "up",
          "legendFormat": "{{instance}}"
        }
      ]
    },
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "sum(rate(http_requests_total[5m])) by (service)",
          "legendFormat": "{{service}}"
        }
      ]
    }
  ]
}
```

### 2. Service Dashboards

#### API Gateway
```json
{
  "title": "API Gateway",
  "panels": [
    {
      "title": "Request Volume",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total{job=\"api-gateway\"}[5m])",
          "legendFormat": "{{status}}"
        }
      ]
    },
    {
      "title": "Response Time",
      "type": "heatmap",
      "targets": [
        {
          "expr": "rate(http_request_duration_seconds_bucket{job=\"api-gateway\"}[5m])",
          "format": "heatmap"
        }
      ]
    }
  ]
}
```

#### Search Services
```json
{
  "title": "Search Services",
  "panels": [
    {
      "title": "Query Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(search_queries_total[5m])",
          "legendFormat": "queries/sec"
        }
      ]
    },
    {
      "title": "Result Quality",
      "type": "gauge",
      "targets": [
        {
          "expr": "search_result_quality_score",
          "legendFormat": "quality"
        }
      ]
    }
  ]
}
```

## Alert Rules

### Service Alerts
```yaml
# /etc/grafana/provisioning/alerting/service_alerts.yml
apiVersion: 1

groups:
  - name: Service Alerts
    rules:
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: Service {{ $labels.instance }} is down
          
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: High error rate on {{ $labels.instance }}
```

## Dashboard Templates

### Service Template
```json
{
  "title": "${service} Dashboard",
  "templating": {
    "list": [
      {
        "name": "service",
        "type": "query",
        "query": "label_values(up, job)"
      }
    ]
  },
  "panels": [
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total{job=\"$service\"}[5m])"
        }
      ]
    }
  ]
}
```

## Docker Configuration

```yaml
version: '3.8'

services:
  grafana:
    image: grafana/grafana:10.2.0
    ports:
      - "3001:3000"
    volumes:
      - ./config/grafana:/etc/grafana
      - grafana_data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    restart: unless-stopped

volumes:
  grafana_data:
```

## Dashboard Organization

### Folder Structure
```
Dashboards/
├── Overview/
│   ├── System Overview
│   └── Service Health
├── Services/
│   ├── API Gateway
│   ├── Search Services
│   ├── Data Services
│   └── Infrastructure
├── Business/
│   ├── Search Analytics
│   └── Content Metrics
└── Development/
    ├── Debug
    └── Testing
```

## Notification Channels

### Email
```yaml
apiVersion: 1

notifiers:
  - name: Email
    type: email
    uid: email1
    settings:
      addresses: alerts@example.com
```

### Slack
```yaml
apiVersion: 1

notifiers:
  - name: Slack
    type: slack
    uid: slack1
    settings:
      url: https://hooks.slack.com/services/xxx/yyy/zzz
      recipient: "#alerts"
```

## User Management

### Roles
```yaml
apiVersion: 1

roles:
  - name: ServiceViewer
    permissions:
      - action: "dashboards:read"
        scope: "folders:id:1"
  
  - name: ServiceEditor
    permissions:
      - action: "dashboards:write"
        scope: "folders:id:1"
```

## API Configuration

### Authentication
```yaml
apiVersion: 1

auth:
  - name: API Keys
    type: apikey
    role: Editor
    expiration: 30d
```

## Performance Optimization

### Dashboard Settings
```json
{
  "refresh": "1m",
  "timeOptions": [
    "5m",
    "15m",
    "30m",
    "1h",
    "3h",
    "6h",
    "12h",
    "24h"
  ],
  "timePicker": {
    "refresh_intervals": [
      "5s",
      "10s",
      "30s",
      "1m",
      "5m",
      "15m",
      "30m",
      "1h"
    ]
  }
}
```

## Backup Configuration

### Backup Script
```bash
#!/bin/bash
BACKUP_DIR="/backup/grafana"
DATE=$(date +%Y%m%d)

# Backup dashboards
curl -H "Authorization: Bearer $API_KEY" \
     http://localhost:3001/api/dashboards \
     -o $BACKUP_DIR/dashboards_$DATE.json

# Backup datasources
curl -H "Authorization: Bearer $API_KEY" \
     http://localhost:3001/api/datasources \
     -o $BACKUP_DIR/datasources_$DATE.json

# Backup alert rules
curl -H "Authorization: Bearer $API_KEY" \
     http://localhost:3001/api/alert-rules \
     -o $BACKUP_DIR/alerts_$DATE.json
``` 