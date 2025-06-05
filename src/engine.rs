use rdkafka::message::BorrowedMessage;
use tracing::{info, instrument};

use crate::{
    build_graph::{BackendSVG, BuildGraph},
    data::Bounds,
    message::{DigitizerMessage, UnpackMessage},
    Cache, CollectType, Find, Finder, Mode, Topics, UserBounds,
};

pub(crate) struct Engine {
    cache: Cache,
    collect_mode: CollectType,
    topics: Topics,
    output: Mode,
}

impl Engine {
    pub(crate) fn new(collect_mode: CollectType, topics: Topics, output: Mode, find: Find) -> Self {
        let finder = Finder::from_cli(find).expect("");
        Engine {
            cache: Default::default(),
            collect_mode,
            topics,
            output,
        }
    }

    #[instrument(skip_all)]
    pub(crate) fn process_message(&mut self, message: &BorrowedMessage) {
        match self.collect_mode {
            CollectType::Traces => {
                if let Some(msg) = message.unpack_trace_message(self.topics.trace_topic.as_str()) {
                    self.cache.push_trace(&msg);
                }
            }
            CollectType::Events => {
                if let Some(msg) =
                    message.unpack_event_list_message(self.topics.digitiser_event_topic.as_str())
                {
                    self.cache.push_events(&msg);
                }
            }
            CollectType::All => {
                if let Some(msg) = message.unpack_message(&self.topics) {
                    match msg {
                        DigitizerMessage::Trace(msg) => self.cache.push_trace(&msg),
                        DigitizerMessage::EventList(msg) => {
                            self.cache.push_event_list_to_trace(&msg)
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn get_count(&self) -> usize {
        match self.collect_mode {
            CollectType::Traces => self.cache.get_num_traces(),
            CollectType::Events => self.cache.get_num_events(),
            CollectType::All => self.cache.get_num_traces_with_events(),
        }
    }

    #[instrument(skip_all)]
    pub(crate) fn output(&self, user_bounds: &UserBounds) {
        match &self.output {
            Mode::File(output_to_file) => {
                info!("Outputting");
                match self.collect_mode {
                    CollectType::Traces => {
                        info!(
                            "Outputting {} Digitiser Traces",
                            self.cache.iter_traces().len()
                        );
                        for (metadata, traces) in self.cache.iter_traces() {
                            info!("Outputting Frame {:?} Traces", metadata);
                            info!("Outputting {} Traces", traces.traces.len());
                            for (channel, trace) in &traces.traces {
                                info!("Outputting Channel {channel}");
                                let mut bounds = Bounds::from_trace(&trace).expect("");
                                bounds.ammend_with_user_input(user_bounds);
                                let graph = BuildGraph::<BackendSVG<'_>>::new(
                                    320,
                                    240,
                                    bounds.time_range(),
                                    bounds.intensity_range(),
                                );

                                graph
                                    .save_trace_graph(&output_to_file.path, &trace)
                                    .expect("");
                            }
                        }
                    }
                    CollectType::Events => {}
                    CollectType::All => {
                        info!(
                            "Outputting {} Digitiser Traces",
                            self.cache.iter_traces().len()
                        );
                        for (metadata, traces) in self.cache.iter_traces() {
                            info!("Outputting Frame {:?} Traces", metadata);
                            info!("Outputting {} Traces", traces.traces.len());
                            for (channel, trace) in &traces.traces {
                                info!("Outputting Channel {channel}");
                                let mut bounds = Bounds::from_trace(&trace).expect("");
                                bounds.ammend_with_user_input(user_bounds);
                                let graph = BuildGraph::<BackendSVG<'_>>::new(
                                    800,
                                    600,
                                    bounds.time_range(),
                                    bounds.intensity_range(),
                                );

                                let path_buf = graph
                                    .build_path(&output_to_file.path, metadata, *channel)
                                    .expect("extension should write");
                                graph.save_trace_graph(&path_buf, &trace).expect("");
                            }
                        }
                    }
                }
            }
        }
    }
}
