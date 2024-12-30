# Technology Stack

This document outlines the technology stack used in the Sonoma Search project.

## Programming Languages and Frameworks

### Backend Services
- **Language**: Rust
- **Web Framework**: Actix-web
- **Database ORM**: SQLx
- **Search Engine**: Elasticsearch
- **Metrics**: Prometheus client

### Frontend
- **Framework**: Web-based interface (specific framework details to be added)
- **API Communication**: HTTP/REST

## Databases and Storage

### Primary Database
- **PostgreSQL**: Main data store for structured data
  - Stores crawled page metadata
  - Manages service state
  - Handles relationship data

### Search Engine
- **Elasticsearch**: Full-text search and indexing
  - Document storage and retrieval
  - Full-text search capabilities
  - Ranking and scoring

## Infrastructure and DevOps

### Containerization
- **Docker**: Container runtime
- **Docker Compose**: Local development and testing
- **Docker Registry**: Container image storage

### CI/CD
- **GitHub Actions**: Automated build and deployment pipeline
  - Automated testing
  - Container image building
  - Deployment automation

### Monitoring and Observability
- **Prometheus**: Metrics collection and storage
  - Service metrics
  - System metrics
  - Custom business metrics

- **Grafana**: Metrics visualization and dashboards
  - Real-time monitoring
  - Historical data analysis
  - Alert management

## Service Ports

### Application Ports
- Web Frontend: 3000
- API Gateway: 8000
- Elasticsearch: 9200

### Metrics Ports
- Searcher: 9091
- Ranker: 9092
- Indexer: 9093
- Crawler: 9094
- Storage: 9095

### Monitoring Ports
- Prometheus: 9090
- Grafana: 3001

## Development Tools

### Version Control
- **Git**: Source code management
- **GitHub**: Code hosting and collaboration

### Development Environment
- **Rust Toolchain**:
  - Cargo: Package manager
  - Rustfmt: Code formatter
  - Clippy: Linter

### Testing
- **Unit Testing**: Rust's built-in testing framework
- **Integration Testing**: Custom test suites
- **Load Testing**: (specific tools to be added) 