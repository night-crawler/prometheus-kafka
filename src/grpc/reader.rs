use hex;
use log::{error, trace};
use rdkafka::Message;
use tonic::{Request, Response, Status};

use prometheus_kafka::prometheus_reader_server::PrometheusReader;
use prometheus_kafka::WriteRequest;

use crate::grpc::convert::Convert;
use crate::KafkaStorage;

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}

pub struct GrpcPrometheusReader {
    storage: KafkaStorage,
}

impl GrpcPrometheusReader {
    pub fn new(storage: KafkaStorage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl PrometheusReader for GrpcPrometheusReader {
    async fn receive(&self, request: Request<WriteRequest>) -> Result<Response<()>, Status> {
        let write_request: WriteRequest = request.into_inner();
        let messages = write_request.convert();
        let futures = messages
            .iter()
            .map(|message| self.storage.store(message))
            .collect::<Vec<_>>();

        for future in futures {
            match future.await {
                Ok((partition, offset)) => {
                    trace!("Committed offset {} for partition {}", offset, partition);
                }
                Err((kafka_error, owned_message)) => {
                    let message_payload_repr = match owned_message.payload() {
                        Some(payload) => {
                            String::from_utf8(payload.to_vec()).unwrap_or(hex::encode(payload))
                        }
                        None => {
                            "no payload".to_string()
                        }
                    };

                    let error_message = format!("Error occurred {}; failed message: {}", kafka_error, message_payload_repr);
                    error!("{}", error_message);
                    return Err(Status::internal(error_message));
                }
            }
        }

        Ok(Response::new(()))
    }
}
