//! Recurrence model: rules for repeating calendar events.

use chrono::{DateTime, Utc};

/// How often a recurring event repeats.
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Defines when a recurrence stops generating occurrences.
pub enum RecurrenceEnd {
    Never,
    /// Stops after a fixed number of occurrences.
    AfterOccurrences(u32),
    UntilDate(DateTime<Utc>),
}

/// Rule defining how and when an event repeats.
pub struct Recurrence {
    pub frequency: RecurrenceFrequency,
    /// `1` = every period, `2` = every other period, etc.
    ///
    /// ex : frequency: Weekly + interval: 2  =  Every 2 week
    pub interval: u32,
    pub end: RecurrenceEnd,
}
