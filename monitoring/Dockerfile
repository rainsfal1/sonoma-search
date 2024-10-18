FROM rust:1.81 as builder
WORKDIR /usr/src/monitoring
COPY . .
RUN cat Cargo.toml
RUN rm -f Cargo.lock
RUN cargo update --verbose
RUN cargo tree --all-features
RUN cargo build --release --verbose

FROM debian:bookworm-slim  
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/monitoring/target/release/monitoring /usr/local/bin/monitoring
CMD ["monitoring"]