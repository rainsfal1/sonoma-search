# PostgreSQL Schema

This document describes the PostgreSQL database schema and configuration for the Sonoma Search engine.

## Database Schema

### Documents Table
```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    content TEXT,
    content_type TEXT,
    language TEXT,
    crawled_at TIMESTAMP WITH TIME ZONE,
    last_modified TIMESTAMP WITH TIME ZONE,
    size INTEGER,
    status TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_documents_url ON documents (url);
CREATE INDEX idx_documents_status ON documents (status);
CREATE INDEX idx_documents_crawled_at ON documents (crawled_at);
CREATE INDEX idx_documents_metadata ON documents USING GIN (metadata);
```

### Queue Items Table
```sql
CREATE TABLE queue_items (
    id UUID PRIMARY KEY,
    queue_name TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    payload JSONB,
    status TEXT DEFAULT 'pending',
    scheduled_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT idx_queue_priority UNIQUE (queue_name, priority, status)
);

CREATE INDEX idx_queue_items_status ON queue_items (status);
CREATE INDEX idx_queue_items_scheduled ON queue_items (scheduled_at);
```

### Domain Metadata Table
```sql
CREATE TABLE domain_metadata (
    domain TEXT PRIMARY KEY,
    robots_txt TEXT,
    crawl_delay INTEGER,
    last_crawled_at TIMESTAMP WITH TIME ZONE,
    error_count INTEGER DEFAULT 0,
    status TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_domain_metadata_status ON domain_metadata (status);
```

### Search Analytics Table
```sql
CREATE TABLE search_analytics (
    id UUID PRIMARY KEY,
    query TEXT NOT NULL,
    results_count INTEGER,
    page_number INTEGER,
    user_agent TEXT,
    ip_address TEXT,
    session_id TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_search_analytics_query ON search_analytics (query);
CREATE INDEX idx_search_analytics_created ON search_analytics (created_at);
```

## Database Configuration

### Connection Settings
```ini
# postgresql.conf
max_connections = 500
shared_buffers = 8GB
work_mem = 128MB
maintenance_work_mem = 1GB
effective_cache_size = 24GB
synchronous_commit = off

# Connection Pooling
max_prepared_transactions = 0
max_worker_processes = 8
max_parallel_workers_per_gather = 4
max_parallel_workers = 8

# WAL Settings
wal_level = replica
max_wal_size = 4GB
min_wal_size = 1GB
checkpoint_completion_target = 0.9
```

### Performance Tuning

```ini
# Query Planning
random_page_cost = 1.1
effective_io_concurrency = 200
default_statistics_target = 100

# Memory Settings
huge_pages = try
temp_buffers = 256MB
max_stack_depth = 7MB
dynamic_shared_memory_type = posix

# Background Writer
bgwriter_delay = 200ms
bgwriter_lru_maxpages = 100
bgwriter_lru_multiplier = 2.0
```

## Maintenance Procedures

### Vacuum Settings
```sql
-- Regular VACUUM
VACUUM ANALYZE documents;
VACUUM ANALYZE queue_items;
VACUUM ANALYZE domain_metadata;
VACUUM ANALYZE search_analytics;

-- Full VACUUM (when needed)
VACUUM FULL documents;
```

### Index Maintenance
```sql
-- Reindex tables
REINDEX TABLE documents;
REINDEX TABLE queue_items;
REINDEX TABLE domain_metadata;
REINDEX TABLE search_analytics;
```

## Backup Configuration

### Continuous Archiving
```ini
# postgresql.conf
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/archive/%f'
archive_timeout = 60

# Recovery Settings
restore_command = 'cp /var/lib/postgresql/archive/%f %p'
recovery_target_timeline = 'latest'
```

### Backup Script
```bash
#!/bin/bash
BACKUP_DIR="/backup/postgresql"
DATE=$(date +%Y%m%d)

# Full backup
pg_dump -Fc sonoma > $BACKUP_DIR/full_$DATE.dump

# Schema only
pg_dump -s sonoma > $BACKUP_DIR/schema_$DATE.sql

# Individual tables
for table in documents queue_items domain_metadata search_analytics; do
    pg_dump -t $table sonoma > $BACKUP_DIR/${table}_${DATE}.sql
done
```

## Security Configuration

### Access Control
```sql
-- Create roles
CREATE ROLE readonly;
CREATE ROLE readwrite;
CREATE ROLE admin;

-- Grant privileges
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO readwrite;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO admin;

-- Create users
CREATE USER searcher_service WITH PASSWORD 'xxx';
CREATE USER crawler_service WITH PASSWORD 'xxx';
CREATE USER storage_service WITH PASSWORD 'xxx';

-- Assign roles
GRANT readonly TO searcher_service;
GRANT readwrite TO crawler_service;
GRANT admin TO storage_service;
```

### Connection Security
```ini
# pg_hba.conf
hostssl all             all             10.0.0.0/8             scram-sha-256
hostssl all             all             172.16.0.0/12          scram-sha-256
hostssl all             all             192.168.0.0/16         scram-sha-256
```

## Monitoring Queries

### Performance Monitoring
```sql
-- Active queries
SELECT pid, age(clock_timestamp(), query_start), usename, query
FROM pg_stat_activity
WHERE query != '<IDLE>' AND query NOT ILIKE '%pg_stat_activity%'
ORDER BY query_start desc;

-- Table statistics
SELECT schemaname, relname, seq_scan, seq_tup_read, 
       idx_scan, idx_tup_fetch, n_tup_ins, n_tup_upd, n_tup_del
FROM pg_stat_user_tables;

-- Index usage
SELECT schemaname, tablename, indexname, idx_scan, 
       idx_tup_read, idx_tup_fetch
FROM pg_stat_user_indexes;
```

### Health Checks
```sql
-- Database size
SELECT pg_size_pretty(pg_database_size('sonoma'));

-- Table sizes
SELECT relname, pg_size_pretty(pg_total_relation_size(relid))
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(relid) DESC;

-- Cache hit ratio
SELECT sum(heap_blks_read) as heap_read, 
       sum(heap_blks_hit)  as heap_hit, 
       sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) as ratio
FROM pg_statio_user_tables;
```

## Migration Procedures

### Version Control
```sql
CREATE TABLE schema_versions (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    script_name TEXT NOT NULL
);

-- Track migrations
INSERT INTO schema_versions (version, description, script_name)
VALUES (1, 'Initial schema', '001_initial_schema.sql');
```

### Migration Script
```bash
#!/bin/bash
MIGRATIONS_DIR="/app/migrations"
CURRENT_VERSION=$(psql -t -c "SELECT MAX(version) FROM schema_versions;")

for migration in $(ls $MIGRATIONS_DIR/*.sql | sort); do
    version=$(basename $migration | cut -d_ -f1)
    if [ "$version" -gt "$CURRENT_VERSION" ]; then
        psql -f $migration
        description=$(head -n1 $migration | grep -o '".*"' | tr -d '"')
        psql -c "INSERT INTO schema_versions (version, description, script_name) 
                 VALUES ($version, '$description', '$(basename $migration)');"
    fi
done
``` 