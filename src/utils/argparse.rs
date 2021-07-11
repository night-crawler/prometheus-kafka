use clap::{App, Arg, ArgMatches};

pub fn prepare_server_arg_matches() -> ArgMatches {
    App::new("prometheus-kafka")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("prometheus-kafka")
        .arg(
            Arg::new("listen")
                .short('l')
                .long("listen")
                .about("Listen gRPC address and port")
                .takes_value(true)
                .required(true)
                .default_value("[::1]:50051"),
        )
        .arg(
            Arg::new("brokers")
                .short('b')
                .long("brokers")
                .about("Broker list in kafka format")
                .takes_value(true)
                .required(true)
                .default_value("localhost:9092"),
        )
        .arg(
            Arg::new("log-conf")
                .long("log-conf")
                .about("Configure the logging format (example: 'rdkafka=trace')")
                .takes_value(true),
        )
        .arg(
            Arg::new("topic")
                .long("topic")
                .about("Topic name")
                .takes_value(true)
                .default_value("out")
                .required(true),
        )
        .get_matches()
}
