use crate::{Cli, CollectMode, data::Cache};
use rdkafka::{
    message::BorrowedMessage,
    Message,
};
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

fn unpack_message<'a, F, G, M>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
    id: F,
    unpack: G,
) -> Option<M>
where
    F: Fn(&'a [u8]) -> bool,
    G: Fn(&'a [u8]) -> Result<M, flatbuffers::InvalidFlatbuffer>,
{
    if m.topic() == topic {
        if let Some(payload) = m.payload() {
            if id(payload) {
                return unpack(payload).ok();
            }
        }
    }
    None
}

fn unpack_trace_message<'a>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
) -> Option<DigitizerAnalogTraceMessage<'a>> {
    unpack_message(
        m,
        topic,
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    )
}

fn unpack_event_list_message<'a>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
) -> Option<DigitizerEventListMessage<'a>> {
    unpack_message(
        m,
        topic,
        digitizer_event_list_message_buffer_has_identifier,
        root_as_digitizer_event_list_message,
    )
}

pub(crate) fn process_message(args: &Cli, cache: &mut Cache, message: &BorrowedMessage) {
    match args.collect {
        CollectMode::Traces => {
            if let Some(msg) = unpack_trace_message(&message, args.trace_topic.as_str()) {
                cache.push_trace(&msg);
            }
        }
        CollectMode::Events => {
            if let Some(msg) =
                unpack_event_list_message(&message, args.digitiser_event_topic.as_str())
            {
                cache.push_events(&msg);
            }
        }
        CollectMode::All => {
            if let Some(msg) = unpack_trace_message(&message, args.trace_topic.as_str()) {
                cache.push_trace(&msg);
            } else if let Some(msg) =
                unpack_event_list_message(&message, args.digitiser_event_topic.as_str())
            {
                cache.push_event_list_to_trace(&msg);
            }
        }
    }
}