# Build stage
FROM rust:1.81 AS builder

# Set the working directory
WORKDIR /usr/src/indexer

# Copy the entire project including the source files and the Cargo.toml
COPY . .

# Set SQLx to use offline mode (if applicable)
ENV SQLX_OFFLINE=true

# Build the project in release mode
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install required dependencies
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libssl3 \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create the directory for the indexer app
RUN mkdir -p /app

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/indexer/target/release/indexer /app/indexer

# Copy any additional configuration files
COPY --from=builder /usr/src/indexer/.sqlx /app/.sqlx

# Set the working directory inside the container
WORKDIR /app

# Ensure the executable has the right permissions
RUN chmod +x /app/indexer

# Set the environment variables (optional, can also use .env file if needed)
# ENV DATABASE_URL=your_database_url
# ENV ELASTICSEARCH_URL=your_elasticsearch_url

# Expose necessary ports (adjust as per your project requirements)
EXPOSE 8080
EXPOSE 9200

# Run the indexer application
CMD ["/app/indexer"]
