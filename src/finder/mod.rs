use std::marker::PhantomData;

//mod find_by_date;
//mod find;
use chrono::{DateTime, Utc};
//pub(crate) use find_by_date::{FindByDate, TraceFinderByKafkaTimestamp};

use rdkafka::{consumer::{BaseConsumer, Consumer}, message::BorrowedMessage, Message, Offset, TopicPartitionList};

use crate::{cli_structs::Steps, message::FBMessage};

type Timestamp = DateTime<Utc>;

pub(crate) trait FinderType<'a> {
    type Msg : FBMessage<'a>;

    fn topic(&self) -> &str;
    //fn get_message(&self, message: &'a BorrowedMessage<'a>) -> Option<Self::Msg>;
}

pub(crate) struct Finder<'a, M>
{
    topic: &'a str,
    phantom: PhantomData<M>
}

impl<'a, M> Finder<'a, M> {
    pub(crate) fn new(topic: &'a str) -> Self {
        Self {
            topic,
            phantom: PhantomData,
        }
    }
}

impl<'a, M> FinderType<'a> for Finder<'a, M> where
    M: FBMessage<'a,> {
    type Msg = M;

    fn topic(&self) -> &str {
        self.topic
    }
}

pub(crate) struct FindEngine<'a> {
    consumer: &'a BaseConsumer,
    steps: &'a Steps,
    tpl: TopicPartitionList,
}

impl<'a> FindEngine<'a> {
    pub(crate) fn new(
        consumer: &'a BaseConsumer,
        steps: &'a Steps,
    ) -> Self {
        Self {
            consumer,
            steps,
            tpl: TopicPartitionList::new()
        }
    }

    fn setup<F : FinderType<'a>>(&mut self, finder: &F) {
        self.tpl = TopicPartitionList::with_capacity(1);
        self.tpl.add_partition(finder.topic(), 0);
    }

    fn get_offset<F : FinderType<'a>>(&mut self, finder: &F) -> i64 {
        match self.tpl.find_partition(finder.topic(), 0).expect("").offset() {
            Offset::OffsetTail(offset) => offset,
            _ => unreachable!()
        }
    }

    fn set_offset<F : FinderType<'a>>(&mut self, finder: &F, offset: i64) {
        self.tpl
            .set_partition_offset(finder.topic(), 0, Offset::OffsetTail(offset))
            .unwrap();

        self.consumer
            .assign(&self.tpl)
            .unwrap();
    }

    fn get_current_message<F : FBMessage<'a>>(&mut self) -> Option<F> {
        self.consumer
            .iter()
            .next()
            .and_then(Result::ok)
            .and_then(FBMessage::from_borrowed_message)
    }

    /// Jumps through the consumer in steps of size `step_size` until it finds the 
    /// 
    fn set_offset_to_last_index_with_timestamp_after<F : FinderType<'a>>(&mut self, finder: &F, start: i64, step_size: i64, target: Timestamp) -> Option<(i64,Timestamp)> {
        let mut index = start;
        self.set_offset(finder, index);
        let mut earliest = self.get_current_message::<F::Msg>()?.timestamp();
        while earliest > target {
            self.set_offset(finder, index + step_size);
            let new_timestamp = self.get_current_message::<F::Msg>()?.timestamp();
            if new_timestamp <= target {
                break;
            } else {
                index += step_size;
                earliest = new_timestamp;
            }
        }
        Some((index, earliest))
    }
 
    // Seeks through the kafka topic for the first 
    fn set_offset_to_first_index_with_timestamp_before<F : FinderType<'a>>(&mut self, finder: &F, start: i64, target: Timestamp) -> Option<(i64,Timestamp)> {
        let mut index = start;
        let mut earliest = self.get_current_message::<F::Msg>()?.timestamp();
        for step in (0..self.steps.num_step_passes).rev() {
            let step_size = self.steps.min_step_size*self.steps.step_mul_coef.pow(step);
            (index, earliest) = self.set_offset_to_last_index_with_timestamp_after(finder, index, step_size, target)?;
        }
        Some((index, earliest))
    }

    fn poll_for_next_message_with_timestamp_after_or_equal<M : FBMessage<'a>>(&self, timestamp: DateTime<Utc>) -> Option<M> {
        for msg in self.consumer.iter() {
            if let Some(msg) = msg
                .ok()
                .and_then(FBMessage::from_borrowed_message)
                .filter(|msg : &M|msg.timestamp() >= timestamp) {
                return Some(msg)
            }
        }
        None
    }

    pub(crate) fn find<F : FinderType<'a>, Filter>(&mut self, finder: &F, start: i64, target: Timestamp, filter: Filter) -> Option<F::Msg>
    where Filter: Fn(&F::Msg)->bool {
        self.set_offset_to_first_index_with_timestamp_before(finder, start, target);
        let message = self.poll_for_next_message_with_timestamp_after_or_equal::<F::Msg>(target)?;

        let timestamp = message.timestamp();
        if filter(&message) {
            return Some(message);
        }
        for msg in self.consumer.iter() {
            if let Some(msg) = msg.ok().and_then::<F::Msg,_>(FBMessage::from_borrowed_message) {
                if msg.timestamp() == timestamp {
                    if filter(&msg) {
                        return Some(msg);
                    }
                } else {
                    return None;
                }
            }
        }
        None
    }
}