use chrono::Utc;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    TopicPartitionList,
};
use tracing::instrument;

use crate::{
    finder::{
        searcher::Searcher,
        task::{SearchTask, TaskClass},
        SearchResults, SearchStatus, SearchTarget,
    },
    messages::{Cache, EventListMessage, FBMessage, TraceMessage},
};

pub(crate) struct SearchByCapture;
impl TaskClass for SearchByCapture {}

impl<'a> SearchTask<'a, SearchByCapture> {
    /// Performs a FromEnd search.
    /// # Attributes
    /// - target: what to search for.
    #[instrument(skip_all)]
    pub(crate) async fn search(self, target: SearchTarget) -> (StreamConsumer, SearchResults) {
        let start = Utc::now();

        let mut cache = Cache::default();

        let mut tpl = TopicPartitionList::with_capacity(2);
        tpl.add_partition_offset(&self.topics.trace_topic, 0, rdkafka::Offset::End)
            .expect("");
        tpl.add_partition_offset(&self.topics.digitiser_event_topic, 0, rdkafka::Offset::End)
            .expect("");
        self.consumer.assign(&tpl).expect("");
        // TODO
        // Find Digitiser Traces
        self.emit_status(SearchStatus::TraceSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.trace_topic,
            1,
            self.send_status.clone(),
        );

        let trace_results: Vec<TraceMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, target.number)
            .await
            .collect()
            .into();

        // Find Digitiser Event Lists
        self.emit_status(SearchStatus::EventListSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            2 * target.number as i64 + 1,
            self.send_status.clone(),
        );

        let eventlist_results: Vec<EventListMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, 2 * target.number)
            .await
            .collect()
            .into();

        for trace in trace_results.iter() {
            cache.push_trace(&trace.get_unpacked_message().expect(""));
        }

        for eventlist in eventlist_results.iter() {
            cache.push_events(&eventlist.get_unpacked_message().expect(""));
        }
        cache.attach_event_lists_to_trace();

        // Send cache via status
        self.emit_status(SearchStatus::Successful).await;
        let time = Utc::now() - start;
        (self.consumer, SearchResults { cache, time })
    }
}
