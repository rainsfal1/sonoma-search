# Build stage
FROM rust:1.81 AS builder

WORKDIR /usr/src/search-engine

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libssl3 \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app
COPY --from=builder /usr/src/search-engine/target/release/ranker /app/ranker
COPY --from=builder /usr/src/search-engine/.sqlx /app/.sqlx

WORKDIR /app
RUN chmod +x /app/ranker

# Expose metrics port
EXPOSE 9093

CMD ["/app/ranker"]