//!
use chrono::{DateTime, Utc};
use std::collections::{hash_map::Entry, HashMap};
use supermusr_common::{Channel, DigitizerId, Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage,
    dev2_digitizer_event_v2_generated::DigitizerEventListMessage, FrameMetadata,
};
use tracing::error;

trait CreateFromMessage<M> {
    fn create_from_message(msg: &M) -> Self;
}

/// Timeseries of signal intensities. The time and value scaling is not stored here, so interpretation is owner dependent.
type Trace = Vec<Intensity>;

/// Bundles all metadata which uniquely defines each digitiser message.
#[derive(PartialEq, Eq, Hash, Debug)]
struct DigitiserMetadata {
    /// Unique to each digitiser.
    id: DigitizerId,
    /// Unique to each frame.
    timestamp: DateTime<Utc>,
}

/// Encapsulates all traces of a digitiser trace message.
struct DigitiserTrace {
    ///
    traces: HashMap<Channel, Trace>,
    events: Option<DigitiserEventList>,
}

impl CreateFromMessage<DigitizerAnalogTraceMessage<'_>> for DigitiserTrace {
    fn create_from_message(msg: &DigitizerAnalogTraceMessage) -> Self {
        let pairs: Vec<(Channel, Trace)> = msg
            .channels()
            .unwrap()
            .iter()
            .map(|x| (x.channel(), x.voltage().unwrap().iter().collect()))
            .collect();
        let traces: HashMap<Channel, Trace> = HashMap::from_iter(pairs.into_iter());
        DigitiserTrace {
            traces,
            events: None,
        }
    }
}

struct Event {
    time: Time,
    intensity: Intensity,
}
type EventList = Vec<Event>;
type DigitiserEventList = HashMap<Channel, EventList>;

impl CreateFromMessage<DigitizerEventListMessage<'_>> for DigitiserEventList {
    fn create_from_message(msg: &DigitizerEventListMessage) -> Self {
        let mut events = HashMap::<Channel, EventList>::new();
        for (idx, chnl) in msg.channel().unwrap().iter().enumerate() {
            events
                .entry(chnl)
                .or_insert(Default::default())
                .push(Event {
                    time: msg.time().unwrap().get(idx),
                    intensity: msg.voltage().unwrap().get(idx),
                })
        }
        events
    }
}

#[derive(Default)]
pub(crate) struct Cache {
    traces: HashMap<DigitiserMetadata, DigitiserTrace>,
    events: HashMap<DigitiserMetadata, DigitiserEventList>,
}

impl Cache {
    pub(crate) fn push_trace(&mut self, msg: &DigitizerAnalogTraceMessage<'_>) {
        let metadata = DigitiserMetadata {
            id: msg.digitizer_id(),
            timestamp: msg
                .metadata()
                .timestamp()
                .unwrap()
                .clone()
                .try_into()
                .unwrap(),
        };
        match self.traces.entry(metadata) {
            Entry::Occupied(occupied_entry) => {
                error!("Trace already found: {0:?}", occupied_entry.key());
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(DigitiserTrace::create_from_message(msg));
            }
        }
    }
    pub(crate) fn push_events(&mut self, msg: &DigitizerEventListMessage<'_>) {
        let metadata = DigitiserMetadata {
            id: msg.digitizer_id(),
            timestamp: msg
                .metadata()
                .timestamp()
                .unwrap()
                .clone()
                .try_into()
                .unwrap(),
        };
        match self.events.entry(metadata) {
            Entry::Occupied(occupied_entry) => {
                error!("Event list already found: {0:?}", occupied_entry.key());
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(DigitiserEventList::create_from_message(msg));
            }
        }
    }

    pub(crate) fn push_event_list_to_trace(&mut self, msg: &DigitizerEventListMessage<'_>) {
        let metadata = DigitiserMetadata {
            id: msg.digitizer_id(),
            timestamp: msg
                .metadata()
                .timestamp()
                .unwrap()
                .clone()
                .try_into()
                .unwrap(),
        };
        match self.traces.entry(metadata) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().events =
                    Some(DigitiserEventList::create_from_message(msg));
            }
            Entry::Vacant(vacant_entry) => {
                error!("Trace not found: {0:?}", vacant_entry.key());
            }
        }
    }

    pub(crate) fn get_num_traces(&self) -> usize {
        self.traces.len()
    }

    pub(crate) fn get_num_events(&self) -> usize {
        self.events.len()
    }

    pub(crate) fn get_num_traces_with_events(&self) -> usize {
        self.traces.values().filter(|x|x.events.is_some()).count()
    }
}
