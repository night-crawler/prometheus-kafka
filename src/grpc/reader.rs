use log::{error, trace};
use tonic::{Request, Response, Status};

use prometheus_kafka::prometheus_reader_server::PrometheusReader;
use prometheus_kafka::WriteRequest;

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
    async fn receive(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<()>, Status> {
        let write_request: WriteRequest = request.into_inner();
        match self.storage.store(write_request).await {
            Ok((partition, offset)) => {
                trace!("Committed offset {} for partition {}", offset, partition);
            }
            Err((kafka_error, _message)) => {
                error!("Error occurred: {}", kafka_error);
            }
        };

        Ok(Response::new(()))
    }
}