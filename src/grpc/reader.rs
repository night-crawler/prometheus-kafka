use std::sync::Arc;

use futures::future::join_all;
use hex;
use log::{error, trace};
use rdkafka::error::KafkaError;
use rdkafka::Message;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use tokio::task::JoinError;
use tonic::{Request, Response, Status};

use prometheus_kafka::prometheus_reader_server::PrometheusReader;
use prometheus_kafka::WriteRequest;

use crate::grpc::convert::Convert;
use crate::kafka::storage::PrometheusKafkaMessage;
use crate::KafkaStorage;

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}

pub struct GrpcPrometheusReader {
    storage: Arc<KafkaStorage>,
    dry_run: bool,
}

impl GrpcPrometheusReader {
    pub fn new(storage: KafkaStorage, dry_run: bool) -> Self {
        Self { storage: Arc::from(storage), dry_run }
    }
}

enum ProcessResult {
    Success {
        partition: i32,
        offset: i64,
    },
    KafkaError {
        kafka_error: KafkaError,
        owned_message: OwnedMessage,
    },
    JoinError {
        join_error: JoinError
    },
}

fn process_delivery_result(delivery_result: OwnedDeliveryResult) -> ProcessResult {
    match delivery_result {
        Ok((partition, offset)) => ProcessResult::Success { partition, offset },
        Err((kafka_error, owned_message)) => ProcessResult::KafkaError { kafka_error, owned_message }
    }
}

async fn store(storage: Arc<KafkaStorage>, message: PrometheusKafkaMessage) -> OwnedDeliveryResult {
    storage.store(&message).await
}

#[tonic::async_trait]
impl PrometheusReader for GrpcPrometheusReader {
    async fn receive(&self, request: Request<WriteRequest>) -> Result<Response<()>, Status> {
        if self.dry_run {
            return Ok(Response::new(()));
        }

        let write_request: WriteRequest = request.into_inner();
        let messages = write_request.convert();

        let spawned_iter = messages.into_iter()
            .map(|message| store(self.storage.clone(), message))
            .map(|fut| tokio::spawn(fut));

        let spawn_results = join_all(spawned_iter).await;

        let process_results = spawn_results.into_iter()
            .map(|spawn_result| {
                match spawn_result {
                    Ok(delivery_result) => process_delivery_result(delivery_result),
                    Err(join_error) => ProcessResult::JoinError { join_error }
                }
            });

        let mut errors = vec![];

        for process_result in process_results {
            match process_result {
                ProcessResult::Success { offset, partition } => {
                    trace!("Committed offset: {}, partition: {}", offset, partition);
                }
                ProcessResult::KafkaError { kafka_error, owned_message } => {
                    let message_payload_repr = match owned_message.payload() {
                        Some(payload) => {
                            String::from_utf8(payload.to_vec()).unwrap_or(hex::encode(payload))
                        }
                        None => "no payload".to_string()
                    };

                    let error_repr = format!("Kafka error: {}; failed message: {}", kafka_error, message_payload_repr);
                    error!("{}", error_repr);
                    errors.push(error_repr);
                }
                ProcessResult::JoinError { join_error } => {
                    let error_repr = format!("Join error: {}", join_error);
                    error!("{}", error_repr);
                    errors.push(error_repr)
                }
            }
        }

        return if errors.is_empty() {
            Ok(Response::new(()))
        } else {
            Err(Status::data_loss(errors.join(";\n")))
        };
    }
}
