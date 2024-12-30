# Deployment Guide

This document outlines the deployment process for the Sonoma Search engine.

## Prerequisites

### Hardware Requirements
- CPU: 8+ cores
- RAM: 32GB+ recommended
- Storage: 100GB+ SSD
- Network: 1Gbps+

### Software Requirements
- Docker Engine 24.0+
- Docker Compose 2.0+
- SSL certificates
- Domain names configured
- Access to container registry

## Deployment Architecture

### Production Environment
```
                           [Load Balancer]
                                 |
                   +------------+------------+
                   |            |            |
            [Web Frontend] [API Gateway] [Monitoring]
                   |            |            |
        +---------+------------+--------+    |
        |         |            |        |    |
   [Searcher] [Ranker]    [Indexer] [Crawler] 
        |         |            |        |    
        +----+----+------------+--------+    
             |                          
      [Elasticsearch]              [PostgreSQL]
```

## Configuration

### 1. Environment Variables
```bash
# .env.production
# API Gateway
API_GATEWAY_PORT=8000
API_GATEWAY_HOST=0.0.0.0
API_RATE_LIMIT=100

# Services
SEARCHER_PORT=8001
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

# SSL/TLS
SSL_CERT_PATH=/etc/ssl/certs/sonoma.crt
SSL_KEY_PATH=/etc/ssl/private/sonoma.key
```

### 2. Docker Compose
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  nginx:
    image: nginx:latest
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./config/nginx:/etc/nginx/conf.d
      - ./ssl:/etc/ssl
    depends_on:
      - web
      - api-gateway

  web:
    image: sonoma/web-frontend:${TAG}
    environment:
      - NODE_ENV=production
      - API_GATEWAY_URL=https://api.sonoma.com

  api-gateway:
    image: sonoma/api-gateway:${TAG}
    environment:
      - RUST_LOG=info
      - PORT=${API_GATEWAY_PORT}
    depends_on:
      - ranker
      - indexer
      - crawler
      - storage

  ranker:
    image: sonoma/ranker:${TAG}
    environment:
      - RUST_LOG=info
      - PORT=${RANKER_PORT}

  indexer:
    image: sonoma/indexer:${TAG}
    environment:
      - RUST_LOG=info
      - PORT=${INDEXER_PORT}
    depends_on:
      - elasticsearch
      - postgres

  crawler:
    image: sonoma/crawler:${TAG}
    environment:
      - RUST_LOG=info
      - PORT=${CRAWLER_PORT}
    depends_on:
      - postgres

  storage:
    image: sonoma/storage:${TAG}
    environment:
      - RUST_LOG=info
      - PORT=${STORAGE_PORT}
    depends_on:
      - postgres

  postgres:
    image: postgres:16
    environment:
      - POSTGRES_DB=${POSTGRES_DB}
      - POSTGRES_USER=${POSTGRES_USER}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
    environment:
      - node.name=es01
      - cluster.name=sonoma-prod
      - discovery.type=single-node
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms8g -Xmx8g"
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./config/prometheus:/etc/prometheus
      - prometheus_data:/prometheus
    ports:
      - "${PROMETHEUS_PORT}:9090"

  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./config/grafana:/etc/grafana
      - grafana_data:/var/lib/grafana
    ports:
      - "${GRAFANA_PORT}:3000"

volumes:
  postgres_data:
  elasticsearch_data:
  prometheus_data:
  grafana_data:
```

## Deployment Steps

### 1. Initial Setup
```bash
# Clone repository
git clone https://github.com/your-org/sonoma-search.git
cd sonoma-search

# Create production config
cp .env.example .env.production
nano .env.production

