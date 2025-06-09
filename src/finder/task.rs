use rdkafka::consumer::BaseConsumer;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{info, error, instrument};

use crate::{
    finder::{searcher::Searcher, MessageFinder, SearchStatus, SearchTarget},
    messages::{
        Cache, EventListMessage, FBMessage,
        TraceMessage,
    },
    Select, Topics,
};

pub(crate) struct SearchTask<'a> {
    consumer: BaseConsumer,
    send_status: &'a mpsc::Sender<SearchStatus>,
    select: &'a Select,
    topics: &'a Topics,
}

impl<'a> SearchTask<'a> {
    pub(crate) fn new(consumer: BaseConsumer, send_status: &'a mpsc::Sender<SearchStatus>, select: &'a Select, topics: &'a Topics) -> Self {
        Self {
            consumer,
            send_status,
            select,
            topics,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        //let mut status = self.status.lock().expect("Status");
        //status.replace(new_status);
        if let Err(e) = self.send_status.send(new_status).await {
            panic!("{e}");
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn search(
        self,
        target: SearchTarget,
    ) -> (BaseConsumer, Cache) {
        let steps = &self.select.step;
        let mut cache = Cache::default();

        let send_status = self.send_status;

        // Find Digitiser Traces
        self.emit_status(SearchStatus::TraceSearchInProgress(0,steps.num_step_passes + 1)).await;

        let searcher = Searcher::new(&self.consumer, &self.topics.trace_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            self.emit_status(SearchStatus::TraceSearchInProgress(steps.num_step_passes - 1 - step,steps.num_step_passes + 1)).await;
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp).await;
        }

        self.emit_status(SearchStatus::TraceSearchInProgress(steps.num_step_passes,steps.num_step_passes + 1)).await;

        let results: Vec<TraceMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t| t >= target.timestamp).await
            .acquire_while(|msg| target.filter_trace_by_channel_and_digtiser_id(msg), target.number).await
            .collect()
            .into();

        self.emit_status(SearchStatus::TraceSearchInProgress(steps.num_step_passes + 1,steps.num_step_passes + 1)).await;

        for trace in results.iter() {
            cache.push_trace(&trace.get_unpacked_message().expect(""));
        }

        // Find Digitiser Event Lists
        self.emit_status(SearchStatus::EventListSearchInProgress(0,steps.num_step_passes + 1)).await;

        let searcher = Searcher::new(&self.consumer, &self.topics.digitiser_event_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            self.emit_status(SearchStatus::EventListSearchInProgress(steps.num_step_passes - 1 - step,steps.num_step_passes + 1)).await;
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp).await;
        }

        self.emit_status(SearchStatus::EventListSearchInProgress(steps.num_step_passes,steps.num_step_passes + 1)).await;

        let results: Vec<EventListMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t| t >= target.timestamp).await
            .acquire_while(|msg| target.filter_eventlist_digtiser_id(msg), target.number).await
            .collect()
            .into();

        self.emit_status(SearchStatus::EventListSearchInProgress(steps.num_step_passes + 1,steps.num_step_passes + 1)).await;

        for eventlist in results.iter() {
            cache.push_event_list_to_trace(&eventlist.get_unpacked_message().expect(""));
        }

        // Send cache via status
        self.emit_status(SearchStatus::Successful).await;

        (self.consumer, cache)
    }
}