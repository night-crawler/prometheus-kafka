use std::time::Duration;

use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use crate::grpc::reader::prometheus_kafka::WriteRequest;

pub struct KafkaStorage {
    topics: Vec<String>,
    producer: FutureProducer,
}

impl KafkaStorage {
    pub fn new(config: ClientConfig, topics: Vec<&str>) -> Self {
        let producer = config.create().expect("Failed to create a producer");
        Self { producer, topics: topics.into_iter().map(|topic| topic.to_string()).collect() }
    }

    pub async fn store(&self, _write_request: WriteRequest) -> OwnedDeliveryResult {
        let record = FutureRecord::to("aaa")
            .key("aaa")
            .partition(-1)
            .payload("www");
        self.producer.send(record, Duration::from_secs(0)).await
    }
}
