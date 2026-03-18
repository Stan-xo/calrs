//! Event model: a time-bound calendar entry, optionally recurring.

use chrono::{DateTime, Utc};
use super::recurrence::Recurrence;

/// Progression state of a calendar event.
pub enum EventState {
    Planned,
    Confirmed,
    Finished,
}

/// A time-bound calendar entry with optional recurrence.
///
/// Recurring events generate virtual occurrences based on the `recurrence` rule.
/// A detached occurrence gets its date added to `exceptions` on the parent,
/// and its `parent_id` set to the parent event.
pub struct Event {
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    /// If `true`, the time part of the datetimes is ignored at display time.
    pub full_day: bool,
    pub state: EventState,
    pub recurrence: Option<Recurrence>,
    /// Dates excluded from recurrence generation (detached or deleted occurrences).
    pub exceptions: Option<Vec<DateTime<Utc>>>,
    /// Set if this event was detached from a recurring series.
    pub parent_id: Option<u64>,
}