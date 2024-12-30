# API Gateway

The API Gateway serves as the central entry point for all client requests, providing routing, authentication, rate limiting, and request/response transformation.

## Overview

The API Gateway manages all incoming requests from clients, routes them to appropriate backend services, and handles cross-cutting concerns such as authentication, rate limiting, and response caching. It provides a unified API interface for the frontend while abstracting the complexity of the microservices architecture.

## Features

- Request routing
- Authentication and authorization
- Rate limiting
- Response caching
- Request/response transformation
- API documentation
- Metrics exposure
- Health checks

## API Endpoints

### Search
- **Endpoint**: `/api/search`
- **Method**: GET
- **Parameters**:
  - `q`: Search query string
  - `page`: Page number (optional)
  - `size`: Results per page (optional)
  - `sort`: Sort order (optional)
  - `filters`: Search filters (optional)
- **Response**: Search results from Searcher service

### Suggestions
- **Endpoint**: `/api/suggest`
- **Method**: GET
- **Parameters**:
  - `q`: Partial query string
  - `limit`: Number of suggestions (optional)
- **Response**: Search suggestions

### Document Management
- **Endpoint**: `/api/documents`
- **Method**: POST
- **Parameters**: Document data
- **Response**: Storage service response

### Crawl Management
- **Endpoint**: `/api/crawl`
- **Method**: POST
- **Parameters**: URLs to crawl
- **Response**: Crawler service response

### Health Check
- **Endpoint**: `/health`
- **Method**: GET
- **Response**: Gateway and service health status

## Configuration

The gateway can be configured through environment variables:

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

## Metrics

The service exposes Prometheus metrics at `/metrics:9090`:

- `http_requests_total`: Total number of HTTP requests
- `http_request_duration_seconds`: Request duration histogram
- `http_errors_total`: Total number of HTTP errors
- `cache_hits_total`: Total number of cache hits
- `cache_misses_total`: Total number of cache misses
- `rate_limit_hits_total`: Total number of rate limit hits

## Request Flow

1. **Request Reception**
   - Parse request
   - Validate headers
   - Extract authentication
   - Load balancing

2. **Pre-processing**
   - Authentication check
   - Rate limit check
   - Request validation
   - Cache lookup

3. **Request Routing**
   - Service discovery
   - Load balancing
   - Circuit breaking
   - Timeout handling

4. **Post-processing**
   - Response transformation
   - Cache update
   - Metrics collection
   - Error handling

## Security

1. **Authentication**
   - API key validation
   - JWT validation
   - OAuth2 support
   - Rate limiting

2. **Authorization**
   - Role-based access
   - Scope validation
   - IP whitelisting
   - Request validation

3. **Security Headers**
   - CORS configuration
   - Content Security Policy
   - XSS protection
   - HSTS

## Dependencies

- **Searcher Service**: Search functionality
- **Ranker Service**: Result ranking
- **Storage Service**: Data storage
- **Indexer Service**: Document indexing
- **Crawler Service**: Web crawling

## Error Handling

The gateway implements comprehensive error handling:

- Invalid requests
- Service unavailability
- Timeout handling
- Rate limit exceeded
- Authentication failures
- Authorization failures

## Development

### Local Setup

1. Set required environment variables
2. Run the service:
   ```bash
   cargo run --bin gateway
   ```

### Testing

```bash
# Run unit tests
cargo test --bin gateway

# Run integration tests
cargo test --test gateway_integration

# Run load tests
cargo test --test load_tests
```

## Deployment

The service is deployed as a Docker container:

```bash
# Build the container
docker build -t api-gateway -f services/gateway/Dockerfile .

# Run the container
docker run -p 8000:8000 -p 9090:9090 api-gateway
```

## Monitoring

The service can be monitored through:

1. Prometheus metrics at `/metrics:9090`
2. Grafana dashboards (pre-configured)
3. Log output (structured JSON logs)

## Performance Considerations

1. **Caching**
   - Response caching
   - Route caching
   - DNS caching
   - Connection pooling

2. **Load Balancing**
   - Round-robin
   - Least connections
   - Resource-based
   - Health checks

3. **Circuit Breaking**
   - Failure thresholds
   - Recovery time
   - Fallback responses
   - Partial availability

4. **Rate Limiting**
   - Per-client limits
   - Global limits
   - Burst handling
   - Token bucket algorithm 