use std::sync::{Arc, Mutex};

use rdkafka::consumer::BaseConsumer;
use tokio::sync::{mpsc, oneshot};

use crate::{finder::{searcher::Searcher, InitSearchResponse, MessageFinder, SearchStatus, SearchTarget}, messages::{Cache, CreateFromMessage, DigitiserEventList, DigitiserTrace, EventListMessage, FBMessage, TraceMessage}, Select, Topics};

#[derive(Clone)]
pub(crate) struct SearchEngine {
    consumer: Arc<Mutex<BaseConsumer>>,
    select: Arc<Mutex<Select>>,
    topics: Arc<Mutex<Topics>>
}

impl SearchEngine {
    pub(crate) fn new(consumer: BaseConsumer, select: &Select, topics: &Topics) -> Self {
        Self {
            consumer: Arc::new(Mutex::new(consumer)),
            select: Arc::new(Mutex::new(select.clone())),
            topics: Arc::new(Mutex::new(topics.clone()))
        }
    }

    pub(crate) async fn search(&mut self, send_status: mpsc::Sender<SearchStatus>, target: SearchTarget) -> Cache {
        let consumer = self.consumer.lock().expect("");
        let topics = &self.topics.lock().expect("");

        let steps = &self.select.lock().expect("").step;

        let mut cache = Cache::default();

        // Find Digitiser Traces
        let searcher = Searcher::new(&consumer, &topics.trace_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t|t > target.timestamp);
        }

        let results : Vec<TraceMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t|t >= target.timestamp)
            .acquire_while(|msg|target.filter_trace_by_channel_and_digtiser_id(msg))
            .collect()
            .into();

        for trace in results.iter() {
            cache.push_trace(&trace.get_unpacked_message().expect(""));
        }
        
        // Find Digitiser Event Lists
        let searcher = Searcher::new(&consumer, &topics.trace_topic, 1, send_status.clone());
        let mut iter = searcher.iter_backstep();
        for step in (0..steps.num_step_passes).rev() {
            let sz = steps.min_step_size * steps.step_mul_coef.pow(step);
            iter.step_size(sz)
                .backstep_until_time(|t|t > target.timestamp);
        }

        let results : Vec<EventListMessage> = iter
            .collect()
            .iter_forward()
            .move_until(|t|t >= target.timestamp)
            .acquire_while(|msg|target.filter_eventlist_digtiser_id(msg))
            .collect()
            .into();

        for eventlist in results.iter() {
            cache.push_event_list_to_trace(&eventlist.get_unpacked_message().expect(""));
        }

        // Return cache
        cache
    }
}

impl MessageFinder for SearchEngine {
    fn init_search(&mut self, target: SearchTarget) -> Option<InitSearchResponse> {
        let (send_halt, mut recv_halt) = oneshot::channel();
        let (send_finished, recv_finished) = oneshot::channel();
        let (send_status, recv_status) = mpsc::channel(10);

        let mut engine = self.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(()) = recv_halt.try_recv() {
                    return;
                }
                tokio::select!{
                    results = engine.search(send_status,target) => {
                        if let Err(_e) = send_finished.send(results) {
                            return;
                        }
                        return;
                    }
                }
            }
        });
        
        Some(InitSearchResponse {
            send_halt,
            recv_finished,
            recv_status
        })
    }
}