//!
//!
use clap::{Args, Parser, Subcommand};
use data::Cache;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::BorrowedMessage,
    Message,
};

use std::net::SocketAddr;
use supermusr_common::{
    init_tracer,
    tracer::{TracerEngine, TracerOptions},
    CommonKafkaOpts,
};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage,
    },
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
        DigitizerEventListMessage,
    },
};
use tracing::warn;

mod data;
mod message_handling;

/// [clap] derived stuct to parse command line arguments.
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group.
    #[clap(long)]
    consumer_group: String,

    /// Kafka trace topic.
    #[clap(long)]
    trace_topic: String,

    /// Kafka digitiser event list topic.
    #[clap(long)]
    digitiser_event_topic: String,

    /// Kafka frame event list topic.
    #[clap(long)]
    frame_event_topic: String,

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

#[derive(Clone, Subcommand)]
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
    path: String,
}

/// Entry point.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let _tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace
    ));

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        Some(&[
            args.trace_topic.as_str(),
            args.digitiser_event_topic.as_str(),
        ]),
    )?;

    let mut cache = Cache::default();

    while args.collect.get_count() < args.num {
        match consumer.recv().await {
            Ok(m) => {
                args.collect.process_message(&mut cache, m);
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        }
    }
    Ok(())
}
