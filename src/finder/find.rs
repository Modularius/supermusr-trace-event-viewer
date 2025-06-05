use chrono::{DateTime, Duration, Utc};

type Index = i64;
type Timestamp = DateTime<Utc>;

pub(crate) enum State {
    TooEarly,
    WithinBuffer,
    After,
}

pub(crate) struct Find {
    target: Timestamp,
    ///
    time_precision: Duration,
    /// Ammount to subtract off the solution.
    offset_buffer: Index,

    state: State,
    step: Index,
    current_index: Index,
    current_timestamp: Timestamp,
}
