use rdkafka::message::BorrowedMessage;

use crate::{build_graph::BuildGraph, data::Bounds, message_handling::UnpackMessage, Cache, CollectMode, OutputMode, Topics, UserBounds};

pub(crate) struct Engine {
    cache: Cache,
    collect_mode: CollectMode,
    topics: Topics,
    output: OutputMode,
}

impl Engine {
    pub(crate) fn new(collect_mode: CollectMode, topics: Topics, output: OutputMode) -> Self {
        Engine {
            cache: Default::default(),
            collect_mode,
            topics,
            output
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

    pub(crate) fn output(&self, user_bounds: &UserBounds) {
        match &self.output {
            OutputMode::File(output_to_file) => {
                match self.collect_mode {
                    CollectMode::Traces => {
                        for (metadata, traces) in self.cache.iter_traces() {
                            for (channel, trace) in &traces.traces {
                                let mut bounds = Bounds::from_trace(&trace).expect("");
                                bounds.ammend_with_user_input(user_bounds);
                                let graph = BuildGraph::new(320,240,bounds.time_range(), bounds.intensity_range());

                                graph.save_trace_graph(&output_to_file.path, &trace).expect("");
                            }
                        }
                    },
                    CollectMode::Events => {},
                    CollectMode::All => {}
                }
            },
        }
    }
}