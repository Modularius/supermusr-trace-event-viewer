mod engine;
mod searcher;
mod task;

use strum::Display;
use supermusr_common::{Channel, DigitizerId};
use tokio::sync::{mpsc, oneshot};

use crate::{
    messages::{Cache, EventListMessage, FBMessage, TraceMessage},
    Timestamp,
};

pub(crate) use engine::SearchEngine;

#[derive(Default)]
pub(crate) enum SearchStatus {
    #[default]
    Off,
    Text(String),
    TraceSearchInProgress(u32,u32),
    EventListSearchInProgress(u32,u32),
    Halted,
    Successful,
}

#[derive(Default, Clone)]
pub(crate) struct SearchTarget {
    pub(crate) timestamp: Timestamp,
    pub(crate) channels: Vec<Channel>,
    pub(crate) digitiser_ids: Vec<DigitizerId>,
    pub(crate) number: usize,
}

impl SearchTarget {
    pub(crate) fn filter_trace_by_channel_and_digtiser_id(&self, msg: &TraceMessage) -> bool {
        //self.channels.
        //    iter()
        //    .any(|&c| msg.has_channel(c)) ||
        self.digitiser_ids
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
    fn init_search(&mut self, target: SearchTarget) -> bool;
    
    fn status(&mut self) -> Option<SearchStatus>;
    
    fn cache(&mut self) -> Option<Cache>;

    async fn run(&mut self);
}
