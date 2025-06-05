use std::{error::Error, marker::PhantomData, ops::Range};

use chrono::{DateTime, Utc};
use rdkafka::{
    consumer::{BaseConsumer, Consumer},
    error::KafkaError,
    message::BorrowedMessage,
    Message, Offset, TopicPartitionList,
};
use supermusr_common::{Channel, DigitizerId};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage;
use tracing::info;

use crate::{
    cli_structs::Steps,
    message::{FBMessage, TraceMessage},
    Cache, DigitizerMessage, Topics, UnpackMessage,
};

type Index = i64;
type Timestamp = DateTime<Utc>;

fn get_timestamp_at_index(
    consumer: &BaseConsumer,
    topic: &str,
    index: Index,
) -> anyhow::Result<Timestamp> {
    let mut tpl = TopicPartitionList::new();
    let offset = Offset::OffsetTail(index);
    tpl.add_partition_offset(topic, 0, offset)?;
    consumer.assign(&tpl)?;
    match consumer.poll(None) {
        Some(res) => res?
            .timestamp()
            .to_millis()
            .and_then(Timestamp::from_timestamp_millis)
            .ok_or(anyhow::anyhow!("No Message Found")),
        None => Err(anyhow::anyhow!("No Message Polled")),
    }
}

pub(crate) trait FinderByTimestamp<'a> {
    type Key;
    type Output: FBMessage<'a>;

    fn get_timestamp_at_index(&self, index: Index) -> anyhow::Result<Timestamp>;
    fn get_next_message_with_timestamp_after(&self, timestamp: &Timestamp) -> Option<Self::Output>;
    fn get_next_message_with_timestamp_equal_and_criteria(
        &self,
        key: Self::Key,
        first_message: &Self::Output,
        timestamp: &Timestamp,
    ) -> Option<Self::Output>;
}

fn push_trace(
    cache: &mut Cache,
    channel: Channel,
    dig_trace_msg: &DigitizerAnalogTraceMessage,
) -> Option<DigitizerId> {
    info!("Found trace");
    if dig_trace_msg
        .channels()
        .unwrap()
        .iter()
        .find(|c| c.channel() == channel)
        .is_some()
    {
        info!("Push trace");
        cache.push_trace(&dig_trace_msg);
        Some(dig_trace_msg.digitizer_id())
    } else {
        None
    }
}

pub(crate) struct TraceFinderByKafkaTimestamp<'a> {
    consumer: &'a BaseConsumer,
    topic: &'a str,
}

impl<'a> TraceFinderByKafkaTimestamp<'a> {
    pub(crate) fn new(consumer: &'a BaseConsumer, topic: &'a str) -> Self {
        Self { consumer, topic }
    }
}

impl<'a> FinderByTimestamp<'a> for TraceFinderByKafkaTimestamp<'a> {
    type Output = TraceMessage<'a>;
    type Key = Channel;

    fn get_timestamp_at_index(&self, index: Index) -> anyhow::Result<Timestamp> {
        get_timestamp_at_index(self.consumer, self.topic, index)
    }

    fn get_next_message_with_timestamp_after(&self, timestamp: &Timestamp) -> Option<Self::Output> {
        for msg in self.consumer.iter() {
            if let Some(trace) = msg
                .ok()
                .and_then(TraceMessage::from_borrowed_message)
                .filter(|trace| trace.timestamp() > *timestamp)
            {
                return Some(trace);
            }
        }
        None
    }

    fn get_next_message_with_timestamp_equal_and_criteria(
        &self,
        cache: &mut Cache,
        key: Self::Key,
        first_message: &Self::Output,
        timestamp: &Timestamp,
    ) -> Option<Self::Output> {
        let mut dig_id = None;
        if let Some(did) = push_trace(cache, key, &first_message.get_unpacked_message().expect(""))
        {
            dig_id = Some(did);
        } else {
            for msg in self.consumer.iter() {
                if let Some(trace) = msg
                    .ok()
                    .and_then(TraceMessage::from_borrowed_message)
                    .filter(|trace| trace.timestamp() == *timestamp)
                    .filter(|trace| {
                        trace
                            .get_unpacked_message()
                            .and_then(|trace| trace.channels())
                            .and_then(|channels| {
                                channels.iter().find(|channel| channel.channel() == key)
                            })
                            .is_some()
                    })
                {
                    push_trace(cache, key, &trace.get_unpacked_message().expect(""));
                    return Some(trace);
                }
            }
        }
        None
    }
}

pub(crate) struct FindByDate<'a, F: FinderByTimestamp<'a>> {
    step: Steps,
    finder: F,
}

impl<F: FinderByTimestamp> FindByDate<F> {
    pub(crate) fn new(finder: F, step: &Steps) -> Self {
        Self {
            step: step.clone(),
            finder,
        }
    }

    pub(crate) fn find<R>(
        &self,
        topics: &Topics,
        timestamp: Timestamp,
        key: F::Key,
    ) -> anyhow::Result<(Index, F::Output)> {
        let (idx, tmstp) = self.find_first_index_with_timestamp_before(timestamp)?;
        let first_message = self
            .finder
            .get_next_message_with_timestamp_after(&timestamp);
        let first_message = self
            .finder
            .get_next_message_with_timestamp_equal_and_criteria(key, &timestamp);

        if let Some(first_message) = first_message {
            let timestamp = first_message.timestamp().expect("");
            info!("Seeking Timestamp {timestamp}");
            let mut dig_id = None;
            if let Some(did) = push_trace(&mut cache, args.select.channel, dig_msg) {
                dig_id = Some(did);
            } else {
                for msg in consumer.iter() {
                    match msg {
                        Ok(message) => {
                            if let Some(dig_msg) = message.unpack_message(&args.topics) {
                                if dig_msg.is_timestamp_equal_to(timestamp) {
                                    if let Some(did) =
                                        push_trace(&mut cache, args.select.channel, dig_msg)
                                    {
                                        dig_id = Some(did);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }

    // Seeks through the kafka topic for the first
    pub(crate) fn find_first_index_with_timestamp_before(
        &self,
        target_timestamp: Timestamp,
    ) -> anyhow::Result<(Index, Timestamp)> {
        let mut index = 1;
        let mut earliest_timestamp = self.finder.get_timestamp_at_index(index)?;
        for step in (0..self.step.num_step_passes).rev() {
            let step_size = self.step.min_step_size * self.step.step_mul_coef.pow(step);
            info!("Step: {step_size}");
            (index, earliest_timestamp) =
                self.find_last_index_with_timestamp_after(index, step_size, target_timestamp)?;
        }
        Ok((index, earliest_timestamp))
    }

    /// Jumps through the consumer in steps of size `step_size` until it finds the
    ///
    pub(crate) fn find_last_index_with_timestamp_after(
        &self,
        start: Index,
        step_size: Index,
        target_timestamp: Timestamp,
    ) -> anyhow::Result<(Index, Timestamp)> {
        let mut index = start;
        let mut earliest_timestamp = self.finder.get_timestamp_at_index(index)?;
        while earliest_timestamp > target_timestamp {
            info!("{index}, {earliest_timestamp}");
            let new_timestamp = self.finder.get_timestamp_at_index(index + step_size)?;
            if new_timestamp <= target_timestamp {
                break;
            } else {
                index += step_size;
                earliest_timestamp = new_timestamp;
            }
        }
        Ok((index, earliest_timestamp))
    }
}
