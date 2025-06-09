use std::time::Duration;

use rdkafka::{
    consumer::{BaseConsumer, Consumer}, util::Timeout, Message, TopicPartitionList
};
use tokio::sync::mpsc;
use tracing::{info, instrument};

use crate::{finder::SearchStatus, messages::FBMessage, Timestamp};

pub(crate) struct Searcher<'a, M> {
    consumer: &'a BaseConsumer,
    topic: String,
    offset: i64,
    tpl: TopicPartitionList,
    send_status: mpsc::Sender<SearchStatus>,
    results: Vec<M>,
}

impl<'a, M> Searcher<'a, M> {
    #[instrument(skip_all)]
    pub(crate) fn new(
        consumer: &'a BaseConsumer,
        topic: &str,
        offset: i64,
        send_status: mpsc::Sender<SearchStatus>,
    ) -> Self {
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition(topic, 0);
        Self {
            consumer,
            offset,
            topic: topic.to_owned(),
            tpl,
            send_status,
            results: Default::default(),
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(send_status: &mpsc::Sender<SearchStatus>, new_status: SearchStatus) {
        //let mut status = self.status.lock().expect("Status");
        //status.replace(new_status);
        send_status.send(new_status).await.expect("");
    }

    #[instrument(skip_all)]
    pub(crate) fn iter_backstep(self) -> BackstepIter<'a, M> {
        BackstepIter {
            inner: self,
            step_size: None,
        }
    }

    #[instrument(skip_all)]
    pub(crate) fn iter_forward(self) -> ForwardSearchIter<'a, M> {
        ForwardSearchIter {
            inner: self,
            message: None,
        }
    }

    fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }
}

impl<'a, M> Into<Vec<M>> for Searcher<'a, M> {
    #[instrument(skip_all)]
    fn into(self) -> Vec<M> {
        self.results
    }
}

impl<'a, M> Searcher<'a, M>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all)]
    async fn message(&mut self, offset: i64) -> Option<M> {
        self.tpl
            .set_partition_offset(self.topic.as_str(), 0, rdkafka::Offset::OffsetTail(offset))
            .expect("");

        self.consumer.assign(&self.tpl).expect("");
        let msg : Option<M> = self.consumer
            .iter()
            .next()
            .and_then(Result::ok)
            .and_then(FBMessage::from_borrowed_message);
        match &msg {
            Some(msg) => self.send_status.send(SearchStatus::Text(format!("Message at offset {offset}: timestamp: {0}", msg.timestamp()))),
            None => self.send_status.send(SearchStatus::Text(format!{"Message at offset {offset} failed"})),
        }.await.expect("");
        msg
    }
}

pub(crate) struct BackstepIter<'a, M> {
    inner: Searcher<'a, M>,
    step_size: Option<i64>,
}

impl<'a, M> BackstepIter<'a, M> {
    pub(crate) fn step_size(&mut self, step_size: i64) -> &mut Self {
        self.step_size = Some(step_size);
        self
    }

    pub(crate) fn collect(self) -> Searcher<'a, M> {
        self.inner
    }
}

impl<'a, M> BackstepIter<'a, M>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all)]
    pub(crate) async fn backstep_until_time<F: Fn(Timestamp) -> bool>(&mut self, f: F) -> &mut Self {
        let mut offset = self.inner.offset;
        let mut earliest = {
            match self.inner.message(offset).await {
                Some(message) => message.timestamp(),
                None => return self
            }
        };
        self.inner.message(offset).await.expect("").timestamp();
        //info!("{offset} Earliest {earliest}");
        while f(earliest) {
            let new_offset = offset + self.step_size.expect("");
            match self.inner.message(new_offset).await {
                Some(message) => { 
                    let new_timestamp = message.timestamp();
                    //info!("New {new_timestamp}");
                    if f(new_timestamp) {
                        offset = new_offset;
                        earliest = new_timestamp;
                    } else {
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
        self.inner.set_offset(offset);
        self
    }
}

pub(crate) struct ForwardSearchIter<'a, M> {
    inner: Searcher<'a, M>,
    message: Option<M>,
}

impl<'a, M> ForwardSearchIter<'a, M> {
    pub(crate) fn collect(self) -> Searcher<'a, M> {
        self.inner
    }
}

impl<'a, M> ForwardSearchIter<'a, M>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all)]
    pub(crate) async fn move_until<F: Fn(Timestamp) -> bool>(mut self, f: F) -> Self {
        while let Some(msg) = self.inner.consumer.poll(Timeout::After(Duration::from_millis(100))) {
            if let Some(msg) = msg
                .ok()
                .and_then(FBMessage::from_borrowed_message)
                .filter(|m| f(FBMessage::timestamp(m)))
            {
                self.message = Some(msg);
                self.inner.send_status.send(SearchStatus::Text(format!("Message timestamp: {0}", self.message.as_ref().expect("").timestamp()))).await.expect("");
                break;
            }
        }
        self
    }

    #[instrument(skip_all)]
    pub(crate) async fn acquire_while<F: Fn(&M) -> bool>(mut self, f: F) -> Self {
        if let Some(first_message) = self.message.take() {
            let timestamp = first_message.timestamp();
            if f(&first_message) {
                self.inner.results.push(first_message);
            }

            let messages = self
                .inner
                .consumer
                .iter()
                .flat_map(Result::ok)
                .flat_map::<Option<M>, _>(FBMessage::from_borrowed_message);


            for msg in messages {
                self.inner.send_status.send(SearchStatus::Text(format!("Message timestamp: {0}", msg.timestamp()))).await.expect("");
                if msg.timestamp() == timestamp {
                    if f(&msg) {
                        self.inner.results.push(msg);
                    }
                } else {
                    break;
                }
            }
        }
        self
    }
}
