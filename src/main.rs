use log::info;
use rdkafka::ClientConfig;
use structopt::StructOpt;
use tonic::transport::Server;

use crate::grpc::reader::GrpcPrometheusReader;
use crate::grpc::reader::prometheus_kafka::prometheus_reader_server::PrometheusReaderServer;
use crate::kafka::storage::KafkaStorage;
use crate::utils::argparse::AppOptions;
use crate::utils::logging::setup_logger;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = AppOptions::from_args();
    setup_logger(true, opts.log_conf.as_ref().map(|s| s.as_str()));

    info!("Options: {:#?}", opts);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(opts.worker_threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mut kafka_config = ClientConfig::new();

            kafka_config
                .set("bootstrap.servers", opts.brokers)
                .set("batch.num.messages", opts.batch_size.to_string())
                .set("linger.ms", opts.batch_linger_ms.to_string());

            let kafka_producer = KafkaStorage::new(kafka_config, &opts.topic);
            let reader = GrpcPrometheusReader::new(kafka_producer);

            Server::builder()
                // .http2_keepalive_interval(Some(core::time::Duration::from_millis(20000)))
                // .concurrency_limit_per_connection(1024)
                // .max_concurrent_streams(Some(64))
                .add_service(PrometheusReaderServer::new(reader))
                .serve(opts.listen)
                .await
        })?;

    Ok(())
}