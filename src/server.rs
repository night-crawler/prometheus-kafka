use tonic::{Request, Response, Status, transport::Server};

use prometheus_kafka::WriteRequest;
use prometheus_kafka::prometheus_reader_server::{PrometheusReader, PrometheusReaderServer};

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyPrometheusReader;

#[tonic::async_trait]
impl PrometheusReader for MyPrometheusReader {
    async fn receive(
        &self,
        request: tonic::Request<WriteRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("{:?}", request);
        Ok(tonic::Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let reader = MyPrometheusReader::default();

    Server::builder()
        .add_service(PrometheusReaderServer::new(reader))
        .serve(addr)
        .await?;

    Ok(())
}