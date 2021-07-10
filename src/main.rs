pub mod utils {
    pub mod logging;
    pub mod argparse;
}

use std::time::Duration;
use log::{info, trace};
use clap::{App, Arg};
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use tonic::{Request, Response, Status, transport::Server};

use prometheus_kafka::prometheus_reader_server::{PrometheusReader, PrometheusReaderServer};
use prometheus_kafka::WriteRequest;
use rdkafka::message::OwnedMessage;
use rdkafka::error::KafkaError;

use crate::utils::logging::setup_logger;
use crate::utils::argparse::prepare_server_arg_matches;

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}

pub struct MyPrometheusReader {
    kafka_producer: KafkaProducer,
}

pub struct KafkaProducer {
    topics: Vec<String>,
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(config: ClientConfig, topics: Vec<&str>) -> Self {
        let producer = config.create().expect("Failed to create a producer");
        Self { producer, topics: topics.into_iter().map(|topic| topic.to_string()).collect() }
    }

    pub async fn store(&self, write_request: WriteRequest) -> OwnedDeliveryResult {
        let record = FutureRecord::to("aaa")
            .key("aaa")
            .partition(-1)
            .payload("www");
        self.producer.send(record, Duration::from_secs(0)).await
    }
}

impl MyPrometheusReader {
    fn new(producer: KafkaProducer) -> Self {
        Self { kafka_producer: producer }
    }
}

#[tonic::async_trait]
impl PrometheusReader for MyPrometheusReader {
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
                trace!("Error occurred: {}", kafka_error);
            }
        };

        Ok(Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = prepare_server_arg_matches();
    setup_logger(true, matches.value_of("log-conf"));

    let mut kafka_config = ClientConfig::new();
    kafka_config
        .set("bootstrap.servers", matches.value_of("brokers").expect("Specify brokers"))
        .set("queue.buffering.max.ms", "0");

    let addr = matches.value_of("listen").expect("Specify listen address").parse()?;
    let topics: Vec<_> = matches.values_of("topics").unwrap().collect();
    let kafka_producer = KafkaProducer::new(kafka_config, topics);
    let reader = MyPrometheusReader::new(kafka_producer);

    Server::builder()
        .add_service(PrometheusReaderServer::new(reader))
        .serve(addr)
        .await?;

    Ok(())
}