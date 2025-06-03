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
    let find_engine = FindEngine::new(&consumer, &args.select.step);
    let traces = find_engine.find(&trace_finder, 1, args.select.timestamp, |x|x.has_channel(args.select.channel));
    let dig_id = traces.unwrap().d;
    let eventlists = find_engine.find(&eventlist_finder, 1, args.select.timestamp, |x|x.has_channel(args.select.channel));
    //let timestamp = "2025-05-31 17:38:00.0 UTC".parse()?;
    let tf = TraceFinderByKafkaTimestamp::new(&consumer, &args.topics.trace_topic);
    let trace_finder = FindByDate::new(tf, &args.select.step);
    let did = trace_finder.find(&args.topics, args.select.timestamp, args.select.channel);

    let (idx, tmstp) = trace_finder.find_first_index_with_timestamp_before(args.select.timestamp)?;
    let first_message = get_next_message_with_timestamp_after(&consumer, &args.topics, &args.select.timestamp);
    if let Some(first_message) = first_message {
        let dig_msg = first_message.unpack_message(&args.topics).unwrap();
        let timestamp: DateTime<Utc> = dig_msg.timestamp().unwrap();
        info!("Seeking Timestamp {timestamp}");
        let mut dig_id = None;
        if let Some(did) = push_trace(&mut cache, args.select.channel, dig_msg) {
            dig_id = Some(did);
        } else {
            for msg in consumer.iter() {
                match msg {
                    Ok(message) => 
                        if let Some(dig_msg) = message.unpack_message(&args.topics) {
                            if dig_msg.is_timestamp_equal_to(timestamp) {
                                    if let Some(did) = push_trace(&mut cache, args.select.channel, dig_msg) {
                                        dig_id = Some(did);
                                        break;
                                    }
                            }
                        },
                    Err(_) => {},
                }
            }
        }

        if let Some(dig_id) = dig_id {
            let ef = TraceFinderByKafkaTimestamp::new(&consumer, &args.topics.digitiser_event_topic);
            let event_finder = FindByDate::new(ef, &args.select.step);

            event_finder.find_first_index_with_timestamp_before(args.select.timestamp)?;
            let first_message = get_next_message_with_timestamp_after(&consumer, &args.topics, &args.select.timestamp);
            if let Some(first_message) = first_message {
                let dig_msg = first_message.unpack_message(&args.topics).unwrap();
                let timestamp: DateTime<Utc> = dig_msg.timestamp().unwrap();
                info!("Seeking Timestamp {timestamp}");
                if push_event_list_if_valid(&mut cache,dig_id,dig_msg).is_none() {
                    for msg in consumer.iter() {
                        match msg {
                            Ok(message) => 
                                if let Some(dig_msg) = message.unpack_message(&args.topics) {
                                    if dig_msg.is_timestamp_equal_to(timestamp) {
                                        if push_event_list_if_valid(&mut cache,dig_id,dig_msg).is_some() {
                                            break;
                                        }
                                    }
                                },
                            Err(_) => {},
                        }
                    }
                }
            }
        }
    }


    
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
    Ok(())
}


fn push_event_list_if_valid(cache: &mut Cache, dig_id: DigitizerId, dig_msg: DigitizerMessage) -> Option<()> {
    if let DigitizerMessage::EventList(dig_eventlist_msg) = dig_msg {
        info!("Found EventList");
        if dig_eventlist_msg.digitizer_id() == dig_id {
            info!("Push EventList");
            cache.push_event_list_to_trace(&dig_eventlist_msg);
            Some(())
        } else {
            None
        }
    } else {
        unreachable!()
    }
}

fn get_next_message_with_timestamp_after<'a>(consumer: &'a BaseConsumer, topics: &Topics, timestamp: &DateTime<Utc>) -> Option<BorrowedMessage<'a>> {
    for msg in consumer.iter() {
        match msg {
            Ok(message) => 
                if let Some(dig_msg) = message.unpack_message(topics) {
                    if dig_msg.timestamp()
                        .is_some_and(|ts : DateTime<Utc>|ts > *timestamp) {
                            return Some(message);
                    }
                }
            Err(_) => {},
        }
    }
    None
}
