DO $$ BEGIN
  IF EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
    CREATE EXTENSION IF NOT EXISTS timescaledb;
    PERFORM create_hypertable('access_log', 'event_ts');
    PERFORM add_retention_policy('access_log', INTERVAL '365 days');
  END IF;
END $$;
