# Crawler Service

The Crawler service is responsible for discovering, fetching, and processing web pages for indexing in the search engine.

## Overview

The Crawler service systematically browses and downloads web pages, respects robots.txt rules, manages crawl priorities, and handles the discovery of new URLs. It works in conjunction with the Indexer service to make content searchable.

## Features

- Distributed crawling
- Robots.txt compliance
- Politeness policies
- URL deduplication
- Content extraction
- Metrics exposure
- RESTful API endpoints

## API Endpoints

### Submit URLs
- **Endpoint**: `/api/submit`
- **Method**: POST
- **Parameters**:
  ```json
  {
    "urls": [
      {
        "url": "string",
        "priority": integer,
        "depth": integer
      }
    ]
  }
  ```
- **Response**:
  ```json
  {
    "accepted": integer,
    "rejected": integer,
    "errors": [
      {
        "url": "string",
        "reason": "string"
      }
    ]
  }
  ```

### Crawl Status
- **Endpoint**: `/api/status`
- **Method**: GET
- **Parameters**:
  - `url`: URL to check status for
- **Response**:
  ```json
  {
    "url": "string",
    "status": "pending|crawling|completed|failed",
    "lastCrawled": "timestamp",
    "nextCrawl": "timestamp",
    "error": "string"
  }
  ```

### Health Check
- **Endpoint**: `/health`
- **Method**: GET
- **Response**: Status of the service

## Configuration

The service can be configured through environment variables:

```env
STORAGE_SERVICE_URL=http://storage:8003
PORT=8005
RUST_LOG=info
MAX_DEPTH=5
POLITENESS_DELAY=1000
MAX_CONCURRENT_REQUESTS=100
USER_AGENT="SonomaBot/1.0"
```

## Metrics

The service exposes Prometheus metrics at `/metrics:9094`:

- `pages_crawled_total`: Total number of pages crawled
- `crawl_errors_total`: Total number of crawl errors
- `crawl_latency_seconds`: Crawl latency histogram
- `urls_discovered_total`: Total number of URLs discovered
- `robots_txt_requests_total`: Total number of robots.txt requests
- `bandwidth_bytes_total`: Total bandwidth used
- `active_crawlers`: Current number of active crawlers

## Crawling Process

1. **URL Discovery**
   - Seed URLs
   - Link extraction
   - Sitemap parsing
   - URL normalization

2. **Politeness**
   - Robots.txt parsing
   - Rate limiting
   - Crawl delay respect
   - Domain rotation

3. **Content Fetching**
   - HTTP requests
   - Response validation
   - Content type checking
   - Error handling

4. **Processing**
   - Content extraction
   - Link extraction
   - Metadata collection
   - Content validation

## Dependencies

- **Storage Service**: URL queue and metadata storage
- **Indexer Service**: Document indexing
- **HTTP Client**: Web page fetching

## Error Handling

The service implements comprehensive error handling:

- Network errors
- Invalid URLs
- Rate limiting
- Timeout handling
- Content processing errors
- Storage service errors

## Development

### Local Setup

1. Set required environment variables
2. Run the service:
   ```bash
   cargo run --bin crawler
   ```

### Testing

```bash
# Run unit tests
cargo test --bin crawler

# Run integration tests
cargo test --test crawler_integration

# Run politeness tests
cargo test --test politeness_tests
```

## Deployment

The service is deployed as a Docker container:

```bash
# Build the container
docker build -t crawler -f services/crawler/Dockerfile .

# Run the container
docker run -p 8005:8005 -p 9094:9094 crawler
```

## Monitoring

The service can be monitored through:

1. Prometheus metrics at `/metrics:9094`
2. Grafana dashboards (pre-configured)
3. Log output (structured JSON logs)

## Performance Considerations

- Connection pooling
- DNS caching
- Resource limits
- Concurrent crawling
- Memory management
- Bandwidth throttling
- Domain-based rate limiting

## Politeness Policies

1. **Robots.txt**
   - Parse and respect robots.txt rules
   - Cache robots.txt contents
   - Handle various robots.txt formats

2. **Rate Limiting**
   - Per-domain delays
   - Concurrent request limits
   - Bandwidth throttling
   - Request distribution

3. **Error Handling**
   - Backoff strategies
   - Retry policies
   - Error classification
   - Domain blacklisting 