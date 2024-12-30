# Docker Setup

This document describes how to deploy the Sonoma Search engine using Docker and Docker Compose.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- 16GB RAM minimum
- 50GB disk space minimum

## Service Images

All services are containerized using Docker:

```bash
services/
├── api-gateway/Dockerfile
├── crawler/Dockerfile
├── indexer/Dockerfile
├── ranker/Dockerfile
├── searcher/Dockerfile
├── storage/Dockerfile
└── web/Dockerfile
```

## Docker Compose Configuration

The system is orchestrated using Docker Compose. Here's the basic structure:

```yaml
version: '3.8'

services:
  # Frontend Layer
  web:
    build: 
      context: .
      dockerfile: services/web/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - API_GATEWAY_URL=http://api-gateway:8000
    depends_on:
      - api-gateway

  api-gateway:
    build:
      context: .
      dockerfile: services/api-gateway/Dockerfile
    ports:
      - "8000:8000"
      - "9090:9090"
    environment:
      - SEARCHER_URL=http://searcher:8001
      - RANKER_URL=http://ranker:8002
      - STORAGE_URL=http://storage:8003
      - INDEXER_URL=http://indexer:8004
      - CRAWLER_URL=http://crawler:8005
    depends_on:
      - searcher
      - ranker
      - storage
      - indexer
      - crawler

  # Search Layer
  searcher:
    build:
      context: .
      dockerfile: services/searcher/Dockerfile
    ports:
      - "8001:8001"
      - "9091:9091"
    environment:
      - ELASTICSEARCH_URL=http://elasticsearch:9200
      - RANKER_URL=http://ranker:8002
    depends_on:
      - elasticsearch
      - ranker

  ranker:
    build:
      context: .
      dockerfile: services/ranker/Dockerfile
    ports:
      - "8002:8002"
      - "9092:9092"
    environment:
      - STORAGE_URL=http://storage:8003
    volumes:
      - ./models:/models
    depends_on:
      - storage

  # Data Layer
  indexer:
    build:
      context: .
      dockerfile: services/indexer/Dockerfile
    ports:
      - "8004:8004"
      - "9093:9093"
    environment:
      - ELASTICSEARCH_URL=http://elasticsearch:9200
      - STORAGE_URL=http://storage:8003
    depends_on:
      - elasticsearch
      - storage

  crawler:
    build:
      context: .
      dockerfile: services/crawler/Dockerfile
    ports:
      - "8005:8005"
      - "9094:9094"
    environment:
      - STORAGE_URL=http://storage:8003
    depends_on:
      - storage

  storage:
    build:
      context: .
      dockerfile: services/storage/Dockerfile
    ports:
      - "8003:8003"
      - "9095:9095"
    environment:
      - DATABASE_URL=postgresql://user:password@postgres:5432/sonoma
    depends_on:
      - postgres

  # Infrastructure
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
    ports:
      - "9200:9200"
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data

  postgres:
    image: postgres:16
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=sonoma
    volumes:
      - postgres_data:/var/lib/postgresql/data

  # Monitoring
  prometheus:
    image: prom/prometheus:v2.45.0
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus:/etc/prometheus
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

  grafana:
    image: grafana/grafana:10.2.0
    ports:
      - "3001:3000"
    volumes:
      - ./config/grafana:/etc/grafana/provisioning
      - grafana_data:/var/lib/grafana

volumes:
  elasticsearch_data:
  postgres_data:
  prometheus_data:
  grafana_data:
```

## Deployment Steps

1. **Clone Repository**
   ```bash
   git clone https://github.com/your-org/sonoma-search.git
   cd sonoma-search
   ```

2. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Build Images**
   ```bash
   docker compose build
   ```

4. **Start Services**
   ```bash
   docker compose up -d
   ```

5. **Verify Deployment**
   ```bash
   docker compose ps
   ```

## Service Health Checks

Check the health of all services:

```bash
# API Gateway
curl http://localhost:8000/health

# Searcher
curl http://localhost:8001/health

# Ranker
curl http://localhost:8002/health

# Storage
curl http://localhost:8003/health

# Indexer
curl http://localhost:8004/health

# Crawler
curl http://localhost:8005/health
```

## Resource Requirements

Recommended resource allocation per service:

| Service        | CPU | Memory | Disk  |
|---------------|-----|---------|-------|
| Web Frontend  | 1   | 512MB   | 1GB   |
| API Gateway   | 2   | 1GB     | 1GB   |
| Searcher      | 2   | 2GB     | 1GB   |
| Ranker        | 2   | 2GB     | 5GB   |
| Indexer       | 2   | 2GB     | 1GB   |
| Crawler       | 2   | 1GB     | 1GB   |
| Storage       | 2   | 1GB     | 1GB   |
| Elasticsearch | 4   | 4GB     | 20GB  |
| PostgreSQL    | 2   | 2GB     | 10GB  |
| Prometheus    | 1   | 1GB     | 10GB  |
| Grafana       | 1   | 512MB   | 1GB   |

## Common Operations

### View Logs
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f [service-name]
```

### Restart Service
```bash
docker compose restart [service-name]
```

### Scale Service
```bash
docker compose up -d --scale [service-name]=[instances]
```

### Update Service
```bash
# Pull latest images
docker compose pull

# Rebuild service
docker compose build [service-name]

# Update running service
docker compose up -d --no-deps [service-name]
```

## Troubleshooting

1. **Service Won't Start**
   - Check logs: `docker compose logs [service-name]`
   - Verify environment variables
   - Check resource availability

2. **Service Unhealthy**
   - Check health endpoint
   - Verify dependencies are running
   - Check resource usage

3. **Performance Issues**
   - Monitor resource usage
   - Check container metrics
   - Verify resource limits

4. **Network Issues**
   - Check service discovery
   - Verify port mappings
   - Check network connectivity 