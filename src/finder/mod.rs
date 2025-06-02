mod find_by_date;
mod find;
pub(crate) use find_by_date::{FindByDate, FinderByTimestamp, TraceFinderByKafkaTimestamp};

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use rdkafka::{consumer::{BaseConsumer, Consumer}, error::KafkaResult, message::BorrowedMessage, Message, Offset, TopicPartitionList};
/*
use crate::{message_handling, CollectType, Engine, Find, Topics};
enum FindModeRange {
    Between(DateTime<Utc>,DateTime<Utc>),
    After(DateTime<Utc>),
    Before(DateTime<Utc>),
}

enum FindMode {
    All(FindModeRange),
    Num(usize, FindModeRange),
    Next(usize),
    Prev(usize)
}

pub(crate) struct Finder<'a> {
    mode: FindMode,
    phantom: PhantomData<&'a ()>
}

impl<'a> Finder<'a> {
    pub(crate) fn from_cli(find: Find) -> Result<Self, ()> {
        let range = if let Some((from, until)) = Option::zip(find.from, find.until) {
            Ok(FindModeRange::Between(from, until))
        } else if let Some(from) = find.from {
            Ok(FindModeRange::After(from))
        } else if let Some(until) = find.until {
            Ok(FindModeRange::Before(until))
        } else {
            Err(())
        }?;
        let me = Self {
            mode: if let Some(num) = find.num {
                FindMode::Num(num, range)
            } else {
                FindMode::All(range)
            },
            phantom: PhantomData
        };
        Ok(me)
    }

    pub(crate) fn subscribe_consumer(&self, consumer: &BaseConsumer, topics: &Topics, collect: &CollectType) -> anyhow::Result<()> {
        let mut tpl = TopicPartitionList::new();
        let num = match self.mode {
            FindMode::All(_) => 0,
            FindMode::Num(num, _) => num,
            FindMode::Next(num) => num,
            FindMode::Prev(num) => num,
        };
        let offset = Offset::OffsetTail(num.try_into()?);
        match collect {
            CollectType::Traces => {
                tpl.add_partition_offset(topics.trace_topic.as_str(), 0, offset)?;
            }
            CollectType::Events => {
                tpl.add_partition_offset(topics.digitiser_event_topic.as_str(), 0, offset)?;
            },
            CollectType::All => {
                tpl.add_partition_offset(topics.trace_topic.as_str(), 0, offset)?;
                tpl.add_partition_offset(topics.digitiser_event_topic.as_str(), 0, offset)?;
            },
        }
        consumer.assign(&tpl)?;
        Ok(())
    }
    
    pub(crate) fn iter(self, consumer: &'a BaseConsumer, engine: &'a Engine) -> FinderIter<'a> {
        FinderIter {
            engine,
            consumer,
            finder: self,
        }
    }
}

pub(crate) struct FinderIter<'a> {
    finder: Finder<'a>,
    consumer: &'a BaseConsumer,
    engine: &'a Engine,
}

impl<'a> Iterator for FinderIter<'a> {
    type Item = KafkaResult<BorrowedMessage<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if match self.finder.mode {
            FindMode::All(_) => true,
            FindMode::Num(num, _) => self.engine.get_count() >= num
        } {
            return None;
        }
            match self.consumer.poll(None) {
                Some(message) => match message {
                    Ok(message) => {
                        if message.topic()
                        message.unpack_trace_message(topic)
                        todo!()
                    }
                    Err(_) => todo!(),
                },
                None => todo!(),
        }
    }
} */