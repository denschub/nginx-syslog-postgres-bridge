CREATE TABLE access_log (
  id UUID NOT NULL,
  event_ts TIMESTAMP WITH TIME ZONE NOT NULL,
  PRIMARY KEY (id, event_ts),

  hostname TEXT NOT NULL,

  server_name TEXT,
  server_port INTEGER,

  client_addr TEXT,
  client_forwarded_for TEXT,
  client_referer TEXT,
  client_ua TEXT,

  req_host TEXT,
  req_length BIGINT,
  req_method TEXT,
  req_proto TEXT,
  req_scheme TEXT,
  req_uri TEXT,

  res_body_length BIGINT,
  res_duration FLOAT,
  res_length BIGINT,
  res_status INTEGER,

  upstream_addr TEXT,
  upstream_bytes_received BIGINT,
  upstream_bytes_sent BIGINT,
  upstream_cache_status TEXT,
  upstream_connect_time FLOAT,
  upstream_host TEXT,
  upstream_response_length BIGINT,
  upstream_response_time FLOAT,
  upstream_status INTEGER
);

CREATE INDEX access_log_hostname_idx ON access_log(hostname);

SELECT create_hypertable('access_log', 'event_ts');
SELECT add_retention_policy('access_log', INTERVAL '365 days');
