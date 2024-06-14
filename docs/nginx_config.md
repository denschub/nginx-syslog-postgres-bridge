# nginx configuration

As for the log format, a specific and relatively compact JSON format is required. The following config can be made available globally in the `http {}` block:

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

To send access log entries, set the following, either globally in `http {}` or for a specific `server {}` block:

```
access_log syslog:server=nginx-syslog-bridge.example.com:514,nohostname postgres_bridge_json;
```
