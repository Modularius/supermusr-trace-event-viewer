//!
use chrono::{DateTime, Utc};
use std::{collections::HashMap, ops::Range};
use supermusr_common::{Channel, DigitizerId, Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage,
    dev2_digitizer_event_v2_generated::DigitizerEventListMessage,
};

use crate::UserBounds;

#[derive(Clone)]
pub(crate) struct Bounds {
    /// Minimum time bin to graph.
    time_min: Time,
    /// Maximum time bin to graph.
    time_max: Time,
    /// Minimum intensity value to graph.
    intensity_min: Intensity,
    /// Maximum intensity value to graph.
    intensity_max: Intensity,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            time_min: Time::MAX,
            time_max: Time::MIN,
            intensity_min: Intensity::MAX,
            intensity_max: Intensity::MIN,
        }
    }
}

impl Bounds {
    pub(crate) fn merge(self, bound: Self) -> Self {
        Self {
            time_min: self.time_min.min(bound.time_min),
            time_max: self.time_max.max(bound.time_max),
            intensity_min: self.intensity_min.min(bound.intensity_min),
            intensity_max: self.intensity_max.max(bound.intensity_max),
        }
    }

    pub(crate) fn from_trace(trace: &Trace) -> Option<Self> {
        Option::zip(trace.iter().min(), trace.iter().max()).map(
            |(&intensity_min, &intensity_max)| Self {
                time_min: Time::default(),
                time_max: trace.len() as Time,
                intensity_min,
                intensity_max,
            },
        )
    }

    pub(crate) fn from_digitiser_trace(trace: &DigitiserTrace) -> Option<Self> {
        let mut bounds = trace.traces.values().map(Self::from_trace).flatten();
        let trace_bound = bounds
            .next()
            .map(|first| bounds.fold(first, Self::merge));

        if let Some(event_bound) = trace.events.as_ref().map(|events| {
            Self::from_digitiser_events_list(events)
        }).flatten() {
            trace_bound.map(|tb|tb.merge(event_bound))
        } else {
            trace_bound
        }
    }

    pub(crate) fn from_events_list(events: &EventList) -> Option<Self> {
        let time = Option::zip(
            events.iter().map(|e| e.time).min(),
            events.iter().map(|e| e.time).max(),
        );
        let intensity = Option::zip(
            events.iter().map(|e| e.intensity).min(),
            events.iter().map(|e| e.intensity).max(),
        );
        Option::zip(time, intensity).map(
            |((time_min, time_max), (intensity_min, intensity_max))| Self {
                time_min,
                time_max,
                intensity_min,
                intensity_max,
            },
        )
    }

    pub(crate) fn from_digitiser_events_list(events: &DigitiserEventList) -> Option<Self> {
        let mut bounds = events.values().map(Self::from_events_list).flatten();
        bounds.next().map(|first| bounds.fold(first, Self::merge))
    }

    pub(crate) fn ammend_with_user_input(&mut self, bounds: &UserBounds) {
        if let Some(time_min) = bounds.time_min {
            self.time_min = time_min;
        }
        if let Some(time_max) = bounds.time_max {
            self.time_max = time_max;
        }
        if let Some(intensity_min) = bounds.intensity_min {
            self.intensity_min = intensity_min;
        }
        if let Some(intensity_max) = bounds.intensity_max {
            self.intensity_max = intensity_max;
        }
    }

    pub(crate) fn time_range(&self) -> Range<f32> {
        self.time_min as f32 .. self.time_max as f32
    }

    pub(crate) fn intensity_range(&self) -> Range<f32> {
        self.intensity_min as f32 .. self.intensity_max as f32
    }
}

pub(crate) trait CreateFromMessage<M> {
    fn create_from_message(msg: &M) -> Self;
}

/// Timeseries of signal intensities. The time and value scaling is not stored here, so interpretation is owner dependent.
pub(crate) type Trace = Vec<Intensity>;

/// Bundles all metadata which uniquely defines each digitiser message.
#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct DigitiserMetadata {
    /// Unique to each digitiser.
    pub(crate) id: DigitizerId,
    /// Unique to each frame.
    pub(crate) timestamp: DateTime<Utc>,
}

/// Encapsulates all traces of a digitiser trace message.
pub(crate) struct DigitiserTrace {
    ///
    pub(crate) traces: HashMap<Channel, Trace>,
    pub(crate) events: Option<DigitiserEventList>,
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

pub(crate) struct Event {
    pub(crate) time: Time,
    pub(crate) intensity: Intensity,
}
pub(crate) type EventList = Vec<Event>;
pub(crate) type DigitiserEventList = HashMap<Channel, EventList>;

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
