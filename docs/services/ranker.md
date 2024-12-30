# Ranker Service

The Ranker service is responsible for scoring and ranking search results based on various relevance factors and algorithms.

## Overview

The Ranker service receives search results from the Searcher service and applies sophisticated ranking algorithms to ensure the most relevant results appear first. It considers multiple factors including text relevance, page authority, freshness, and user engagement metrics.

## Features

- Advanced ranking algorithms
- Real-time result scoring
- Configurable ranking factors
- Metrics exposure for monitoring
- RESTful API endpoints

## API Endpoints

### Rank
- **Endpoint**: `/api/rank`
- **Method**: POST
- **Parameters**:
  ```json
  {
    "query": "string",
    "documents": [
      {
        "id": "string",
        "title": "string",
        "url": "string",
        "content": "string",
        "metadata": {
          "pageRank": float,
          "lastUpdated": "timestamp",
          "inboundLinks": integer
        }
      }
    ]
  }
  ```
- **Response**:
  ```json
  {
    "ranked_results": [
      {
        "id": "string",
        "score": float,
        "factors": {
          "textRelevance": float,
          "pageAuthority": float,
          "freshness": float
        }
      }
    ]
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
PORT=8002
RUST_LOG=info
MODEL_PATH=/models/ranker
```

## Metrics

The service exposes Prometheus metrics at `/metrics:9092`:

- `ranking_requests_total`: Total number of ranking requests
- `ranking_latency_seconds`: Ranking request latency histogram
- `ranking_errors_total`: Total number of ranking errors
- `model_load_errors_total`: Total number of model loading errors
- `cache_hits_total`: Total number of cache hits
- `cache_misses_total`: Total number of cache misses

## Ranking Factors

The service considers multiple factors when ranking results:

1. **Text Relevance**
   - Term frequency
   - Inverse document frequency
   - Field weights (title, content, URL)
   - Query term proximity

2. **Page Authority**
   - PageRank score
   - Domain authority
   - Inbound link count
   - URL structure

3. **Content Quality**
   - Content freshness
   - Content length
   - Content structure
   - Media richness

4. **User Engagement**
   - Click-through rate
   - Bounce rate
   - Time on page
   - Return visits

## Dependencies

- **Storage Service**: Access to page metrics and historical data
- **Machine Learning Models**: Pre-trained ranking models

## Error Handling

The service implements comprehensive error handling:

- Invalid request parameters
- Model loading failures
- Storage service communication errors
- Internal processing errors

## Development

### Local Setup

1. Download required ML models
2. Set required environment variables
3. Run the service:
   ```bash
   cargo run --bin ranker
   ```

### Testing

```bash
# Run unit tests
cargo test --bin ranker

# Run integration tests
cargo test --test ranker_integration

# Run model validation tests
cargo test --test model_validation
```

## Deployment

The service is deployed as a Docker container:

```bash
# Build the container
docker build -t ranker -f services/ranker/Dockerfile .

# Run the container
docker run -p 8002:8002 -p 9092:9092 ranker
```

## Monitoring

The service can be monitored through:

1. Prometheus metrics at `/metrics:9092`
2. Grafana dashboards (pre-configured)
3. Log output (structured JSON logs)

## Performance Considerations

- Model caching
- Batch processing
- Resource limits
- Result caching
- Concurrent request handling
- Model optimization 