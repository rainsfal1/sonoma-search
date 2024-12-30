# Troubleshooting Guide

This document provides guidance for troubleshooting common issues in the Sonoma Search engine.

## Common Issues

### Service Health Checks

#### API Gateway Issues
```bash
# Check API Gateway status
curl -I http://localhost:8000/health

# Check logs
docker compose logs api-gateway

# Common issues:
# 1. Connection refused - Check if service is running
# 2. 502 Bad Gateway - Check downstream services
# 3. 429 Too Many Requests - Rate limit exceeded
```

#### Search Service Issues
```bash
# Check Searcher status
curl -I http://localhost:8001/health

# Check Elasticsearch connection
curl http://localhost:9200/_cluster/health

# Common issues:
# 1. Slow search responses - Check Elasticsearch performance
# 2. No search results - Check index status
# 3. Connection timeouts - Check network connectivity
```

#### Crawler Issues
```bash
# Check Crawler status
curl -I http://localhost:8004/health

# Check crawl queue
curl http://localhost:8004/queue/status

# Common issues:
# 1. Stuck queue - Check for failed jobs
# 2. High memory usage - Check for memory leaks
# 3. Rate limiting - Check crawler delays
```

### Database Issues

#### PostgreSQL Troubleshooting
```sql
-- Check active connections
SELECT * FROM pg_stat_activity;

-- Check slow queries
SELECT pid, age(clock_timestamp(), query_start), usename, query
FROM pg_stat_activity
WHERE state != 'idle'
  AND query NOT ILIKE '%pg_stat_activity%'
ORDER BY query_start desc;

-- Check table bloat
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) as total_size,
    pg_size_pretty(pg_table_size(schemaname || '.' || tablename)) as table_size,
    pg_size_pretty(pg_indexes_size(schemaname || '.' || tablename)) as index_size
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC
LIMIT 10;
```

#### Elasticsearch Issues
```bash
# Check cluster health
curl -X GET "localhost:9200/_cluster/health?pretty"

# Check indices
curl -X GET "localhost:9200/_cat/indices?v"

# Check shard allocation
curl -X GET "localhost:9200/_cat/shards?v"

# Common issues:
# 1. Red cluster status - Check unassigned shards
# 2. High disk usage - Check index sizes
# 3. Slow queries - Check query performance
```

### Memory Issues

#### Memory Leak Detection
```rust
// Add memory profiling
#[cfg(debug_assertions)]
use memory_profiler::MemoryProfiler;

fn main() {
    #[cfg(debug_assertions)]
    let profiler = MemoryProfiler::new();

    // Your application code

    #[cfg(debug_assertions)]
    profiler.report();
}

// Monitor specific allocations
#[derive(Debug)]
struct MemoryStats {
    allocations: usize,
    deallocations: usize,
    current_bytes: usize,
}

impl MemoryStats {
    fn record_allocation(&mut self, size: usize) {
        self.allocations += 1;
        self.current_bytes += size;
    }

    fn record_deallocation(&mut self, size: usize) {
        self.deallocations += 1;
        self.current_bytes -= size;
    }
}
```

#### Resource Monitoring
```bash
#!/bin/bash
# monitor-resources.sh

while true; do
    echo "=== $(date) ==="
    
    echo "Memory Usage:"
    free -h
    
    echo "Top Memory Processes:"
    ps aux --sort=-%mem | head -n 5
    
    echo "Docker Container Stats:"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"
    
    sleep 60
done
```

### Network Issues

#### Network Diagnostics
```bash
# Check network connectivity
for service in api-gateway ranker indexer crawler storage; do
    echo "Testing $service..."
    nc -zv localhost $(grep ${service^^}_PORT .env | cut -d= -f2)
done

# Check DNS resolution
for host in elasticsearch postgres api-gateway; do
    echo "Resolving $host..."
    dig +short $host
done

# Monitor network traffic
tcpdump -i any port 8000 -w capture.pcap
```

#### Connection Pool Issues
```rust
// Connection pool monitoring
#[derive(Debug)]
struct PoolMetrics {
    active_connections: AtomicUsize,
    idle_connections: AtomicUsize,
    total_connections: AtomicUsize,
    connection_timeouts: AtomicUsize,
}

impl PoolMetrics {
    fn record_connection_acquired(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    fn record_connection_released(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        self.idle_connections.fetch_add(1, Ordering::SeqCst);
    }

    fn record_connection_timeout(&self) {
        self.connection_timeouts.fetch_add(1, Ordering::SeqCst);
    }
}
```

