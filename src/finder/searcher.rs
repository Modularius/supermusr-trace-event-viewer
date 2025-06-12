use std::time::Duration;

use rdkafka::{
    consumer::{BaseConsumer, Consumer}, util::Timeout, TopicPartitionList
};
use tokio::sync::mpsc;
use tracing::instrument;

use crate::{finder::SearchStatus, messages::FBMessage, Timestamp};

pub(crate) struct Searcher<'a, M> {
    consumer: &'a BaseConsumer,
    topic: String,
    offset: i64,
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
        let mut tpl = TopicPartitionList::with_capacity(1);
        tpl.add_partition_offset(topic, 0, rdkafka::Offset::End).expect("");
        consumer.assign(&tpl).expect("");
        Self {
            consumer,
            offset,
            topic: topic.to_owned(),
            send_status,
            results: Default::default(),
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(send_status: &mpsc::Sender<SearchStatus>, new_status: SearchStatus) {
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

    pub(crate) fn get_offset(&self) -> i64 {
        self.offset
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
        self.consumer.seek(&self.topic, 0, rdkafka::Offset::OffsetTail(offset), Duration::from_millis(1)).expect("");
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
        //self.inner.message(offset).await.expect("").timestamp();
        
        while f(earliest) {
            let new_offset = offset + self.step_size.expect("Size step should have been set. This should never fail.");
            match self.inner.message(new_offset).await {
                Some(message) => { 
                    let new_timestamp = message.timestamp();
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
    pub(crate) async fn acquire_while<F: Fn(&M) -> bool>(mut self, f: F, number: usize) -> Self {
        if let Some(first_message) = self.message.take() {
            let mut timestamp = first_message.timestamp();
            if f(&first_message) {
                self.inner.results.push(first_message);
            }

            let mut messages = self
                .inner
                .consumer
                .iter()
                .flat_map(Result::ok)
                .flat_map::<Option<M>, _>(FBMessage::from_borrowed_message);

            for _ in 0..number {
                while let Some(msg) = messages.next() {
                    self.inner.send_status.send(SearchStatus::Text(format!("Message timestamp: {0}", msg.timestamp()))).await.expect("");
                    let new_timestamp = msg.timestamp();
                    if new_timestamp == timestamp {
                        if f(&msg) {
                            self.inner.results.push(msg);
                        }
                    } else {
                        timestamp = new_timestamp;
                        break;
                    }
                }
            }
        }
        self
    }
}
