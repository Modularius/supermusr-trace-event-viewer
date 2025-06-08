use rdkafka::consumer::BaseConsumer;
use tokio::{sync::{mpsc, oneshot::{self, error::TryRecvError, Receiver}}, task::JoinHandle};
use tracing::{info, instrument};

use crate::{
    finder::{searcher::Searcher, InitSearchResponse, MessageFinder, SearchStatus, SearchTarget},
    messages::{
        Cache, EventListMessage, FBMessage,
        TraceMessage,
    },
    Select, Topics,
};

pub(crate) struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    /// 
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: Option<BaseConsumer>,
    /// When another instance of [Self] is finished with the [BaseConsumer] object,
    /// it is passed back via this channel.
    recv_consumer: Option<Receiver<BaseConsumer>>,
    /// 
    select: Select,
    topics: Topics,
    /// When a search is in progress
    handle: Option<JoinHandle<()>>,
}

impl SearchEngine {
    pub(crate) fn new(consumer: BaseConsumer, select: &Select, topics: &Topics) -> Self {
        Self {
            consumer: Some(consumer),
            recv_consumer: None,
            select: select.clone(),
            topics: topics.clone(),
            handle: None,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(send_status: &mpsc::Sender<SearchStatus>, new_status: SearchStatus) {
        //let mut status = self.status.lock().expect("Status");
        //status.replace(new_status);
        send_status.send(new_status).await.expect("");
    }

    #[instrument(skip_all)]
    pub(crate) async fn search(
        &mut self,
        send_status: mpsc::Sender<SearchStatus>,
        target: SearchTarget,
    ) {
        let steps = &self.select.step;
        let mut cache = Cache::default();

        // Find Digitiser Traces
        Self::emit_status(&send_status, SearchStatus::TraceSearchInProgress(0,steps.num_step_passes + 1)).await;

        let searcher = Searcher::new(self.consumer.as_ref().expect(""), &self.topics.trace_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            Self::emit_status(&send_status, SearchStatus::TraceSearchInProgress(steps.num_step_passes - 1 - step,steps.num_step_passes + 1)).await;
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp);
        }

        Self::emit_status(&send_status, SearchStatus::TraceSearchInProgress(steps.num_step_passes,steps.num_step_passes + 1)).await;

        let results: Vec<TraceMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t| t >= target.timestamp)
            .acquire_while(|msg| target.filter_trace_by_channel_and_digtiser_id(msg))
            .collect()
            .into();

        Self::emit_status(&send_status, SearchStatus::TraceSearchInProgress(steps.num_step_passes + 1,steps.num_step_passes + 1)).await;

        for trace in results.iter() {
            cache.push_trace(&trace.get_unpacked_message().expect(""));
        }

        // Find Digitiser Event Lists
        Self::emit_status(&send_status, SearchStatus::EventListSearchInProgress(0,steps.num_step_passes + 1)).await;

        let searcher = Searcher::new(self.consumer.as_ref().expect(""), &self.topics.trace_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            Self::emit_status(&send_status, SearchStatus::EventListSearchInProgress(steps.num_step_passes - 1 - step,steps.num_step_passes + 1)).await;
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp);
        }

        Self::emit_status(&send_status, SearchStatus::EventListSearchInProgress(steps.num_step_passes,steps.num_step_passes + 1)).await;

        let results: Vec<EventListMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t| t >= target.timestamp)
            .acquire_while(|msg| target.filter_eventlist_digtiser_id(msg))
            .collect()
            .into();

        Self::emit_status(&send_status, SearchStatus::EventListSearchInProgress(steps.num_step_passes + 1,steps.num_step_passes + 1)).await;

        for eventlist in results.iter() {
            cache.push_event_list_to_trace(&eventlist.get_unpacked_message().expect(""));
        }

        // Send cache via status
        Self::emit_status(&send_status, SearchStatus::Successful(cache)).await;
    }
}

impl Drop for SearchEngine {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.as_ref() {
            handle.abort();
        }
    }
}

impl MessageFinder for SearchEngine {
    #[instrument(skip_all)]
    fn init_search(&mut self, target: SearchTarget) -> Option<InitSearchResponse> {
        self.consumer.take().and_then(|consumer| {
            let (send_consumer, recv_consumer) = oneshot::channel();
            let (send_halt, mut recv_halt) = oneshot::channel();
            let (send_status, recv_status) = mpsc::channel(10);

            self.recv_consumer = Some(recv_consumer);

            let mut engine = SearchEngine::new(consumer, &self.select, &self.topics);

            self.handle = Some(tokio::spawn(async move {
                loop {
                    if let Ok(()) = recv_halt.try_recv() {
                        send_status.clone().send(SearchStatus::Halted).await.expect("");
                        break;
                    }
                    tokio::select! {
                        _ = engine.search(send_status,target) => {
                            break;
                        }
                    }
                }
                if let Err(_) = send_consumer.send(engine.consumer.take().expect("")) {

                }
            }));
            Some(InitSearchResponse {
                send_halt,
                recv_status,
            })
        })
    }

    fn retrieve_consumer(&mut self) {
        if let Some(recv_consumer) = self.recv_consumer.as_mut() {
            match recv_consumer.try_recv() {
                Ok(consumer) => {
                    self.consumer = Some(consumer)
                },
                Err(e) => match e {
                    TryRecvError::Empty => {},
                    TryRecvError::Closed => {
                        self.recv_consumer = None;
                    },
                },
            }
        }
    }
}