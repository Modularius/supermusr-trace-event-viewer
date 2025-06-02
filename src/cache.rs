//!
use std::collections::{hash_map::{self, Entry}, HashMap};
use supermusr_common::{Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage,
    dev2_digitizer_event_v2_generated::DigitizerEventListMessage,
};
use tracing::{error, info};

use crate::data::{CreateFromMessage, DigitiserEventList, DigitiserMetadata, DigitiserTrace};

#[derive(Default)]
pub(crate) struct Cache {
    traces: HashMap<DigitiserMetadata, DigitiserTrace>,
    events: HashMap<DigitiserMetadata, DigitiserEventList>,
    bounds: Option<((Time, Time), (Intensity, Intensity))>,
}

impl Cache {
    pub(crate) fn push_trace(&mut self, msg: &DigitizerAnalogTraceMessage<'_>) {
        info!("New Trace");
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

    pub(crate) fn get_num_traces(&self) -> usize {
        self.traces.len()
    }

    pub(crate) fn get_num_events(&self) -> usize {
        self.events.len()
    }

    pub(crate) fn get_num_traces_with_events(&self) -> usize {
        self.traces.values().filter(|x| x.events.is_some()).count()
    }

    pub(crate) fn iter_traces(&self) -> hash_map::Iter<'_, DigitiserMetadata, DigitiserTrace> {
        self.traces.iter()
    }

    pub(crate) fn iter_events(&self) -> hash_map::Iter<'_, DigitiserMetadata, DigitiserEventList> {
        self.events.iter()
    }

    pub(crate) fn push_events(&mut self, msg: &DigitizerEventListMessage<'_>) {
        info!("New Events");
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
        info!("New Event for Trace");
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
                info!("Found Trace for Events");
                occupied_entry.get_mut().events =
                    Some(DigitiserEventList::create_from_message(msg));
            }
            Entry::Vacant(vacant_entry) => {
                error!("Trace not found: {0:?}", vacant_entry.key());
            }
        }
    }
}
