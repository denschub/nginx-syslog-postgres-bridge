# 3.0.0 (unreleased)

This version moves away from individual `INSERT` statements for each individual request. Instead, database insertions are batched via large `INSERT INTO ... SELECT * FROM UNNEST` queries. This significantly increases the throughput, while also reducing database server load.

- A new setting, `--insert-batch-size`/`INSERT_BATCH_SIZE`, is available to set the size of batch insertions.

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
