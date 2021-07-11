use log::info;
use rdkafka::ClientConfig;
use tonic::transport::Server;

use crate::grpc::reader::GrpcPrometheusReader;
use crate::grpc::reader::prometheus_kafka::prometheus_reader_server::PrometheusReaderServer;
use crate::kafka::writer::KafkaProducer;
use crate::utils::argparse::prepare_server_arg_matches;
use crate::utils::logging::setup_logger;

pub mod grpc {
    pub mod reader;
}

pub mod kafka {
    pub mod writer;
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

    let topics: Vec<_> = matches.values_of("topics").unwrap().collect();
    info!("Relying incoming requests to kafka topics: {:?}", topics);

    let kafka_producer = KafkaProducer::new(kafka_config, topics);
    let reader = GrpcPrometheusReader::new(kafka_producer);

    Server::builder()
        .add_service(PrometheusReaderServer::new(reader))
        .serve(addr)
        .await?;

    Ok(())
}