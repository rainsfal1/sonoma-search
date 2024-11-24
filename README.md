# Search Engine Microservices

A distributed search engine implementation using microservices architecture.

## Services
- Crawler: Web crawler service
- Indexer: Document indexing service
- Ranker: Search ranking service
- Searcher: Search API service
- Storage: Shared storage library

## Development
```bash
cd deploy/docker
docker-compose up
```

## Monitoring
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000

## Architecture
```
Services → Prometheus → Grafana
         ↓
    PostgreSQL
         ↑
    RabbitMQ (Coming Soon)
```
