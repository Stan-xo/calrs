//! Reminder model.

/// Delay before a calendar item at which a reminder should trigger.
pub enum ReminderDelay {
    Minutes5,
    Minutes10,
    Minutes15,
    Minutes30,
    Hour1,
    Hour2,
    Hour6,
    Hour12,
    Day1,
    Day2,
    Day3,
    Week1,
    Week2,
    Week3,
    Month1,
    Year1,
}

/// A single reminder associated with a calendar item.
pub struct Reminder {
    pub active: bool,
    pub delay: ReminderDelay,
}
