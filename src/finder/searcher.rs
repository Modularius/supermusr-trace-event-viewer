use std::time::Duration;

use rdkafka::{
    consumer::{BaseConsumer, Consumer, StreamConsumer},
    util::Timeout,
    TopicPartitionList,
};
use tokio::sync::mpsc;
use tracing::instrument;

use crate::{finder::SearchStatus, messages::FBMessage, Timestamp};

/// Object to search through the broker from a given offset, on a given topic, for messages of type `M`.
pub(crate) struct Searcher<'a, M, C> {
    /// Reference to the Kafka consumer.
    consumer: &'a C,
    /// Topic to search on.
    topic: String,
    /// Current offset.
    offset: i64,
    /// Send channel, along which status messages should be sent.
    send_status: mpsc::Sender<SearchStatus>,
    /// Results accumulate here.
    results: Vec<M>,
}

impl<'a, M, C : Consumer> Searcher<'a, M, C> {
    /// Creates a new instance, and assigns the given topic to the broker's consumer.
    ///
    /// # Attributes
    /// - consumer: the broker's consumer to use.
    /// - topic: the topic to search on.
    /// - offset: the offset to search from.
    /// - send_status: send channel, along which status messages should be sent.
    #[instrument(skip_all)]
    pub(crate) fn new(
        consumer: &'a C,
        topic: &str,
        offset: i64,
        send_status: mpsc::Sender<SearchStatus>,
    ) -> Self {
        let mut tpl = TopicPartitionList::with_capacity(1);
        tpl.add_partition_offset(topic, 0, rdkafka::Offset::End)
            .expect("");
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
    pub(crate) async fn emit_status(
        send_status: &mpsc::Sender<SearchStatus>,
        new_status: SearchStatus,
    ) {
        send_status.send(new_status).await.expect("");
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a backstep iterator.
    pub(crate) fn iter_backstep(self) -> BackstepIter<'a, M, C> {
        BackstepIter {
            inner: self,
            step_size: None,
        }
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a forward iterator.
    pub(crate) fn iter_forward(self) -> ForwardSearchIter<'a, M, C> {
        ForwardSearchIter {
            inner: self,
            message: None,
        }
    }

    /// Sets the offset.
    fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    /// Gets the offset.
    pub(crate) fn get_offset(&self) -> i64 {
        self.offset
    }
}

/// Extracts the results from the searcher, when the user is finished with it.
impl<'a, M, C> Into<Vec<M>> for Searcher<'a, M, C> {
    #[instrument(skip_all)]
    fn into(self) -> Vec<M> {
        self.results
    }
}

impl<'a, M> Searcher<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all)]
    async fn message(&mut self, offset: i64) -> Option<M> {
        self.consumer
            .seek(
                &self.topic,
                0,
                rdkafka::Offset::OffsetTail(offset),
                Duration::from_millis(1),
            )
            .expect("");

        let msg : Option<M> = self
            .consumer
            .recv().await
            .ok()
            .and_then(FBMessage::from_borrowed_message);

        match &msg {
            Some(msg) => self.send_status.send(SearchStatus::Text(format!(
                "Message at offset {offset}: timestamp: {0}",
                msg.timestamp()
            ))),
            None => self.send_status.send(SearchStatus::Text(
                format! {"Message at offset {offset} failed"},
            )),
        }
        .await
        .expect("");
        msg
    }
}

/// Performs a backwards search on the broker from the searcher's offset.
///
/// Note this iterator can only move the [Searcher]'s offset, it cannot accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct BackstepIter<'a, M, C> {
    inner: Searcher<'a, M, C>,
    step_size: Option<i64>,
}

impl<'a, M, C> BackstepIter<'a, M, C> {
    /// Sets the
    pub(crate) fn step_size(&mut self, step_size: i64) -> &mut Self {
        self.step_size = Some(step_size);
        self
    }

    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C> {
        self.inner
    }
}

impl<'a, M> BackstepIter<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    /// Repeatedly search the topic backwards, in increments of [Self::step_size],
    /// until the given predicate of the message's timestamp is satisfied.
    ///
    /// # Attributes
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is later than the target.
    #[instrument(skip_all)]
    pub(crate) async fn backstep_until_time<F: Fn(Timestamp) -> bool>(
        &mut self,
        f: F,
    ) -> &mut Self {
        let mut offset = self.inner.offset;
        let mut earliest = {
            match self.inner.message(offset).await {
                Some(message) => message.timestamp(),
                None => return self,
            }
        };

        while f(earliest) {
            let new_offset = offset
                + self
                    .step_size
                    .expect("Size step should have been set. This should never fail.");
            match self.inner.message(new_offset).await {
                Some(message) => {
                    let new_timestamp = message.timestamp();
                    if f(new_timestamp) {
                        offset = new_offset;
                        earliest = new_timestamp;
                    } else {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }
        self.inner.set_offset(offset);
        self
    }
}

/// Searches on a topic forwards, one message at a time.
///
/// Note this iterator can both move the [Searcher]'s offset and accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct ForwardSearchIter<'a, M, C> {
    inner: Searcher<'a, M, C>,
    message: Option<M>,
}

impl<'a, M, C> ForwardSearchIter<'a, M, C> {
    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C> {
        self.inner
    }
}

impl<'a, M> ForwardSearchIter<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    /// Steps forward, message by message, until the given predicate fails.
    ///
    /// # Attributes
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is earlier than the target.
    #[instrument(skip_all)]
    pub(crate) async fn move_until<F: Fn(Timestamp) -> bool>(mut self, f: F) -> Self {
        while let Ok(msg) = self
            .inner
            .consumer
            .recv().await
        {
            if let Some(msg) = FBMessage::from_borrowed_message(msg)
                .filter(|m| f(FBMessage::timestamp(m)))
            {
                self.message = Some(msg);
                self.inner
                    .send_status
                    .send(SearchStatus::Text(format!(
                        "Message timestamp: {0}",
                        self.message.as_ref().expect("").timestamp()
                    )))
                    .await
                    .expect("");
                break;
            }
        }
        self
    }

    /// Steps forward, message by message, acquiring messages which satisfy the predicate, until the given number of messages are obtained. [TODO]
    ///
    /// # Attributes
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is earlier than the target.
    #[instrument(skip_all)]
    pub(crate) async fn acquire_while<F: Fn(&M) -> bool>(mut self, f: F, number: usize) -> Self {
        if let Some(first_message) = self.message.take() {
            let mut timestamp = first_message.timestamp();
            if f(&first_message) {
                self.inner.results.push(first_message);
            }

            let mut messages : Option<M> = self
                .inner
                .consumer
                .recv().await
                .ok()
                .and_then(FBMessage::from_borrowed_message);

            for _ in 0..number {
                while let Some(msg) = messages {
                    
                    messages = self.inner.consumer.recv().await.ok()
                        .and_then(FBMessage::from_borrowed_message);
                    
                    self.inner
                        .send_status
                        .send(SearchStatus::Text(format!(
                            "Message timestamp: {0}",
                            msg.timestamp()
                        )))
                        .await
                        .expect("");
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
