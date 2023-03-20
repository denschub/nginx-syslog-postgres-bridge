FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/nginx-syslog-postgres-bridge /usr/local/bin/nginx-syslog-postgres-bridge
CMD ["nginx-syslog-postgres-bridge"]
