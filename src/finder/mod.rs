mod engine;
mod searcher;

use supermusr_common::{Channel, DigitizerId};
use tokio::sync::{mpsc, oneshot};

use crate::{
    messages::{Cache, DigitiserTrace, EventListMessage, FBMessage, TraceMessage},
    Timestamp,
};

pub(crate) use engine::SearchEngine;

pub(crate) struct InitSearchResponse {
    pub(crate) send_halt: oneshot::Sender<()>,
    pub(crate) recv_finished: oneshot::Receiver<Cache>,
    pub(crate) recv_status: mpsc::Receiver<SearchStatus>,
}

#[derive(Default, Clone)]
pub(crate) enum SearchStatus {
    #[default]
    Off,
    On,
}

#[derive(Default, Clone)]
pub(crate) struct SearchTarget {
    pub(crate) timestamp: Timestamp,
    pub(crate) channels: Vec<Channel>,
    pub(crate) digitiser_ids: Vec<DigitizerId>,
}

impl SearchTarget {
    pub(crate) fn filter_trace_by_channel_and_digtiser_id(&self, msg: &TraceMessage) -> bool {
        self.channels.iter().any(|&c| msg.has_channel(c))
            || self
                .digitiser_ids
                .iter()
                .any(|&d: &u8| msg.digitiser_id() == d)
    }

    pub(crate) fn filter_eventlist_digtiser_id(&self, msg: &EventListMessage) -> bool {
        self.digitiser_ids
            .iter()
            .any(|&d: &u8| msg.digitiser_id() == d)
    }
}

pub(crate) trait MessageFinder {
    fn init_search(&mut self, target: SearchTarget) -> Option<InitSearchResponse>;
}
