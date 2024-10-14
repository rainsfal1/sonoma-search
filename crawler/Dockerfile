# Build stage
FROM rust:1.81 AS builder

WORKDIR /usr/src/search-engine

# Copy the entire project, including the .sqlx directory
COPY . .

# Set SQLx to use offline mode
ENV SQLX_OFFLINE=true

# Build the crawler
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y openssl ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Create the directory for the crawler
RUN mkdir -p /app

# Copy the built executable
COPY --from=builder /usr/src/search-engine/target/release/crawler /app/crawler

# Copy the .sqlx directory
COPY --from=builder /usr/src/search-engine/.sqlx /app/.sqlx

# Copy the config file
COPY --from=builder /usr/src/search-engine/crawler/config.toml /app/config.toml

WORKDIR /app

# Make sure the crawler executable has the right permissions
RUN chmod +x /app/crawler

# Use the full path to the crawler executable
CMD ["/app/crawler"]
