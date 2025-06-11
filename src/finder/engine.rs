use rdkafka::consumer::BaseConsumer;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, instrument};

use crate::{
    finder::{task::SearchTask, MessageFinder, SearchStatus, SearchTarget},
    messages::Cache,
    Select, Topics,
};

pub(crate) struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    /// 
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: Option<BaseConsumer>,
    target: Option<SearchTarget>,
    /// When another instance of [Self] is finished with the [BaseConsumer] object,
    /// it is passed back via this channel.
    send_init: mpsc::Sender<(BaseConsumer, SearchTarget)>,
    recv_halt: mpsc::Receiver<BaseConsumer>,
    recv_results: mpsc::Receiver<(BaseConsumer, Cache)>,
    recv_status: mpsc::Receiver<SearchStatus>,
    status: Option<SearchStatus>,
    cache: Option<Cache>,
    // 
    //select: Select,
    //topics: Topics,
    /// When a search is in progress
    handle: JoinHandle<()>,
}

impl SearchEngine {
    pub(crate) fn new(consumer: BaseConsumer, select: &Select, topics: &Topics) -> Self {
        let select = select.clone();
        let topics = topics.clone();

        let (send_init, mut recv_init) = mpsc::channel(1);
        let (send_results, recv_results) = mpsc::channel(1);
        let (send_halt, recv_halt) = mpsc::channel(1);
        let (send_status, recv_status) = mpsc::channel(1);
        Self {
            consumer: Some(consumer),
            send_init,
            recv_halt,
            recv_results,
            recv_status,
            target: None,
            status: None,
            cache: None,
            handle: tokio::spawn(async move {
                loop {
                    let (consumer, target) = recv_init.recv().await.expect("");
                    
                    let task = SearchTask::new(consumer, &send_status, &select, &topics);
                    let (consumer, cache) = task.search(target).await;
                    
                    send_results.send((consumer, cache)).await.expect("");
                }
            }),
        }
    }
}


impl Drop for SearchEngine {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl MessageFinder for SearchEngine {
    #[instrument(skip_all)]
    fn init_search(&mut self, target: SearchTarget) -> bool {
        if self.consumer.is_some() {
            self.target = Some(target);
        }
        self.consumer.is_some()
    }
    
    fn status(&mut self) -> Option<SearchStatus> {
        self.status.take()
    }
    
    fn cache(&mut self) -> Option<Cache> {
        self.cache.take()
    }

    async fn run(&mut self) {
        if let Some(target) = self.target.take() {
            if let Some(consumer) = self.consumer.take() {
                if let Err(_) = self.send_init.send((consumer, target)).await {
                    error!("send_init failed");
                }
            } else {
                error!("Missing Consumer");
            }
        }

        if !self.recv_results.is_empty() {
            if let Some((consumer, cache)) = self.recv_results.recv().await {
                self.consumer = Some(consumer);
                self.cache = Some(cache);
            }
        }
        if !self.recv_halt.is_empty() {
            if let Some(consumer) = self.recv_halt.recv().await {
                self.consumer = Some(consumer);
            }
        }
        if !self.recv_status.is_empty() {
            if let Some(status) = self.recv_status.recv().await {
                self.status = Some(status);
            }
        }
    }
}

