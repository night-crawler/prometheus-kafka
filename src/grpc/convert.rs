use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Add, Deref};
use std::time::SystemTime;
use std::vec;

use chrono::{DateTime, Utc};

use crate::grpc::reader::prometheus_kafka::Label;
use crate::grpc::reader::prometheus_kafka::WriteRequest;
use crate::kafka::storage::Message;

const DEFAULT_NAME: &str = "__UNKNOWN_NAME__";

pub trait Convert {
    fn convert(&self) -> Vec<Message>;
}

impl Convert for WriteRequest {
    fn convert(&self) -> Vec<Message> {
        let mut messages: Vec<Message> = vec![];

        for ts in self.timeseries.iter() {
            let labels_map: HashMap<&str, &str> = labels_to_map(&ts.labels);
            let name = labels_map.get("__name__").unwrap_or(&DEFAULT_NAME).deref();

            for sample in ts.samples.iter() {
                let value = sample.value.to_string();
                // timestamp is in ms format, see pkg/timestamp/timestamp.go
                let timestamp = ms_to_utc_rfc3339(sample.timestamp);
                messages.push(Message::new(name, &value, &timestamp, &labels_map));
            }

            for exemplar in ts.exemplars.iter() {
                let additional_labels = labels_to_map(&exemplar.labels);
                let labels_map = merge(&labels_map, &additional_labels);
                let value = exemplar.value.to_string();
                let timestamp = ms_to_utc_rfc3339(exemplar.timestamp);
                messages.push(Message::new(name, &value, &timestamp, &labels_map));
            }
        }

        messages
    }
}

fn merge<K, V>(map1: &HashMap<K, V>, map2: &HashMap<K, V>) -> HashMap<K, V> where K: Eq + Hash + Copy, V: Copy {
    map1.iter()
        .chain(map2.iter())
        .map(|(&k, &v)| (k, v))
        .collect()
}

fn labels_to_map(labels: &[Label]) -> HashMap<&str, &str> {
    labels.iter()
        .map(|label| (label.name.as_str(), label.value.as_str()))
        .collect()
}

fn ms_to_utc_rfc3339(ms: i64) -> String {
    let dt: DateTime<Utc> = SystemTime::UNIX_EPOCH
        .add(core::time::Duration::from_millis(ms as u64))
        .into();
    dt.to_rfc3339()
}