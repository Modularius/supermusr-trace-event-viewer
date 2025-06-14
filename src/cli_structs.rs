use clap::{Args, Subcommand, ValueEnum};
use supermusr_common::{Channel, Intensity, Time};

use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Args)]
pub(crate) struct Topics {
    /// Kafka trace topic.
    #[clap(long)]
    pub(crate) trace_topic: String,

    /// Kafka digitiser event list topic.
    #[clap(long)]
    pub(crate) digitiser_event_topic: String,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct UserBounds {
    /// Minimum time bin to graph, derived from input if left unspecified.
    #[clap(long)]
    pub(crate) time_min: Option<Time>,

    /// Maximum time bin to graph, derived from input if left unspecified.
    #[clap(long)]
    pub(crate) time_max: Option<Time>,

    /// Minimum intensity value to graph, derived from input if left unspecified.
    #[clap(long)]
    pub(crate) intensity_min: Option<Intensity>,

    /// Maximum intensity value to graph, derived from input if left unspecified.
    #[clap(long)]
    pub(crate) intensity_max: Option<Intensity>,
}

/*
#[derive(Clone, Subcommand)]
pub(crate) enum Mode {
    /// Outputs image to file.
    File(OutputToFile),
    // /// Outputs image to server.
    //Server(OutputToFile),
}
    */

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum CollectType {
    /// Collects the given number of traces.
    Traces,
    /// Collects the given number of event lists.
    Events,
    /// Collects the given number of traces and their corresponding eventlists.
    All,
}

/// * If `from` is the only field set, then it finds the first frame at or after `from`.
/// * If `until` is the only field set, then it finds the first frame before `until`.
/// * If `from` and `num` are the only fields set, then it finds the first `num` frames at or after `from`.
/// * If `until` and `num` are the only fields set, then it finds the first `num` frames before `until`.
/// * If `from` and `until` are the only fields set, then it finds all frames between `from` (inclusive) and `until` (exclusive).
/// * If `from`, `until` and `num` are the only fields set, then it finds at most the first `num` frames between `from` (inclusive) and `until` (exclusive).
#[derive(Clone, Debug, Args)]
pub(crate) struct Select {
    /// The timestamp of the frame to search for.
    #[clap(long, default_value = "Utc::now()")]
    pub(crate) timestamp: DateTime<Utc>,

    #[clap(flatten)]
    pub(crate) step: Steps,

    // /// The digitiser Id to search for.
    // #[clap(long)]
    // pub(crate) digitiser_id: DigitizerId,
    /// The channel to search for.
    #[clap(long)]
    pub(crate) channel: Channel,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct Steps {
    /// The min step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "50")]
    pub(crate) min_step_size: i64,

    /// The max step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "10")]
    pub(crate) step_mul_coef: i64,

    /// The max step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "5")]
    pub(crate) num_step_passes: u32,
}
