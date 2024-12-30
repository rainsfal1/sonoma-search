# Indexer Service

The Indexer service is responsible for processing and indexing documents into Elasticsearch, making them searchable through the search engine.

## Overview

The Indexer service receives documents from the Crawler service, processes them for optimal search performance, and indexes them into Elasticsearch. It handles document preprocessing, text analysis, and maintains index health.

## Features

- Document preprocessing and analysis
- Bulk indexing capabilities
- Index management and optimization
- Schema management
- Metrics exposure for monitoring
- RESTful API endpoints

## API Endpoints

### Index Document
- **Endpoint**: `/api/index`
- **Method**: POST
- **Parameters**:
  ```json
  {
    "documents": [
      {
        "id": "string",
        "url": "string",
        "title": "string",
        "content": "string",
        "metadata": {
          "crawledAt": "timestamp",
          "lastModified": "timestamp",
          "contentType": "string",
          "language": "string"
        }
      }
    ]
  }
  ```
- **Response**:
  ```json
  {
    "indexed": integer,
    "failed": integer,
    "errors": [
      {
        "id": "string",
        "error": "string"
      }
    ]
  }
  ```

### Reindex
- **Endpoint**: `/api/reindex`
- **Method**: POST
- **Parameters**: Optional source and target index names
- **Response**: Status of reindex operation

### Health Check
- **Endpoint**: `/health`
- **Method**: GET
- **Response**: Status of the service

## Configuration

The service can be configured through environment variables:

```env
ELASTICSEARCH_URL=http://elasticsearch:9200
STORAGE_SERVICE_URL=http://storage:8003
PORT=8004
RUST_LOG=info
BATCH_SIZE=1000
```

## Metrics

The service exposes Prometheus metrics at `/metrics:9093`:

- `documents_indexed_total`: Total number of documents indexed
- `indexing_errors_total`: Total number of indexing errors
- `indexing_latency_seconds`: Indexing latency histogram
- `batch_size_current`: Current batch size
- `elasticsearch_bulk_requests_total`: Total number of bulk requests
- `elasticsearch_bulk_errors_total`: Total number of bulk errors

## Document Processing

The service performs several processing steps:

1. **Text Extraction**
   - HTML cleaning
   - Content extraction
   - Metadata extraction

2. **Text Analysis**
   - Language detection
   - Text normalization
   - Stop word removal
   - Stemming/lemmatization

3. **Enrichment**
   - Entity extraction
   - Keyword extraction
   - Topic classification
   - Sentiment analysis

4. **Optimization**
   - Content summarization
   - Field optimization
   - Index optimization

## Dependencies

- **Elasticsearch**: Document storage and indexing
- **Storage Service**: Access to document metadata
- **NLP Models**: Text analysis and processing

## Error Handling

The service implements comprehensive error handling:

- Invalid document format
- Elasticsearch connection issues
- Storage service communication errors
- Processing pipeline errors
- Bulk indexing failures

## Development

### Local Setup

1. Ensure Elasticsearch is running
2. Set required environment variables
3. Run the service:
   ```bash
   cargo run --bin indexer
   ```

### Testing

```bash
# Run unit tests
cargo test --bin indexer

# Run integration tests
cargo test --test indexer_integration

# Run pipeline tests
cargo test --test pipeline_tests
```

## Deployment

The service is deployed as a Docker container:

```bash
# Build the container
docker build -t indexer -f services/indexer/Dockerfile .

# Run the container
docker run -p 8004:8004 -p 9093:9093 indexer
```

## Monitoring

The service can be monitored through:

1. Prometheus metrics at `/metrics:9093`
2. Grafana dashboards (pre-configured)
3. Log output (structured JSON logs)

## Performance Considerations

- Batch processing optimization
- Index sharding and replication
- Document pipeline optimization
- Memory management
- Concurrent processing
- Index refresh intervals
- Bulk operation sizing 