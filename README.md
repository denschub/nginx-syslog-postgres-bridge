# nginx-syslog-postgres-bridge

A bridge to connect nginx' syslog output for `access_log` to a PostgreSQL database. Transferring `error_log` is not supported.

## Usage

The server can be configured with CLI arguments and with Environmental variables. Run with `--help` for details.

## Data consistency and completeness

nginx does not store failed deliveries. If this service is down, log lines will simply be dropped by nginx. Invalid datagrams will be dropped. Log lines that do not fit within a single UDP datagram (~65KiB) will, [as spec'ed][rfc5426], result in an incomplete JSON document and thus be dropped as well.

The data resulting from this tool should be considered good enough for simple statistical analysis and occasional tracing. It is not replacing a full end-to-end tracing setup with a coverage guarantee.

## Security considerations

This bridge does not run any authentication, authorization, or validation. Any valid JSON datagram received will be stored in the database. While it would be possible to allow-list certain source IPs in this application, doing that in a firewall probably makes more sense.

All data sent to this application is sent unencrypted over UDP. While there are syslog transport mechanisms via TCP and encryption, [nginx does not support those][nginx-syslog]. If logging data is sent over an untrusted network, encrypted tunneling is recommended since the log format includes PII (namely, the user's IP).

## Performance considerations

Because nginx is just firing UDP datagrams towards this application with no regard for anything, this application is designed to process incoming UDP traffic as fast as possible. Each incoming UDP datagram is immediately spawned off into a different task to make room for more UDP traffic. This results in an application that can handle pretty much all traffic - but due to background processing queues and the latency of storing things into a database, memory usage can grow. The `queue-size` setting limits how many valid log entries can be stored asynchronously. The default value of 10k entries guarantees that even in the worst case of 10k log entries each using 64KiB (which, in practice, is impossible), the app is limited to ~650MiB memory usage.

In a local benchmark, with nginx, PostgreSQL, and this application sharing a Docker environment with 5 CPU cores of an Apple M1 Max, I was able to achieve a peak traffic of ~22k req/s. This bridge was able to handle all the incoming log entries without issues. However, during the 30 seconds of burn-in test, a total of ~660k requests have been responded to. Most of them did not immediately end up in the database, as PostgreSQL inserts in their current form are rather slow, so the memory usage peaked at ~480 MiB. Backfilling those log entries into the database took roughly 8 minutes. It's therefore not a good idea to expose this application to a constant load of more than 1k req/s.

Short bursts of traffic with longer pauses in between to clear the backlog are fine. Note, however, that this application uses `jemalloc` is its allocator, the application will allocate a lot of memory for the queue, and it will take a while for this memory to be returned to the system. If handling large spikes of traffic is a concern, check [`jemalloc`s tuning documentation][jemalloc-tuning] for information on how to free memory faster.

For constantly high loads, this application can be optimized by a) batching `INSERT` queries in transactions and b) running transactions in parallel. However, as the currently possible load exceeds any load realistically expected in its environment, these optimizations are ignored for now.

## Required nginx configuration

As for the log format, a specific and relatively compact JSON format is required. The following config can be made available globally:

```
log_format postgres_bridge_json escape=json '{'
  '"hostname":"$hostname",'
  '"ts":"$msec",'
  '"server":{'
    '"name":"$server_name",'
    '"port":"$server_port"'
  '},"client":{'
    '"addr":"$remote_addr",'
    '"forwarded_for":"$http_x_forwarded_for",'
    '"referer":"$http_referer",'
    '"ua":"$http_user_agent"'
  '},"req":{'
    '"host":"$host",'
    '"length":"$request_length",'
    '"method":"$request_method",'
    '"proto":"$server_protocol",'
    '"scheme":"$scheme",'
    '"uri":"$request_uri"'
  '},"res":{'
    '"body_length":"$body_bytes_sent",'
    '"duration":"$request_time",'
    '"length":"$bytes_sent",'
    '"status":"$status"'
  '},"upstream":{'
    '"addr":"$upstream_addr",'
    '"bytes_received":"$upstream_bytes_received",'
    '"bytes_sent":"$upstream_bytes_sent",'
    '"cache_status":"$upstream_cache_status",'
    '"connect_time":"$upstream_connect_time",'
    '"host":"$proxy_host",'
    '"response_length":"$upstream_response_length",'
    '"response_time":"$upstream_response_time",'
    '"status":"$upstream_status"'
  '}'
'}';
```

To send access log entries, set the following, either globally or for a specific `server {}` block:

```
access_log syslog:server=nginx-syslog-bridge.example.com:514,nohostname postgres_bridge_json;
```

[jemalloc-tuning]: https://github.com/jemalloc/jemalloc/blob/dev/TUNING.md
[nginx-syslog]: https://nginx.org/en/docs/syslog.html
[rfc5426]: https://www.rfc-editor.org/rfc/rfc5426
