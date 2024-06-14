# nginx-syslog-postgres-bridge

A bridge to connect nginx' syslog output for `access_log` to a PostgreSQL database. Transferring `error_log` is not supported.

It is highly recommended that the PostgreSQL used for this tool supports the TimescaleDB extension. It works fine with either the cloud-hosted option, or a [self-hosted TimescaleDB][selfhosted-timescale]. If TimescaleDB support is detected, the database migrations automatically set up the `access_log` table as a Hypertable, with partitioning on the `event_ts`, and a 365 day retention policy.

Running this on a plain PostgreSQL works - but performance will take a hit for larger datasets, especially query performance. You also have to manually delete old entries if you want to.

## Data consistency and completeness

nginx does not store failed deliveries. If this service is down, log lines will simply be dropped by nginx. Invalid datagrams will be dropped. Log lines that do not fit within a single UDP datagram (~65KiB) will, [as spec'ed][rfc5426], result in an incomplete JSON document and thus be dropped as well.

The data resulting from this tool should be considered good enough for simple statistical analysis and occasional tracing. It does not replace a full end-to-end tracing setup with a coverage guarantee.

## Security considerations

This bridge does not run any authentication, authorization, or validation. Any valid JSON datagram received will be stored in the database. While it would be possible to allow-list certain source IPs in this application, doing that in a firewall probably makes more sense.

All data sent to this application is sent unencrypted over UDP. While there are syslog transport mechanisms via TCP and encryption, [nginx does not support those][nginx-syslog]. If logging data is sent over an untrusted network, encrypted tunneling is recommended since the log format includes PII (namely, the user's IP).

## Performance considerations

Because nginx is just firing UDP datagrams towards this application with no regard for anything, this application is designed to process incoming UDP traffic as fast as possible. Each incoming UDP datagram is immediately spawned off into a different task to make room for more UDP traffic. This results in an application that can handle pretty much all traffic - but due to background processing queues and the latency of storing things in a database, memory usage can grow. The `queue-size` setting limits how many valid log entries can be stored asynchronously. The default value of 10k entries guarantees that even in the worst case of 10k log entries, each using 64KiB (which, in practice, is impossible), the app is limited to ~650MiB memory usage.

In a local benchmark, with nginx, PostgreSQL, and this application sharing a Docker environment with 5 CPU cores of an Apple M1 Max, I was able to achieve a peak traffic of ~22k req/s. This bridge was able to handle all the incoming log entries without issues. However, during the 30 seconds of the burn-in test, a total of ~660k requests have been responded to. Most of them did not immediately end up in the database, as PostgreSQL inserts in their current form are rather slow, so the memory usage peaked at ~480 MiB. Backfilling those log entries into the database took roughly 8 minutes. It's therefore not a good idea to expose this application to a constant load of more than 1k req/s.

Short bursts of traffic with longer pauses in between to clear the backlog are fine. Note, however, that this application uses `jemalloc` is its allocator, the application will allocate a lot of memory for the queue, and it will take a while for this memory to be returned to the system. If handling large spikes of traffic is a concern, check [`jemalloc`s tuning documentation][jemalloc-tuning] for information on how to free memory faster.

For constantly high loads, this application can be optimized by a) batching `INSERT` queries in transactions and b) running transactions in parallel. However, as the currently possible load exceeds any load realistically expected in its environment, these optimizations are ignored for now.

## Required nginx configuration

nginx needs to be configured with a special log format. [Check the dedicated documentation page for details](./docs/nginx_config.md).

## Deployment and configuration

A container image is pushed to [the GitHub Container registry at `ghcr.io/denschub/nginx-syslog-postgres-bridge:latest`][ghcr], and [to Docker Hub as `denschub/nginx-syslog-postgres-bridge:latest`][dockerhub]. The container exposes port 8514.

Configuration of the server is done with either environment variables or via CLI arguments. Make sure to set `NGINXPG_DATABASE`/`--database-url` to a valid PostgreSQL connection URL like `postgres://postgres@127.0.0.1/nginx_logs`. The database needs to exist before starting the server, but the server startup procedure will take care of all database migrations.

Released binaries are available for all stable releases. Check the [Releases section on GitHub][github-releases] for the latest release, and you'll find a `.zip` with a pre-built binary.

Additional settings are available, for example a custom limit for the maximum queue length. Run with `--help` to see all details.

## License

[MIT](/LICENSE).

[dockerhub]: https://hub.docker.com/repository/docker/denschub/nginx-syslog-postgres-bridge/general
[ghcr]: https://github.com/denschub/nginx-syslog-postgres-bridge/pkgs/container/nginx-syslog-postgres-bridge
[github-releases]: https://github.com/denschub/nginx-syslog-postgres-bridge/releases
[jemalloc-tuning]: https://github.com/jemalloc/jemalloc/blob/dev/TUNING.md
[nginx-syslog]: https://nginx.org/en/docs/syslog.html
[rfc5426]: https://www.rfc-editor.org/rfc/rfc5426
[selfhosted-timescale]: https://docs.timescale.com/self-hosted/latest
