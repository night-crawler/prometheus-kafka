## Run debug (dry-run)

```shell
cargo run --bin prometheus-kafka-server -- \
  --worker-threads=16 \
  --batch-size=1000 \
  --batch-linger-ms=100 \
  --log-conf prometheus_kafka=info \
  --dry-run=true
```

## Run release (dry-run)

```shell
cargo run --release --bin prometheus-kafka-server -- \
  --worker-threads=16 \
  --batch-size=1000 \
  --batch-linger-ms=100 \
  --log-conf prometheus_kafka=info \
  --dry-run=true
```

## Flame graph

```bash
cargo flamegraph --min-width 0.001 --bin prometheus-kafka-server -- \
  --worker-threads=16 \
  --batch-size=1000 \
  --batch-linger-ms=100 \
  --log-conf prometheus_kafka=info \
  --dry-run=true

```

## Import a sample

```shell
grpcurl \
  -plaintext \
  -import-path ./proto \
  -proto transport.proto \
  -d @ \
  localhost:50051 \
  prometheus.PrometheusReader/Receive < medium.json
```

## Benchmark

```shell
ghz \
  --skipTLS \
  --insecure \
  --cpus=16 \
  --connections=16 \
  --concurrency=1000 \
  --keepalive=12s \
  --duration=10s \
  --import-paths=./proto \
  --proto=transport.proto \
  --data-file=./payload/medium.json \
  --call=prometheus.PrometheusReader/Receive \
  0.0.0.0:50051
```
