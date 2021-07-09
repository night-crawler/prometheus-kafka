use tonic::{transport::Server, Request, Response, Status};

pub mod prometheus_kafka {
    tonic::include_proto!("prometheus"); // The string specified here must match the proto package name
}
