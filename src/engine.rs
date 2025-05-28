use rdkafka::message::BorrowedMessage;

use crate::{message_handling::UnpackMessage, Cache, CollectMode, Topics};

pub(crate) struct Engine {
    cache: Cache,
    collect_mode: CollectMode,
    topics: Topics,
}

impl Engine {
    pub(crate) fn new(collect_mode: CollectMode, topics: Topics) -> Self {
        Engine {
            cache: Default::default(),
            collect_mode,
            topics,
        }
    }

    

    pub(crate) fn process_message(&mut self, message: &BorrowedMessage) {
        match self.collect_mode {
            CollectMode::Traces => {
                if let Some(msg) = message.unpack_trace_message(self.topics.trace_topic.as_str()) {
                    self.cache.push_trace(&msg);
                }
            }
            CollectMode::Events => {
                if let Some(msg) =
                    message.unpack_event_list_message(self.topics.digitiser_event_topic.as_str())
                {
                    self.cache.push_events(&msg);
                }
            }
            CollectMode::All => {
                if let Some(msg) = message.unpack_trace_message(self.topics.trace_topic.as_str()) {
                    self.cache.push_trace(&msg);
                } else if let Some(msg) =
                    message.unpack_event_list_message(self.topics.digitiser_event_topic.as_str())
                {
                    self.cache.push_event_list_to_trace(&msg);
                }
            }
        }
    }

    pub(crate) fn get_count(&self) -> usize {
        match self.collect_mode {
            CollectMode::Traces => self.cache.get_num_traces(),
            CollectMode::Events => self.cache.get_num_events(),
            CollectMode::All => self.cache.get_num_traces_with_events()
        }
    }
}