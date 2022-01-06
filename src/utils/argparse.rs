use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "prometheus-kafka")]
pub struct AppOptions {
    /// Tokio runtime worker threads
    #[structopt(short, long, default_value = "1")]
    pub worker_threads: usize,

    /// Listen gRPC address and port
    #[structopt(short, long, default_value = "0.0.0.0:50051")]
    pub listen: SocketAddr,

    /// Broker list in kafka format
    #[structopt(short, long, default_value = "localhost:9092")]
    pub brokers: String,

    /// Output topic name
    #[structopt(short, long, default_value = "out")]
    pub topic: String,

    /// Queue buffering time
    #[structopt(long, default_value = "2000")]
    pub batch_linger_ms: usize,

    /// Queue batch size
    #[structopt(long, default_value = "500")]
    pub batch_size: usize,

    /// Configure the logging format (example: 'rdkafka=trace')
    #[structopt(long)]
    pub log_conf: Option<String>,
}
