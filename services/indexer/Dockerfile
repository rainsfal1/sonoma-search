# Build stage
FROM rust:1.81 AS builder

WORKDIR /usr/src/search-engine

# Copy the entire project
COPY . .

# Set SQLx to use offline mode
ENV SQLX_OFFLINE=true

# Build the indexer
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install required dependencies including Elasticsearch client libraries
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libssl3 \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create the directory for the indexer
RUN mkdir -p /app

# Copy the built executable
COPY --from=builder /usr/src/search-engine/target/release/indexer /app/indexer

# Copy the .sqlx directory
COPY --from=builder /usr/src/search-engine/.sqlx /app/.sqlx

WORKDIR /app

# Make sure the indexer executable has the right permissions
RUN chmod +x /app/indexer

# Expose metrics port
EXPOSE 9091

CMD ["/app/indexer"]