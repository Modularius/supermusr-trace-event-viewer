//!
//!
use build_graph::{BackendSVG, BuildGraph};
use cache::Cache;
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use cli_structs::{CollectType, Select, Mode, Topics, UserBounds};
use data::Bounds;
use finder::{FindEngine, Finder};
use message::{FBMessage, UnpackMessage};
//use engine::Engine;
//use finder::Finder;
use supermusr_common::{
    init_tracer, tracer::{TracerEngine, TracerOptions}, Channel, CommonKafkaOpts, DigitizerId
};
//use cache::Cache;
use rdkafka::{
    consumer::{BaseConsumer, CommitMode, Consumer}, error::KafkaError, message::BorrowedMessage
};
use supermusr_streaming_types::dev2_digitizer_event_v2_generated::digitizer_event_list_message_buffer_has_identifier;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tracing::{info,warn};

use crate::{data::EventList, message::{EventListMessage, TraceMessage}};

mod cli_structs;
mod data;
mod output;
mod build_graph;
mod cache;
//mod engine;
mod message_handling;
mod finder;
mod message;

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

    #[clap(flatten)]
    select: Select,

    /// Which data to collect.
    #[clap(long)]
    collect: CollectType,

    /// Subcommand to execute.
    #[command(subcommand)]
    mode: Mode,
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

    let mut cache = Cache::default();

    let trace_finder = Finder::<'_,TraceMessage>::new(&args.topics.trace_topic);
    let eventlist_finder = Finder::<'_,EventListMessage>::new(&args.topics.digitiser_event_topic);
    let mut find_engine = FindEngine::new(&consumer, &args.select.step);
    let trace = find_engine.find(&trace_finder, 1, args.select.timestamp, |x|x.has_channel(args.select.channel));
    let digitiser_id = trace.as_ref().expect("").digitiser_id();
    let eventlist = find_engine.find(&eventlist_finder, 1, args.select.timestamp, |evlist|evlist.digitiser_id() == digitiser_id);

    if let Some((trace,eventlist)) = Option::zip(trace,eventlist) {
        cache.push_trace(&trace.get_unpacked_message().expect(""));
        cache.push_event_list_to_trace(&eventlist.get_unpacked_message().expect(""));
        match args.mode {
            Mode::File(output_to_file) => {
                info!("Outputting {} Digitiser Traces", cache.iter_traces().len());
                for (metadata, traces) in cache.iter_traces() {
                    info!("Outputting Frame {:?} Traces", metadata);
                    info!("Outputting {} Traces", traces.traces.len());
                    for (channel, trace) in &traces.traces {
                        info!("Outputting Channel {channel}");
                        let mut bounds = Bounds::from_trace(&trace).expect("");
                        bounds.ammend_with_user_input(&args.bounds);
                        let graph = BuildGraph::<BackendSVG<'_>>::new(800,600,bounds.time_range(), bounds.intensity_range());

                        let path_buf = graph.build_path(&output_to_file.path, metadata, *channel).expect("extension should write");
                        let eventlist = traces.events.as_ref().and_then(|ev|ev.get(channel));
                        graph.save_trace_graph(&path_buf, &trace, eventlist).expect("");
                    }
                }
            },
        }
    }
    Ok(())
}
