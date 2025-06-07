use rdkafka::{consumer::{BaseConsumer, Consumer}, TopicPartitionList};
use tokio::sync::mpsc;

use crate::{finder::SearchStatus, messages::FBMessage, Timestamp};

pub(crate) struct Searcher<'a, M> {
    consumer: &'a BaseConsumer,
    topic: String,
    offset: i64,
    tpl: TopicPartitionList,
    send_status: mpsc::Sender<SearchStatus>,
    results: Vec<M>
}

impl<'a, M> Searcher<'a, M>{
    pub(crate) fn new(consumer: &'a BaseConsumer, topic: &str, offset: i64, send_status: mpsc::Sender<SearchStatus>) -> Self {
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition(topic, 0);
        Self {
            consumer,
            offset,
            topic: topic.to_owned(),
            tpl,
            send_status,
            results: Default::default()
        }
    }

    pub(crate) fn iter_backstep(self) -> BackstepIter<'a, M> {
        BackstepIter {
            inner: self,
            step_size: None,
        }
    }

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
    fn into(self) -> Vec<M> {
        self.results
    }
}

impl<'a, M> Searcher<'a, M> where M: FBMessage<'a> {
    fn message(&mut self, offset: i64) -> Option<M> {
        self.tpl.set_partition_offset(self.topic.as_str(),0,rdkafka::Offset::OffsetTail(offset)).expect("");
        
        self.consumer.assign(&self.tpl).expect("");
        self.consumer
            .iter()
            .next()
            .and_then(Result::ok)
            .and_then(FBMessage::from_borrowed_message)
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

impl<'a, M> BackstepIter<'a, M> where M : FBMessage<'a> {
    pub(crate) fn backstep_until_time<F : Fn(Timestamp) -> bool>(&mut self, f : F) -> &mut Self {
        let mut offset = self.inner.offset;
        let mut earliest = self.inner.message(offset).expect("").timestamp();
        while f(earliest) {
            let new_offset = offset + self.step_size.expect("");
            let new_timestamp = self.inner.message(new_offset).expect("").timestamp();
            if f(new_timestamp) {
                offset = new_offset;
                earliest = new_timestamp;
            } else {
                break;
            }
        }
        self.inner.set_offset(offset);
        self
    }
}


pub(crate) struct ForwardSearchIter<'a, M> {
    inner: Searcher<'a, M>,
    message: Option<M>
}

impl <'a, M> ForwardSearchIter<'a, M> {
    pub(crate) fn collect(self) -> Searcher<'a, M> {
        self.inner
    }
}

impl<'a, M> ForwardSearchIter<'a, M>
where
    M : FBMessage<'a> {
    pub(crate) fn move_until<F : Fn(Timestamp) -> bool>(mut self, f : F) -> Self {
        for msg in self.inner.consumer.iter() {
            if let Some(msg) = msg
                .ok()
                .and_then(FBMessage::from_borrowed_message)
                .filter(|m|f(FBMessage::timestamp(m)))
            {
                self.message = Some(msg);
            }
        }
        self
    }
    
    pub(crate) fn acquire_while<F : Fn(&M) -> bool>(mut self, f : F) -> Self {
        let first_message = self.message.take().expect("");
        let timestamp = first_message.timestamp();
        if f(&first_message) {
            self.inner.results.push(first_message);
        }

        let messages = self.inner
            .consumer.iter()
            .flat_map(Result::ok)
            .flat_map::<Option<M>,_>(FBMessage::from_borrowed_message);

        for msg in messages {
            if msg.timestamp() == timestamp {
                if f(&msg) {
                    self.inner.results.push(msg);
                }
            } else {
                break;
            }
        }
        self
    }
}