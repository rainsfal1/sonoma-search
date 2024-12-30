# Environment Configuration

This document describes the environment configuration for the Sonoma Search engine across different deployment environments.

## Overview

The system uses environment variables for configuration, with different values for development, staging, and production environments. Configuration is managed through `.env` files and Docker Compose environment variables.

## Common Environment Variables

### API Gateway
```env
PORT=8000
RUST_LOG=info
SEARCHER_URL=http://searcher:8001
RANKER_URL=http://ranker:8002
STORAGE_URL=http://storage:8003
INDEXER_URL=http://indexer:8004
CRAWLER_URL=http://crawler:8005
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60
CACHE_TTL=300
```

### Searcher Service
```env
PORT=8001
RUST_LOG=info
ELASTICSEARCH_URL=http://elasticsearch:9200
RANKER_URL=http://ranker:8002
CACHE_SIZE=1000
SEARCH_TIMEOUT=5000
```

### Ranker Service
```env
PORT=8002
RUST_LOG=info
STORAGE_URL=http://storage:8003
MODEL_PATH=/models/ranker
CACHE_SIZE=1000
BATCH_SIZE=50
```

### Storage Service
```env
PORT=8003
RUST_LOG=info
DATABASE_URL=postgresql://user:password@postgres:5432/sonoma
MAX_POOL_SIZE=20
QUEUE_BATCH_SIZE=100
```

### Indexer Service
```env
PORT=8004
RUST_LOG=info
ELASTICSEARCH_URL=http://elasticsearch:9200
STORAGE_URL=http://storage:8003
BATCH_SIZE=1000
INDEX_REFRESH_INTERVAL=30s
```

### Crawler Service
```env
PORT=8005
RUST_LOG=info
STORAGE_URL=http://storage:8003
MAX_DEPTH=5
POLITENESS_DELAY=1000
MAX_CONCURRENT_REQUESTS=100
USER_AGENT="SonomaBot/1.0"
```

### Web Frontend
```env
PORT=3000
API_GATEWAY_URL=http://api-gateway:8000
NODE_ENV=production
ANALYTICS_ENABLED=true
```

## Environment-Specific Configuration

### Development
```env
# Logging
RUST_LOG=debug
NODE_ENV=development

# Resources
MAX_POOL_SIZE=5
CACHE_SIZE=100
BATCH_SIZE=10

# Features
ANALYTICS_ENABLED=false
RATE_LIMIT_REQUESTS=1000
CACHE_TTL=60

# Infrastructure
ELASTICSEARCH_REPLICAS=0
POSTGRES_MAX_CONNECTIONS=100
```

### Staging
```env
# Logging
RUST_LOG=info
NODE_ENV=staging

# Resources
MAX_POOL_SIZE=10
CACHE_SIZE=500
BATCH_SIZE=50

# Features
ANALYTICS_ENABLED=true
RATE_LIMIT_REQUESTS=500
CACHE_TTL=300

# Infrastructure
ELASTICSEARCH_REPLICAS=1
POSTGRES_MAX_CONNECTIONS=200
```

### Production
```env
# Logging
RUST_LOG=warn
NODE_ENV=production

# Resources
MAX_POOL_SIZE=20
CACHE_SIZE=1000
BATCH_SIZE=100

# Features
ANALYTICS_ENABLED=true
RATE_LIMIT_REQUESTS=100
CACHE_TTL=600

# Infrastructure
ELASTICSEARCH_REPLICAS=2
POSTGRES_MAX_CONNECTIONS=500
```

## Infrastructure Configuration

### Elasticsearch
```yaml
# Development
cluster.name: sonoma-dev
discovery.type: single-node
xpack.security.enabled: false
index.number_of_replicas: 0

# Staging
cluster.name: sonoma-staging
discovery.type: zen
xpack.security.enabled: true
index.number_of_replicas: 1

# Production
cluster.name: sonoma-prod
discovery.type: zen
xpack.security.enabled: true
index.number_of_replicas: 2
```

### PostgreSQL
```conf
# Development
max_connections = 100
shared_buffers = 1GB
work_mem = 32MB
maintenance_work_mem = 256MB

# Staging
max_connections = 200
shared_buffers = 4GB
work_mem = 64MB
maintenance_work_mem = 512MB

# Production
max_connections = 500
shared_buffers = 8GB
work_mem = 128MB
maintenance_work_mem = 1GB
```

## Security Configuration

### SSL/TLS
```env
# Development
SSL_ENABLED=false

# Staging
SSL_ENABLED=true
SSL_CERT_PATH=/etc/ssl/staging/cert.pem
SSL_KEY_PATH=/etc/ssl/staging/key.pem

# Production
SSL_ENABLED=true
SSL_CERT_PATH=/etc/ssl/prod/cert.pem
SSL_KEY_PATH=/etc/ssl/prod/key.pem
```

### Authentication
```env
# Development
AUTH_ENABLED=false
JWT_SECRET=development-secret

# Staging
AUTH_ENABLED=true
JWT_SECRET=staging-secret
JWT_EXPIRY=3600

# Production
AUTH_ENABLED=true
JWT_SECRET=production-secret
JWT_EXPIRY=1800
```

## Monitoring Configuration

### Prometheus
```yaml
# Development
global:
  scrape_interval: 30s
  evaluation_interval: 30s

# Staging
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Production
global:
  scrape_interval: 10s
  evaluation_interval: 10s
```

### Grafana
```ini
# Development
[auth]
disable_login_form = true

# Staging
[auth]
disable_login_form = false
oauth_auto_login = true

# Production
[auth]
disable_login_form = false
oauth_auto_login = true
```

## Configuration Management

1. **Local Development**
   ```bash
   # Copy example config
   cp .env.example .env.development
   
   # Edit configuration
   vim .env.development
   
   # Use configuration
   export $(cat .env.development | xargs)
   ```

2. **Docker Deployment**
   ```bash
   # Use environment file
   docker compose --env-file .env.production up -d
   ```

3. **Kubernetes Deployment**
   ```bash
   # Create config map
   kubectl create configmap sonoma-config --from-env-file=.env.production
   
   # Create secrets
   kubectl create secret generic sonoma-secrets --from-env-file=.env.secrets
   ```

## Validation and Testing

1. **Configuration Validation**
   ```bash
   # Validate environment
   ./scripts/validate-env.sh .env.production
   
   # Test configuration
   ./scripts/test-config.sh
   ```

2. **Security Checks**
   ```bash
   # Scan for secrets
   ./scripts/scan-secrets.sh
   
   # Validate SSL configuration
   ./scripts/check-ssl.sh
   ``` 

# Services
API_GATEWAY_PORT=8000
RANKER_PORT=8002
INDEXER_PORT=8003
CRAWLER_PORT=8004
STORAGE_PORT=8005

# Databases
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_DB=sonoma
POSTGRES_USER=production_user
POSTGRES_PASSWORD=<secure-password>

# Elasticsearch
ELASTICSEARCH_URL=http://elasticsearch:9200
ELASTICSEARCH_USERNAME=elastic
ELASTICSEARCH_PASSWORD=<secure-password>

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3001

# Web Frontend
PORT=3000
API_GATEWAY_URL=http://api-gateway:8000
NODE_ENV=production
``` 