FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libjemalloc2 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/nginx-syslog-postgres-bridge /usr/local/bin/nginx-syslog-postgres-bridge
COPY ./scripts/jemalloc-wrapper /usr/local/bin/jemalloc-wrapper
ENTRYPOINT [ "jemalloc-wrapper" ]
CMD ["nginx-syslog-postgres-bridge"]
