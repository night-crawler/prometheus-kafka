```shell
cargo run --bin prometheus-kafka-server -- --log-conf prometheus_kafka=trace 
```

## Import a sample

```shell
grpcurl \
  -plaintext \
  -import-path ./proto \
  -proto transport.proto \
  -d @ \
  localhost:50051 \
  prometheus.PrometheusReader/Receive < sample.json
```

## Benchmark

```shell
ghz \
  --skipTLS \
  --insecure \
  --cpus=16 \
  --connections=100 \
  --concurrency=2500 \
  --keepalive=12s \
  --duration=10s \
  --import-paths ./proto \
  --proto transport.proto \
  --data-file ./sample.json \
  --call prometheus.PrometheusReader/Receive \
  0.0.0.0:50051
```
