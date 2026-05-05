# 3.1.0

If your webserver and this bridge are running on the same host, you can now set the `LISTEN_ADDR` to a unix socket path (like `unix:/var/run/ngxslpg.sock`) as an alternative to the UDP socket. This cuts some network overhead if needed.

This is not a breaking change: the default behavior remains unchanged and previously working values continue to work.

# 3.0.1

This version does not contain any functional changes. It only updates third-party dependencies.

# 3.0.0

This version moves away from individual `INSERT` statements for each individual request. Instead, database insertions are batched via large `INSERT INTO ... SELECT * FROM UNNEST` queries. This significantly increases the throughput, while also reducing database server load.

- **Potentially breaking**: The short setting CLI flags, like `-d` for `--database-url` have been removed. If you used those, please migrate to the long form names or environment variables.
- **Potentially breaking**: The default value for `QUEUE_SIZE` has been dropped to 50. Unless you handle hundreds of requests per second, this should not matter to you.
- A new setting, `--insert-batch-size`/`INSERT_BATCH_SIZE`, is available to set the size of batch insertions. The default batch size is 10, which should be fine for small setups.
- Another setting, `--insert-timeout`/`INSERT_TIMEOUT`, exists to throttle `INSERT` queries. If the buffer doesn't immediately reaches `INSERT_BATCH_SIZE`, the tool waits `INSERT_TIMEOUT` milliseconds before inserting the log entries into the database. Defaults to 1 second.
- You can now set the log level and log output format. Run with `--help` to see the available flags and values.

# 2.1.4

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.1.3

This version does not contain any functional changes. It only updates third-party dependencies. Users of the binary releases or the official container images should be aware that there was a switch from Debian to Alpine as the base system. `libjemalloc` is no longer available in the container.

# 2.1.2

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.1.1

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.1.0

This version introduces a new setting, `--threads`/`THREADS` that allows limiting the number of worker threads and the size of the database connection pool. If this flag is not set, the number of available CPU cores will be used, which matches the current behavior.

# 2.0.3

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.0.2

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.0.1

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.0.0

The first public release. It's named `2.0.0`, because I have been using this service for two years internally and made some breaking changes while I was preparing this project for the public.
