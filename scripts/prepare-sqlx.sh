#!/bin/bash

# Source environment variables from .env
set -a
source .env
set +a

# Extract username and database name from DATABASE_URL
DB_USER=$(echo $DATABASE_URL | sed -n 's/.*:\/\/\([^:]*\):.*/\1/p')
DB_NAME=$(echo $DATABASE_URL | sed -n 's/.*\/\([^?]*\).*/\1/p')

# Start a temporary Postgres container
docker run --name temp-postgres \
  -e POSTGRES_USER=$DB_USER \
  -e POSTGRES_PASSWORD=$POSTGRES_PASSWORD \
  -e POSTGRES_DB=$DB_NAME \
  -p 5432:5432 \
  -d postgres:15

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to start..."
until docker exec temp-postgres pg_isready -U $DB_USER; do
  sleep 1
done

# Initialize database
docker cp services/storage/src/scripts/init.sql temp-postgres:/init.sql
docker exec temp-postgres psql -U $DB_USER -d $DB_NAME -f /init.sql

# Generate SQLx prepare files
echo "Running SQLx prepare..."
cargo sqlx prepare --workspace

# Cleanup
docker stop temp-postgres
docker rm temp-postgres
