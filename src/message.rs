use chrono::{DateTime, Utc};
use rdkafka::{message::BorrowedMessage, Message};
use supermusr_common::{Channel, DigitizerId};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
        DigitizerAnalogTraceMessage
    },
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier,
        root_as_digitizer_event_list_message,
        DigitizerEventListMessage
    }
};

pub(crate) struct TraceMessage<'a> {
    message: BorrowedMessage<'a>,
    timestamp: DateTime<Utc>,
    digitiser_id: DigitizerId,
}

impl<'a> TraceMessage<'a> {
    pub(crate) fn has_channel(&self, channel: Channel) -> bool {
        self.get_unpacked_message()
            .and_then(|d|d.channels())
            .and_then(|c|c.iter().find(|c|c.channel() == channel))
            .is_some()
    }
}

impl<'a> FBMessage<'a> for TraceMessage<'a> {
    type UnpackedMessage = DigitizerAnalogTraceMessage<'a>;
    
    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage> {
        self.message.unpack_trace_message()
    }

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self> {
        let trace = message.unpack_trace_message()?;

        let timestamp = trace.metadata()
            .timestamp()
            .cloned()
            .map(TryInto::try_into)
            .map(Result::ok)
            .flatten()?;
        let digitiser_id = trace.digitizer_id();

        Some(Self {
            message,
            timestamp,
            digitiser_id
        })
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn digitiser_id(&self) -> DigitizerId {
        self.digitiser_id
    }
}

pub(crate) struct EventListMessage<'a> {
    message: BorrowedMessage<'a>,
    timestamp: DateTime<Utc>,
    digitiser_id: DigitizerId,

}

impl<'a> FBMessage<'a> for EventListMessage<'a> {
    type UnpackedMessage = DigitizerEventListMessage<'a>;

    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage> {
        self.message.unpack_event_list_message()
    }

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self> {
        let evlist = message.unpack_event_list_message()?;

        let timestamp = evlist.metadata()
            .timestamp()
            .cloned()
            .map(TryInto::try_into)
            .map(Result::ok)
            .flatten()?;

        let digitiser_id = evlist.digitizer_id();

        Some(Self {
            message,
            timestamp,
            digitiser_id
        })
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn digitiser_id(&self) -> DigitizerId {
        self.digitiser_id
    }
}

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
    /*fn unpack_message(&'a self, topics: &'a Topics) -> Option<DigitizerMessage<'a>> {
        self.unpack_trace_message(&topics.trace_topic)
            .map(DigitizerMessage::Trace)
            .or(self
                .unpack_event_list_message(&topics.digitiser_event_topic)
                .map(DigitizerMessage::EventList)
            )
    }*/
    fn unpack_trace_message(&'a self) -> Option<DigitizerAnalogTraceMessage<'a>>;
    fn unpack_event_list_message(&'a self) -> Option<DigitizerEventListMessage<'a>>;
}

impl<'a> UnpackMessage<'a> for BorrowedMessage<'a> {
    fn unpack_trace_message(&'a self) -> Option<DigitizerAnalogTraceMessage<'a>> {
        self.payload()
            .filter(|payload|digitizer_analog_trace_message_buffer_has_identifier(payload))
            .and_then(|payload|root_as_digitizer_analog_trace_message(payload).ok())
    }

    fn unpack_event_list_message(&'a self) -> Option<DigitizerEventListMessage<'a>> {
        self.payload()
            .filter(|payload|digitizer_event_list_message_buffer_has_identifier(payload))
            .and_then(|payload|root_as_digitizer_event_list_message(payload).ok())
    }
}

pub(crate) trait FBMessage<'a> : Sized {
    type UnpackedMessage;

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self>;
    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage>;
    fn timestamp(&self) -> DateTime<Utc>;
    fn digitiser_id(&self) -> DigitizerId;
}
