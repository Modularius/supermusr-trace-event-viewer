//!
//!
use clap::{Args, Parser, Subcommand, ValueEnum};
use engine::Engine;
use supermusr_common::{
    init_tracer, tracer::{TracerEngine, TracerOptions}, CommonKafkaOpts, Intensity, Time
};
use cache::Cache;
use rdkafka::{
    TopicPartitionList,
    consumer::{CommitMode, BaseConsumer, Consumer},
    error::KafkaError,
    util::Timeout,
    Offset
};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tracing::{info,warn};

mod data;
mod output_to_file;
mod build_graph;
mod cache;
mod engine;
mod message_handling;

#[derive(Clone, Debug, Args)]
struct Topics {
    /// Kafka trace topic.
    #[clap(long)]
    trace_topic: String,

    /// Kafka digitiser event list topic.
    #[clap(long)]
    digitiser_event_topic: String,
}

#[derive(Clone, Debug, Args)]
struct UserBounds {
    /// Minimum time bin to graph, derived from input if left unspecified.
    #[clap(long)]
    time_min: Option<Time>,

    /// Maximum time bin to graph, derived from input if left unspecified.
    #[clap(long)]
    time_max: Option<Time>,

    /// Minimum intensity value to graph, derived from input if left unspecified.
    #[clap(long)]
    intensity_min: Option<Intensity>,

    /// Maximum intensity value to graph, derived from input if left unspecified.
    #[clap(long)]
    intensity_max: Option<Intensity>,
}

/// [clap] derived stuct to parse command line arguments.
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group.
    #[clap(long)]
    consumer_group: String,

    #[clap(flatten)]
    topics: Topics,

    #[clap(flatten)]
    bounds: UserBounds,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used.
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
    #[clap(long, default_value = "")]
    otel_namespace: String,

    /// Endpoint on which OpenMetrics flavour metrics are available.
    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// Which data to collect.
    #[clap(long)]
    collect: CollectMode,

    /// How much data to collect.
    #[clap(long)]
    num: usize,

    /// Subcommand to execute.
    #[command(subcommand)]
    mode: OutputMode,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CollectMode {
    /// Collects the given number of traces.
    Traces,
    /// Collects the given number of event lists.
    Events,
    /// Collects the given number of traces and their corresponding eventlists.
    All,
}

#[derive(Clone, Subcommand)]
enum OutputMode {
    /// Outputs image to file.
    File(OutputToFile),
}

#[derive(Clone, Parser)]
struct OutputToFile {
    #[clap(long)]
    path: PathBuf,
}

pub fn create_default_consumer(
    broker_address: &String,
    username: &Option<String>,
    password: &Option<String>,
    consumer_group: &String,
    topics_to_subscribe: Option<&[&str]>,
) -> Result<BaseConsumer, KafkaError> {
    // Setup consumer with arguments and default parameters.
    let consumer: BaseConsumer = supermusr_common::generate_kafka_client_config(broker_address, username, password)
        .set("group.id", consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    // Subscribe to if topics are provided.
    if let Some(topics_to_subscribe) = topics_to_subscribe {
        // Note this fails if the topics list is empty
        consumer.subscribe(topics_to_subscribe)?;
    }

    Ok(consumer)
}


/// Entry point.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let _tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace.clone()
    ));

    let consumer = create_default_consumer(
        &args.common_kafka_options.broker,
        &args.common_kafka_options.username,
        &args.common_kafka_options.password,
        &args.consumer_group,
        None,
    )?;

    let mut tpl = TopicPartitionList::new();
    tpl.add_partition_offset(args.topics.trace_topic.as_str(), 0, Offset::OffsetTail(args.num.try_into()?))?;
    tpl.add_partition_offset(args.topics.digitiser_event_topic.as_str(), 0, Offset::OffsetTail(args.num.try_into()?))?;
    consumer.assign(&tpl)?;

    let timeout = Timeout::After(Duration::from_millis(100));

    let mut engine = Engine::new(args.collect, args.topics, args.mode);
    
    info!("Starting Loop");

    while engine.get_count() < args.num {
        match consumer.poll(None) {
            Some(Ok(m)) => {
                info!("New Message");
                engine.process_message(&m);
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Some(Err(e)) => warn!("Kafka error: {}", e),
            None => warn!("No message"),
        }
    }

    engine.output(&args.bounds);
    Ok(())
}
