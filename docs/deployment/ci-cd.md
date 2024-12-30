# CI/CD Pipeline

This document describes the Continuous Integration and Continuous Deployment (CI/CD) pipeline implemented using GitHub Actions.

## Overview

The CI/CD pipeline automates building, testing, and deploying the Sonoma Search engine. It ensures code quality, runs tests, and manages deployments across different environments.

## Pipeline Structure

```mermaid
graph TD
    A[Push to Branch] --> B[Code Quality]
    B --> C[Build]
    C --> D[Test]
    D --> E{Branch?}
    E -->|main| F[Deploy to Production]
    E -->|staging| G[Deploy to Staging]
    E -->|develop| H[Deploy to Development]
```

## GitHub Actions Workflows

### Main Workflow

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, staging, develop ]
  pull_request:
    branches: [ main, staging ]

jobs:
  quality:
    name: Code Quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
          
      - name: Check formatting
        run: cargo fmt -- --check
        
      - name: Run clippy
        run: cargo clippy -- -D warnings

  build:
    name: Build
    needs: quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
        
      - name: Login to Docker Registry
        uses: docker/login-action@v2
        with:
          registry: ${{ secrets.DOCKER_REGISTRY }}
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
          
      - name: Build and push images
        run: |
          docker compose build
          docker compose push

  test:
    name: Test
    needs: build
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
      elasticsearch:
        image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
        env:
          discovery.type: single-node
          xpack.security.enabled: false
        ports:
          - 9200:9200
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: cargo test --workspace
        env:
          DATABASE_URL: postgresql://test:test@localhost:5432/test
          ELASTICSEARCH_URL: http://localhost:9200

  deploy-dev:
    name: Deploy to Development
    if: github.ref == 'refs/heads/develop'
    needs: test
    runs-on: ubuntu-latest
    environment: development
    steps:
      - name: Deploy to Development
        uses: your-deployment-action@v1
        with:
          environment: development
          credentials: ${{ secrets.DEPLOY_CREDENTIALS }}

  deploy-staging:
    name: Deploy to Staging
    if: github.ref == 'refs/heads/staging'
    needs: test
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - name: Deploy to Staging
        uses: your-deployment-action@v1
        with:
          environment: staging
          credentials: ${{ secrets.DEPLOY_CREDENTIALS }}

  deploy-prod:
    name: Deploy to Production
    if: github.ref == 'refs/heads/main'
    needs: test
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Deploy to Production
        uses: your-deployment-action@v1
        with:
          environment: production
          credentials: ${{ secrets.DEPLOY_CREDENTIALS }}
```

## Environment Configuration

### Development
- Automatic deployments from `develop` branch
- Reduced resource allocation
- Debug logging enabled
- Test data subset

### Staging
- Manual approval required
- Production-like environment
- Full dataset
- Performance monitoring

### Production
- Manual approval required
- High availability setup
- Production data
- Full monitoring

## Secret Management

Required secrets in GitHub:

```yaml
# Docker Registry
DOCKER_REGISTRY: Container registry URL
DOCKER_USERNAME: Registry username
DOCKER_PASSWORD: Registry password

# Deployment
DEPLOY_CREDENTIALS: Deployment credentials
SSH_PRIVATE_KEY: SSH key for deployment

# Environment-specific
DEV_ENV_FILE: Development environment variables
STAGING_ENV_FILE: Staging environment variables
PROD_ENV_FILE: Production environment variables
```

## Quality Gates

1. **Code Quality**
   - Rust formatting (rustfmt)
   - Linting (clippy)
   - Code coverage
   - SAST scanning

2. **Testing**
   - Unit tests
   - Integration tests
   - End-to-end tests
   - Performance tests

3. **Security**
   - Dependency scanning
   - Container scanning
   - Secret scanning
   - License compliance

## Deployment Process

1. **Pre-deployment**
   - Backup verification
   - Health checks
   - Resource verification
   - Configuration validation

2. **Deployment**
   - Rolling updates
   - Blue-green deployment
   - Canary releases
   - Rollback preparation

3. **Post-deployment**
   - Health verification
   - Smoke tests
   - Performance validation
   - Monitoring alerts

## Monitoring and Alerts

1. **Deployment Metrics**
   - Deployment frequency
   - Lead time
   - Change failure rate
   - Mean time to recovery

2. **Pipeline Metrics**
   - Build duration
   - Test coverage
   - Success rate
   - Resource usage

## Rollback Procedures

1. **Automatic Rollback**
   - Failed health checks
   - Error rate threshold
   - Performance degradation
   - Critical security issues

2. **Manual Rollback**
   ```bash
   # Revert to previous version
   git revert HEAD
   git push

   # Manual container rollback
   docker compose pull [previous-tag]
   docker compose up -d
   ```

## Troubleshooting

1. **Pipeline Failures**
   - Check action logs
   - Verify secrets
   - Check resource limits
   - Validate configurations

2. **Deployment Failures**
   - Check deployment logs
   - Verify credentials
   - Check network connectivity
   - Validate environment variables 