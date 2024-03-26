FROM debian:bookworm as builder
RUN apt-get update && \
    apt-get install -y build-essential curl git && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
COPY . .
RUN cargo install --locked --path . --root ./out

FROM debian:bookworm-slim
WORKDIR /app
RUN \
    groupadd --gid 10001 app && \
    useradd --uid 10001 --gid 10001 --home /app --create-home app && \
    apt-get update && apt-get -y dist-upgrade && \
    apt-get -y install ca-certificates libjemalloc2 && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

USER app:app
COPY --from=builder /app/out/bin/nginx-syslog-postgres-bridge /app

EXPOSE 8514/udp
CMD ["/app/nginx-syslog-postgres-bridge"]
