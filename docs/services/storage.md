# Storage Service

The Storage service is responsible for managing persistent data storage and providing a unified interface for data access across all services.

## Overview

The Storage service acts as an abstraction layer over PostgreSQL, providing structured data storage, queuing capabilities, and data access patterns for other services. It ensures data consistency and provides optimized access patterns for different use cases.

## Features

- Structured data storage
- Queue management
- Data access patterns
- Schema management
- Metrics exposure
- RESTful API endpoints

## API Endpoints

### Store Document
- **Endpoint**: `/api/documents`
- **Method**: POST
- **Parameters**:
  ```json
  {
    "documents": [
      {
        "id": "string",
        "url": "string",
        "metadata": {
          "crawledAt": "timestamp",
          "lastModified": "timestamp",
          "contentType": "string",
          "size": integer
        },
        "status": "string"
      }
    ]
  }
  ```
- **Response**:
  ```json
  {
    "stored": integer,
    "failed": integer,
    "errors": [
      {
        "id": "string",
        "error": "string"
      }
    ]
  }
  ```

### Queue Management
- **Endpoint**: `/api/queue`
- **Method**: POST
- **Parameters**:
  ```json
  {
    "operation": "push|pop|peek",
    "queue": "string",
    "items": [
      {
        "id": "string",
        "priority": integer,
        "payload": object
      }
    ]
  }
  ```
- **Response**:
  ```json
  {
    "success": boolean,
    "items": [
      {
        "id": "string",
        "payload": object
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
DATABASE_URL=postgresql://user:password@postgres:5432/sonoma
PORT=8003
RUST_LOG=info
MAX_POOL_SIZE=20
QUEUE_BATCH_SIZE=100
```

## Metrics

The service exposes Prometheus metrics at `/metrics:9095`:

- `database_connections_active`: Current active database connections
- `database_queries_total`: Total number of database queries
- `database_errors_total`: Total number of database errors
- `query_latency_seconds`: Query latency histogram
- `queue_size`: Current size of each queue
- `queue_operations_total`: Total number of queue operations

## Data Models

### Documents
```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    crawled_at TIMESTAMP WITH TIME ZONE,
    last_modified TIMESTAMP WITH TIME ZONE,
    content_type TEXT,
    size INTEGER,
    status TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Queues
```sql
CREATE TABLE queue_items (
    id UUID PRIMARY KEY,
    queue_name TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    payload JSONB,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    scheduled_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    INDEX idx_queue_priority (queue_name, priority, status)
);
```

## Dependencies

- **PostgreSQL**: Primary data store
- **SQLx**: Database ORM and query builder

## Error Handling

The service implements comprehensive error handling:

- Database connection issues
- Query failures
- Data validation errors
- Concurrent access issues
- Queue operation failures

## Development

### Local Setup

1. Start PostgreSQL
2. Run migrations
3. Set environment variables
4. Run the service:
   ```bash
   cargo run --bin storage
   ```

### Testing

```bash
# Run unit tests
cargo test --bin storage

# Run integration tests
cargo test --test storage_integration

# Run migration tests
cargo test --test migration_tests
```

## Deployment

The service is deployed as a Docker container:

```bash
# Build the container
docker build -t storage -f services/storage/Dockerfile .

# Run the container
docker run -p 8003:8003 -p 9095:9095 storage
```

## Monitoring

The service can be monitored through:

1. Prometheus metrics at `/metrics:9095`
2. Grafana dashboards (pre-configured)
3. Log output (structured JSON logs)

## Performance Considerations

- Connection pooling
- Query optimization
- Index management
- Transaction management
- Batch operations
- Cache strategies
- Queue performance

## Data Management

1. **Backup Strategy**
   - Regular backups
   - Point-in-time recovery
   - Backup verification
   - Retention policies

2. **Data Cleanup**
   - Old data removal
   - Queue cleanup
   - Index maintenance
   - Vacuum operations

3. **Schema Management**
   - Migration handling
   - Version control
   - Rollback procedures
   - Schema validation 