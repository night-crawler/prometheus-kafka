```shell
cargo run --bin prometheus-kafka-server 
```

```shell
grpcurl \
  -plaintext \
  -import-path ./proto \
  -proto transport.proto \
  -d @ \
  localhost:50051 \
  prometheus.PrometheusReader/Receive < sample.json
```
