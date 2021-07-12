use log::info;
use rdkafka::ClientConfig;
use tonic::transport::Server;

use crate::grpc::reader::GrpcPrometheusReader;
use crate::grpc::reader::prometheus_kafka::prometheus_reader_server::PrometheusReaderServer;
use crate::kafka::storage::KafkaStorage;
use crate::utils::argparse::prepare_server_arg_matches;
use crate::utils::logging::setup_logger;

pub mod grpc {
    pub mod reader;
    pub mod convert;
}

pub mod kafka {
    pub mod storage;
}

pub mod utils {
    pub mod logging;
    pub mod argparse;
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
    info!("Listening {}", addr);

    let topic = matches.value_of("topic").expect("Specify kafka topic");
    info!("Relying incoming requests to kafka topic: {:?}", topic);

    let kafka_producer = KafkaStorage::new(kafka_config, topic);
    let reader = GrpcPrometheusReader::new(kafka_producer);

    Server::builder()
        .add_service(PrometheusReaderServer::new(reader))
        .serve(addr)
        .await?;

    Ok(())
}