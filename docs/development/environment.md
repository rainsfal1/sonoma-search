# Development Environment Setup

This document describes how to set up a development environment for the Sonoma Search engine.

## Prerequisites

### Required Software
- Rust 1.70+ (`rustup`)
- Docker 24.0+
- Docker Compose 2.0+
- Git 2.0+
- Node.js 18+ (for web frontend)
- PostgreSQL 16 client tools
- curl
- jq

### System Requirements
- 16GB RAM minimum
- 50GB disk space
- Unix-like OS (Linux/macOS)
- Python 3.8+ (for scripts)

## Initial Setup

### 1. Clone Repository
```bash
# Clone the repository
git clone https://github.com/rainsfal1/sonoma-search.git
cd sonoma-search

# Initialize git hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

### 2. Rust Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt
rustup component add clippy
rustup component add rust-analyzer

# Install cargo tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-expand
cargo install sqlx-cli
```

### 3. Environment Configuration
```bash
# Copy example environment files
cp .env.example .env
cp config/development/* config/

# Generate development certificates
./scripts/generate-certs.sh

# Set up database
./scripts/setup-database.sh
```

### 4. Dependencies Installation
```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql-client \
    protobuf-compiler

# Install Node.js dependencies
cd web
npm install
cd ..
```

## Development Workflow

### 1. Start Development Services
```bash
# Start infrastructure services
docker compose -f docker-compose.dev.yml up -d postgres elasticsearch

# Wait for services to be ready
./scripts/wait-for-services.sh

# Run database migrations
sqlx migrate run

# Start development servers
cargo watch -x run
```

### 2. Code Style and Linting
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run security audit
cargo audit
```

### 3. Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*_integration'

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

## IDE Setup

### VS Code
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.inlayHints.enable": true,
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

### Required Extensions
- rust-analyzer
- crates
- Better TOML
- Docker
- PostgreSQL
- ElasticSearch

## Service Development

### 1. API Gateway
```bash
# Start with hot reload
cargo watch -x 'run --bin api-gateway'

# Run tests
cargo test -p api-gateway
```

### 2. Search Services
```bash
# Start searcher
cargo watch -x 'run --bin searcher'

# Start ranker
cargo watch -x 'run --bin ranker'

# Run tests
cargo test -p searcher -p ranker
```

### 3. Data Services
```bash
# Start crawler
cargo watch -x 'run --bin crawler'

# Start indexer
cargo watch -x 'run --bin indexer'

# Start storage
cargo watch -x 'run --bin storage'

# Run tests
cargo test -p crawler -p indexer -p storage
```

### 4. Web Frontend
```bash
# Start development server
cd web
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

## Database Development

### PostgreSQL
```bash
# Connect to database
psql postgresql://user:password@localhost:5432/sonoma

# Create new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert migration
sqlx migrate revert
```

### Elasticsearch
```bash
# Create index
curl -X PUT "localhost:9200/documents" -H "Content-Type: application/json" -d @config/elasticsearch/index.json

# Test search
curl -X GET "localhost:9200/documents/_search" -H "Content-Type: application/json" -d '{
  "query": {"match_all": {}}
}'
```

## Debugging

### Logging
```bash
# Set log level
export RUST_LOG=debug

# Log to file
export RUST_LOG_STYLE=always
export RUST_LOG=debug > service.log

# Trace SQL queries
export SQLX_OFFLINE=true
export DATABASE_URL="postgresql://user:password@localhost:5432/sonoma"
```

### Profiling
```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bin service-name

# Memory profiling
export RUST_BACKTRACE=1
cargo run --release
```

## Common Issues

### Database Connection
```bash
# Check connection
psql -h localhost -U user -d sonoma

# Reset database
./scripts/reset-database.sh

# Clear data
./scripts/clear-data.sh
```

### Elasticsearch Issues
```bash
# Check cluster health
curl -X GET "localhost:9200/_cluster/health?pretty"

# Reset indexes
./scripts/reset-elasticsearch.sh

# Clear indices
curl -X DELETE "localhost:9200/*"
```

### Docker Issues
```bash
# Reset containers
docker compose down -v
docker compose up -d

# Clean up
docker system prune -af
docker volume prune -f
```

## Performance Testing

### Load Testing
```bash
# Install k6
brew install k6

# Run load test
k6 run tests/load/search_test.js

# Run with metrics
k6 run --out influxdb=http://localhost:8086/k6 tests/load/search_test.js
```

### Benchmarking
```bash
# Install criterion
cargo install criterion

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench benchmark_name
``` 