use log::{error, trace};
use tonic::{Request, Response, Status};

use prometheus_kafka::prometheus_reader_server::PrometheusReader;
use prometheus_kafka::WriteRequest;

use crate::KafkaProducer;

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}

pub struct GrpcPrometheusReader {
    kafka_producer: KafkaProducer,
}


impl GrpcPrometheusReader {
    pub fn new(producer: KafkaProducer) -> Self {
        Self { kafka_producer: producer }
    }
}

#[tonic::async_trait]
impl PrometheusReader for GrpcPrometheusReader {
    async fn receive(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<()>, Status> {
        let write_request: WriteRequest = request.into_inner();
        match self.kafka_producer.store(write_request).await {
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