### Performance Issues

#### Slow Queries
```rust
// Query timing middleware
pub struct QueryTimer;

impl<S, B> Transform<S, ServiceRequest> for QueryTimer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = QueryTimerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(QueryTimerMiddleware { service }))
    }
}

impl<S, B> Service<ServiceRequest> for QueryTimerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let path = req.path().to_owned();

        Box::pin(async move {
            let response = self.service.call(req).await?;
            let duration = start.elapsed();

            if duration > Duration::from_secs(1) {
                log::warn!("Slow query detected: {} took {:?}", path, duration);
            }

            Ok(response)
        })
    }
}
```

#### Performance Profiling
```rust
// Flamegraph generation
#[cfg(feature = "profiling")]
use flamegraph::{self, Options};

fn main() {
    #[cfg(feature = "profiling")]
    {
        let options = Options::default();
        flamegraph::generate_flamegraph(&options, || {
            // Your application code
        });
    }
}

// Performance metrics
lazy_static! {
    static ref REQUEST_HISTOGRAM: Histogram = register_histogram!(
        "request_duration_seconds",
        "Request duration in seconds",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();

    static ref QUERY_GAUGE: Gauge = register_gauge!(
        "active_queries",
        "Number of active queries"
    ).unwrap();
}
```

### Error Handling

#### Error Logging
```rust
// Structured error logging
#[derive(Debug, Serialize)]
struct ErrorLog {
    timestamp: DateTime<Utc>,
    error_type: String,
    message: String,
    stack_trace: String,
    context: HashMap<String, Value>,
}

impl ErrorLog {
    fn new(error: &Error, context: HashMap<String, Value>) -> Self {
        Self {
            timestamp: Utc::now(),
            error_type: error.type_id().to_string(),
            message: error.to_string(),
            stack_trace: format!("{:?}", error.backtrace()),
            context,
        }
    }

    fn log(&self) {
        log::error!("Error: {}", serde_json::to_string(&self).unwrap());
    }
}
```

#### Error Recovery
```rust
// Circuit breaker pattern
pub struct CircuitBreaker {
    state: AtomicUsize,
    failure_threshold: u32,
    reset_timeout: Duration,
    last_failure: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            state: AtomicUsize::new(0), // 0: Closed, 1: Open, 2: Half-Open
            failure_threshold,
            reset_timeout,
            last_failure: AtomicU64::new(0),
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.is_open() {
            return Err(/* Circuit is open */);
        }

        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(error)
            }
        }
    }
}
```

### Debugging Tools

#### Logging Configuration
```rust
// Configure logging
pub fn setup_logging() {
    use env_logger::{Builder, Target};
    use log::LevelFilter;

    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

// Structured logging
#[derive(Debug, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    module: String,
    line: u32,
    context: HashMap<String, Value>,
}
```

#### Debugging Middleware
```rust
// Request/Response debugging middleware
pub struct DebugMiddleware;

impl<S, B> Transform<S, ServiceRequest> for DebugMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DebugMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DebugMiddlewareService { service }))
    }
}

impl<S, B> Service<ServiceRequest> for DebugMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        log::debug!("Request: {:?}", req);

        Box::pin(async move {
            let response = self.service.call(req).await?;
            log::debug!("Response: {:?}", response);
            Ok(response)
        })
    }
}
```

### Recovery Procedures

#### Database Recovery
```bash
#!/bin/bash
# database-recovery.sh

# Backup current state
pg_dump -h localhost -U postgres sonoma > backup.sql

# Stop services
docker compose stop

# Reset database
dropdb -h localhost -U postgres sonoma
createdb -h localhost -U postgres sonoma

# Restore from backup
psql -h localhost -U postgres sonoma < backup.sql

# Restart services
docker compose up -d
```

#### Index Recovery
```bash
#!/bin/bash
# index-recovery.sh

# Backup indices
curl -X PUT "localhost:9200/_snapshot/backup/snapshot_1?wait_for_completion=true"

# Delete corrupted indices
curl -X DELETE "localhost:9200/documents"

# Recreate indices
curl -X PUT "localhost:9200/documents" -H "Content-Type: application/json" -d @config/elasticsearch/index.json

# Restore from backup
curl -X POST "localhost:9200/_snapshot/backup/snapshot_1/_restore"

# Verify recovery
curl -X GET "localhost:9200/_cat/indices?v"
``` 