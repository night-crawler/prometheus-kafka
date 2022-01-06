use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use log::info;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct KafkaStorage {
    topic: String,
    producer: FutureProducer,
    num_written: AtomicUsize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrometheusKafkaMessage {
    labels: HashMap<String, String>,
    name: String,
    value: String,
    timestamp: String,
}

impl PrometheusKafkaMessage {
    pub fn new(name: &str, value: &str, timestamp: &str, labels: &HashMap<&str, &str>) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            timestamp: timestamp.to_string(),
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }
}

impl KafkaStorage {
    pub fn new(config: ClientConfig, topic: &str) -> Self {
        let producer = config.create().expect("Failed to create a producer");
        Self {
            producer,
            topic: topic.to_string(),
            num_written: AtomicUsize::default(),
        }
    }

    pub async fn store(&self, message: &PrometheusKafkaMessage) -> OwnedDeliveryResult {
        let uuid = Uuid::new_v4().to_string();
        let payload = serde_json::to_string(message).expect("Could not serialize a message");
        let record = FutureRecord::to(self.topic.as_str())
            .key(uuid.as_str())
            .partition(-1)
            .payload(payload.as_str());

        let result = self.producer.send(record, Duration::from_secs(0)).await;
        let num = self.num_written.fetch_add(1, Ordering::Relaxed);

        if num % 10000 == 0 {
            info!("Written {} messages", num);
        }

        result
    }
}
