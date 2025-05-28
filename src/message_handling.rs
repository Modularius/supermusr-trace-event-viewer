use rdkafka::{
    message::BorrowedMessage,
    Message,
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
    message: &'a BorrowedMessage<'a>,
    topic: &str,
    id: F,
    unpack: G,
) -> Option<M>
where
    F: Fn(&'a [u8]) -> bool,
    G: Fn(&'a [u8]) -> Result<M, flatbuffers::InvalidFlatbuffer>,
{
    if message.topic() == topic {
        if let Some(payload) = message.payload() {
            if id(payload) {
                return unpack(payload).ok();
            }
        }
    }
    None
}

pub(crate) trait UnpackMessage<'a> {
    fn unpack_trace_message(&'a self, topic: &str) -> Option<DigitizerAnalogTraceMessage<'a>>;
    fn unpack_event_list_message(&'a self, topic: &str) -> Option<DigitizerEventListMessage<'a>>;
}

impl<'a> UnpackMessage<'a> for BorrowedMessage<'a> {
    fn unpack_trace_message(&'a self, topic: &str) -> Option<DigitizerAnalogTraceMessage<'a>> {
        unpack_message(
            self,
            topic,
            digitizer_analog_trace_message_buffer_has_identifier,
            root_as_digitizer_analog_trace_message,
        )
    }

    fn unpack_event_list_message(&'a self, topic: &str) -> Option<DigitizerEventListMessage<'a>> {
        unpack_message(
            self,
            topic,
            digitizer_event_list_message_buffer_has_identifier,
            root_as_digitizer_event_list_message,
        )
    }
}