# Configure SSL
mkdir -p ssl
cp /path/to/ssl/certs/* ssl/
```

### 2. Build Images
```bash
# Set version tag
export TAG=$(git describe --tags)

# Build all images
docker compose -f docker-compose.prod.yml build

# Push to registry
docker compose -f docker-compose.prod.yml push
```

### 3. Database Setup
```bash
# Run migrations
docker compose -f docker-compose.prod.yml run --rm api-gateway \
    ./scripts/run-migrations.sh

# Initialize Elasticsearch indices
docker compose -f docker-compose.prod.yml run --rm searcher \
    ./scripts/init-elasticsearch.sh
```

### 4. Deploy Services
```bash
# Start all services
docker compose -f docker-compose.prod.yml up -d

# Verify deployment
docker compose -f docker-compose.prod.yml ps

# Check logs
docker compose -f docker-compose.prod.yml logs -f
```

## Monitoring

### 1. Health Checks
```bash
# API Gateway
curl -I https://api.sonoma.com/health

# Services
curl -I http://localhost:${SEARCHER_PORT}/health
curl -I http://localhost:${RANKER_PORT}/health
curl -I http://localhost:${INDEXER_PORT}/health
curl -I http://localhost:${CRAWLER_PORT}/health
curl -I http://localhost:${STORAGE_PORT}/health
```

### 2. Metrics
```bash
# Prometheus metrics
curl http://localhost:${PROMETHEUS_PORT}/metrics

# Service metrics
curl http://localhost:${SEARCHER_PORT}/metrics
curl http://localhost:${RANKER_PORT}/metrics
curl http://localhost:${INDEXER_PORT}/metrics
curl http://localhost:${CRAWLER_PORT}/metrics
curl http://localhost:${STORAGE_PORT}/metrics
```

## Backup and Recovery

### 1. Database Backup
```bash
# Backup PostgreSQL
docker compose -f docker-compose.prod.yml exec postgres \
    pg_dump -U ${POSTGRES_USER} ${POSTGRES_DB} > backup.sql

# Backup Elasticsearch
curl -X PUT "localhost:9200/_snapshot/backup" -H "Content-Type: application/json" -d '{
  "type": "fs",
  "settings": {
    "location": "/backup/elasticsearch"
  }
}'
curl -X PUT "localhost:9200/_snapshot/backup/snapshot_1?wait_for_completion=true"
```

### 2. Recovery
```bash
# Restore PostgreSQL
cat backup.sql | docker compose -f docker-compose.prod.yml exec -T postgres \
    psql -U ${POSTGRES_USER} ${POSTGRES_DB}

# Restore Elasticsearch
curl -X POST "localhost:9200/_snapshot/backup/snapshot_1/_restore"
```

## Scaling

### 1. Horizontal Scaling
```bash
# Scale services
docker compose -f docker-compose.prod.yml up -d --scale searcher=3
docker compose -f docker-compose.prod.yml up -d --scale ranker=2
docker compose -f docker-compose.prod.yml up -d --scale crawler=4
```

### 2. Vertical Scaling
```bash
# Update resource limits
docker compose -f docker-compose.prod.yml up -d --scale searcher=3 \
    --memory=4g --cpus=2
```

## Troubleshooting

### 1. Service Issues
```bash
# Check service logs
docker compose -f docker-compose.prod.yml logs searcher
docker compose -f docker-compose.prod.yml logs ranker

# Restart service
docker compose -f docker-compose.prod.yml restart searcher

# Check resource usage
docker stats
```

### 2. Database Issues
```bash
# Check PostgreSQL logs
docker compose -f docker-compose.prod.yml logs postgres

# Check Elasticsearch health
curl http://localhost:9200/_cluster/health?pretty

# Rebuild indices
./scripts/rebuild-indices.sh
```

## Security

### 1. SSL/TLS Configuration
```nginx
# nginx/conf.d/default.conf
server {
    listen 443 ssl;
    server_name api.sonoma.com;

    ssl_certificate /etc/ssl/certs/sonoma.crt;
    ssl_certificate_key /etc/ssl/private/sonoma.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://api-gateway:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 2. Firewall Rules
```bash
# Allow necessary ports
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow from 10.0.0.0/8 to any port 9200 proto tcp
```

## Maintenance

### 1. Updates
```bash
# Pull latest images
docker compose -f docker-compose.prod.yml pull

# Update services
docker compose -f docker-compose.prod.yml up -d

# Clean up
docker system prune -af
```

### 2. Monitoring
```bash
# Check disk usage
df -h

# Check memory usage
free -h

# Check service health
./scripts/health-check.sh
``` 