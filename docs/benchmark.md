# Benchmarking the Bridge for Fun and Profit

This document shouldn't be taken as a source of "what you can expect if you run this in production", but more a general showcase in _one_ possible scenario.

Benchmarks were run on 2025-12-14. I used two of the smallest available Hetzner Cloud VMs with dedicated CPU cores. These were called CCX13, had two dedicated CPU cores, 8 GiB of RAM, and 80 GiB of local NVMe storage. According to `/proc/cpuinfo`, these VMs ran on an AMD EPYC cpu with 2.4 GHz, but I don't know the exact cores.

The first machine ran nginx version 1.29.4, and also ran the benchmarks based on `wrk` 4.1.0. The second machine ran PostgreSQL 18.1, and also ran v3.0.0 of the bridge. I decided to have the bridge and database on the same machine to avoid testing server bandwidth. However, both servers were connected using Hetzner's "private network" feature, which held 6 Gbit/s consistently. PostgreSQL, nginx, and the bridge all ran in Docker containers for ease of testing. The CPU and memory provided are from `docker stats`.

To give nginx an as-high-as-possible throughput, I used a simple server that just returned a static string:

```plain
server {
  listen 80 default_server reuseport;
  listen [::]:80 default_server reuseport;
  server_name _;

  add_header Content-Type text/plain;
  return 200 "meow";
}
```

## Default settings

This test ran with the bridge's default settings and no custom queue sizes, so `INSERT_BATCH_SIZE=10` and `QUEUE_SIZE=50`. I used a `wrk` lua script to throttle requests, let it run for 60 seconds, and checked the peak values for the containers;

| Throughput | Bridge CPU peak | Bridge RAM peak | DB CPU peak | DB RAM peak |
| ---------- | --------------- | --------------- | ----------- | ----------- |
| 10 req/s   | 0.39%           | 840 KiB         | 1.54%       | 168.4 MiB   |
| 100 req/s  | 1.22%           | 860 KiB         | 1.32%       | 168.4 MiB   |
| 1000 req/s | 9.55%           | 1.14 MiB        | 11.93%      | 171.4 MiB   |
| 5400 req/s | 49.77 %         | 1.16 MiB        | 64.73 %     | 194.6 MiB   |

Pushing it higher than 5.5k req/s resulted in requests that weren't logged anymore. That was not a system resource limitation per se, but the insert batches of 10 rows didn't complete fast enough, so the `QUEUE_SIZE` filled up and dropped input packets.

## Pushing the limits

If I let `wrk` run as fast as it can, it fired around 47k req/s to nginx. I ran a 60 seconds test, in which wrk fired 2.80 million requests. Only 454k ended up in the database so only 16.1% of request got logged. The bridge peaked at 80.8% CPU and 1.53 MiB RAM. The DB peaked at 58.77% CPU and 188.5 MiB RAM.

To increase throughput, I set `INSERT_BATCH_SIZE=2000` and `QUEUE_SIZE=4000`. In general, having `QUEUE_SIZE` a two-times your `INSERT_BATCH_SIZE` is a good idea if you're optimizing for constant load - there really isn't a point in keeping more than two batches in your input queue - if you can't write to the DB fast enough, there's no point in keeping much more than that around.

With those settings, I ran another 60 second `wrk` test, and fired 2.87 million requests. Out of those, 2.52 made it into the databasse, or 87.1% of everything. The bridge peaked at 97.34% CPU and 6.77 MiB RAM. The database reached 67.03 % CPU and 217.9 MiB RAM. These results show that the bottleneck in v3.0.0 of the bridge is the parsing-side of things, and not directly the database layer.

Based on the 87% storage rate, one could estimate that a 40k req/s is a handle'able load. I couldn't get a `wrk` delay script to be precise enough to throttle to that, and I also couldn't get other benchmark tools to work - `autocannon`, for example, was always too slow. So I couldn't 100% verify the 40k req/s throughput, but since this heavily depends on your individual CPU anyway, there isn't too much point in capturing precise numbers.

This document should, at least, demonstrate that this bridge is easily to handle tens of thousands of requests with a really small resource footprint, and I hope you have an idea how to benchmark it on your own infrastructure if needed.
